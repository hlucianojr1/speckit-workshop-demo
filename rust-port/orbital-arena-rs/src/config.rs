//! Simulation configuration: `SimConfig` resource + `RunMode` (data-model.md §Resources).

use std::path::PathBuf;

use bevy::prelude::*;

/// Number of orbiting bodies spawned around the anchor (research.md's "Body count" note:
/// fixed at 5 as an implementation default, not a user-facing requirement).
pub const BODY_COUNT: usize = 5;

/// Target distance between the anchor and each orbiting body (world units).
pub const ORBIT_RADIUS: f64 = 150.0;

/// Radius of the arena boundary circle (world units); must exceed `ORBIT_RADIUS` with
/// margin so orbiting bodies stay visually inside it.
pub const ARENA_RADIUS: f64 = 220.0;

/// Gizmo circle radius used to draw each body (world units).
pub const BODY_VISUAL_RADIUS: f64 = 10.0;

/// Launch-time run mode: normal windowed observation, or automated screenshot capture.
#[derive(Debug, Clone, PartialEq)]
pub enum RunMode {
    Windowed,
    Screenshot {
        warmup_frames: u32,
        output_path: PathBuf,
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
