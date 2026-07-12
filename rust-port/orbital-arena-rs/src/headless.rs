//! Headless CSV/digest mode (sandbox-headless.spec.md) — T045 of the Full-Fidelity
//! Closure backlog: a windowless, render-free run that simulates N fixed steps and
//! emits the reference CSV trace plus a final digest — the cross-language golden-check
//! instrument (sandbox-scenes.spec.md §8.1).

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::body::{Body, SpawnIndex};
use crate::config::FIXED_STEP_SECONDS;
use crate::config::{RunMode, SimConfig};
use crate::constraint::PhysicsPlugin;
use crate::digest::state_digest;
use crate::engine_rng::EngineRngPlugin;
use crate::free_particles::FreeParticlePlugin;
use crate::orbital::{ArenaConfig, ArenaPlugin};
use crate::rng::RngPlugin;
use crate::vfx::VfxPlugin;

/// Sandbox override of the arena's game-default `half_extent` (5.0), chosen to fit the
/// view (sandbox-scenes.spec.md §4.5).
const ORBITAL_HALF_EXTENT: f32 = 2.0;
const ORBITAL_PLAYERS: u8 = 2;

/// Scenes the headless runner can currently simulate. The CLI accepts all five spec
/// scene names (sandbox-headless.spec.md §2); variants are added here as their scenes
/// land (US9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadlessScene {
    /// The default scene (reference display name "rope").
    Rope,
    /// The embedded Orbital Arena match (sandbox-scenes.spec.md §4.5/§6, US8).
    Orbital,
}

impl HeadlessScene {
    /// Display name matching the reference stdout line (`scene=<display name>`).
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Rope => "rope",
            Self::Orbital => "orbital arena",
        }
    }
}

/// Errors mirroring the reference `headless_status` enum (Article 1: status values,
/// never panics).
#[derive(Debug)]
pub enum HeadlessError {
    InvalidArguments,
    WriteFailed(io::Error),
}

impl std::fmt::Display for HeadlessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidArguments => write!(f, "invalid arguments (frames must be > 0)"),
            Self::WriteFailed(e) => write!(f, "cannot write trace output: {e}"),
        }
    }
}

/// Successful run summary.
pub struct HeadlessReport {
    pub digest: u64,
    pub frames_written: u32,
    pub scene: HeadlessScene,
}

impl HeadlessReport {
    /// The reference stdout line: `trace_digest=<%016x> frames=<N> scene=<name>`.
    #[must_use]
    pub fn summary_line(&self) -> String {
        format!(
            "trace_digest={:016x} frames={} scene={}",
            self.digest,
            self.frames_written,
            self.scene.display_name()
        )
    }
}

/// Builds the windowless app for a scene: `MinimalPlugins` only — no window, no GL, no
/// wall clock (`TimeUpdateStrategy::ManualDuration` advances virtual time by exactly one
/// fixed step per `update()`).
fn build_app(seed: u64, frames: u32, scene: HeadlessScene) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SimConfig {
        seed,
        run_mode: RunMode::Headless { frames },
    });
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        FIXED_STEP_SECONDS,
    )));
    app.add_plugins(RngPlugin);
    app.add_plugins(EngineRngPlugin);
    match scene {
        HeadlessScene::Rope => {
            app.add_plugins(PhysicsPlugin)
                .add_plugins(VfxPlugin)
                .add_plugins(FreeParticlePlugin);
        }
        HeadlessScene::Orbital => {
            app.add_plugins(ArenaPlugin {
                config: ArenaConfig {
                    seed,
                    player_count: ORBITAL_PLAYERS,
                    half_extent: ORBITAL_HALF_EXTENT,
                    particle_capacity: 500,
                },
            });
        }
    }
    app
}

/// First and last body positions in spawn order (`rope0` / `ropeN` CSV columns; 0.0
/// when the scene has no bodies, per the reference).
fn endpoint_positions(world: &mut World) -> (f64, f64, f64, f64) {
    let mut min: Option<(u32, f64, f64)> = None;
    let mut max: Option<(u32, f64, f64)> = None;
    let mut query = world.query::<(&SpawnIndex, &Body)>();
    for (index, body) in query.iter(world) {
        let entry = (index.0, body.position.x, body.position.y);
        if min.is_none_or(|(i, _, _)| entry.0 < i) {
            min = Some(entry);
        }
        if max.is_none_or(|(i, _, _)| entry.0 > i) {
            max = Some(entry);
        }
    }
    let (_, r0x, r0y) = min.unwrap_or((0, 0.0, 0.0));
    let (_, rnx, rny) = max.unwrap_or((0, 0.0, 0.0));
    (r0x, r0y, rnx, rny)
}

/// Runs `frames` fixed steps and writes one CSV row per step (byte-normative format,
/// sandbox-headless.spec.md §4). Wall clock is never consulted.
pub fn run_headless(
    seed: u64,
    frames: u32,
    out_path: &Path,
    scene: HeadlessScene,
) -> Result<HeadlessReport, HeadlessError> {
    if frames == 0 {
        return Err(HeadlessError::InvalidArguments);
    }

    let mut app = build_app(seed, frames, scene);
    // Priming update: runs Startup (scene spawn) and initializes virtual time; the
    // first FixedUpdate step is only observed from the second update() onward
    // (recorded Bevy ManualDuration behavior — see the crate's test suite pattern).
    app.update();

    let mut csv = String::new();
    csv.push_str("frame,sim_time,digest,rope0_x,rope0_y,ropeN_x,ropeN_y\n");

    // Accumulated like the reference (`m_sim_time += dt` per substep), not multiplied.
    let mut sim_time = 0.0f64;
    let mut digest = 0u64;
    for frame in 0..frames {
        app.update(); // exactly one fixed step
        sim_time += FIXED_STEP_SECONDS;
        digest = state_digest(app.world_mut());
        let (r0x, r0y, rnx, rny) = endpoint_positions(app.world_mut());
        // Reference row format: %u,%.9f,%016llx,%.9f,%.9f,%.9f,%.9f
        let _ = writeln!(
            csv,
            "{frame},{sim_time:.9},{digest:016x},{r0x:.9},{r0y:.9},{rnx:.9},{rny:.9}"
        );
    }

    fs::write(out_path, csv).map_err(HeadlessError::WriteFailed)?;

    Ok(HeadlessReport {
        digest,
        frames_written: frames,
        scene,
    })
}
