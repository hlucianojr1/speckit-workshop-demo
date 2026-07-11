//! Shared screen-to-world cursor conversion, factored out once a second feature
//! (interactive VFX/particle bursts, Phase A4) needed the same helper as the well-drag
//! feature (Phase A2/A3). Requires rendering (Camera/Window).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// Converts the primary window's current cursor position to world space via the
/// (assumed single) active camera. Returns `None` gracefully if there is no primary
/// window, no camera, or the cursor is outside the window — never panics.
#[must_use]
pub fn cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.iter().next()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = cameras.iter().next()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}
