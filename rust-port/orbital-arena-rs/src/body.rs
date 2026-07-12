//! `Body`/`Anchor`/`SpawnIndex` components (data-model.md §Components).

use bevy::math::DVec2;
use bevy::prelude::*;

/// A simulated body: position/velocity are `f64` (Article VI) — `f32` conversion happens
/// only at render time (`visuals.rs`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Body {
    pub position: DVec2,
    pub velocity: DVec2,
    /// `0.0` marks an immovable body (the anchor); the constraint solver never applies a
    /// position correction to a body whose `inverse_mass` is `0.0`.
    pub inverse_mass: f64,
}

/// Zero-sized marker for the single central, immovable body.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Anchor;

/// Verlet previous-frame position (sandbox-scenes.spec.md §5 step 1: `prev`). Anchors
/// carry this component too (initialized to their fixed position) but their `Prev` is
/// never read — the integration system skips anchors entirely.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Prev(pub DVec2);

/// Stable spawn-order index, independent of `Entity` allocation details, used to order
/// `SimulationSnapshot::bodies` deterministically (data-model.md's robustness note).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SpawnIndex(pub u32);
