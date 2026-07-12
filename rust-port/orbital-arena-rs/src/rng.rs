//! `RngPlugin`: seeds a single `StdRng` from `SimConfig.seed XOR 0x564658` at `Startup`
//! (contracts/plugins.md `RngPlugin`, data-model.md §Resources `DeterministicRng`).
//! The XOR fold matches sandbox-scenes.spec.md §9's "Emitter seed: scene seed XOR
//! 0x564658" — this stream feeds the VFX emitter only and never the scene's own rng
//! (rng.spec.md), preserving RNG stream isolation (§10 guarantee).

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::config::SimConfig;

/// Emitter seed fold constant (sandbox-scenes.spec.md §9).
const VFX_EMITTER_SEED_XOR: u64 = 0x0056_4658;

/// Wraps the single seeded RNG stream used for VFX emitter bursts (never body placement
/// or free-particle spawn — those draw from `EngineRngRes`, `rng.spec.md` §3). Article VI
/// forbids `OsRng`/`thread_rng` in sim paths — this resource is the only VFX RNG entry
/// point for the crate.
#[derive(Resource)]
pub struct DeterministicRng(pub StdRng);

pub struct RngPlugin;

impl Plugin for RngPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, insert_rng);
    }
}

/// Public so other `Startup` systems (e.g. `constraint::spawn_bodies`) can order themselves
/// `.after(insert_rng)` without needing a dedicated `SystemSet`.
pub fn insert_rng(mut commands: Commands, config: Res<SimConfig>) {
    commands.insert_resource(DeterministicRng(StdRng::seed_from_u64(
        config.seed ^ VFX_EMITTER_SEED_XOR,
    )));
}
