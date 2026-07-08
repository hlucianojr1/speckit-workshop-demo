// Orbital Arena — player-controlled gravity well: radial attraction, capture
// predicate, and kinematic stepping.
//
// Constitutional articles satisfied:
//   - 1 (no exceptions), 2 (no RTTI)
//   - 5, 10 (determinism: pure functions, no rng, no global state)
//   - 6 (real-time: no allocation anywhere in this module)
//   - 9 (fairness: no player-index parameter exists anywhere in this API)
//
// Forces apply directly to vfx::particle_pool::live_particles() spans (research.md
// D1) — NOT via physics::constraint_solver.

#pragma once

namespace orbital_arena {

// Player avatar. Value type, POD (data-model.md).
struct gravity_well {
    float position[2]{};
    float velocity[2]{};
    float strength{0.0f};          // normalized 0..1 (player input, FR-003)
    float influence_radius{2.0f};  // identical for all wells (FR-001)
    float capture_radius{0.15f};   // identical for all wells
    bool  active{true};            // false when player departs (FR-013)
};

inline constexpr float kMaxWellSpeed  = 2.0f;   // units/s, identical for all (FR-002)
inline constexpr float kMaxPullAccel  = 40.0f;  // accel at min clamped distance, strength=1
inline constexpr float kMinDistanceSq = 0.01f;  // inverse-square clamp (no div-by-zero)

// Radial acceleration (units/s^2) exerted by `well` on a point at `pos`.
// - Zero if !well.active, strength == 0, or distance > influence_radius (FR-003).
// - Magnitude: strength * kMaxPullAccel * (kMinDistanceSq / max(dist_sq, kMinDistanceSq)),
//   i.e. inverse-square with clamped minimum distance; direction: toward well.position.
void radial_acceleration(const gravity_well& well,
                         const float pos[2],
                         float out_accel[2]) noexcept;

// True iff `pos` is strictly inside well.capture_radius (squared-distance compare,
// no sqrt — FR-004) AND well.active. Boundary (exactly equal) is NOT captured.
[[nodiscard]] bool is_within_capture(const gravity_well& well,
                                     const float pos[2]) noexcept;

// Advances the well one fixed step: velocity := steer * kMaxWellSpeed with the
// magnitude capped at kMaxWellSpeed (FR-002), position += velocity * dt, then
// position clamped to [-half_extent, +half_extent]^2 (US1 scenario 4).
// `strength_input` is clamped to 0..1 before assignment. Inactive wells do not move.
void step_well(gravity_well& well,
               const float steer[2],
               float strength_input,
               float half_extent,
               double dt) noexcept;

}  // namespace orbital_arena
