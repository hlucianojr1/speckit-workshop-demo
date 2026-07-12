//! T047 (US6): the `rope` scene's exact geometry per sandbox-scenes.spec.md §4.1 —
//! 24 formula-placed bodies (node 0 anchored), 23 rest-0.30 constraints connecting
//! consecutive nodes, and 32 free particles each consuming exactly 5 `EngineRng` draws
//! in x/y/vx/vy/radius order.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

use std::time::Duration;

use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use orbital_arena_rs::body::{Anchor, Body, SpawnIndex};
use orbital_arena_rs::config::{
    RunMode, SimConfig, ROPE_ANCHOR_X, ROPE_ANCHOR_Y, ROPE_FREE_PARTICLE_COUNT, ROPE_NODES,
    ROPE_REST_LENGTH, ROPE_TILT_ANGLE,
};
use orbital_arena_rs::constraint::{ConstraintLink, PhysicsPlugin};
use orbital_arena_rs::engine_rng::EngineRngPlugin;
use orbital_arena_rs::free_particles::{FreeParticle, FreeParticlePlugin};
use orbital_arena_rs::rng::RngPlugin;
use orbital_arena_rs::rng_mt::EngineRng;

/// Builds the rope scene without stepping it, so bodies/particles are exactly the
/// `Startup`-spawned state (zero physics/verlet has run yet).
fn build_unstepped(seed: u64) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SimConfig {
        seed,
        run_mode: RunMode::Windowed,
    });
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    app.add_plugins(RngPlugin);
    app.add_plugins(EngineRngPlugin);
    app.add_plugins(PhysicsPlugin);
    app.add_plugins(FreeParticlePlugin);
    // Startup only — no `app.update()` — so `FixedUpdate` systems have not run.
    app.update();
    app
}

#[test]
fn twenty_four_bodies_at_exact_formula_positions_with_node_zero_anchored() {
    let mut app = build_unstepped(42);
    let world = app.world_mut();

    let mut bodies: Vec<(u32, DVec2, bool)> = world
        .query::<(&SpawnIndex, &Body, Has<Anchor>)>()
        .iter(world)
        .map(|(idx, body, is_anchor)| (idx.0, body.position, is_anchor))
        .collect();
    bodies.sort_by_key(|(idx, _, _)| *idx);

    assert_eq!(bodies.len(), ROPE_NODES);

    let dx = ROPE_TILT_ANGLE.sin() * ROPE_REST_LENGTH;
    let dy = -ROPE_TILT_ANGLE.cos() * ROPE_REST_LENGTH;
    for (i, (idx, position, is_anchor)) in bodies.iter().enumerate() {
        assert_eq!(*idx, i as u32);
        let expected = DVec2::new(ROPE_ANCHOR_X + dx * i as f64, ROPE_ANCHOR_Y + dy * i as f64);
        assert_eq!(*position, expected, "node {i} position mismatch");
        assert_eq!(*is_anchor, i == 0, "node {i} anchor flag mismatch");
    }
}

#[test]
fn twenty_three_constraints_connect_consecutive_nodes_at_rest_length() {
    let mut app = build_unstepped(42);
    let world = app.world_mut();

    let mut indices_by_entity: Vec<(Entity, u32)> = world
        .query_filtered::<(Entity, &SpawnIndex), With<Body>>()
        .iter(world)
        .map(|(entity, idx)| (entity, idx.0))
        .collect();
    indices_by_entity.sort_by_key(|(entity, _)| *entity);
    let index_of = |e: Entity| -> u32 {
        indices_by_entity
            .iter()
            .find(|(entity, _)| *entity == e)
            .expect("every constrained entity has a SpawnIndex")
            .1
    };

    let mut links: Vec<(u32, u32, f64)> = world
        .query::<&ConstraintLink>()
        .iter(world)
        .map(|link| (index_of(link.a), index_of(link.b), link.rest_length))
        .collect();
    links.sort_by_key(|(a, b, _)| (*a, *b));

    assert_eq!(links.len(), ROPE_NODES - 1);
    for (i, (a, b, rest_length)) in links.iter().enumerate() {
        assert_eq!(*a, i as u32, "link {i} should start at node {i}");
        assert_eq!(*b, (i + 1) as u32, "link {i} should end at node {}", i + 1);
        assert_eq!(*rest_length, ROPE_REST_LENGTH);
    }
}

#[test]
fn thirty_two_free_particles_each_consume_exactly_five_draws_in_spec_order() {
    let mut app = build_unstepped(42);
    let world = app.world_mut();

    let mut particles: Vec<(u32, FreeParticle)> = world
        .query::<(&SpawnIndex, &FreeParticle)>()
        .iter(world)
        .map(|(idx, particle)| (idx.0, *particle))
        .collect();
    particles.sort_by_key(|(idx, _)| *idx);

    assert_eq!(particles.len(), ROPE_FREE_PARTICLE_COUNT);

    // Re-derive the expected draws directly from `EngineRng` (independent of any ECS
    // plumbing) to confirm the spawn system consumes exactly 5 draws per particle in
    // x, y, vx, vy, radius order — zero more, zero fewer.
    let mut rng = EngineRng::new(42);
    for (i, (idx, particle)) in particles.iter().enumerate() {
        assert_eq!(*idx, i as u32);
        let x = rng.next_double_unit() * 6.0 - 3.0;
        let y = rng.next_double_unit() * 2.0 - 1.0;
        let vx = rng.next_double_unit() * 2.0 - 1.0;
        let vy = rng.next_double_unit() * 2.0 - 1.0;
        let radius = 0.04 + rng.next_double_unit() * 0.04;
        assert_eq!(particle.position, DVec2::new(x, y), "particle {i} position");
        assert_eq!(
            particle.velocity,
            DVec2::new(vx, vy),
            "particle {i} velocity"
        );
        assert_eq!(particle.radius, radius, "particle {i} radius");
    }
}
