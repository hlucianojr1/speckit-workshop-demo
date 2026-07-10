//! `RngPlugin`: seeds a single `StdRng` from `SimConfig.seed` at `Startup`
//! (contracts/plugins.md `RngPlugin`, data-model.md §Resources `DeterministicRng`).

use bevy::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

use crate::config::SimConfig;

/// Wraps the single seeded RNG stream used for initial body-placement jitter
/// (rng.spec.md §3 `construct`). Article VI forbids `OsRng`/`thread_rng` in sim paths —
/// this resource is the only RNG entry point for the crate.
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
    commands.insert_resource(DeterministicRng(StdRng::seed_from_u64(config.seed)));
}
