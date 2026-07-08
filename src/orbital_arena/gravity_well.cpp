// Orbital Arena gravity well — see include/orbital_arena/gravity_well.h for the contract.

#include "orbital_arena/gravity_well.h"

#include <cmath>

namespace orbital_arena {

void radial_acceleration(const gravity_well& well,
                         const float pos[2],
                         float out_accel[2]) noexcept {
    out_accel[0] = 0.0f;
    out_accel[1] = 0.0f;
    if (!well.active || well.strength <= 0.0f) {
        return;  // exact zero (US1 scenario 3, FR-013)
    }

    const float dx = well.position[0] - pos[0];
    const float dy = well.position[1] - pos[1];
    const float dist_sq = dx * dx + dy * dy;
    if (dist_sq > well.influence_radius * well.influence_radius) {
        return;  // beyond influence radius: exactly zero (FR-003)
    }
    if (dist_sq == 0.0f) {
        return;  // coincident: direction undefined, exert nothing (deterministic)
    }

    const float clamped_sq = dist_sq < kMinDistanceSq ? kMinDistanceSq : dist_sq;
    const float magnitude = well.strength * kMaxPullAccel * (kMinDistanceSq / clamped_sq);
    const float inv_dist = 1.0f / std::sqrt(dist_sq);
    out_accel[0] = magnitude * dx * inv_dist;
    out_accel[1] = magnitude * dy * inv_dist;
}

bool is_within_capture(const gravity_well& well, const float pos[2]) noexcept {
    if (!well.active) {
        return false;
    }
    const float dx = well.position[0] - pos[0];
    const float dy = well.position[1] - pos[1];
    // Squared-distance compare, no sqrt (FR-004); strict: boundary is NOT captured.
    return dx * dx + dy * dy < well.capture_radius * well.capture_radius;
}

namespace {

[[nodiscard]] float clamp_unit(float value) noexcept {
    if (value < 0.0f) {
        return 0.0f;
    }
    if (value > 1.0f) {
        return 1.0f;
    }
    return value;
}

[[nodiscard]] float clamp_abs(float value, float bound) noexcept {
    if (value < -bound) {
        return -bound;
    }
    if (value > bound) {
        return bound;
    }
    return value;
}

}  // namespace

void step_well(gravity_well& well,
               const float steer[2],
               float strength_input,
               float half_extent,
               double dt) noexcept {
    if (!well.active) {
        return;  // departed players' wells neither move nor change strength (FR-013)
    }

    well.strength = clamp_unit(strength_input);

    float vx = steer[0] * kMaxWellSpeed;
    float vy = steer[1] * kMaxWellSpeed;
    const float speed_sq = vx * vx + vy * vy;
    const float max_sq = kMaxWellSpeed * kMaxWellSpeed;
    if (speed_sq > max_sq) {
        const float scale = kMaxWellSpeed / std::sqrt(speed_sq);
        vx *= scale;
        vy *= scale;
    }
    well.velocity[0] = vx;
    well.velocity[1] = vy;

    well.position[0] = clamp_abs(well.position[0] + vx * static_cast<float>(dt), half_extent);
    well.position[1] = clamp_abs(well.position[1] + vy * static_cast<float>(dt), half_extent);
}

}  // namespace orbital_arena
