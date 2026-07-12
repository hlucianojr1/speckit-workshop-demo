//! FR-013 (US4): the distance-constraint solver moves connected bodies toward their
//! target separation, and never moves the anchor body.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use orbital_arena_rs::body::{Anchor, Body, Prev};
use orbital_arena_rs::config::{RunMode, SimConfig, ROPE_ANCHOR_X, ROPE_ANCHOR_Y};
use orbital_arena_rs::constraint::{ConstraintLink, PhysicsPlugin};
use orbital_arena_rs::rng::RngPlugin;

fn build_app(seed: u64) -> App {
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
    app.add_plugins(PhysicsPlugin);
    app
}

#[test]
fn separation_converges_toward_rest_length_when_displaced() {
    let mut app = build_app(42);
    app.update(); // Startup spawn + first fixed tick.

    let orbiter = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, Without<Anchor>>()
            .iter(world)
            .next()
            .expect("at least one orbiting body must exist")
    };

    let rest_length = {
        let world = app.world_mut();
        let mut query = world.query::<&ConstraintLink>();
        query
            .iter(world)
            .find(|link| link.a == orbiter || link.b == orbiter)
            .expect("orbiter has a constraint link")
            .rest_length
    };

    // Displace the orbiter far beyond its rest length with zero injected velocity, so the
    // constraint solver alone is responsible for pulling it back in. `Prev` must move with
    // `position` (sandbox-scenes.spec.md §7 release_held_node semantics) — otherwise verlet
    // integration reads the stale pre-teleport `Prev` and injects a spurious velocity.
    let distance_before = 1_000.0_f64;
    {
        let world = app.world_mut();
        let mut body = world
            .get_mut::<Body>(orbiter)
            .expect("orbiter has a Body component");
        body.position = DVec2::new(distance_before, 0.0);
        body.velocity = DVec2::ZERO;
        let mut prev = world
            .get_mut::<Prev>(orbiter)
            .expect("orbiter has a Prev component");
        prev.0 = DVec2::new(distance_before, 0.0);
    }

    app.update();

    let distance_after = {
        let world = app.world_mut();
        world
            .get::<Body>(orbiter)
            .expect("orbiter has a Body component")
            .position
            .length()
    };

    assert!(
        (distance_after - rest_length).abs() < (distance_before - rest_length).abs(),
        "expected separation to move closer to rest_length: before={distance_before}, \
         after={distance_after}, rest_length={rest_length}"
    );
}

#[test]
fn anchor_position_never_changes_across_any_number_of_constraints() {
    let mut app = build_app(42);
    for _ in 0..30 {
        app.update();
    }

    let world = app.world_mut();
    let anchor = world
        .query_filtered::<Entity, With<Anchor>>()
        .iter(world)
        .next()
        .expect("anchor entity must exist");
    let body = world
        .get::<Body>(anchor)
        .expect("anchor has a Body component");
    // The rope's anchor is node 0 at (ROPE_ANCHOR_X, ROPE_ANCHOR_Y) — not the origin
    // (sandbox-scenes.spec.md §4.1) — but it must never move regardless.
    assert_eq!(body.position, DVec2::new(ROPE_ANCHOR_X, ROPE_ANCHOR_Y));
    assert_eq!(body.velocity, DVec2::ZERO);
}
