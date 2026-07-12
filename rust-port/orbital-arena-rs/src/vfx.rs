//! VFX particle system (specs/transform/vfx-particle-system.spec.md): a fixed-capacity,
//! render-only particle pool with gravity-only force application (wind/turbulence
//! descoped per the spec's §7). Never read by physics or game-rule systems.

use bevy::math::DVec3;
use bevy::prelude::*;
use rand::Rng;

use crate::rng::DeterministicRng;

/// Pool capacity (sandbox-scenes.spec.md §9: 2048).
pub const VFX_POOL_CAPACITY: usize = 2048;
/// Lifetime shared by every VFX particle (sandbox-scenes.spec.md §9: "Lifetime 0.6 s
/// (min == max)") — the reference's one shared pool/emitter has exactly one lifetime
/// value, so this is a crate-wide constant rather than a per-particle field.
pub const VFX_LIFETIME_SECONDS: f64 = 0.6;
/// Spawn color shared by every VFX particle, from every burst source (sandbox-scenes.spec.md
/// §9: RGBA (1.0, 0.85, 0.4, 1.0) normalized) — the reference has exactly one shared
/// emitter per scene, so bounce sparks and interactive click bursts share this color too.
pub const VFX_SPAWN_COLOR: [f32; 4] = [1.0, 0.85, 0.4, 1.0];
/// Gravity-only force (spec §7 scope reduction), matching the reference's downward pull.
const GRAVITY_ACCEL: DVec3 = DVec3::new(0.0, -9.81, 0.0);

/// One VFX particle (spec §2.1). Position/velocity/color/size are render-boundary
/// precision; `remaining_lifetime_seconds` is the `f64` time accumulator (Article 5/VI).
#[derive(Component, Debug, Clone, Copy)]
pub struct VfxParticle {
    pub position: DVec3,
    pub velocity: DVec3,
    pub remaining_lifetime_seconds: f64,
    pub color: [f32; 4],
    pub size: f32,
}

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, tick_vfx);
    }
}

/// Spawns up to `count` particles at `position` with a randomized outward velocity,
/// clamped to the pool's remaining free capacity (spec §3.1 `try_spawn`: partial success,
/// never all-or-nothing, never a panic on exhaustion). Returns the number actually spawned.
pub fn try_spawn_burst(
    commands: &mut Commands,
    live_count: usize,
    position: DVec3,
    count: usize,
    lifetime_seconds: f64,
    color: [f32; 4],
    rng: &mut DeterministicRng,
) -> usize {
    let free = VFX_POOL_CAPACITY.saturating_sub(live_count);
    let spawned = count.min(free);
    for _ in 0..spawned {
        let angle = rng.0.random_range(0.0..std::f64::consts::TAU);
        // Speed/size ranges per sandbox-scenes.spec.md §9.
        let speed = rng.0.random_range(1.5..4.0);
        let size = rng.0.random_range(0.02..0.05) as f32;
        let velocity = DVec3::new(angle.cos() * speed, angle.sin() * speed, 0.0);
        commands.spawn(VfxParticle {
            position,
            velocity,
            remaining_lifetime_seconds: lifetime_seconds,
            color,
            size,
        });
    }
    spawned
}

/// Applies gravity, integrates velocity into position, then ages/retires (spec §3.2
/// `tick` delegating to `age_and_retire`) — despawning particles whose remaining
/// lifetime reaches <= 0. `dt == 0.0` is a no-op (matches every particle simply not
/// moving/aging).
fn tick_vfx(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut VfxParticle)>,
) {
    let dt = time.delta_secs_f64();
    if dt == 0.0 {
        return;
    }
    for (entity, mut particle) in &mut particles {
        particle.velocity += GRAVITY_ACCEL * dt;
        let velocity = particle.velocity;
        particle.position += velocity * dt;
        particle.remaining_lifetime_seconds -= dt;
        if particle.remaining_lifetime_seconds <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, SeedableRng};

    fn test_rng() -> DeterministicRng {
        DeterministicRng(StdRng::seed_from_u64(1))
    }

    #[test]
    fn try_spawn_burst_spawns_exactly_requested_when_capacity_allows() {
        let mut app = App::new();
        let mut rng = test_rng();
        let spawned = try_spawn_burst(
            &mut app.world_mut().commands(),
            0,
            DVec3::ZERO,
            10,
            0.5,
            [1.0, 1.0, 1.0, 1.0],
            &mut rng,
        );
        app.world_mut().flush();
        assert_eq!(spawned, 10);
        let mut query = app.world_mut().query::<&VfxParticle>();
        assert_eq!(query.iter(app.world()).count(), 10);
    }

    #[test]
    fn try_spawn_burst_partial_success_when_pool_nearly_full() {
        let mut app = App::new();
        let mut rng = test_rng();
        let spawned = try_spawn_burst(
            &mut app.world_mut().commands(),
            VFX_POOL_CAPACITY - 3,
            DVec3::ZERO,
            10,
            0.5,
            [1.0, 1.0, 1.0, 1.0],
            &mut rng,
        );
        assert_eq!(
            spawned, 3,
            "should clamp to remaining free capacity, not error"
        );
    }

    #[test]
    fn try_spawn_burst_spawns_zero_when_pool_full() {
        let mut app = App::new();
        let mut rng = test_rng();
        let spawned = try_spawn_burst(
            &mut app.world_mut().commands(),
            VFX_POOL_CAPACITY,
            DVec3::ZERO,
            5,
            0.5,
            [1.0, 1.0, 1.0, 1.0],
            &mut rng,
        );
        assert_eq!(spawned, 0);
    }

    #[test]
    fn tick_vfx_applies_gravity_and_integrates_position() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Time::<Fixed>::from_hz(60.0));
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.add_systems(FixedUpdate, tick_vfx);
        let entity = app
            .world_mut()
            .spawn(VfxParticle {
                position: DVec3::ZERO,
                velocity: DVec3::ZERO,
                remaining_lifetime_seconds: 10.0,
                color: [1.0, 1.0, 1.0, 1.0],
                size: 1.0,
            })
            .id();
        app.update();
        app.update();
        let particle = app.world().get::<VfxParticle>(entity).unwrap();
        assert!(
            particle.velocity.y < 0.0,
            "gravity should pull velocity downward"
        );
        assert!(
            particle.position.y < 0.0,
            "position should integrate downward"
        );
    }

    #[test]
    fn tick_vfx_despawns_particle_at_end_of_lifetime() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Time::<Fixed>::from_hz(60.0));
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.add_systems(FixedUpdate, tick_vfx);
        let entity = app
            .world_mut()
            .spawn(VfxParticle {
                position: DVec3::ZERO,
                velocity: DVec3::ZERO,
                remaining_lifetime_seconds: 0.001,
                color: [1.0, 1.0, 1.0, 1.0],
                size: 1.0,
            })
            .id();
        app.update();
        app.update();
        assert!(app.world().get::<VfxParticle>(entity).is_none());
    }
}
