//! `PhysicsPlugin`: spawns the anchor/orbiting bodies + constraint links at `Startup`, and
//! runs the sorted-order distance-constraint solver in `FixedUpdate`
//! (contracts/plugins.md `PhysicsPlugin`, data-model.md §Components `ConstraintLink`,
//! research.md R6).

use std::f64::consts::TAU;

use bevy::math::DVec2;
use bevy::prelude::*;
use rand::Rng;

use crate::body::{Anchor, Body, SpawnIndex};
use crate::config::{BODY_COUNT, ORBIT_RADIUS};
use crate::rng::{insert_rng, DeterministicRng};

/// A distance constraint connecting exactly two bodies (data-model.md §Components).
#[derive(Component, Debug, Clone, Copy)]
pub struct ConstraintLink {
    pub a: Entity,
    pub b: Entity,
    pub rest_length: f64,
}

/// Pre-sized per-solve sort buffer (R6): reserved once at `Startup`, `clear()`+repopulated
/// every `FixedUpdate` call — capacity is never exceeded since the link count is fixed
/// after startup (Article IV zero-allocation design).
#[derive(Resource, Debug, Default)]
pub struct SortedLinks(pub Vec<(Entity, Entity, f64)>);

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SortedLinks::default())
            .add_systems(Startup, spawn_bodies.after(insert_rng))
            .add_systems(FixedUpdate, (integrate_bodies, solve_constraints).chain());
    }
}

/// Spawns one immovable anchor at the origin plus `BODY_COUNT` orbiting bodies arranged in
/// a hub topology (every orbiter linked directly to the anchor), with RNG-jittered initial
/// angle/radius (FR-004, FR-007).
fn spawn_bodies(mut commands: Commands, mut rng: ResMut<DeterministicRng>) {
    let anchor = commands
        .spawn((
            Body {
                position: DVec2::ZERO,
                velocity: DVec2::ZERO,
                inverse_mass: 0.0,
            },
            Anchor,
            SpawnIndex(0),
        ))
        .id();

    let mut orbiters = Vec::with_capacity(BODY_COUNT);
    for i in 0..BODY_COUNT {
        let base_angle = (i as f64 / BODY_COUNT as f64) * TAU;
        let angle = base_angle + rng.0.random_range(-0.2..0.2);
        let radius = ORBIT_RADIUS + rng.0.random_range(-10.0..10.0);
        let position = DVec2::new(angle.cos() * radius, angle.sin() * radius);

        // Tangential velocity so a rigid distance constraint alone produces
        // orbit/pendulum-like circular motion around the anchor with no gravity system.
        let tangent = DVec2::new(-angle.sin(), angle.cos());
        let speed = 90.0 * (ORBIT_RADIUS / radius).sqrt();
        let velocity = tangent * speed;

        let entity = commands
            .spawn((
                Body {
                    position,
                    velocity,
                    inverse_mass: 1.0,
                },
                SpawnIndex((i + 1) as u32),
            ))
            .id();
        orbiters.push(entity);
    }

    for &orbiter in &orbiters {
        commands.spawn(ConstraintLink {
            a: anchor,
            b: orbiter,
            rest_length: ORBIT_RADIUS,
        });
    }

    commands.insert_resource(SortedLinks(Vec::with_capacity(orbiters.len())));
}

/// Semi-implicit Euler position integration (Article VI: `f64` accumulators/state).
fn integrate_bodies(time: Res<Time<Fixed>>, mut bodies: Query<&mut Body>) {
    let dt = time.delta_secs_f64();
    for mut body in &mut bodies {
        if body.inverse_mass > 0.0 {
            let velocity = body.velocity;
            body.position += velocity * dt;
        }
    }
}

/// Sorted-order distance-constraint solver (physics-constraint.spec.md §4.1, research.md
/// R6): constraints are gathered and sorted by the canonical `(min(a,b), max(a,b))` key
/// via `Entity`'s built-in `Ord` every tick, then solved in that fixed order so the result
/// never depends on ECS query/storage iteration order.
fn solve_constraints(
    mut sorted: ResMut<SortedLinks>,
    links: Query<&ConstraintLink>,
    mut bodies: Query<&mut Body>,
) {
    sorted.0.clear();
    for link in &links {
        sorted.0.push((link.a, link.b, link.rest_length));
    }
    sorted
        .0
        .sort_unstable_by_key(|&(a, b, _)| if a <= b { (a, b) } else { (b, a) });

    for &(a, b, rest_length) in sorted.0.iter() {
        let Ok([mut body_a, mut body_b]) = bodies.get_many_mut([a, b]) else {
            continue;
        };
        let delta = body_b.position - body_a.position;
        let distance = delta.length();
        if distance < 1e-9 {
            continue;
        }
        let inverse_mass_sum = body_a.inverse_mass + body_b.inverse_mass;
        if inverse_mass_sum <= 0.0 {
            continue;
        }
        let diff = (distance - rest_length) / distance;
        let correction = delta * (diff / inverse_mass_sum);
        let inverse_mass_a = body_a.inverse_mass;
        let inverse_mass_b = body_b.inverse_mass;
        body_a.position += correction * inverse_mass_a;
        body_b.position -= correction * inverse_mass_b;
    }
}
