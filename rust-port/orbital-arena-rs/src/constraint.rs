//! `PhysicsPlugin`: spawns the 24-node rope chain + constraint links at `Startup`, and
//! runs true verlet integration + the sorted-order distance-constraint solver in
//! `FixedUpdate` (sandbox-scenes.spec.md §4.1/§5, physics-constraint.spec.md §4.1,
//! research.md R6). Rewritten from the earlier hub-star/velocity-integration demo to
//! match the C++ reference's rope scene exactly (Full-Fidelity Closure backlog, T048).

use bevy::math::DVec2;
use bevy::prelude::*;

use crate::body::{Anchor, Body, Prev, SpawnIndex};
use crate::config::{
    FIXED_STEP_SECONDS, GRAVITY_Y, ROPE_ANCHOR_X, ROPE_ANCHOR_Y, ROPE_NODES, ROPE_REST_LENGTH,
    ROPE_TILT_ANGLE, SOLVER_ITERATIONS,
};

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
            .add_systems(Startup, spawn_bodies)
            .add_systems(FixedUpdate, (integrate_bodies, solve_constraints).chain());
    }
}

/// Spawns the 24-node rope chain (sandbox-scenes.spec.md §4.1): node *i* at
/// `x = sin(tilt)*rest*i`, `y = anchor_y - cos(tilt)*rest*i`; node 0 is the only anchor.
/// Pure formula — consumes ZERO rng draws, preserving the reference's rng draw order
/// (free particles are the first draws from the scene's rng stream).
fn spawn_bodies(mut commands: Commands) {
    let dx = ROPE_TILT_ANGLE.sin() * ROPE_REST_LENGTH;
    let dy = -ROPE_TILT_ANGLE.cos() * ROPE_REST_LENGTH;

    let mut nodes = Vec::with_capacity(ROPE_NODES);
    for i in 0..ROPE_NODES {
        let position = DVec2::new(ROPE_ANCHOR_X + dx * i as f64, ROPE_ANCHOR_Y + dy * i as f64);
        let is_anchor = i == 0;
        let mut entity = commands.spawn((
            Body {
                position,
                velocity: DVec2::ZERO,
                inverse_mass: if is_anchor { 0.0 } else { 1.0 },
            },
            Prev(position),
            SpawnIndex(i as u32),
        ));
        if is_anchor {
            entity.insert(Anchor);
        }
        nodes.push(entity.id());
    }

    for pair in nodes.windows(2) {
        let [a, b] = pair else { continue };
        commands.spawn(ConstraintLink {
            a: *a,
            b: *b,
            rest_length: ROPE_REST_LENGTH,
        });
    }

    commands.insert_resource(SortedLinks(Vec::with_capacity(ROPE_NODES - 1)));
}

/// True verlet integration (sandbox-scenes.spec.md §5 step 1): `next = cur + (cur -
/// prev) - g*dt^2` on y; `prev <- cur`. Anchors never move. This replaces the earlier
/// velocity-based Euler step — the rope has no persistent `Body.velocity` state, matching
/// the reference (`Body.velocity` stays `DVec2::ZERO` for rope nodes; it exists on the
/// shared component only because other scenes' bodies use it).
fn integrate_bodies(mut bodies: Query<(&mut Body, &mut Prev), Without<Anchor>>) {
    let dt = FIXED_STEP_SECONDS; // independent of Time<Fixed> (see config.rs doc comment)
    for (mut body, mut prev) in &mut bodies {
        let cur = body.position;
        let next = cur + (cur - prev.0) - DVec2::new(0.0, GRAVITY_Y * dt * dt);
        body.position = next;
        prev.0 = cur;
    }
}

/// Sorted-order distance-constraint solver (physics-constraint.spec.md §4.1, research.md
/// R6): constraints are gathered and sorted by the canonical `(min(a,b), max(a,b))` key
/// via `Entity`'s built-in `Ord` every tick, then solved in that fixed order so the result
/// never depends on ECS query/storage iteration order. Runs `SOLVER_ITERATIONS` (8)
/// projection passes per fixed step, matching the reference `constraint_solver::solve`.
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

    for _ in 0..SOLVER_ITERATIONS {
        for &(a, b, rest_length) in &sorted.0 {
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
}
