//! Rust/Bevy visual port of the Orbital Arena simulation (specs/004-rust-bevy-visual-port).
//!
//! Behavior is a language-agnostic-spec-driven port of `specs/transform/*.spec.md`, bound
//! by `specs/transform/rust-constitution.md`.
#![deny(unsafe_code)]

pub mod alloc;
pub mod background;
pub mod body;
pub mod color_ramp;
pub mod config;
pub mod constraint;
pub mod cursor;
pub mod digest;
pub mod engine_rng;
pub mod frame_budget;
pub mod free_particles;
pub mod headless;
pub mod orbital;
pub mod rng;
pub mod rng_mt;
pub mod screenshot;
pub mod vfx;
pub mod visuals;

use bevy::prelude::*;

use crate::body::{Body, SpawnIndex};
use crate::free_particles::FreeParticle;

/// The one process-wide global allocator (Article IV / FR-014): every binary and test
/// linking this crate shares this single instrumented allocator.
#[global_allocator]
static GLOBAL_ALLOCATOR: alloc::CountingAllocator = alloc::CountingAllocator;

/// Recorded positions/velocities of all bodies (and, for scenes that have them, free
/// particles) at a given tick, used by the determinism tests to compare two runs
/// (spec.md Key Entities, data-model.md §Simulation State Snapshot; extended for the
/// rope scene where body geometry is a pure formula — free particles are the
/// seed-dependent state, sandbox-scenes.spec.md §8). Test-only type — not a runtime
/// resource used by the app itself.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationSnapshot {
    pub seed: u64,
    pub tick: u32,
    pub bodies: Vec<(f64, f64, f64, f64)>,
    pub particles: Vec<(f64, f64, f64, f64)>,
}

/// Builds a `SimulationSnapshot` ordered by each entity's stable `SpawnIndex`, not by
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

    let mut particle_query = world.query::<(&SpawnIndex, &FreeParticle)>();
    let mut indexed_particles: Vec<(u32, (f64, f64, f64, f64))> = particle_query
        .iter(world)
        .map(|(index, particle)| {
            (
                index.0,
                (
                    particle.position.x,
                    particle.position.y,
                    particle.velocity.x,
                    particle.velocity.y,
                ),
            )
        })
        .collect();
    indexed_particles.sort_unstable_by_key(|(index, _)| *index);

    SimulationSnapshot {
        seed,
        tick,
        bodies: indexed.into_iter().map(|(_, values)| values).collect(),
        particles: indexed_particles
            .into_iter()
            .map(|(_, values)| values)
            .collect(),
    }
}
