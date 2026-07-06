// GoogleTest for engine_demo::vfx::particle_pool.

#include "engine_demo/vfx/particle.h"

#include <array>
#include <cstddef>
#include <gtest/gtest.h>

namespace {

using engine_demo::allocator;
using engine_demo::vfx::emit_result;
using engine_demo::vfx::particle_pool;
using engine_demo::vfx::spawn_params;
using engine_demo::vfx::vfx_status;

spawn_params make_params(float x, double lifetime) noexcept {
    spawn_params p{};
    p.position[0] = x;
    p.position[1] = 0.0f;
    p.position[2] = 0.0f;
    p.velocity[0] = 1.0f;
    p.velocity[1] = 2.0f;
    p.velocity[2] = 3.0f;
    p.lifetime_seconds = lifetime;
    p.color[0] = 0.5f;
    p.color[1] = 0.6f;
    p.color[2] = 0.7f;
    p.color[3] = 1.0f;
    p.size = 2.0f;
    return p;
}

TEST(particle_pool, spawn_within_capacity_activates_particles_with_initialized_fields) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 4};

    const spawn_params params[] = {make_params(1.0f, 1.0), make_params(2.0f, 2.0)};
    const emit_result result = pool.try_spawn(params);

    EXPECT_EQ(result.status, vfx_status::ok);
    EXPECT_EQ(result.spawned, 2u);
    ASSERT_EQ(pool.live_count(), 2u);
    EXPECT_EQ(pool.free_count(), 2u);

    const auto live = pool.live_particles();
    EXPECT_FLOAT_EQ(live[0].position[0], 1.0f);
    EXPECT_DOUBLE_EQ(live[0].remaining_lifetime_seconds, 1.0);
    EXPECT_FLOAT_EQ(live[0].velocity[1], 2.0f);
    EXPECT_FLOAT_EQ(live[0].size, 2.0f);
    EXPECT_FLOAT_EQ(live[1].position[0], 2.0f);
    EXPECT_DOUBLE_EQ(live[1].remaining_lifetime_seconds, 2.0);
}

TEST(particle_pool, spawn_beyond_free_capacity_reports_pool_exhausted_and_preserves_existing) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 2};

    const spawn_params first[] = {make_params(1.0f, 1.0)};
    ASSERT_EQ(pool.try_spawn(first).status, vfx_status::ok);

    const spawn_params overflow[] = {make_params(9.0f, 9.0), make_params(9.0f, 9.0)};
    const emit_result result = pool.try_spawn(overflow);

    EXPECT_EQ(result.status, vfx_status::pool_exhausted);
    EXPECT_EQ(result.spawned, 1u);  // only 1 free slot remained
    EXPECT_EQ(pool.free_count(), 0u);
    // The pre-existing particle must be untouched by the failed portion of the request.
    EXPECT_FLOAT_EQ(pool.live_particles()[0].position[0], 1.0f);
}

TEST(particle_pool, spawn_against_zero_capacity_pool_reports_zero_spawned) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 0};

    const spawn_params params[] = {make_params(1.0f, 1.0)};
    const emit_result result = pool.try_spawn(params);

    EXPECT_EQ(result.status, vfx_status::pool_exhausted);
    EXPECT_EQ(result.spawned, 0u);
    EXPECT_EQ(pool.capacity(), 0u);
}

TEST(particle_pool, age_and_retire_zero_dt_is_a_no_op) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 2};
    const spawn_params params[] = {make_params(1.0f, 1.0)};
    ASSERT_EQ(pool.try_spawn(params).status, vfx_status::ok);

    pool.age_and_retire(0.0);

    ASSERT_EQ(pool.live_count(), 1u);
    EXPECT_DOUBLE_EQ(pool.live_particles()[0].remaining_lifetime_seconds, 1.0);
}

TEST(particle_pool, age_and_retire_retires_particle_whose_lifetime_reaches_exactly_zero) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 2};
    const spawn_params params[] = {make_params(1.0f, 0.5)};
    ASSERT_EQ(pool.try_spawn(params).status, vfx_status::ok);

    pool.age_and_retire(0.5);  // remaining_lifetime_seconds becomes exactly 0.0

    EXPECT_EQ(pool.live_count(), 0u);
    EXPECT_EQ(pool.free_count(), 2u);
}

TEST(particle_pool, age_and_retire_advances_surviving_particles) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 2};
    const spawn_params params[] = {make_params(1.0f, 5.0)};
    ASSERT_EQ(pool.try_spawn(params).status, vfx_status::ok);

    pool.age_and_retire(1.0);

    ASSERT_EQ(pool.live_count(), 1u);
    EXPECT_DOUBLE_EQ(pool.live_particles()[0].remaining_lifetime_seconds, 4.0);
}

// SC-002: sustained continuous spawning against an exhausted pool always reports
// pool_exhausted and never crashes.
TEST(particle_pool, sustained_spawn_against_exhausted_pool_always_reports_exhaustion) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 1};

    const spawn_params fill[] = {make_params(0.0f, 1.0e9)};  // never expires during this test
    ASSERT_EQ(pool.try_spawn(fill).status, vfx_status::ok);
    ASSERT_EQ(pool.free_count(), 0u);

    for (int frame = 0; frame < 10000; ++frame) {
        const spawn_params attempt[] = {make_params(1.0f, 1.0)};
        const emit_result result = pool.try_spawn(attempt);
        ASSERT_EQ(result.status, vfx_status::pool_exhausted);
        ASSERT_EQ(result.spawned, 0u);
    }
    EXPECT_EQ(pool.live_count(), 1u);
}

// FR-008 / Article 6: steady-state operations must not allocate.
TEST(particle_pool, steady_state_spawn_and_retire_perform_no_allocation) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 8};  // construction may allocate

    const std::size_t bytes_before = alloc.bytes_used();
    const spawn_params params[] = {make_params(1.0f, 1.0), make_params(2.0f, 1.0)};
    ASSERT_EQ(pool.try_spawn(params).status, vfx_status::ok);
    pool.age_and_retire(0.1);
    const std::size_t bytes_after = alloc.bytes_used();

    EXPECT_EQ(bytes_before, bytes_after);
}

// FR-010 / SC-004: identical call sequences must yield identical resulting state.
TEST(particle_pool, identical_call_sequence_produces_identical_state_across_pools) {
    std::array<std::byte, 4096> buffer_a{};
    std::array<std::byte, 4096> buffer_b{};
    allocator alloc_a{buffer_a.data(), buffer_a.size()};
    allocator alloc_b{buffer_b.data(), buffer_b.size()};
    particle_pool pool_a{alloc_a, 4};
    particle_pool pool_b{alloc_b, 4};

    const spawn_params params[] = {make_params(1.0f, 1.0), make_params(2.0f, 0.4)};
    ASSERT_EQ(pool_a.try_spawn(params).status, vfx_status::ok);
    ASSERT_EQ(pool_b.try_spawn(params).status, vfx_status::ok);

    pool_a.age_and_retire(0.4);
    pool_b.age_and_retire(0.4);

    const auto live_a = pool_a.live_particles();
    const auto live_b = pool_b.live_particles();
    ASSERT_EQ(live_a.size(), live_b.size());
    for (std::size_t i = 0; i < live_a.size(); ++i) {
        EXPECT_EQ(live_a[i].remaining_lifetime_seconds, live_b[i].remaining_lifetime_seconds);
        EXPECT_EQ(live_a[i].position[0], live_b[i].position[0]);
        EXPECT_EQ(live_a[i].position[1], live_b[i].position[1]);
        EXPECT_EQ(live_a[i].position[2], live_b[i].position[2]);
    }
}

}  // namespace
