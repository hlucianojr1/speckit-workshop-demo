//! Rust/Bevy visual port of the Orbital Arena simulation (specs/004-rust-bevy-visual-port).
//!
//! Behavior is a language-agnostic-spec-driven port of `specs/transform/*.spec.md`, bound
//! by `specs/transform/rust-constitution.md`.
#![deny(unsafe_code)]

pub mod alloc;
pub mod body;
pub mod config;
pub mod constraint;
pub mod cursor;
pub mod frame_budget;
pub mod free_particles;
pub mod game;
pub mod rng;
pub mod screenshot;
pub mod vfx;
pub mod visuals;

use bevy::prelude::*;

use crate::body::{Body, SpawnIndex};

/// The one process-wide global allocator (Article IV / FR-014): every binary and test
/// linking this crate shares this single instrumented allocator.
#[global_allocator]
static GLOBAL_ALLOCATOR: alloc::CountingAllocator = alloc::CountingAllocator;

/// Recorded positions/velocities of all bodies at a given tick, used by the determinism
/// tests to compare two runs (spec.md Key Entities, data-model.md §Simulation State
/// Snapshot). Test-only type — not a runtime resource used by the app itself.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationSnapshot {
    pub seed: u64,
    pub tick: u32,
    pub bodies: Vec<(f64, f64, f64, f64)>,
}

/// Builds a `SimulationSnapshot` ordered by each body's stable `SpawnIndex`, not by
/// `Entity` (data-model.md's robustness note; research.md R5).
pub fn capture_snapshot(app: &mut App, seed: u64, tick: u32) -> SimulationSnapshot {
    let world = app.world_mut();
    let mut query = world.query::<(&SpawnIndex, &Body)>();
    let mut indexed: Vec<(u32, (f64, f64, f64, f64))> = query
        .iter(world)
        .map(|(index, body)| {
            (
                index.0,
                (
                    body.position.x,
                    body.position.y,
                    body.velocity.x,
                    body.velocity.y,
                ),
            )
        })
        .collect();
    indexed.sort_unstable_by_key(|(index, _)| *index);

    SimulationSnapshot {
        seed,
        tick,
        bodies: indexed.into_iter().map(|(_, values)| values).collect(),
    }
}
