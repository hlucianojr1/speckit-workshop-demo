//! Sandbox free-particle physics (specs/transform/sandbox-free-particles.spec.md),
//! default (rope-like) scene variant only per the spec's §6 scope reduction: light
//! gravity + bounce off a rectangular boundary + a VFX collision spark on every bounce.
//! Only wired into the `constraint` scene (§4.2c) — the `arena` scene has its own,
//! rule-isolated particle field (`game::FieldParticle`) and this module is never added
//! there.

use bevy::math::DVec2;
use bevy::prelude::*;
use rand::Rng;

use crate::rng::{insert_rng, DeterministicRng};
use crate::vfx::try_spawn_burst;

/// Initial free-particle population (reference: 32).
pub const FREE_PARTICLE_COUNT: usize = 32;
/// Rectangular world bounds free particles bounce within.
pub const HALF_WIDTH: f64 = 260.0;
pub const HALF_HEIGHT: f64 = 190.0;
/// Light gravity — weaker than any rigid-body gravity, "for visual interest" per the
/// reference comment.
const GRAVITY_Y: f64 = -40.0;
/// Velocity damping applied on each bounce (reference: 0.85).
const BOUNCE_DAMPING: f64 = 0.85;
/// Spark burst size per bounce (reference `kVfxSparkCount`: 6).
const SPARK_COUNT: usize = 6;
const SPARK_LIFETIME_SECONDS: f64 = 0.6;
const SPARK_COLOR: [f32; 4] = [1.0, 0.75, 0.35, 1.0];

#[derive(Component, Debug, Clone, Copy)]
pub struct FreeParticle {
    pub position: DVec2,
    pub velocity: DVec2,
    pub radius: f64,
}

pub struct FreeParticlePlugin;

impl Plugin for FreeParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_free_particles.after(insert_rng))
            .add_systems(FixedUpdate, step_free_particles);
    }
}

fn spawn_free_particles(mut commands: Commands, mut rng: ResMut<DeterministicRng>) {
    for _ in 0..FREE_PARTICLE_COUNT {
        let x = rng.0.random_range(-HALF_WIDTH..HALF_WIDTH);
        let y = rng.0.random_range(-HALF_HEIGHT..HALF_HEIGHT);
        let angle = rng.0.random_range(0.0..std::f64::consts::TAU);
        let speed = rng.0.random_range(20.0..80.0);
        commands.spawn(FreeParticle {
            position: DVec2::new(x, y),
            velocity: DVec2::new(angle.cos() * speed, angle.sin() * speed),
            radius: rng.0.random_range(2.5..5.5),
        });
    }
}

/// Integrates every free particle by one fixed tick (spec §3 default variant): light
/// gravity, then bounce off the rectangular boundary with damping, spawning a VFX
/// collision spark at the contact point on every bounce (spec §5 / vfx-particle-system
/// spec's usage pattern).
fn step_free_particles(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut particles: Query<&mut FreeParticle>,
    vfx_particles: Query<(), With<crate::vfx::VfxParticle>>,
    mut rng: ResMut<DeterministicRng>,
) {
    let dt = time.delta_secs_f64();
    let mut live_vfx = vfx_particles.iter().count();

    for mut particle in &mut particles {
        particle.velocity.y += GRAVITY_Y * dt;
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
                SPARK_LIFETIME_SECONDS,
                SPARK_COLOR,
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
