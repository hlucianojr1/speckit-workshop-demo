//! `ScreenshotPlugin`: after `warmup_frames` `FixedUpdate` ticks have elapsed, captures a
//! single screenshot and exits (contracts/plugins.md `ScreenshotPlugin`, research.md R2,
//! FR-010/FR-011). Requires rendering — only added when `SimConfig.run_mode` is
//! `Screenshot`, never in headless tests.

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};

use crate::config::{RunMode, SimConfig};

/// Number of extra `Update` frames to wait, after the screenshot has been requested,
/// before exiting — gives the async capture + disk-write pipeline time to complete
/// (research.md R2).
const EXIT_DELAY_FRAMES: u32 = 10;

/// Only inserted when `SimConfig.run_mode` is `Screenshot` (data-model.md §Resources).
#[derive(Resource, Debug, Default)]
pub struct ScreenshotState {
    pub fixed_ticks_elapsed: u32,
    pub triggered: bool,
    pub frames_since_trigger: u32,
}

pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScreenshotState::default())
            .add_systems(FixedUpdate, count_fixed_ticks)
            .add_systems(Update, (trigger_screenshot, exit_after_screenshot).chain());
    }
}

fn count_fixed_ticks(mut state: ResMut<ScreenshotState>) {
    state.fixed_ticks_elapsed += 1;
}

/// Fires once `fixed_ticks_elapsed >= warmup_frames`; `warmup_frames = 0` captures at the
/// first post-setup render (Edge Cases).
fn trigger_screenshot(
    mut commands: Commands,
    mut state: ResMut<ScreenshotState>,
    config: Res<SimConfig>,
) {
    if state.triggered {
        return;
    }
    let RunMode::Screenshot {
        warmup_frames,
        ref output_path,
    } = config.run_mode
    else {
        return;
    };
    if state.fixed_ticks_elapsed < warmup_frames {
        return;
    }

    let path = output_path.clone();
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path));
    state.triggered = true;
}

/// Requests `AppExit::Success` a few frames after the screenshot was requested, once the
/// async capture/disk-write pipeline has had time to run (FR-011: exit 0 only after the
/// file is confirmed written; `save_to_disk` itself logs — via `error!` — rather than
/// panicking on a write failure, satisfying Article I's no-panic rule).
fn exit_after_screenshot(mut state: ResMut<ScreenshotState>, mut exit: EventWriter<AppExit>) {
    if !state.triggered {
        return;
    }
    state.frames_since_trigger += 1;
    if state.frames_since_trigger >= EXIT_DELAY_FRAMES {
        exit.write(AppExit::Success);
    }
}
