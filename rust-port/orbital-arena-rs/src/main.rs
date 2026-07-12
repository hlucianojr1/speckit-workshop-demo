//! `orbital-arena-rs` binary: CLI parsing (contracts/cli.md) + `App` assembly
//! (contracts/plugins.md Composition).
#![deny(unsafe_code)]

use std::path::PathBuf;

use bevy::prelude::*;
use clap::{Parser, Subcommand, ValueEnum};

use orbital_arena_rs::background::BackgroundPlugin;
use orbital_arena_rs::config::{RunMode, SimConfig};
use orbital_arena_rs::constraint::PhysicsPlugin;
use orbital_arena_rs::engine_rng::EngineRngPlugin;
use orbital_arena_rs::frame_budget::FrameBudgetPlugin;
use orbital_arena_rs::free_particles::FreeParticlePlugin;
use orbital_arena_rs::orbital::drag::WellDragPlugin;
use orbital_arena_rs::orbital::visuals::ArenaVisualsPlugin;
use orbital_arena_rs::orbital::{ArenaConfig, ArenaPlugin};
use orbital_arena_rs::rng::RngPlugin;
use orbital_arena_rs::screenshot::ScreenshotPlugin;
use orbital_arena_rs::vfx::VfxPlugin;
use orbital_arena_rs::visuals::VisualsPlugin;

/// Sandbox override of the arena's game-default `half_extent` (5.0), chosen to fit the
/// view (sandbox-scenes.spec.md §4.5).
const ORBITAL_HALF_EXTENT: f32 = 2.0;
const ORBITAL_PLAYERS: u8 = 2;

/// Which scene to run. The five reference scene names (+ aliases) are accepted per
/// sandbox-headless.spec.md §2; `constraint` is the pre-T048 alias of `rope`, and
/// `arena` the port's historical alias of `orbital`. Scenes whose ports have not
/// landed yet (pendulum/cloth/storm — US9) are accepted by the parser but exit with a
/// clear error at runtime.
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Scene {
    #[default]
    #[value(alias = "orbital", alias = "orbital_arena")]
    Arena,
    #[value(alias = "rope")]
    Constraint,
    #[value(alias = "pendulum_tower")]
    Pendulum,
    Cloth,
    #[value(alias = "particle_storm")]
    Storm,
}

/// `orbital-arena-rs [--seed <u64>] [--scene <arena|constraint>]` — windowed by default;
/// `screenshot` subcommand for automated evidence capture (contracts/cli.md).
#[derive(Parser, Debug)]
#[command(name = "orbital-arena-rs", version, about)]
struct Cli {
    /// RNG seed for initial body-placement jitter (FR-007). Ignored when the `screenshot`
    /// subcommand supplies its own `--seed`.
    #[arg(long, default_value_t = 42)]
    seed: u64,

    /// Which scene to run (Phase A2 extension).
    #[arg(long, value_enum, default_value_t = Scene::Arena)]
    scene: Scene,

    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Subcommand, Debug)]
enum Mode {
    /// Run for `warmup_frames` fixed ticks, then write a screenshot to `out` and exit 0.
    Screenshot {
        #[arg(long)]
        warmup_frames: u32,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long, value_enum)]
        scene: Option<Scene>,
    },
    /// Windowless run: simulate N fixed steps, write a CSV trace, print the digest
    /// (sandbox-headless.spec.md; T045).
    Headless {
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long, default_value_t = 600)]
        frames: u32,
        #[arg(long, default_value = "trace.csv")]
        out: PathBuf,
        #[arg(long, value_enum)]
        scene: Option<Scene>,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Some(Mode::Headless {
        seed,
        frames,
        out,
        scene,
    }) = cli.mode
    {
        run_headless_mode(
            seed.unwrap_or(cli.seed),
            frames,
            &out,
            scene.unwrap_or(cli.scene),
        );
        return;
    }

    let (sim_config, scene) = match cli.mode {
        Some(Mode::Screenshot {
            warmup_frames,
            out,
            seed,
            scene,
        }) => (
            SimConfig {
                seed: seed.unwrap_or(cli.seed),
                run_mode: RunMode::Screenshot {
                    warmup_frames,
                    output_path: out,
                },
            },
            scene.unwrap_or(cli.scene),
        ),
        None => (
            SimConfig {
                seed: cli.seed,
                run_mode: RunMode::Windowed,
            },
            cli.scene,
        ),
        Some(Mode::Headless { .. }) => unreachable!("handled above"),
    };

    let is_screenshot_mode = matches!(sim_config.run_mode, RunMode::Screenshot { .. });
    let seed = sim_config.seed;

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(sim_config)
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(RngPlugin)
        .add_plugins(FrameBudgetPlugin);

    match scene {
        Scene::Arena => {
            app.add_plugins(ArenaPlugin {
                config: ArenaConfig {
                    seed,
                    player_count: ORBITAL_PLAYERS,
                    half_extent: ORBITAL_HALF_EXTENT,
                    particle_capacity: 500,
                },
            })
            .add_plugins(ArenaVisualsPlugin)
            .add_plugins(WellDragPlugin);
        }
        Scene::Constraint => {
            app.add_plugins(BackgroundPlugin)
                .add_plugins(EngineRngPlugin)
                .add_plugins(PhysicsPlugin)
                .add_plugins(VfxPlugin)
                .add_plugins(FreeParticlePlugin)
                .add_plugins(VisualsPlugin);
        }
        Scene::Pendulum | Scene::Cloth | Scene::Storm => {
            eprintln!(
                "orbital-arena-rs: scene not implemented yet \
                 (Full-Fidelity Closure backlog, US9 / T069\u{2013}T071)"
            );
            std::process::exit(3);
        }
    }

    if is_screenshot_mode {
        app.add_plugins(ScreenshotPlugin);
    }

    app.add_systems(Update, exit_on_escape);

    app.run();
}

/// Parity with the reference sandbox's "Esc quit" control hint (sandbox-hud.spec.md §2).
fn exit_on_escape(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

/// Headless dispatch: maps CLI scene names onto runnable headless scenes, runs the
/// trace, prints the reference summary line. Exit codes per sandbox-headless.spec.md
/// §2: 0 success, 2 argument error (clap's default), 3 run failure.
fn run_headless_mode(seed: u64, frames: u32, out: &std::path::Path, scene: Scene) {
    use orbital_arena_rs::headless::{run_headless, HeadlessScene};

    let headless_scene = match scene {
        Scene::Constraint => HeadlessScene::Rope,
        Scene::Arena => HeadlessScene::Orbital,
        Scene::Pendulum | Scene::Cloth | Scene::Storm => {
            eprintln!(
                "orbital-arena-rs: scene not implemented yet \
                 (Full-Fidelity Closure backlog, US9 / T069\u{2013}T071)"
            );
            std::process::exit(3);
        }
    };

    match run_headless(seed, frames, out, headless_scene) {
        Ok(report) => println!("{}", report.summary_line()),
        Err(error) => {
            eprintln!("orbital-arena-rs: headless run failed: {error}");
            std::process::exit(3);
        }
    }
}
