// GoogleTest for engine_demo::vfx::emitter — User Story 1 (Emit a Visual Effect Burst).
// See specs/004-particle-vfx-subsystem/contracts/emitter.md for the full contract.
//
// Independence check (FR-005, FR-012): this file never includes or constructs
// engine_demo::physics::constraint_solver or engine_demo::ecs::world.

#include "engine_demo/vfx/emitter.h"

#include <array>
#include <chrono>
#include <cmath>
#include <gtest/gtest.h>

namespace {

using engine_demo::allocator;
using engine_demo::vfx::cone_shape;
using engine_demo::vfx::emit_result;
using engine_demo::vfx::emitter;
using engine_demo::vfx::emitter_config;
using engine_demo::vfx::gravity_force;
using engine_demo::vfx::particle;
using engine_demo::vfx::particle_pool;
using engine_demo::vfx::point_shape;
using engine_demo::vfx::sphere_shape;
using engine_demo::vfx::vfx_status;
using engine_demo::vfx::wind_force;

emitter_config make_point_config(std::uint64_t seed) {
    emitter_config cfg{};
    cfg.shape = point_shape{{0.0f, 0.0f, 0.0f}};
    cfg.seed = seed;
    cfg.speed_min = 2.0f;
    cfg.speed_max = 2.0f;
    cfg.lifetime_min_seconds = 1.0;
    cfg.lifetime_max_seconds = 1.0;
    cfg.size_min = 1.0f;
    cfg.size_max = 1.0f;
    return cfg;
}

TEST(emitter, try_emit_point_shape_burst_initializes_all_particles) {
    std::array<std::byte, 8192> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 64};
    emitter e{alloc, pool, make_point_config(1)};

    emit_result result = e.try_emit(20);

    EXPECT_EQ(result.status, vfx_status::ok);
    EXPECT_EQ(result.spawned, 20u);
    EXPECT_EQ(pool.live_count(), 20u);
}

TEST(emitter, tick_advances_position_and_decreases_lifetime) {
    std::array<std::byte, 8192> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 8};
    emitter e{alloc, pool, make_point_config(2)};

    ASSERT_EQ(e.try_emit(1).status, vfx_status::ok);
    particle before = pool.live_particles()[0];

    e.tick(0.1);

    particle after = pool.live_particles()[0];
    EXPECT_DOUBLE_EQ(after.remaining_lifetime_seconds, before.remaining_lifetime_seconds - 0.1);
    // With zero forces attached, velocity is unchanged and position advances by velocity*dt.
    EXPECT_FLOAT_EQ(after.position[0], before.position[0] + before.velocity[0] * 0.1f);
    EXPECT_FLOAT_EQ(after.position[1], before.position[1] + before.velocity[1] * 0.1f);
}

// Edge case (FR-006): a request larger than free capacity is rejected entirely.
TEST(emitter, try_emit_beyond_free_capacity_spawns_nothing) {
    std::array<std::byte, 8192> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 5};
    emitter e{alloc, pool, make_point_config(3)};

    ASSERT_EQ(e.try_emit(3).status, vfx_status::ok);
    ASSERT_EQ(pool.live_count(), 3u);

    emit_result result = e.try_emit(3);  // only 2 free slots remain
    EXPECT_EQ(result.status, vfx_status::pool_exhausted);
    EXPECT_EQ(result.spawned, 0u);
    EXPECT_EQ(pool.live_count(), 3u) << "no particle from the rejected request may become active";
}

// Edge case (research.md §7): a request larger than the pool's total capacity is
// rejected without consuming m_rng — proven indirectly via a determinism comparison.
TEST(emitter, try_emit_beyond_total_capacity_does_not_consume_rng) {
    std::array<std::byte, 8192> buffer_a{};
    std::array<std::byte, 8192> buffer_b{};
    allocator alloc_a{buffer_a.data(), buffer_a.size()};
    allocator alloc_b{buffer_b.data(), buffer_b.size()};
    particle_pool pool_a{alloc_a, 5};
    particle_pool pool_b{alloc_b, 5};
    emitter emitter_a{alloc_a, pool_a, make_point_config(4)};
    emitter emitter_b{alloc_b, pool_b, make_point_config(4)};

    // emitter_a makes an over-capacity request first; emitter_b does not.
    emit_result rejected = emitter_a.try_emit(10);
    EXPECT_EQ(rejected.status, vfx_status::pool_exhausted);
    EXPECT_EQ(rejected.spawned, 0u);

    // Both now issue an identical valid request; if the rejected call above consumed no
    // rng draws, the two emitters must still produce byte-identical spawn results.
    emit_result result_a = emitter_a.try_emit(3);
    emit_result result_b = emitter_b.try_emit(3);
    ASSERT_EQ(result_a.status, vfx_status::ok);
    ASSERT_EQ(result_b.status, vfx_status::ok);

    eastl::span<const particle> live_a = pool_a.live_particles();
    eastl::span<const particle> live_b = pool_b.live_particles();
    ASSERT_EQ(live_a.size(), live_b.size());
    for (std::size_t i = 0; i < live_a.size(); ++i) {
        EXPECT_FLOAT_EQ(live_a[i].position[0], live_b[i].position[0]);
        EXPECT_FLOAT_EQ(live_a[i].velocity[0], live_b[i].velocity[0]);
    }
}

// Edge case (FR-013): a non-positive sampled lifetime is silently skipped, not a failure.
TEST(emitter, try_emit_skips_non_positive_lifetime_candidates) {
    std::array<std::byte, 8192> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 8};

    emitter_config cfg = make_point_config(5);
    cfg.lifetime_min_seconds = -1.0;
    cfg.lifetime_max_seconds = -1.0;  // every candidate is guaranteed non-positive
    emitter e{alloc, pool, cfg};

    emit_result result = e.try_emit(4);
    EXPECT_EQ(result.status, vfx_status::ok);
    EXPECT_EQ(result.spawned, 0u) << "all candidates should have been skipped, not spawned";
    EXPECT_EQ(pool.live_count(), 0u);
}

// Edge case: tick(0.0) is a no-op.
TEST(emitter, tick_zero_dt_is_noop) {
    std::array<std::byte, 8192> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 4};
    emitter e{alloc, pool, make_point_config(6)};

    ASSERT_EQ(e.try_emit(1).status, vfx_status::ok);
    particle before = pool.live_particles()[0];

    e.tick(0.0);

    particle after = pool.live_particles()[0];
    EXPECT_DOUBLE_EQ(after.remaining_lifetime_seconds, before.remaining_lifetime_seconds);
    EXPECT_FLOAT_EQ(after.position[0], before.position[0]);
    EXPECT_FLOAT_EQ(after.position[1], before.position[1]);
}

TEST(emitter, add_force_succeeds_up_to_capacity_then_reports_invalid_argument) {
    std::array<std::byte, 8192> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 4};
    emitter e{alloc, pool, make_point_config(7)};

    // Capacity is 4 (kMaxForces); the first 4 adds must succeed.
    EXPECT_EQ(e.add_force(gravity_force{}), vfx_status::ok);
    EXPECT_EQ(e.add_force(wind_force{}), vfx_status::ok);
    EXPECT_EQ(e.add_force(gravity_force{}), vfx_status::ok);
    EXPECT_EQ(e.add_force(wind_force{}), vfx_status::ok);

    // The 5th add must fail without disturbing the previously attached forces.
    EXPECT_EQ(e.add_force(gravity_force{}), vfx_status::invalid_argument);

    ASSERT_EQ(e.try_emit(1).status, vfx_status::ok);
    particle before = pool.live_particles()[0];
    e.tick(0.1);
    particle after = pool.live_particles()[0];
    // 2x gravity{0,-9.81,0} + 2x wind{0,0,0} = {0, -19.62, 0} total acceleration.
    float const expected_vy = before.velocity[1] + (-19.62f) * 0.1f;
    EXPECT_NEAR(after.velocity[1], expected_vy, 1e-4f);
}

// ---------------------------------------------------------------------------------
// User Story 2 - Shape Particle Motion with Composable Forces
// ---------------------------------------------------------------------------------

TEST(emitter, cone_shape_directions_stay_within_half_angle) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 200};

    emitter_config cfg = make_point_config(10);
    cone_shape shape{};
    shape.origin[0] = 0.0f;
    shape.origin[1] = 0.0f;
    shape.origin[2] = 0.0f;
    shape.direction[0] = 0.0f;
    shape.direction[1] = 1.0f;
    shape.direction[2] = 0.0f;
    shape.half_angle_radians = 0.3f;
    cfg.shape = shape;
    cfg.speed_min = 1.0f;
    cfg.speed_max = 1.0f;
    emitter e{alloc, pool, cfg};

    ASSERT_EQ(e.try_emit(100).status, vfx_status::ok);
    float const cos_half_angle = std::cos(0.3f);

    for (const particle& p : pool.live_particles()) {
        // velocity == direction * speed (speed == 1), so velocity is already the unit
        // direction vector; dot with the cone's axis must stay within half_angle_radians.
        float const dot = p.velocity[1];  // direction (0,1,0) dot velocity == velocity.y
        EXPECT_GE(dot, cos_half_angle - 1e-4f)
            << "sampled direction fell outside the configured cone half-angle";
    }
}

TEST(emitter, sphere_shape_positions_stay_within_radius) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 200};

    emitter_config cfg = make_point_config(11);
    sphere_shape shape{};
    shape.center[0] = 5.0f;
    shape.center[1] = -2.0f;
    shape.center[2] = 0.0f;
    shape.radius = 3.0f;
    cfg.shape = shape;
    emitter e{alloc, pool, cfg};

    ASSERT_EQ(e.try_emit(100).status, vfx_status::ok);

    for (const particle& p : pool.live_particles()) {
        float const dx = p.position[0] - shape.center[0];
        float const dy = p.position[1] - shape.center[1];
        float const dz = p.position[2] - shape.center[2];
        float const dist_sq = dx * dx + dy * dy + dz * dz;
        EXPECT_LE(dist_sq, shape.radius * shape.radius + 1e-3f)
            << "sampled position fell outside the configured sphere radius";
    }
}

TEST(emitter, gravity_and_wind_compose_additively_over_multiple_ticks) {
    std::array<std::byte, 8192> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 4};
    emitter e{alloc, pool, make_point_config(12)};

    ASSERT_EQ(e.add_force(gravity_force{{0.0f, -9.81f, 0.0f}}), vfx_status::ok);
    ASSERT_EQ(e.add_force(wind_force{{2.0f, 0.0f, 0.0f}}), vfx_status::ok);
    ASSERT_EQ(e.try_emit(1).status, vfx_status::ok);

    particle const initial = pool.live_particles()[0];
    double const dt = 0.1;
    for (int i = 0; i < 3; ++i) {
        e.tick(dt);
    }

    particle const after = pool.live_particles()[0];
    float const expected_vx = initial.velocity[0] + 2.0f * static_cast<float>(3 * dt);
    float const expected_vy = initial.velocity[1] + (-9.81f) * static_cast<float>(3 * dt);
    EXPECT_NEAR(after.velocity[0], expected_vx, 1e-3f);
    EXPECT_NEAR(after.velocity[1], expected_vy, 1e-3f);
}

// ---------------------------------------------------------------------------------
// User Story 3 (SC-002) - perf smoke test: 500 live particles with gravity + wind +
// turbulence attached must tick comfortably under the 2 ms/frame budget.
// ---------------------------------------------------------------------------------

TEST(emitter, tick_500_particles_with_forces_stays_well_under_frame_budget) {
    std::array<std::byte, 131072> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 500};

    emitter_config cfg = make_point_config(20);
    cfg.lifetime_min_seconds = 100.0;
    cfg.lifetime_max_seconds = 100.0;  // long-lived so none retire mid-benchmark
    emitter e{alloc, pool, cfg};
    ASSERT_EQ(e.add_force(gravity_force{}), vfx_status::ok);
    ASSERT_EQ(e.add_force(wind_force{{1.0f, 0.0f, 0.0f}}), vfx_status::ok);
    ASSERT_EQ(e.add_force(engine_demo::vfx::turbulence_force{0.5f, 1.0f}), vfx_status::ok);
    ASSERT_EQ(e.try_emit(500).status, vfx_status::ok);
    ASSERT_EQ(pool.live_count(), 500u);

    auto const start = std::chrono::steady_clock::now();
    e.tick(1.0 / 60.0);
    auto const elapsed = std::chrono::steady_clock::now() - start;
    double const elapsed_ms =
        std::chrono::duration<double, std::milli>(elapsed).count();

    // Generous CI-safe bound: the SC-002 budget is 2 ms; assert well above that to avoid
    // flaking on loaded CI hardware while still catching a real regression.
    EXPECT_LT(elapsed_ms, 10.0)
        << "tick() of 500 particles took " << elapsed_ms << " ms (budget: 2 ms nominal)";
}

// SC-006 (emitter-mediated): repeated try_emit/tick through the actual emitter API,
// including its own scratch buffer and force list, never allocates past construction.
TEST(emitter, sustained_emit_and_tick_do_not_grow_allocator_usage) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 32};
    emitter e{alloc, pool, make_point_config(30)};
    ASSERT_EQ(e.add_force(gravity_force{}), vfx_status::ok);
    ASSERT_EQ(e.add_force(wind_force{{1.0f, 0.0f, 0.0f}}), vfx_status::ok);

    ASSERT_EQ(e.try_emit(1).status, vfx_status::ok);  // warm up: force list already sized
    std::size_t const bytes_after_construction = alloc.bytes_used();

    for (int frame = 0; frame < 2000; ++frame) {
        e.tick(1.0 / 60.0);
        (void)e.try_emit(1);  // pool likely full/near-full; ok or pool_exhausted, never allocates
    }

    EXPECT_EQ(alloc.bytes_used(), bytes_after_construction)
        << "no allocation may occur after the emitter's initial construction";
}

}  // namespace
