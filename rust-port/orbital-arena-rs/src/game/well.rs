//! Gravity well physics (orbital-arena-gravity-well.spec.md).
//!
//! A DIFFERENT subsystem from `crate::constraint`'s rigid-link solver — gravity wells are
//! radial-attraction fields with a capture radius, not distance constraints. They share no
//! formulas (see the spec's Scope note).

use bevy::math::DVec2;
use bevy::prelude::*;

/// Max well speed, units/s (spec §2 reference value). Unused by the deterministic patrol
/// substitute in `game::drive_wells`, kept for spec fidelity / future real-input driving.
pub const MAX_WELL_SPEED: f64 = 2.0;
/// Max pull acceleration at zero clamped distance, strength 1 (spec §2).
pub const MAX_PULL_ACCEL: f64 = 40.0;
/// Minimum squared distance clamp — prevents a singularity at the well's own position
/// (spec §3.1).
pub const MIN_DISTANCE_SQ: f64 = 0.01;

/// A player-controlled gravity well (spec §2).
#[derive(Component, Debug, Clone, Copy)]
pub struct GravityWell {
    pub player_index: u8,
    pub position: DVec2,
    pub velocity: DVec2,
    pub strength: f64,
    pub influence_radius: f64,
    pub capture_radius: f64,
    pub active: bool,
}

/// Radial acceleration `well` exerts on a point at `pos` (spec §3.1). Zero if inactive,
/// zero-strength, or beyond `influence_radius`; otherwise inverse-square with a clamped
/// minimum distance, directed from `pos` toward `well.position`.
#[must_use]
pub fn radial_acceleration(well: &GravityWell, pos: DVec2) -> DVec2 {
    if !well.active || well.strength <= 0.0 {
        return DVec2::ZERO;
    }
    let delta = well.position - pos;
    let distance_sq = delta.length_squared();
    if distance_sq > well.influence_radius * well.influence_radius {
        return DVec2::ZERO;
    }
    if distance_sq < 1.0e-12 {
        return DVec2::ZERO;
    }
    let clamped_distance_sq = distance_sq.max(MIN_DISTANCE_SQ);
    let magnitude = well.strength * MAX_PULL_ACCEL * (MIN_DISTANCE_SQ / clamped_distance_sq);
    delta.normalize() * magnitude
}

/// True iff `pos` is strictly inside `well.capture_radius` and the well is active
/// (spec §3.2).
#[must_use]
pub fn is_within_capture(well: &GravityWell, pos: DVec2) -> bool {
    well.active
        && (pos - well.position).length_squared() < well.capture_radius * well.capture_radius
}

/// Kinematic step (spec §3.3): steer → clamped velocity → position, clamped to the
/// arena's square bounds. Not used by the deterministic patrol substitute in
/// `game::drive_wells` (there is no interactive input to steer with in a screenshot
/// exercise), but implemented and tested for spec fidelity.
pub fn step_well(
    well: &mut GravityWell,
    steer: DVec2,
    strength_input: f64,
    half_extent: f64,
    dt: f64,
) {
    if !well.active {
        return;
    }
    let steer_len = steer.length();
    let clamped_len = steer_len.min(1.0);
    let steer_dir = if steer_len > 1.0e-9 {
        steer / steer_len
    } else {
        DVec2::ZERO
    };
    well.velocity = steer_dir * clamped_len * MAX_WELL_SPEED;
    well.position += well.velocity * dt;
    well.position.x = well.position.x.clamp(-half_extent, half_extent);
    well.position.y = well.position.y.clamp(-half_extent, half_extent);
    well.strength = strength_input.clamp(0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_well(position: DVec2) -> GravityWell {
        GravityWell {
            player_index: 0,
            position,
            velocity: DVec2::ZERO,
            strength: 1.0,
            influence_radius: 100.0,
            capture_radius: 10.0,
            active: true,
        }
    }

    #[test]
    fn acceleration_is_zero_when_inactive() {
        let mut well = active_well(DVec2::ZERO);
        well.active = false;
        assert_eq!(
            radial_acceleration(&well, DVec2::new(5.0, 0.0)),
            DVec2::ZERO
        );
    }

    #[test]
    fn acceleration_is_zero_beyond_influence_radius() {
        let well = active_well(DVec2::ZERO);
        assert_eq!(
            radial_acceleration(&well, DVec2::new(1000.0, 0.0)),
            DVec2::ZERO
        );
    }

    #[test]
    fn acceleration_points_toward_well() {
        let well = active_well(DVec2::new(10.0, 0.0));
        let accel = radial_acceleration(&well, DVec2::ZERO);
        assert!(
            accel.x > 0.0,
            "accel should point toward +x well: {accel:?}"
        );
        assert!(accel.y.abs() < 1.0e-9);
    }

    #[test]
    fn acceleration_never_exceeds_max_at_min_distance() {
        let well = active_well(DVec2::ZERO);
        // At/near the well's own position, magnitude is clamped, not infinite.
        let accel = radial_acceleration(&well, DVec2::new(0.001, 0.0));
        assert!(
            accel.length() <= MAX_PULL_ACCEL + 1.0e-6,
            "accel = {accel:?}"
        );
    }

    #[test]
    fn capture_boundary_is_exclusive() {
        let well = active_well(DVec2::ZERO);
        assert!(!is_within_capture(&well, DVec2::new(10.0, 0.0))); // exactly on radius
        assert!(is_within_capture(&well, DVec2::new(9.9, 0.0)));
    }

    #[test]
    fn step_clamps_position_to_arena_bounds() {
        let mut well = active_well(DVec2::new(95.0, 0.0));
        step_well(&mut well, DVec2::new(1.0, 0.0), 1.0, 100.0, 10.0);
        assert!(well.position.x <= 100.0);
    }

    #[test]
    fn step_clamps_strength_input() {
        let mut well = active_well(DVec2::ZERO);
        step_well(&mut well, DVec2::ZERO, 5.0, 100.0, 1.0 / 60.0);
        assert_eq!(well.strength, 1.0);
    }
}
