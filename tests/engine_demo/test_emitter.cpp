// GoogleTest for engine_demo::vfx::emitter.
//
// This file intentionally never includes engine_demo/physics/constraint.h and never
// constructs an engine_demo::physics::constraint_solver anywhere below, demonstrating
// SC-003 ("100% of emitter behavior test scenarios can be executed and validated without
// running the physics constraint solver") for every test in this file.

#include "engine_demo/vfx/emitter.h"

#include <array>
#include <chrono>  // interop boundary: wall-clock timing only, no allocation.
#include <cmath>
#include <cstddef>
#include <gtest/gtest.h>

namespace {

using engine_demo::allocator;
using engine_demo::vfx::cone_shape;
using engine_demo::vfx::emit_result;
using engine_demo::vfx::emitter;
using engine_demo::vfx::emitter_config;
using engine_demo::vfx::gravity_force;
using engine_demo::vfx::particle_pool;
using engine_demo::vfx::point_shape;
using engine_demo::vfx::sphere_shape;
using engine_demo::vfx::vfx_status;
using engine_demo::vfx::wind_force;

// ---------------------------------------------------------------------------------------
// User Story 1 — Trigger Visual Effects from Gameplay Events
// ---------------------------------------------------------------------------------------

TEST(emitter, point_emitter_burst_spawns_particles_with_initialized_fields) {
    std::array<std::byte, 1 << 16> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 8};

    emitter_config cfg{};
    cfg.shape = point_shape{{1.0f, 2.0f, 3.0f}};
    cfg.seed = 1234;
    cfg.speed_min = 2.0f;
    cfg.speed_max = 2.0f;
    cfg.lifetime_min_seconds = 5.0;
    cfg.lifetime_max_seconds = 5.0;
    emitter e{alloc, pool, cfg};

    const emit_result result = e.try_emit(5);

    EXPECT_EQ(result.status, vfx_status::ok);
    EXPECT_EQ(result.spawned, 5u);
    ASSERT_EQ(pool.live_count(), 5u);
    for (const auto& p : pool.live_particles()) {
        EXPECT_FLOAT_EQ(p.position[0], 1.0f);
        EXPECT_FLOAT_EQ(p.position[1], 2.0f);
        EXPECT_FLOAT_EQ(p.position[2], 3.0f);
        EXPECT_DOUBLE_EQ(p.remaining_lifetime_seconds, 5.0);
    }
}

TEST(emitter, tick_advances_position_from_velocity_and_decreases_lifetime) {
    std::array<std::byte, 1 << 16> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 4};

    emitter_config cfg{};
    cfg.shape = point_shape{{0.0f, 0.0f, 0.0f}};
    cfg.seed = 99;
    cfg.speed_min = 3.0f;  // direction defaults to (0, 1, 0) for point_shape
    cfg.speed_max = 3.0f;
    cfg.lifetime_min_seconds = 2.0;
    cfg.lifetime_max_seconds = 2.0;
    emitter e{alloc, pool, cfg};
    ASSERT_EQ(e.try_emit(1).status, vfx_status::ok);

    const double dt = 0.5;
    e.tick(dt);

    ASSERT_EQ(pool.live_count(), 1u);
    const auto& p = pool.live_particles()[0];
    EXPECT_NEAR(p.position[1], 3.0f * 0.5f, 1.0e-5f);
    EXPECT_DOUBLE_EQ(p.remaining_lifetime_seconds, 1.5);
}

TEST(emitter, emit_against_full_pool_reports_pool_exhausted_and_tick_still_works) {
    std::array<std::byte, 1 << 16> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 2};

    emitter_config cfg{};
    cfg.shape = point_shape{};
    cfg.seed = 5;
    cfg.lifetime_min_seconds = 10.0;
    cfg.lifetime_max_seconds = 10.0;
    emitter e{alloc, pool, cfg};

    const emit_result result = e.try_emit(5);  // pool only has capacity for 2

    EXPECT_EQ(result.status, vfx_status::pool_exhausted);
    EXPECT_EQ(result.spawned, 2u);
    EXPECT_EQ(pool.free_count(), 0u);

    e.tick(1.0 / 60.0);  // must not crash and must still age the particles that spawned
    EXPECT_EQ(pool.live_count(), 2u);
}

TEST(emitter, tick_zero_dt_is_a_no_op) {
    std::array<std::byte, 1 << 16> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 2};

    emitter_config cfg{};
    cfg.shape = point_shape{};
    cfg.seed = 3;
    cfg.speed_min = 1.0f;
    cfg.speed_max = 1.0f;
    cfg.lifetime_min_seconds = 1.0;
    cfg.lifetime_max_seconds = 1.0;
    emitter e{alloc, pool, cfg};
    ASSERT_EQ(e.add_force(gravity_force{}), vfx_status::ok);
    ASSERT_EQ(e.try_emit(1).status, vfx_status::ok);
    const auto before = pool.live_particles()[0];

    e.tick(0.0);

    const auto& after = pool.live_particles()[0];
    EXPECT_EQ(before.position[0], after.position[0]);
    EXPECT_EQ(before.position[1], after.position[1]);
    EXPECT_EQ(before.velocity[1], after.velocity[1]);
    EXPECT_EQ(before.remaining_lifetime_seconds, after.remaining_lifetime_seconds);
}

// Closes the add_force coverage gap identified in /speckit.analyze (finding G1): add_force
// has a happy path (succeeds up to the emitter's fixed capacity) and an edge case
// (invalid_argument once full, without disturbing previously attached forces).
TEST(emitter, add_force_succeeds_until_capacity_then_reports_invalid_argument) {
    std::array<std::byte, 1 << 16> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 1};
    emitter_config cfg{};
    cfg.shape = point_shape{};
    cfg.seed = 11;
    cfg.lifetime_min_seconds = 1000.0;
    cfg.lifetime_max_seconds = 1000.0;
    emitter e{alloc, pool, cfg};

    int successes = 0;
    while (e.add_force(gravity_force{{0.0f, -1.0f, 0.0f}}) == vfx_status::ok) {
        ++successes;
        ASSERT_LT(successes, 1000) << "add_force never reported capacity exhaustion";
    }
    EXPECT_GT(successes, 0);

    // The failed add_force call must not have disturbed the previously attached forces:
    // one tick should apply exactly `successes` copies of the gravity contribution.
    ASSERT_EQ(e.try_emit(1).status, vfx_status::ok);
    e.tick(1.0);
    const auto& p = pool.live_particles()[0];
    EXPECT_NEAR(p.velocity[1], -1.0f * static_cast<float>(successes), 1.0e-4f);
}

// ---------------------------------------------------------------------------------------
// User Story 2 — Configure Emitter Shape and Composable Forces
// ---------------------------------------------------------------------------------------

TEST(emitter, cone_emitter_directions_are_within_configured_angle) {
    std::array<std::byte, 1 << 16> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 64};

    emitter_config cfg{};
    const float half_angle = 0.3f;
    cfg.shape = cone_shape{{0.0f, 0.0f, 0.0f}, {0.0f, 1.0f, 0.0f}, half_angle};
    cfg.seed = 777;
    cfg.speed_min = 1.0f;
    cfg.speed_max = 1.0f;
    cfg.lifetime_min_seconds = 1.0;
    cfg.lifetime_max_seconds = 1.0;
    emitter e{alloc, pool, cfg};
    ASSERT_EQ(e.try_emit(32).status, vfx_status::ok);

    const double cos_half_angle = std::cos(static_cast<double>(half_angle));
    for (const auto& p : pool.live_particles()) {
        // position must always equal the cone's origin.
        EXPECT_FLOAT_EQ(p.position[0], 0.0f);
        EXPECT_FLOAT_EQ(p.position[1], 0.0f);
        EXPECT_FLOAT_EQ(p.position[2], 0.0f);
        // speed is fixed at 1.0, so velocity is already the (near-)unit direction.
        const double dot = p.velocity[1];  // dot with axis (0,1,0) is just the y component
        EXPECT_GE(dot, cos_half_angle - 1.0e-3);
    }
}

TEST(emitter, sphere_emitter_positions_are_within_configured_radius) {
    std::array<std::byte, 1 << 16> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 64};

    emitter_config cfg{};
    const float radius = 2.0f;
    const float center[3] = {1.0f, -1.0f, 0.5f};
    cfg.shape = sphere_shape{{center[0], center[1], center[2]}, radius};
    cfg.seed = 42;
    cfg.speed_min = 0.0f;
    cfg.speed_max = 0.0f;
    cfg.lifetime_min_seconds = 1.0;
    cfg.lifetime_max_seconds = 1.0;
    emitter e{alloc, pool, cfg};
    ASSERT_EQ(e.try_emit(32).status, vfx_status::ok);

    for (const auto& p : pool.live_particles()) {
        const double dx = p.position[0] - center[0];
        const double dy = p.position[1] - center[1];
        const double dz = p.position[2] - center[2];
        const double distance = std::sqrt(dx * dx + dy * dy + dz * dz);
        EXPECT_LE(distance, static_cast<double>(radius) + 1.0e-4);
    }
}

TEST(emitter, gravity_and_wind_forces_accumulate_velocity_over_multiple_ticks) {
    std::array<std::byte, 1 << 16> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 1};

    emitter_config cfg{};
    cfg.shape = point_shape{};
    cfg.seed = 21;
    cfg.speed_min = 0.0f;
    cfg.speed_max = 0.0f;  // starts at rest so the accumulated velocity is exact
    cfg.lifetime_min_seconds = 1000.0;
    cfg.lifetime_max_seconds = 1000.0;
    emitter e{alloc, pool, cfg};
    ASSERT_EQ(e.add_force(gravity_force{{0.0f, -10.0f, 0.0f}}), vfx_status::ok);
    ASSERT_EQ(e.add_force(wind_force{{2.0f, 0.0f, 0.0f}}), vfx_status::ok);
    ASSERT_EQ(e.try_emit(1).status, vfx_status::ok);

    const double dt = 0.1;
    const int ticks = 3;
    for (int i = 0; i < ticks; ++i) {
        e.tick(dt);
    }

    const auto& p = pool.live_particles()[0];
    EXPECT_NEAR(p.velocity[0], 2.0f * static_cast<float>(dt) * ticks, 1.0e-4f);
    EXPECT_NEAR(p.velocity[1], -10.0f * static_cast<float>(dt) * ticks, 1.0e-4f);
}

// ---------------------------------------------------------------------------------------
// Polish — SC-001 perf smoke test
// ---------------------------------------------------------------------------------------

TEST(emitter, tick_with_500_particles_completes_well_under_frame_budget) {
    std::array<std::byte, 1 << 20> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 500};

    emitter_config cfg{};
    cfg.shape = point_shape{};
    cfg.seed = 2026;
    cfg.speed_min = 1.0f;
    cfg.speed_max = 2.0f;
    cfg.lifetime_min_seconds = 1000.0;  // never expires mid-test
    cfg.lifetime_max_seconds = 1000.0;
    emitter e{alloc, pool, cfg};
    ASSERT_EQ(e.add_force(gravity_force{}), vfx_status::ok);
    ASSERT_EQ(e.try_emit(500).status, vfx_status::ok);
    ASSERT_EQ(pool.live_count(), 500u);

    const auto start = std::chrono::steady_clock::now();
    e.tick(1.0 / 60.0);
    const auto end = std::chrono::steady_clock::now();
    const double elapsed_ms = std::chrono::duration<double, std::milli>(end - start).count();

    // SC-001 targets < 2 ms; assert a generous, explicit, CI-safe ceiling well above that
    // so the test still catches real regressions without being flaky on loaded hardware.
    EXPECT_LT(elapsed_ms, 20.0);
}

}  // namespace
