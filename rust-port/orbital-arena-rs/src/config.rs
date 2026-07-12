//! Simulation configuration: `SimConfig` resource + `RunMode` (data-model.md §Resources).

use std::path::PathBuf;

use bevy::prelude::*;

/// Rope scene geometry (sandbox-scenes.spec.md §4.1) — the reference chain, preserved
/// byte-for-byte: node count, rest length, anchor position, and tilt angle.
pub const ROPE_NODES: usize = 24;
pub const ROPE_REST_LENGTH: f64 = 0.30;
pub const ROPE_ANCHOR_X: f64 = 0.0;
pub const ROPE_ANCHOR_Y: f64 = 3.0;
/// ~35 degrees in radians (reference comment) — fixed, not derived.
pub const ROPE_TILT_ANGLE: f64 = 0.61;

/// Free-particle population for the rope scene (reference: 32).
pub const ROPE_FREE_PARTICLE_COUNT: usize = 32;

/// Fixed simulation step (sandbox-scenes.spec.md §3: 1/60 s). Used as a literal `f64`
/// everywhere physics integrates time, rather than `Time<Fixed>::delta_secs_f64()` —
/// Bevy's `Time<Fixed>` quantizes its period to whole nanoseconds internally, which is
/// *not* bit-identical to `1.0 / 60.0`'s IEEE-754 double and silently breaks the
/// cross-language digest parity (sandbox-scenes.spec.md §8).
pub const FIXED_STEP_SECONDS: f64 = 1.0 / 60.0;

/// Verlet gravity magnitude applied to dynamic bodies (§5 step 1: `-9.81*dt^2` on y).
pub const GRAVITY_Y: f64 = 9.81;
/// Constraint-projection iterations per fixed step (§5 step 3).
pub const SOLVER_ITERATIONS: u32 = 8;

/// Radius of the arena boundary circle drawn by `VisualsPlugin` (world units; a rough
/// visual guide only — exact camera/scale work is deferred to the visual-identity
/// closure, sandbox-visual-identity.spec.md / tasks.md US7).
pub const ARENA_RADIUS: f64 = 3.5;

/// Gizmo circle radius used to draw each body (world units; see `ARENA_RADIUS` note).
pub const BODY_VISUAL_RADIUS: f64 = 0.05;

/// Launch-time run mode: normal windowed observation, automated screenshot capture, or
/// the windowless CSV/digest trace (sandbox-headless.spec.md, T045).
#[derive(Debug, Clone, PartialEq)]
pub enum RunMode {
    Windowed,
    Screenshot {
        warmup_frames: u32,
        output_path: PathBuf,
    },
    Headless {
        frames: u32,
    },
}

/// Simulation-wide launch configuration (FR-007, FR-009, FR-010).
#[derive(Resource, Debug, Clone, PartialEq)]
pub struct SimConfig {
    pub seed: u64,
    pub run_mode: RunMode,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            run_mode: RunMode::Windowed,
        }
    }
}
