//! `EngineRng`-backed Bevy resource for scenes requiring C++ digest parity
//! (sandbox-scenes.spec.md §4.1 rope geometry + free particles). Distinct from
//! `rng::DeterministicRng` (StdRng), which remains the crate's rng for non-parity
//! paths (VFX, the `arena` scene). Only wired into the `rope`/`constraint` scene.

use bevy::prelude::*;

use crate::config::SimConfig;
use crate::rng_mt::EngineRng;

/// Wraps the MT19937-based `EngineRng`, seeded exactly like the C++ reference's
/// `engine_demo::sim::rng` (XOR-folded 64-bit seed). The scene's single stream — every
/// rng-consuming spawn draws from this resource, in the spec's exact draw order.
#[derive(Resource)]
pub struct EngineRngRes(pub EngineRng);

pub struct EngineRngPlugin;

impl Plugin for EngineRngPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, insert_engine_rng);
    }
}

/// Public so scene-spawn `Startup` systems can order themselves `.after(insert_engine_rng)`.
pub fn insert_engine_rng(mut commands: Commands, config: Res<SimConfig>) {
    commands.insert_resource(EngineRngRes(EngineRng::new(config.seed)));
}
