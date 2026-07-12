//! Player-controlled gravity well (orbital-arena-gravity-well.spec.md). Pure `f32`
//! arithmetic matching the C++ reference exactly — a rendering/gameplay-feel subsystem,
//! not a `double`-accumulator sim path (spec §5) — since particle live-order and exact
//! position values feed the cross-language state digest (sandbox-scenes.spec.md §8).
//!
//! A DIFFERENT subsystem from `crate::constraint`'s rigid-link solver and from
//! `crate::game`'s Phase A2 screenshot-scale prototype (pixel units, `f64`) that this
//! module supersedes for full-fidelity conformance (Full-Fidelity Closure backlog, US8).

use bevy::math::Vec2;

/// Max well speed, units/s (spec §2).
pub const MAX_WELL_SPEED: f32 = 2.0;
/// Max pull acceleration at zero clamped distance, strength 1 (spec §2).
pub const MAX_PULL_ACCEL: f32 = 40.0;
/// Minimum squared distance clamp — prevents a singularity at the well's own position
/// (spec §3.1).
pub const MIN_DISTANCE_SQ: f32 = 0.01;
/// Identical for every well (spec §2).
pub const DEFAULT_INFLUENCE_RADIUS: f32 = 2.0;
pub const DEFAULT_CAPTURE_RADIUS: f32 = 0.15;

/// A player-controlled gravity well (spec §2).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GravityWell {
    pub position: Vec2,
    pub velocity: Vec2,
    pub strength: f32,
    pub influence_radius: f32,
    pub capture_radius: f32,
    pub active: bool,
}

impl Default for GravityWell {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            strength: 0.0,
            influence_radius: DEFAULT_INFLUENCE_RADIUS,
            capture_radius: DEFAULT_CAPTURE_RADIUS,
            active: true,
        }
    }
}

/// Radial acceleration `well` exerts on a point at `pos` (spec §3.1). Zero if inactive,
/// zero-strength, coincident, or beyond `influence_radius`; otherwise inverse-square
/// with a clamped minimum distance, directed from `pos` toward `well.position`.
#[must_use]
pub fn radial_acceleration(well: &GravityWell, pos: Vec2) -> Vec2 {
    if !well.active || well.strength <= 0.0 {
        return Vec2::ZERO;
    }
    let delta = well.position - pos;
    let dist_sq = delta.length_squared();
    if dist_sq > well.influence_radius * well.influence_radius {
        return Vec2::ZERO;
    }
    if dist_sq == 0.0 {
        return Vec2::ZERO; // coincident: direction undefined, exert nothing (deterministic)
    }
    let clamped_sq = if dist_sq < MIN_DISTANCE_SQ {
        MIN_DISTANCE_SQ
    } else {
        dist_sq
    };
    let magnitude = well.strength * MAX_PULL_ACCEL * (MIN_DISTANCE_SQ / clamped_sq);
    let inv_dist = 1.0 / dist_sq.sqrt();
    Vec2::new(delta.x * inv_dist, delta.y * inv_dist) * magnitude
}

/// True iff `pos` is strictly inside `well.capture_radius` and the well is active
/// (spec §3.2, squared-distance compare, no sqrt).
#[must_use]
pub fn is_within_capture(well: &GravityWell, pos: Vec2) -> bool {
    well.active
        && (well.position - pos).length_squared() < well.capture_radius * well.capture_radius
}

/// Kinematic step (spec §3.3): steer → clamped velocity → position, clamped to the
/// arena's square bounds. `dt` is `f64` (matching the reference's `kTickSeconds`
/// parameter) but narrowed to `f32` before use, since well state is `f32`.
pub fn step_well(
    well: &mut GravityWell,
    steer: Vec2,
    strength_input: f32,
    half_extent: f32,
    dt: f64,
) {
    if !well.active {
        return; // departed players' wells neither move nor change strength
    }
    well.strength = strength_input.clamp(0.0, 1.0);

    let mut v = steer * MAX_WELL_SPEED;
    let speed_sq = v.length_squared();
    let max_sq = MAX_WELL_SPEED * MAX_WELL_SPEED;
    if speed_sq > max_sq {
        let scale = MAX_WELL_SPEED / speed_sq.sqrt();
        v *= scale;
    }
    well.velocity = v;

    let dt = dt as f32;
    well.position.x = (well.position.x + v.x * dt).clamp(-half_extent, half_extent);
    well.position.y = (well.position.y + v.y * dt).clamp(-half_extent, half_extent);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_well(position: Vec2) -> GravityWell {
        GravityWell {
            position,
            velocity: Vec2::ZERO,
            strength: 1.0,
            influence_radius: 100.0,
            capture_radius: 10.0,
            active: true,
        }
    }

    #[test]
    fn acceleration_is_zero_when_inactive() {
        let mut well = active_well(Vec2::ZERO);
        well.active = false;
        assert_eq!(radial_acceleration(&well, Vec2::new(5.0, 0.0)), Vec2::ZERO);
    }

    #[test]
    fn acceleration_is_zero_beyond_influence_radius() {
        let well = active_well(Vec2::ZERO);
        assert_eq!(
            radial_acceleration(&well, Vec2::new(1000.0, 0.0)),
            Vec2::ZERO
        );
    }

    #[test]
    fn acceleration_points_toward_well() {
        let well = active_well(Vec2::new(10.0, 0.0));
        let accel = radial_acceleration(&well, Vec2::ZERO);
        assert!(
            accel.x > 0.0,
            "accel should point toward +x well: {accel:?}"
        );
        assert!(accel.y.abs() < 1.0e-6);
    }

    #[test]
    fn acceleration_never_exceeds_max_at_min_distance() {
        let well = active_well(Vec2::ZERO);
        let accel = radial_acceleration(&well, Vec2::new(0.001, 0.0));
        assert!(
            accel.length() <= MAX_PULL_ACCEL + 1.0e-3,
            "accel = {accel:?}"
        );
    }

    #[test]
    fn capture_boundary_is_exclusive() {
        let well = active_well(Vec2::ZERO);
        assert!(!is_within_capture(&well, Vec2::new(10.0, 0.0))); // exactly on radius
        assert!(is_within_capture(&well, Vec2::new(9.9, 0.0)));
    }

    #[test]
    fn step_clamps_position_to_arena_bounds() {
        let mut well = active_well(Vec2::new(95.0, 0.0));
        step_well(&mut well, Vec2::new(1.0, 0.0), 1.0, 100.0, 10.0);
        assert!(well.position.x <= 100.0);
    }

    #[test]
    fn step_clamps_strength_input() {
        let mut well = active_well(Vec2::ZERO);
        step_well(&mut well, Vec2::ZERO, 5.0, 100.0, 1.0 / 60.0);
        assert_eq!(well.strength, 1.0);
    }

    #[test]
    fn inactive_well_does_not_move() {
        let mut well = active_well(Vec2::new(1.0, 1.0));
        well.active = false;
        step_well(&mut well, Vec2::new(1.0, 0.0), 1.0, 100.0, 1.0 / 60.0);
        assert_eq!(well.position, Vec2::new(1.0, 1.0));
    }
}
