// GoogleTest for engine_demo::ecs::world.

#include "engine_demo/ecs/world.h"

#include <array>
#include <gtest/gtest.h>

namespace {

TEST(world, create_then_destroy_drops_live_count) {
    std::array<std::byte, 1024> buffer{};
    engine_demo::allocator alloc{buffer.data(), buffer.size()};
    engine_demo::ecs::world w{alloc};

    auto h = w.create_entity();
    EXPECT_TRUE(h.is_valid());
    EXPECT_TRUE(w.is_alive(h));
    w.destroy_entity(h);
    EXPECT_EQ(w.live_count(), 0u);
}

// REGRESSION TEST for BUG-003 (CWE-416). Guards the generation bump in destroy_entity.
TEST(world, stale_handle_does_not_revive_after_recycle) {
    std::array<std::byte, 1024> buffer{};
    engine_demo::allocator alloc{buffer.data(), buffer.size()};
    engine_demo::ecs::world w{alloc};

    auto h_old = w.create_entity();
    w.destroy_entity(h_old);

    // create_entity scans forward from the next free slot, so force the pool to wrap
    // and recycle h_old's slot, then grab the handle that lands on the same index.
    engine_demo::ecs::entity_handle h_new{};
    for (std::size_t i = 0; i < 4096; ++i) {
        auto h = w.create_entity();
        if (h.index == h_old.index) {
            h_new = h;
        }
    }
    ASSERT_TRUE(h_new.is_valid()) << "pool wrap must have recycled the destroyed slot";
    EXPECT_FALSE(w.is_alive(h_old)) << "stale handle must not validate against recycled slot";
    EXPECT_TRUE(w.is_alive(h_new));
    EXPECT_NE(h_old.generation, h_new.generation);
}

}  // namespace
