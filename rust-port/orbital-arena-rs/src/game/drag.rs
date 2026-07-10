//! Interactive well control (Phase A2, new capability —
//! specs/transform/orbital-arena-interactive-control.spec.md). Requires rendering
//! (Camera/Window) — only added alongside `GameVisualsPlugin`, never in headless tests.

use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::well::GravityWell;
use super::{WellDragState, HALF_EXTENT};

/// World-space radius within which a press grabs the interactive well (well 0).
const GRAB_RADIUS: f64 = 40.0;
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

/// True iff a press at `cursor_world` should grab the well currently at `well_position`
/// (spec §2: within `GRAB_RADIUS`). Pure and unit-testable, unlike the ECS system that
/// calls it.
#[must_use]
fn is_within_grab_radius(well_position: DVec2, cursor_world: DVec2) -> bool {
    (cursor_world - well_position).length() <= GRAB_RADIUS
}

/// Computes the well's new (position, velocity) for one drag-update frame, clamped to the
/// arena bounds (spec §2). Pure and unit-testable, unlike the ECS system that calls it.
#[must_use]
fn dragged_well_state(current_position: DVec2, cursor_world: DVec2) -> (DVec2, DVec2) {
    let clamped = DVec2::new(
        cursor_world.x.clamp(-HALF_EXTENT, HALF_EXTENT),
        cursor_world.y.clamp(-HALF_EXTENT, HALF_EXTENT),
    );
    (clamped, clamped - current_position)
}

/// Press-and-drag on the interactive well; release resumes its normal driving logic
/// (spec §2). Handles a missing window/camera/cursor gracefully (early-return, no
/// panic) per spec §4.
fn drag_well_system(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut wells: Query<&mut GravityWell>,
    mut drag_state: ResMut<WellDragState>,
) {
    if mouse.just_released(MouseButton::Left) {
        drag_state.dragging = false;
        return;
    }

    let Some(cursor_world) = cursor_world_position(&windows, &cameras) else {
        return;
    };
    let cursor_world = DVec2::new(f64::from(cursor_world.x), f64::from(cursor_world.y));

    let Some(mut well) = wells
        .iter_mut()
        .find(|w| w.player_index == DRAGGABLE_PLAYER_INDEX)
    else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        drag_state.dragging = is_within_grab_radius(well.position, cursor_world);
    }

    if drag_state.dragging && mouse.pressed(MouseButton::Left) {
        let (position, velocity) = dragged_well_state(well.position, cursor_world);
        well.position = position;
        well.velocity = velocity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grab_succeeds_strictly_within_radius() {
        assert!(is_within_grab_radius(
            DVec2::ZERO,
            DVec2::new(GRAB_RADIUS - 1.0, 0.0)
        ));
    }

    #[test]
    fn grab_fails_outside_radius() {
        assert!(!is_within_grab_radius(
            DVec2::ZERO,
            DVec2::new(GRAB_RADIUS + 1.0, 0.0)
        ));
    }

    #[test]
    fn grab_succeeds_exactly_at_radius() {
        assert!(is_within_grab_radius(
            DVec2::ZERO,
            DVec2::new(GRAB_RADIUS, 0.0)
        ));
    }

    #[test]
    fn dragged_well_follows_cursor_within_bounds() {
        let (position, velocity) = dragged_well_state(DVec2::ZERO, DVec2::new(10.0, 20.0));
        assert_eq!(position, DVec2::new(10.0, 20.0));
        assert_eq!(velocity, DVec2::new(10.0, 20.0));
    }

    #[test]
    fn dragged_well_clamps_to_arena_bounds() {
        let far_cursor = DVec2::new(HALF_EXTENT * 10.0, -HALF_EXTENT * 10.0);
        let (position, _velocity) = dragged_well_state(DVec2::ZERO, far_cursor);
        assert_eq!(position.x, HALF_EXTENT);
        assert_eq!(position.y, -HALF_EXTENT);
    }

    #[test]
    fn dragged_well_velocity_is_delta_from_previous_position() {
        let (_, velocity) = dragged_well_state(DVec2::new(5.0, 5.0), DVec2::new(8.0, 1.0));
        assert_eq!(velocity, DVec2::new(3.0, -4.0));
    }
}
