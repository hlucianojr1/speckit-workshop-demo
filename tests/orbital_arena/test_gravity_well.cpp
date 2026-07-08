// GoogleTest for orbital_arena gravity well (T004 — written before the implementation,
// per Constitution Article 7 test-first discipline).

#include "orbital_arena/gravity_well.h"

#include <cmath>
#include <gtest/gtest.h>

namespace {

using orbital_arena::gravity_well;
using orbital_arena::is_within_capture;
using orbital_arena::kMaxPullAccel;
using orbital_arena::kMaxWellSpeed;
using orbital_arena::kMinDistanceSq;
using orbital_arena::radial_acceleration;
using orbital_arena::step_well;

gravity_well make_well(float x, float y, float strength) noexcept {
    gravity_well well{};
    well.position[0] = x;
    well.position[1] = y;
    well.strength = strength;
    return well;
}

TEST(gravity_well, radial_acceleration_inverse_square_magnitude_and_direction) {
    const gravity_well well = make_well(0.0f, 0.0f, 1.0f);
    const float pos[2] = {1.0f, 0.0f};  // dist_sq = 1.0, inside influence_radius 2.0
    float accel[2] = {99.0f, 99.0f};
    radial_acceleration(well, pos, accel);

    // Magnitude: strength * kMaxPullAccel * (kMinDistanceSq / dist_sq), toward the well.
    const float expected = 1.0f * kMaxPullAccel * (kMinDistanceSq / 1.0f);
    EXPECT_FLOAT_EQ(accel[0], -expected);
    EXPECT_FLOAT_EQ(accel[1], 0.0f);
}

TEST(gravity_well, radial_acceleration_scales_linearly_with_strength) {
    const float pos[2] = {0.0f, 1.0f};
    float accel_full[2]{};
    float accel_half[2]{};
    radial_acceleration(make_well(0.0f, 0.0f, 1.0f), pos, accel_full);
    radial_acceleration(make_well(0.0f, 0.0f, 0.5f), pos, accel_half);
    EXPECT_FLOAT_EQ(accel_half[1], accel_full[1] * 0.5f);
}

TEST(gravity_well, radial_acceleration_exactly_zero_at_zero_strength) {
    const gravity_well well = make_well(0.0f, 0.0f, 0.0f);
    const float pos[2] = {0.5f, 0.5f};
    float accel[2] = {99.0f, 99.0f};
    radial_acceleration(well, pos, accel);
    EXPECT_EQ(accel[0], 0.0f);  // exact, not epsilon (US1 scenario 3)
    EXPECT_EQ(accel[1], 0.0f);
}

TEST(gravity_well, radial_acceleration_zero_beyond_influence_radius) {
    gravity_well well = make_well(0.0f, 0.0f, 1.0f);
    well.influence_radius = 2.0f;
    const float pos[2] = {2.5f, 0.0f};
    float accel[2] = {99.0f, 99.0f};
    radial_acceleration(well, pos, accel);
    EXPECT_EQ(accel[0], 0.0f);
    EXPECT_EQ(accel[1], 0.0f);
}

TEST(gravity_well, radial_acceleration_clamped_at_near_zero_distance) {
    const gravity_well well = make_well(0.0f, 0.0f, 1.0f);
    const float pos[2] = {0.01f, 0.0f};  // dist_sq = 1e-4 < kMinDistanceSq
    float accel[2]{};
    radial_acceleration(well, pos, accel);
    // Clamped inverse-square: magnitude never exceeds strength * kMaxPullAccel.
    const float magnitude = std::sqrt(accel[0] * accel[0] + accel[1] * accel[1]);
    EXPECT_LE(magnitude, kMaxPullAccel * 1.0001f);
    EXPECT_FLOAT_EQ(magnitude, kMaxPullAccel);  // at the clamp, full pull applies
    EXPECT_LT(accel[0], 0.0f);                  // still points toward the well
}

TEST(gravity_well, inactive_well_exerts_zero_pull) {
    gravity_well well = make_well(0.0f, 0.0f, 1.0f);
    well.active = false;
    const float pos[2] = {1.0f, 0.0f};
    float accel[2] = {99.0f, 99.0f};
    radial_acceleration(well, pos, accel);
    EXPECT_EQ(accel[0], 0.0f);
    EXPECT_EQ(accel[1], 0.0f);
}

TEST(gravity_well, radial_acceleration_mirror_symmetric) {
    // Article 9 evidence: mirrored inputs produce mirrored outputs.
    const gravity_well well = make_well(0.0f, 0.0f, 0.8f);
    const float right[2] = {1.5f, 0.0f};
    const float left[2] = {-1.5f, 0.0f};
    float accel_right[2]{};
    float accel_left[2]{};
    radial_acceleration(well, right, accel_right);
    radial_acceleration(well, left, accel_left);
    EXPECT_FLOAT_EQ(accel_right[0], -accel_left[0]);
    EXPECT_FLOAT_EQ(accel_right[1], accel_left[1]);
}

TEST(gravity_well, is_within_capture_inside_true_outside_and_boundary_false) {
    gravity_well well = make_well(0.0f, 0.0f, 1.0f);
    well.capture_radius = 0.5f;

    const float inside[2] = {0.3f, 0.0f};
    const float outside[2] = {0.7f, 0.0f};
    const float boundary[2] = {0.5f, 0.0f};  // exactly on the radius: NOT captured
    EXPECT_TRUE(is_within_capture(well, inside));
    EXPECT_FALSE(is_within_capture(well, outside));
    EXPECT_FALSE(is_within_capture(well, boundary));
}

TEST(gravity_well, is_within_capture_false_for_inactive_well) {
    gravity_well well = make_well(0.0f, 0.0f, 1.0f);
    well.capture_radius = 0.5f;
    well.active = false;
    const float inside[2] = {0.1f, 0.0f};
    EXPECT_FALSE(is_within_capture(well, inside));
}

TEST(gravity_well, step_well_caps_diagonal_speed_at_max_well_speed) {
    gravity_well well = make_well(0.0f, 0.0f, 0.0f);
    const float steer[2] = {1.0f, 1.0f};  // naive velocity would be sqrt(2)*kMaxWellSpeed
    step_well(well, steer, 1.0f, 10.0f, 1.0 / 60.0);
    const float speed = std::sqrt(well.velocity[0] * well.velocity[0] +
                                  well.velocity[1] * well.velocity[1]);
    EXPECT_LE(speed, kMaxWellSpeed * 1.0001f);
    EXPECT_FLOAT_EQ(well.strength, 1.0f);
}

TEST(gravity_well, step_well_clamps_position_to_arena_bounds) {
    gravity_well well = make_well(0.0f, 0.0f, 0.0f);
    well.position[0] = 0.99f;
    const float steer[2] = {1.0f, 0.0f};
    const float half_extent = 1.0f;
    for (int i = 0; i < 120; ++i) {  // 2 s at full speed — would overshoot the wall
        step_well(well, steer, 0.5f, half_extent, 1.0 / 60.0);
    }
    EXPECT_LE(well.position[0], half_extent);
    EXPECT_FLOAT_EQ(well.position[0], half_extent);  // pinned at the boundary (US1 sc.4)
}

TEST(gravity_well, step_well_clamps_strength_input_to_unit_range) {
    gravity_well well = make_well(0.0f, 0.0f, 0.0f);
    const float steer[2] = {0.0f, 0.0f};
    step_well(well, steer, 5.0f, 1.0f, 1.0 / 60.0);
    EXPECT_FLOAT_EQ(well.strength, 1.0f);
    step_well(well, steer, -3.0f, 1.0f, 1.0 / 60.0);
    EXPECT_FLOAT_EQ(well.strength, 0.0f);
}

TEST(gravity_well, step_well_inactive_well_never_moves) {
    gravity_well well = make_well(0.25f, -0.5f, 0.3f);
    well.active = false;
    const float steer[2] = {1.0f, 1.0f};
    step_well(well, steer, 1.0f, 5.0f, 1.0 / 60.0);
    EXPECT_FLOAT_EQ(well.position[0], 0.25f);
    EXPECT_FLOAT_EQ(well.position[1], -0.5f);
    EXPECT_FLOAT_EQ(well.velocity[0], 0.0f);
    EXPECT_FLOAT_EQ(well.strength, 0.3f);  // unchanged
}

}  // namespace
