//! Sandbox free-particle physics (specs/transform/sandbox-free-particles.spec.md),
//! default (rope-like) scene variant only per the spec's §6 scope reduction: light
//! gravity + bounce off a rectangular boundary + a VFX collision spark on every bounce.
//! Only wired into the `rope`/`constraint` scene — the `arena` scene has its own,
//! rule-isolated particle field (`game::FieldParticle`) and this module is never added
//! there. Geometry/formulas rewritten to sandbox-scenes.spec.md §4.1/§5 exact values
//! (Full-Fidelity Closure backlog, T050) — spawn draws now come from `EngineRngRes` (the
//! reference rng), matching the C++ draw order (x, y, vx, vy, radius) exactly.

use bevy::math::DVec2;
use bevy::prelude::*;

use crate::body::SpawnIndex;
use crate::config::{FIXED_STEP_SECONDS, GRAVITY_Y, ROPE_FREE_PARTICLE_COUNT};
use crate::engine_rng::{insert_engine_rng, EngineRngRes};
use crate::rng::DeterministicRng;
use crate::vfx::{try_spawn_burst, VFX_LIFETIME_SECONDS, VFX_SPAWN_COLOR};

/// World-space bounce bounds (sandbox-scenes.spec.md §5 step 4, bounce rule): x = ±3.0,
/// y in [-2.0, 2.0].
pub const HALF_WIDTH: f64 = 3.0;
pub const HALF_HEIGHT: f64 = 2.0;
/// Velocity damping applied on each bounce (reference: 0.85).
const BOUNCE_DAMPING: f64 = 0.85;
/// Quarter-strength gravity applied to free particles (reference: `9.81 * 0.25`).
const FREE_PARTICLE_GRAVITY: f64 = GRAVITY_Y * 0.25;
/// Spark burst size per bounce (reference `kVfxSparkCount`: 6).
const SPARK_COUNT: usize = 6;

#[derive(Component, Debug, Clone, Copy)]
pub struct FreeParticle {
    pub position: DVec2,
    pub velocity: DVec2,
    pub radius: f64,
}

/// Render-only motion trail (sandbox-visual-identity.spec.md §4): the last 10 positions
/// in a fixed-size ring buffer, oldest to newest, rolled once per `FixedUpdate` step
/// (never per render frame, and never read by `digest.rs` — trails are explicitly
/// excluded from the state digest, sandbox-scenes.spec.md §10).
#[derive(Component, Debug, Clone, Copy)]
pub struct Trail {
    points: [DVec2; Trail::LEN],
    /// Index the NEXT push will overwrite.
    head: usize,
    /// Number of valid entries so far (saturates at `LEN`).
    filled: usize,
}

impl Trail {
    pub const LEN: usize = 10;

    pub fn new(initial: DVec2) -> Self {
        Self {
            points: [initial; Self::LEN],
            head: 0,
            filled: 1,
        }
    }

    fn push(&mut self, position: DVec2) {
        if let Some(slot) = self.points.get_mut(self.head) {
            *slot = position;
        }
        self.head = (self.head + 1) % Self::LEN;
        self.filled = (self.filled + 1).min(Self::LEN);
    }

    /// Yields points oldest → newest (spec §4: "oldest → newest").
    pub fn iter_oldest_to_newest(&self) -> impl Iterator<Item = DVec2> + '_ {
        let start = (self.head + Self::LEN - self.filled) % Self::LEN;
        (0..self.filled).filter_map(move |i| self.points.get((start + i) % Self::LEN).copied())
    }
}

pub struct FreeParticlePlugin;

impl Plugin for FreeParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_free_particles.after(insert_engine_rng))
            .add_systems(FixedUpdate, (step_free_particles, roll_trails).chain());
    }
}

/// Spawns the reference's 32 free particles, drawing from the scene's `EngineRngRes`
/// stream in the exact order sandbox-scenes.spec.md §4.1 mandates: x, y, vx, vy, radius
/// — 5 draws per particle, none shared with the (rng-free) rope chain construction.
fn spawn_free_particles(mut commands: Commands, mut rng: ResMut<EngineRngRes>) {
    for i in 0..ROPE_FREE_PARTICLE_COUNT {
        let x = rng.0.next_double_unit() * 6.0 - 3.0;
        let y = rng.0.next_double_unit() * 2.0 - 1.0;
        let vx = rng.0.next_double_unit() * 2.0 - 1.0;
        let vy = rng.0.next_double_unit() * 2.0 - 1.0;
        let radius = 0.04 + rng.0.next_double_unit() * 0.04;
        commands.spawn((
            FreeParticle {
                position: DVec2::new(x, y),
                velocity: DVec2::new(vx, vy),
                radius,
            },
            Trail::new(DVec2::new(x, y)),
            // Stable spawn-order index so the state digest iterates free particles in
            // list order (sandbox-scenes.spec.md §8), independent of ECS storage order.
            SpawnIndex(i as u32),
        ));
    }
}

/// Rolls each particle's trail ring buffer once per fixed step (spec §4's "roll/frame"
/// cadence, but "frame" there means simulation step, not render frame).
fn roll_trails(mut particles: Query<(&FreeParticle, &mut Trail)>) {
    for (particle, mut trail) in &mut particles {
        trail.push(particle.position);
    }
}

/// Integrates every free particle by one fixed tick (spec §5 step 4 bounce rule): light
/// gravity, then bounce off the rectangular boundary with damping, spawning a VFX
/// collision spark at the contact point on every bounce (spec §9 / vfx-particle-system
/// spec's usage pattern).
fn step_free_particles(
    mut commands: Commands,
    mut particles: Query<&mut FreeParticle>,
    vfx_particles: Query<(), With<crate::vfx::VfxParticle>>,
    mut rng: ResMut<DeterministicRng>,
) {
    // Literal 1/60s, not `Time<Fixed>::delta_secs_f64()` (see config.rs doc comment on
    // `FIXED_STEP_SECONDS`) — the latter is nanosecond-quantized and silently breaks
    // cross-language digest parity (sandbox-scenes.spec.md §8).
    let dt = FIXED_STEP_SECONDS;
    let mut live_vfx = vfx_particles.iter().count();

    for mut particle in &mut particles {
        particle.velocity.y -= FREE_PARTICLE_GRAVITY * dt;
        let velocity = particle.velocity;
        particle.position += velocity * dt;

        let mut bounced_at: Option<DVec2> = None;
        if particle.position.x < -HALF_WIDTH {
            particle.position.x = -HALF_WIDTH;
            particle.velocity.x = -particle.velocity.x * BOUNCE_DAMPING;
            bounced_at = Some(particle.position);
        } else if particle.position.x > HALF_WIDTH {
            particle.position.x = HALF_WIDTH;
            particle.velocity.x = -particle.velocity.x * BOUNCE_DAMPING;
            bounced_at = Some(particle.position);
        }
        if particle.position.y < -HALF_HEIGHT {
            particle.position.y = -HALF_HEIGHT;
            particle.velocity.y = -particle.velocity.y * BOUNCE_DAMPING;
            bounced_at = Some(particle.position);
        } else if particle.position.y > HALF_HEIGHT {
            particle.position.y = HALF_HEIGHT;
            particle.velocity.y = -particle.velocity.y * BOUNCE_DAMPING;
            bounced_at = Some(particle.position);
        }

        if let Some(contact) = bounced_at {
            let spawned = try_spawn_burst(
                &mut commands,
                live_vfx,
                contact.extend(0.0),
                SPARK_COUNT,
                VFX_LIFETIME_SECONDS,
                VFX_SPAWN_COLOR,
                &mut rng,
            );
            live_vfx += spawned;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn gravity_pulls_velocity_downward() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Time::<Fixed>::from_hz(60.0));
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.insert_resource(DeterministicRng(rand::rngs::StdRng::seed_from_u64(1)));
        app.add_systems(FixedUpdate, step_free_particles);
        let entity = app
            .world_mut()
            .spawn(FreeParticle {
                position: DVec2::ZERO,
                velocity: DVec2::ZERO,
                radius: 3.0,
            })
            .id();
        app.update();
        app.update();
        let particle = app.world().get::<FreeParticle>(entity).unwrap();
        assert!(particle.velocity.y < 0.0);
    }

    #[test]
    fn bounce_reflects_velocity_and_clamps_position() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Time::<Fixed>::from_hz(60.0));
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.insert_resource(DeterministicRng(rand::rngs::StdRng::seed_from_u64(1)));
        app.add_systems(FixedUpdate, step_free_particles);
        let entity = app
            .world_mut()
            .spawn(FreeParticle {
                position: DVec2::new(HALF_WIDTH - 1.0, 0.0),
                velocity: DVec2::new(500.0, 0.0),
                radius: 3.0,
            })
            .id();
        app.update();
        app.update();
        let particle = app.world().get::<FreeParticle>(entity).unwrap();
        assert_eq!(particle.position.x, HALF_WIDTH);
        assert!(
            particle.velocity.x < 0.0,
            "velocity should reflect on bounce"
        );
    }

    #[test]
    fn bounce_spawns_a_vfx_spark_burst() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(Time::<Fixed>::from_hz(60.0));
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        ));
        app.insert_resource(DeterministicRng(rand::rngs::StdRng::seed_from_u64(1)));
        app.add_systems(FixedUpdate, step_free_particles);
        app.world_mut().spawn(FreeParticle {
            position: DVec2::new(HALF_WIDTH - 1.0, 0.0),
            velocity: DVec2::new(500.0, 0.0),
            radius: 3.0,
        });
        app.update();
        app.update();
        let mut query = app.world_mut().query::<&crate::vfx::VfxParticle>();
        assert!(
            query.iter(app.world()).count() > 0,
            "bounce should spawn VFX sparks"
        );
    }
}
