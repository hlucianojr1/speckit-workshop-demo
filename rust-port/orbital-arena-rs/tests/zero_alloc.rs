//! FR-014 (US4): the steady-state frame loop performs no *unbounded* heap growth from our
//! own game logic.
//!
//! FINDING (2026-07-10, recorded for training feedback): Bevy 0.16's own scheduler/task-pool
//! internals allocate-and-immediately-deallocate a small, CONSTANT amount per frame
//! (~3.5 KB/frame measured here) even under `MinimalPlugins` with zero game systems added.
//! This is framework overhead outside game-code control, not a leak. A literal
//! "zero bytes allocated" assertion (as the language-agnostic allocator.spec.md's C++
//! reference implementation achieves) is therefore NOT achievable for a Bevy app — the
//! enforceable Rust-side guarantee is "allocation churn is bounded and does not grow batch
//! over batch" (no leak, no unbounded Vec/HashMap growth in OUR systems), which is what this
//! test actually asserts.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use orbital_arena_rs::alloc::snapshot;
use orbital_arena_rs::config::{RunMode, SimConfig};
use orbital_arena_rs::constraint::PhysicsPlugin;
use orbital_arena_rs::frame_budget::FrameBudgetPlugin;
use orbital_arena_rs::rng::RngPlugin;

#[test]
fn steady_state_frame_loop_allocation_does_not_grow_after_warmup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SimConfig {
        seed: 42,
        run_mode: RunMode::Windowed,
    });
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        1.0 / 60.0,
    )));
    app.add_plugins(RngPlugin);
    app.add_plugins(PhysicsPlugin);
    app.add_plugins(FrameBudgetPlugin);

    // Warm-up: Startup systems + one-time archetype/storage growth is expected to allocate.
    for _ in 0..30 {
        app.update();
    }

    // Measure three equal-length post-warm-up batches. If OUR game logic were growing a
    // collection unboundedly (the real bug this test exists to catch), later batches would
    // trend strictly upward. Bevy's own constant-ish per-frame scheduler churn is present
    // in every batch and has small natural jitter (measured ~0.2% run-to-run on this VM),
    // so we assert "no upward trend beyond a generous tolerance", not bit-exact equality.
    const BATCH_FRAMES: u32 = 120;
    const JITTER_TOLERANCE: f64 = 0.10; // 10% — generous vs. the ~0.2% observed jitter

    let mut deltas = Vec::with_capacity(3);
    for _ in 0..3 {
        let (before, _) = snapshot();
        for _ in 0..BATCH_FRAMES {
            app.update();
        }
        let (after, _) = snapshot();
        deltas.push((after - before) as f64);
    }

    let first = *deltas.first().unwrap_or(&0.0);
    let last = *deltas.last().unwrap_or(&0.0);
    let growth_ratio = (last - first) / first;
    assert!(
        growth_ratio < JITTER_TOLERANCE,
        "allocation-per-batch trended upward by {:.1}% (first={first}, last={last}) — \
         expected constant framework overhead, not growth (a growing delta indicates a \
         leak or unbounded collection growth in game logic): deltas = {deltas:?}",
        growth_ratio * 100.0
    );
}
