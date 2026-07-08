// GoogleTest for cross-run determinism (User Story 3 / SC-004).

#include "engine_demo/vfx/emitter.h"

#include <array>
#include <cstddef>
#include <gtest/gtest.h>

namespace {

using engine_demo::allocator;
using engine_demo::vfx::cone_shape;
using engine_demo::vfx::emitter;
using engine_demo::vfx::emitter_config;
using engine_demo::vfx::gravity_force;
using engine_demo::vfx::particle_pool;
using engine_demo::vfx::sphere_shape;
using engine_demo::vfx::turbulence_force;
using engine_demo::vfx::vfx_status;

// Groups an allocator + particle_pool + emitter so two identically-configured "rigs" can
// be built and driven side by side.
struct rig {
    std::array<std::byte, 1 << 16> buffer{};
    allocator alloc;
    particle_pool pool;
    emitter e;

    rig(std::size_t capacity, const emitter_config& cfg)
        : alloc{buffer.data(), buffer.size()}, pool{alloc, capacity}, e{alloc, pool, cfg} {}
};

TEST(vfx_determinism, identical_seed_and_call_sequence_yields_identical_particle_state) {
    emitter_config cfg{};
    cfg.shape = cone_shape{{0.0f, 0.0f, 0.0f}, {0.0f, 1.0f, 0.0f}, 0.4f};
    cfg.seed = 0xABCDEF0123456789ull;
    cfg.speed_min = 1.0f;
    cfg.speed_max = 3.0f;
    cfg.lifetime_min_seconds = 0.5;
    cfg.lifetime_max_seconds = 2.0;
    cfg.size_min = 0.5f;
    cfg.size_max = 1.5f;

    rig a{32, cfg};
    rig b{32, cfg};
    ASSERT_EQ(a.e.add_force(gravity_force{}), vfx_status::ok);
    ASSERT_EQ(b.e.add_force(gravity_force{}), vfx_status::ok);
    ASSERT_EQ(a.e.add_force(turbulence_force{0.5f, 1.0f}), vfx_status::ok);
    ASSERT_EQ(b.e.add_force(turbulence_force{0.5f, 1.0f}), vfx_status::ok);

    for (int frame = 0; frame < 30; ++frame) {
        if (frame % 5 == 0) {
            ASSERT_EQ(a.e.try_emit(3).status, vfx_status::ok);
            ASSERT_EQ(b.e.try_emit(3).status, vfx_status::ok);
        }
        a.e.tick(1.0 / 60.0);
        b.e.tick(1.0 / 60.0);
    }

    const auto live_a = a.pool.live_particles();
    const auto live_b = b.pool.live_particles();
    ASSERT_EQ(live_a.size(), live_b.size());
    for (std::size_t i = 0; i < live_a.size(); ++i) {
        EXPECT_EQ(live_a[i].remaining_lifetime_seconds, live_b[i].remaining_lifetime_seconds);
        EXPECT_EQ(live_a[i].position[0], live_b[i].position[0]);
        EXPECT_EQ(live_a[i].position[1], live_b[i].position[1]);
        EXPECT_EQ(live_a[i].position[2], live_b[i].position[2]);
        EXPECT_EQ(live_a[i].velocity[0], live_b[i].velocity[0]);
        EXPECT_EQ(live_a[i].velocity[1], live_b[i].velocity[1]);
        EXPECT_EQ(live_a[i].velocity[2], live_b[i].velocity[2]);
        EXPECT_EQ(live_a[i].size, live_b[i].size);
    }
}

TEST(vfx_determinism, replaying_same_burst_from_fresh_emitter_reproduces_initial_attributes) {
    emitter_config cfg{};
    cfg.shape = sphere_shape{{1.0f, 2.0f, 3.0f}, 5.0f};
    cfg.seed = 42;
    cfg.speed_min = 0.0f;
    cfg.speed_max = 4.0f;
    cfg.lifetime_min_seconds = 1.0;
    cfg.lifetime_max_seconds = 1.0;

    rig a{16, cfg};
    rig b{16, cfg};

    ASSERT_EQ(a.e.try_emit(10).status, vfx_status::ok);
    ASSERT_EQ(b.e.try_emit(10).status, vfx_status::ok);

    const auto live_a = a.pool.live_particles();
    const auto live_b = b.pool.live_particles();
    ASSERT_EQ(live_a.size(), 10u);
    ASSERT_EQ(live_b.size(), 10u);
    for (std::size_t i = 0; i < live_a.size(); ++i) {
        EXPECT_EQ(live_a[i].position[0], live_b[i].position[0]);
        EXPECT_EQ(live_a[i].position[1], live_b[i].position[1]);
        EXPECT_EQ(live_a[i].position[2], live_b[i].position[2]);
        EXPECT_EQ(live_a[i].velocity[0], live_b[i].velocity[0]);
    }
}

}  // namespace
