//! `orbital-arena-rs` binary: CLI parsing (contracts/cli.md) + `App` assembly
//! (contracts/plugins.md Composition).
#![deny(unsafe_code)]

use std::path::PathBuf;

use bevy::prelude::*;
use clap::{Parser, Subcommand};

use orbital_arena_rs::config::{RunMode, SimConfig};
use orbital_arena_rs::constraint::PhysicsPlugin;
use orbital_arena_rs::frame_budget::FrameBudgetPlugin;
use orbital_arena_rs::rng::RngPlugin;
use orbital_arena_rs::screenshot::ScreenshotPlugin;
use orbital_arena_rs::visuals::VisualsPlugin;

/// `orbital-arena-rs [--seed <u64>]` — windowed by default; `screenshot` subcommand for
/// automated evidence capture (contracts/cli.md).
#[derive(Parser, Debug)]
#[command(name = "orbital-arena-rs", version, about)]
struct Cli {
    /// RNG seed for initial body-placement jitter (FR-007). Ignored when the `screenshot`
    /// subcommand supplies its own `--seed`.
    #[arg(long, default_value_t = 42)]
    seed: u64,

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
    },
}

fn main() {
    let cli = Cli::parse();

    let sim_config = match cli.mode {
        Some(Mode::Screenshot {
            warmup_frames,
            out,
            seed,
        }) => SimConfig {
            seed: seed.unwrap_or(cli.seed),
            run_mode: RunMode::Screenshot {
                warmup_frames,
                output_path: out,
            },
        },
        None => SimConfig {
            seed: cli.seed,
            run_mode: RunMode::Windowed,
        },
    };

    let is_screenshot_mode = matches!(sim_config.run_mode, RunMode::Screenshot { .. });

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .insert_resource(sim_config)
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_plugins(RngPlugin)
        .add_plugins(PhysicsPlugin)
        .add_plugins(FrameBudgetPlugin)
        .add_plugins(VisualsPlugin);

    if is_screenshot_mode {
        app.add_plugins(ScreenshotPlugin);
    }

    app.run();
}
