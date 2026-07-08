// GoogleTest for orbital_arena input module (T002 — written before the implementation,
// per Constitution Article 7 test-first discipline).

#include "orbital_arena/input.h"

#include <array>
#include <cstddef>
#include <gtest/gtest.h>
#include <limits>

namespace {

using engine_demo::allocator;
using orbital_arena::arena_status;
using orbital_arena::clamp_input;
using orbital_arena::input_frame;
using orbital_arena::input_log;
using orbital_arena::kMaxPlayers;
using orbital_arena::tick_inputs;

input_frame make_frame(float sx, float sy, float strength) noexcept {
    input_frame f{};
    f.steer[0] = sx;
    f.steer[1] = sy;
    f.strength = strength;
    return f;
}

TEST(input, clamp_input_clamps_out_of_range_steer_and_strength) {
    const input_frame clamped = clamp_input(make_frame(3.0f, -7.5f, 42.0f));
    EXPECT_FLOAT_EQ(clamped.steer[0], 1.0f);
    EXPECT_FLOAT_EQ(clamped.steer[1], -1.0f);
    EXPECT_FLOAT_EQ(clamped.strength, 1.0f);

    const input_frame negative = clamp_input(make_frame(-2.0f, 0.25f, -0.5f));
    EXPECT_FLOAT_EQ(negative.steer[0], -1.0f);
    EXPECT_FLOAT_EQ(negative.steer[1], 0.25f);
    EXPECT_FLOAT_EQ(negative.strength, 0.0f);
}

TEST(input, clamp_input_passes_in_range_values_untouched) {
    const input_frame same = clamp_input(make_frame(-0.5f, 1.0f, 0.75f));
    EXPECT_FLOAT_EQ(same.steer[0], -0.5f);
    EXPECT_FLOAT_EQ(same.steer[1], 1.0f);
    EXPECT_FLOAT_EQ(same.strength, 0.75f);
}

TEST(input, clamp_input_maps_nan_to_zero_deterministically) {
    const float nan_value = std::numeric_limits<float>::quiet_NaN();
    const input_frame cleaned = clamp_input(make_frame(nan_value, nan_value, nan_value));
    EXPECT_FLOAT_EQ(cleaned.steer[0], 0.0f);
    EXPECT_FLOAT_EQ(cleaned.steer[1], 0.0f);
    EXPECT_FLOAT_EQ(cleaned.strength, 0.0f);
}

TEST(input, log_round_trips_frames_in_order_with_identity_player_mapping) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    input_log log{alloc, 8};

    for (std::size_t tick = 0; tick < 3; ++tick) {
        tick_inputs frame{};
        for (std::uint8_t player = 0; player < kMaxPlayers; ++player) {
            // Player i's frame always sits at slot i (FR-015 fixed identity mapping).
            frame.players[player] =
                make_frame(static_cast<float>(tick) * 0.1f, static_cast<float>(player) * 0.2f,
                           0.5f);
        }
        ASSERT_EQ(log.append(frame), arena_status::ok);
    }

    ASSERT_EQ(log.size(), 3u);
    for (std::size_t tick = 0; tick < 3; ++tick) {
        const tick_inputs& frame = log.at(tick);
        for (std::uint8_t player = 0; player < kMaxPlayers; ++player) {
            EXPECT_FLOAT_EQ(frame.players[player].steer[0], static_cast<float>(tick) * 0.1f);
            EXPECT_FLOAT_EQ(frame.players[player].steer[1], static_cast<float>(player) * 0.2f);
        }
    }
}

TEST(input, log_full_at_capacity_drops_frame_and_size_stops_growing) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    input_log log{alloc, 2};

    tick_inputs frame{};
    ASSERT_EQ(log.append(frame), arena_status::ok);
    ASSERT_EQ(log.append(frame), arena_status::ok);
    EXPECT_EQ(log.append(frame), arena_status::log_full);
    EXPECT_EQ(log.size(), 2u);
    EXPECT_EQ(log.append(frame), arena_status::log_full);
    EXPECT_EQ(log.size(), 2u);
}

TEST(input, log_reserves_once_and_never_allocates_on_append) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    input_log log{alloc, 16};

    const std::size_t bytes_after_construction = alloc.bytes_used();
    tick_inputs frame{};
    for (int i = 0; i < 16; ++i) {
        ASSERT_EQ(log.append(frame), arena_status::ok);
    }
    // Article 6: reserved once at construction; appends must not allocate.
    EXPECT_EQ(alloc.bytes_used(), bytes_after_construction);
}

TEST(input, log_clear_resets_for_new_match_without_allocating) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    input_log log{alloc, 4};

    tick_inputs frame{};
    ASSERT_EQ(log.append(frame), arena_status::ok);
    const std::size_t bytes_before = alloc.bytes_used();
    log.clear();
    EXPECT_EQ(log.size(), 0u);
    ASSERT_EQ(log.append(frame), arena_status::ok);
    EXPECT_EQ(log.size(), 1u);
    EXPECT_EQ(alloc.bytes_used(), bytes_before);
}

}  // namespace
