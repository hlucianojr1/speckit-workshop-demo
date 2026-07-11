//! `orbital-arena-rs` binary: CLI parsing (contracts/cli.md) + `App` assembly
//! (contracts/plugins.md Composition).
#![deny(unsafe_code)]

use std::path::PathBuf;

use bevy::prelude::*;
use clap::{Parser, Subcommand, ValueEnum};

use orbital_arena_rs::config::{RunMode, SimConfig};
use orbital_arena_rs::constraint::PhysicsPlugin;
use orbital_arena_rs::frame_budget::FrameBudgetPlugin;
use orbital_arena_rs::free_particles::FreeParticlePlugin;
use orbital_arena_rs::game::drag::WellDragPlugin;
use orbital_arena_rs::game::visuals::GameVisualsPlugin;
use orbital_arena_rs::game::GamePlugin;
use orbital_arena_rs::rng::RngPlugin;
use orbital_arena_rs::screenshot::ScreenshotPlugin;
use orbital_arena_rs::vfx::VfxPlugin;
use orbital_arena_rs::visuals::VisualsPlugin;

/// Which scene to run: the Phase A2 Orbital Arena game (gravity wells + capture +
/// scoring, default — compares against the Part 3 reference screenshots) or the
/// original Phase A constraint-solver demo (§4.2's rigid-link "rope").
#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Scene {
    #[default]
    Arena,
    Constraint,
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
}

fn main() {
    let cli = Cli::parse();

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
    };

    let is_screenshot_mode = matches!(sim_config.run_mode, RunMode::Screenshot { .. });

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(sim_config)
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(RngPlugin)
        .add_plugins(FrameBudgetPlugin);

    match scene {
        Scene::Arena => {
            app.add_plugins(GamePlugin)
                .add_plugins(GameVisualsPlugin)
                .add_plugins(WellDragPlugin);
        }
        Scene::Constraint => {
            app.add_plugins(PhysicsPlugin)
                .add_plugins(VfxPlugin)
                .add_plugins(FreeParticlePlugin)
                .add_plugins(VisualsPlugin);
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
