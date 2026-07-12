//! Interactive well control (orbital-arena-interactive-control.spec.md): press-and-drag
//! well 0. Requires rendering (Camera/Window) — only added alongside the windowed app,
//! never in headless tests.

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{ArenaRes, WellDragState};

/// World-space radius within which a press grabs well 0 (spec §2).
const GRAB_RADIUS: f32 = 0.4;
/// Which well responds to mouse drag (spec §2: "reference: well index 0").
const DRAGGABLE_PLAYER_INDEX: u8 = 0;

pub struct WellDragPlugin;

impl Plugin for WellDragPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, drag_well_system);
    }
}

fn cursor_world_position(
    windows: &Query<&Window, With<PrimaryWindow>>,
    cameras: &Query<(&Camera, &GlobalTransform)>,
) -> Option<Vec2> {
    let window = windows.iter().next()?;
    let cursor = window.cursor_position()?;
    let (camera, camera_transform) = cameras.iter().next()?;
    camera.viewport_to_world_2d(camera_transform, cursor).ok()
}

/// Press-and-drag well 0; release resumes its normal autopilot driving (spec §2).
/// Handles a missing window/camera/cursor gracefully (early-return, no panic, spec §4).
fn drag_well_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    arena: Option<ResMut<ArenaRes>>,
    mut drag_state: ResMut<WellDragState>,
) {
    if mouse.just_released(MouseButton::Left) {
        drag_state.dragging = false;
        return;
    }
    let Some(mut arena) = arena else {
        return;
    };
    let Some(cursor_world) = cursor_world_position(&windows, &cameras) else {
        return;
    };
    let half_extent = arena.0.config().half_extent;
    let Some(well_position) = arena.0.wells().first().map(|w| w.position) else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        drag_state.dragging = (cursor_world - well_position).length() <= GRAB_RADIUS;
    }

    if drag_state.dragging && mouse.pressed(MouseButton::Left) {
        let clamped = Vec2::new(
            cursor_world.x.clamp(-half_extent, half_extent),
            cursor_world.y.clamp(-half_extent, half_extent),
        );
        arena
            .0
            .override_well_position(DRAGGABLE_PLAYER_INDEX, clamped);
    }
}
