// GoogleTest for engine_demo::vfx determinism — User Story 3 (Replay-Safe, Real-Time
// Particle Simulation at Scale). See
// specs/004-particle-vfx-subsystem/contracts/emitter.md determinism obligations and
// FR-010 / SC-003.

#include "engine_demo/vfx/emitter.h"

#include <array>
#include <gtest/gtest.h>

namespace {

using engine_demo::allocator;
using engine_demo::vfx::cone_shape;
using engine_demo::vfx::emitter;
using engine_demo::vfx::emitter_config;
using engine_demo::vfx::gravity_force;
using engine_demo::vfx::particle;
using engine_demo::vfx::particle_pool;
using engine_demo::vfx::turbulence_force;
using engine_demo::vfx::vfx_status;
using engine_demo::vfx::wind_force;

emitter_config make_replay_config(std::uint64_t seed) {
    emitter_config cfg{};
    cone_shape shape{};
    shape.origin[0] = 0.0f;
    shape.origin[1] = 0.0f;
    shape.origin[2] = 0.0f;
    shape.direction[0] = 0.0f;
    shape.direction[1] = 1.0f;
    shape.direction[2] = 0.0f;
    shape.half_angle_radians = 0.4f;
    cfg.shape = shape;
    cfg.seed = seed;
    cfg.speed_min = 1.0f;
    cfg.speed_max = 4.0f;
    cfg.lifetime_min_seconds = 5.0;
    cfg.lifetime_max_seconds = 20.0;  // long-lived so particles persist across 1000 frames
    cfg.size_min = 0.5f;
    cfg.size_max = 1.5f;
    return cfg;
}

void expect_identical_snapshots(eastl::span<const particle> a, eastl::span<const particle> b) {
    ASSERT_EQ(a.size(), b.size());
    for (std::size_t i = 0; i < a.size(); ++i) {
        EXPECT_DOUBLE_EQ(a[i].remaining_lifetime_seconds, b[i].remaining_lifetime_seconds);
        EXPECT_FLOAT_EQ(a[i].position[0], b[i].position[0]);
        EXPECT_FLOAT_EQ(a[i].position[1], b[i].position[1]);
        EXPECT_FLOAT_EQ(a[i].position[2], b[i].position[2]);
        EXPECT_FLOAT_EQ(a[i].velocity[0], b[i].velocity[0]);
        EXPECT_FLOAT_EQ(a[i].velocity[1], b[i].velocity[1]);
        EXPECT_FLOAT_EQ(a[i].velocity[2], b[i].velocity[2]);
        EXPECT_FLOAT_EQ(a[i].size, b[i].size);
    }
}

// FR-010 / SC-003: identical seed + identical call sequence over 1,000 frames produces
// byte-identical particle state.
TEST(vfx_determinism, identical_seed_and_sequence_over_1000_frames_is_byte_identical) {
    std::array<std::byte, 65536> buffer_a{};
    std::array<std::byte, 65536> buffer_b{};
    allocator alloc_a{buffer_a.data(), buffer_a.size()};
    allocator alloc_b{buffer_b.data(), buffer_b.size()};
    particle_pool pool_a{alloc_a, 64};
    particle_pool pool_b{alloc_b, 64};

    emitter_config cfg = make_replay_config(2024);
    emitter emitter_a{alloc_a, pool_a, cfg};
    emitter emitter_b{alloc_b, pool_b, cfg};

    ASSERT_EQ(emitter_a.add_force(gravity_force{{0.0f, -9.81f, 0.0f}}), vfx_status::ok);
    ASSERT_EQ(emitter_a.add_force(wind_force{{1.0f, 0.0f, 0.0f}}), vfx_status::ok);
    ASSERT_EQ(emitter_a.add_force(turbulence_force{0.5f, 1.0f}), vfx_status::ok);
    ASSERT_EQ(emitter_b.add_force(gravity_force{{0.0f, -9.81f, 0.0f}}), vfx_status::ok);
    ASSERT_EQ(emitter_b.add_force(wind_force{{1.0f, 0.0f, 0.0f}}), vfx_status::ok);
    ASSERT_EQ(emitter_b.add_force(turbulence_force{0.5f, 1.0f}), vfx_status::ok);

    constexpr double kFixedStep = 1.0 / 60.0;
    for (int frame = 0; frame < 1000; ++frame) {
        if (frame == 0) {
            ASSERT_EQ(emitter_a.try_emit(20).status, vfx_status::ok);
            ASSERT_EQ(emitter_b.try_emit(20).status, vfx_status::ok);
        }
        emitter_a.tick(kFixedStep);
        emitter_b.tick(kFixedStep);
    }

    expect_identical_snapshots(pool_a.live_particles(), pool_b.live_particles());
}

// FR-010: replaying the same burst from a freshly-constructed, identically-seeded
// emitter reproduces identical initial particle attributes.
TEST(vfx_determinism, replaying_burst_from_fresh_emitter_reproduces_initial_attributes) {
    std::array<std::byte, 65536> buffer_first{};
    allocator alloc_first{buffer_first.data(), buffer_first.size()};
    particle_pool pool_first{alloc_first, 32};
    emitter_config cfg = make_replay_config(777);
    emitter emitter_first{alloc_first, pool_first, cfg};
    ASSERT_EQ(emitter_first.try_emit(15).status, vfx_status::ok);

    std::array<std::byte, 65536> buffer_second{};
    allocator alloc_second{buffer_second.data(), buffer_second.size()};
    particle_pool pool_second{alloc_second, 32};
    emitter emitter_second{alloc_second, pool_second, cfg};
    ASSERT_EQ(emitter_second.try_emit(15).status, vfx_status::ok);

    expect_identical_snapshots(pool_first.live_particles(), pool_second.live_particles());
}

}  // namespace
