//! FR-012/FR-015 (US3): same seed + tick count ⇒ identical snapshot; different seed ⇒
//! differing snapshot; omitting `--seed` ⇒ default 42 (FR-007).
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use orbital_arena_rs::capture_snapshot;
use orbital_arena_rs::config::{RunMode, SimConfig};
use orbital_arena_rs::constraint::PhysicsPlugin;
use orbital_arena_rs::rng::RngPlugin;

fn build_headless_app(seed: u64) -> App {
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

fn run_ticks(app: &mut App, ticks: u32) {
    for _ in 0..ticks {
        app.update();
    }
}

#[test]
fn same_seed_same_tick_count_yields_identical_snapshot() {
    let mut app_a = build_headless_app(42);
    let mut app_b = build_headless_app(42);
    run_ticks(&mut app_a, 120);
    run_ticks(&mut app_b, 120);

    let snapshot_a = capture_snapshot(&mut app_a, 42, 120);
    let snapshot_b = capture_snapshot(&mut app_b, 42, 120);
    assert_eq!(snapshot_a, snapshot_b);
}

#[test]
fn different_seed_yields_differing_snapshot() {
    let mut app_baseline = build_headless_app(42);
    let mut app_other = build_headless_app(7);
    run_ticks(&mut app_baseline, 120);
    run_ticks(&mut app_other, 120);

    let snapshot_baseline = capture_snapshot(&mut app_baseline, 42, 120);
    let snapshot_other = capture_snapshot(&mut app_other, 7, 120);
    assert_ne!(snapshot_baseline.bodies, snapshot_other.bodies);
}

#[test]
fn default_sim_config_seed_is_42() {
    let config = SimConfig::default();
    assert_eq!(config.seed, 42);
    assert_eq!(config.run_mode, RunMode::Windowed);
}
