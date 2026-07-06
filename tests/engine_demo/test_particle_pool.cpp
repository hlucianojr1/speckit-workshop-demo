// GoogleTest for engine_demo::vfx::particle_pool.
// See specs/004-particle-vfx-subsystem/contracts/particle_pool.md for the full contract.

#include "engine_demo/vfx/particle.h"

#include <array>
#include <gtest/gtest.h>

namespace {

using engine_demo::allocator;
using engine_demo::vfx::emit_result;
using engine_demo::vfx::particle;
using engine_demo::vfx::particle_pool;
using engine_demo::vfx::spawn_params;
using engine_demo::vfx::vfx_status;

spawn_params make_spawn(float x, double lifetime) {
    spawn_params p{};
    p.position[0] = x;
    p.position[1] = 0.0f;
    p.position[2] = 0.0f;
    p.velocity[0] = 1.0f;
    p.velocity[1] = 2.0f;
    p.velocity[2] = 0.0f;
    p.lifetime_seconds = lifetime;
    p.color[0] = 0.5f;
    p.size = 2.0f;
    return p;
}

TEST(particle_pool, spawn_happy_path_fills_fields_from_spawn_params) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 8};

    spawn_params params[3]{make_spawn(1.0f, 1.0), make_spawn(2.0f, 1.5), make_spawn(3.0f, 2.0)};
    emit_result result = pool.try_spawn(params);

    EXPECT_EQ(result.status, vfx_status::ok);
    EXPECT_EQ(result.spawned, 3u);
    ASSERT_EQ(pool.live_count(), 3u);

    eastl::span<const particle> live = pool.live_particles();
    for (std::size_t i = 0; i < live.size(); ++i) {
        EXPECT_FLOAT_EQ(live[i].position[0], params[i].position[0]);
        EXPECT_FLOAT_EQ(live[i].velocity[0], params[i].velocity[0]);
        EXPECT_DOUBLE_EQ(live[i].remaining_lifetime_seconds, params[i].lifetime_seconds);
        EXPECT_FLOAT_EQ(live[i].color[0], params[i].color[0]);
        EXPECT_FLOAT_EQ(live[i].size, params[i].size);
    }
}

// Edge case (FR-006, research.md §7): all-or-nothing — a request bigger than free
// capacity spawns nothing and leaves existing live particles untouched.
TEST(particle_pool, spawn_beyond_free_capacity_is_all_or_nothing) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 2};

    spawn_params first[1]{make_spawn(9.0f, 1.0)};
    ASSERT_EQ(pool.try_spawn(first).status, vfx_status::ok);
    ASSERT_EQ(pool.live_count(), 1u);

    // free_count() == 1, request 2 -> must be rejected entirely.
    spawn_params too_many[2]{make_spawn(1.0f, 1.0), make_spawn(2.0f, 1.0)};
    emit_result result = pool.try_spawn(too_many);

    EXPECT_EQ(result.status, vfx_status::pool_exhausted);
    EXPECT_EQ(result.spawned, 0u);
    ASSERT_EQ(pool.live_count(), 1u) << "rejected batch must not partially spawn";
    EXPECT_FLOAT_EQ(pool.live_particles()[0].position[0], 9.0f)
        << "existing live particle must be untouched by a rejected request";
}

// Edge case: pool already at capacity (free_count() == 0).
TEST(particle_pool, spawn_when_full_returns_pool_exhausted) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 1};

    spawn_params one[1]{make_spawn(1.0f, 1.0)};
    ASSERT_EQ(pool.try_spawn(one).status, vfx_status::ok);

    spawn_params another[1]{make_spawn(2.0f, 1.0)};
    emit_result result = pool.try_spawn(another);
    EXPECT_EQ(result.status, vfx_status::pool_exhausted);
    EXPECT_EQ(result.spawned, 0u);
    EXPECT_EQ(pool.free_count(), 0u);
}

TEST(particle_pool, age_and_retire_decrements_lifetime_and_retires_at_or_below_zero) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 4};

    spawn_params params[2]{make_spawn(1.0f, 0.35), make_spawn(2.0f, 10.0)};
    ASSERT_EQ(pool.try_spawn(params).status, vfx_status::ok);
    ASSERT_EQ(pool.live_count(), 2u);

    for (int i = 0; i < 5; ++i) {
        pool.age_and_retire(0.1);
    }

    // The 0.35s-lifetime particle must have retired (0.35 - 5*0.1 = -0.15 < 0), leaving
    // only the 10.0s-lifetime particle live.
    ASSERT_EQ(pool.live_count(), 1u);
    EXPECT_FLOAT_EQ(pool.live_particles()[0].position[0], 2.0f);
}

// Edge case: dt == 0.0 is a no-op (paused frame).
TEST(particle_pool, age_and_retire_zero_dt_is_noop) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 2};

    spawn_params params[1]{make_spawn(1.0f, 1.0)};
    ASSERT_EQ(pool.try_spawn(params).status, vfx_status::ok);

    double const lifetime_before = pool.live_particles()[0].remaining_lifetime_seconds;
    pool.age_and_retire(0.0);

    ASSERT_EQ(pool.live_count(), 1u);
    EXPECT_DOUBLE_EQ(pool.live_particles()[0].remaining_lifetime_seconds, lifetime_before);
}

// Determinism (FR-010): identical call sequences produce byte-identical state.
TEST(particle_pool, identical_call_sequence_is_deterministic) {
    std::array<std::byte, 4096> buffer_a{};
    std::array<std::byte, 4096> buffer_b{};
    allocator alloc_a{buffer_a.data(), buffer_a.size()};
    allocator alloc_b{buffer_b.data(), buffer_b.size()};
    particle_pool pool_a{alloc_a, 8};
    particle_pool pool_b{alloc_b, 8};

    spawn_params params[3]{make_spawn(1.0f, 0.5), make_spawn(2.0f, 1.0), make_spawn(3.0f, 1.5)};
    ASSERT_EQ(pool_a.try_spawn(params).status, vfx_status::ok);
    ASSERT_EQ(pool_b.try_spawn(params).status, vfx_status::ok);

    for (int i = 0; i < 10; ++i) {
        pool_a.age_and_retire(0.1);
        pool_b.age_and_retire(0.1);
    }

    ASSERT_EQ(pool_a.live_count(), pool_b.live_count());
    eastl::span<const particle> live_a = pool_a.live_particles();
    eastl::span<const particle> live_b = pool_b.live_particles();
    for (std::size_t i = 0; i < live_a.size(); ++i) {
        EXPECT_DOUBLE_EQ(live_a[i].remaining_lifetime_seconds, live_b[i].remaining_lifetime_seconds);
        EXPECT_FLOAT_EQ(live_a[i].position[0], live_b[i].position[0]);
        EXPECT_FLOAT_EQ(live_a[i].position[1], live_b[i].position[1]);
        EXPECT_FLOAT_EQ(live_a[i].velocity[0], live_b[i].velocity[0]);
    }
}

// SC-006: sustained run (10,000+ frames) at full capacity never crashes, always reports
// pool_exhausted for over-capacity attempts, and never grows allocator usage past the
// pool's one-time construction reserve (Article 6).
TEST(particle_pool, sustained_run_at_capacity_never_allocates_and_never_crashes) {
    std::array<std::byte, 4096> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    particle_pool pool{alloc, 4};

    spawn_params fill[4]{make_spawn(1.0f, 1000.0), make_spawn(2.0f, 1000.0),
                         make_spawn(3.0f, 1000.0), make_spawn(4.0f, 1000.0)};
    ASSERT_EQ(pool.try_spawn(fill).status, vfx_status::ok);
    ASSERT_EQ(pool.free_count(), 0u);

    std::size_t const bytes_after_construction = alloc.bytes_used();

    spawn_params extra[1]{make_spawn(5.0f, 1.0)};
    for (int frame = 0; frame < 10000; ++frame) {
        pool.age_and_retire(0.0);  // long-lived particles never retire; pool stays full
        emit_result over_capacity = pool.try_spawn(extra);
        EXPECT_EQ(over_capacity.status, vfx_status::pool_exhausted);
        EXPECT_EQ(over_capacity.spawned, 0u);
    }

    EXPECT_EQ(pool.live_count(), 4u);
    EXPECT_EQ(alloc.bytes_used(), bytes_after_construction)
        << "no allocation may occur after the pool's initial construction";
}

}  // namespace
