// GoogleTest for orbital_arena power-up system (T012 — written before the
// implementation, per Constitution Article 7 test-first discipline).
//
// Position/kind prediction mirrors the contract's fixed draw order (FR-019):
// x, y from next_double_unit() mapped to [-half_extent, +half_extent], then
// kind from next_u32() % 3. A probe rng seeded identically predicts every spawn.

#include "orbital_arena/powerup.h"

#include "engine_demo/sim/rng.h"

#include <array>
#include <cstddef>
#include <gtest/gtest.h>

namespace {

using engine_demo::allocator;
using engine_demo::sim::rng;
using orbital_arena::gravity_well;
using orbital_arena::kDoublePointsTicks;
using orbital_arena::kFloodBurstCount;
using orbital_arena::kMaxFieldPickups;
using orbital_arena::kPowerupSpawnIntervalTicks;
using orbital_arena::kStrengthSurgeTicks;
using orbital_arena::pickup;
using orbital_arena::powerup_kind;
using orbital_arena::powerup_system;

constexpr float kHalfExtent = 5.0f;

// Runs tick_spawn through one full spawn interval; returns ticks consumed.
std::uint64_t run_one_interval(powerup_system& system, rng& r, std::uint64_t start_tick) {
    for (std::uint32_t i = 0; i < kPowerupSpawnIntervalTicks; ++i) {
        system.tick_spawn(r, start_tick + i, kHalfExtent);
    }
    return start_tick + kPowerupSpawnIntervalTicks;
}

// Counts alive pickups in the fixed slot array.
std::size_t alive_count(const powerup_system& system) {
    std::size_t count = 0;
    for (const pickup& p : system.field_pickups()) {
        if (p.alive) {
            ++count;
        }
    }
    return count;
}

// Returns the first alive pickup slot (asserts one exists).
const pickup& first_alive(const powerup_system& system) {
    for (const pickup& p : system.field_pickups()) {
        if (p.alive) {
            return p;
        }
    }
    ADD_FAILURE() << "no alive pickup";
    return system.field_pickups()[0];
}

gravity_well well_at(float x, float y) noexcept {
    gravity_well well{};
    well.position[0] = x;
    well.position[1] = y;
    well.capture_radius = 0.15f;
    return well;
}

TEST(powerup, spawns_exactly_every_interval_at_seed_determined_position_and_kind) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    powerup_system system{alloc};
    rng r{1234};

    // Predict the draws with an identically seeded probe (fixed order: x, y, kind).
    rng probe{1234};
    const float expected_x =
        static_cast<float>((probe.next_double_unit() * 2.0 - 1.0) * kHalfExtent);
    const float expected_y =
        static_cast<float>((probe.next_double_unit() * 2.0 - 1.0) * kHalfExtent);
    const powerup_kind expected_kind = static_cast<powerup_kind>(probe.next_u32() % 3u);

    // 599 ticks: nothing spawns yet.
    for (std::uint32_t i = 0; i < kPowerupSpawnIntervalTicks - 1; ++i) {
        system.tick_spawn(r, i, kHalfExtent);
    }
    EXPECT_EQ(alive_count(system), 0u);

    // 600th tick: exactly one pickup at the predicted position/kind (FR-009/FR-019).
    system.tick_spawn(r, kPowerupSpawnIntervalTicks - 1, kHalfExtent);
    ASSERT_EQ(alive_count(system), 1u);
    const pickup& spawned = first_alive(system);
    EXPECT_EQ(spawned.position[0], expected_x);
    EXPECT_EQ(spawned.position[1], expected_y);
    EXPECT_EQ(spawned.kind, expected_kind);
    EXPECT_LE(spawned.position[0], kHalfExtent);
    EXPECT_GE(spawned.position[0], -kHalfExtent);
}

TEST(powerup, spawn_sequence_reproducible_across_identically_seeded_runs) {
    std::array<std::byte, 65536> buffer_a{};
    std::array<std::byte, 65536> buffer_b{};
    allocator alloc_a{buffer_a.data(), buffer_a.size()};
    allocator alloc_b{buffer_b.data(), buffer_b.size()};
    powerup_system system_a{alloc_a};
    powerup_system system_b{alloc_b};
    rng rng_a{777};
    rng rng_b{777};

    std::uint64_t tick_a = 0;
    std::uint64_t tick_b = 0;
    for (int interval = 0; interval < 3; ++interval) {
        tick_a = run_one_interval(system_a, rng_a, tick_a);
        tick_b = run_one_interval(system_b, rng_b, tick_b);
    }
    ASSERT_EQ(alive_count(system_a), 3u);
    ASSERT_EQ(alive_count(system_b), 3u);
    const auto pickups_a = system_a.field_pickups();
    const auto pickups_b = system_b.field_pickups();
    for (std::size_t i = 0; i < pickups_a.size(); ++i) {
        EXPECT_EQ(pickups_a[i].alive, pickups_b[i].alive);
        EXPECT_EQ(pickups_a[i].kind, pickups_b[i].kind);
        EXPECT_EQ(pickups_a[i].position[0], pickups_b[i].position[0]);  // bit-exact
        EXPECT_EQ(pickups_a[i].position[1], pickups_b[i].position[1]);
    }
}

TEST(powerup, at_cap_spawn_skipped_timer_continues_and_rng_not_consumed) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    powerup_system system{alloc};
    rng r{99};

    std::uint64_t tick = 0;
    for (std::uint32_t i = 0; i < kMaxFieldPickups; ++i) {
        tick = run_one_interval(system, r, tick);
    }
    ASSERT_EQ(alive_count(system), static_cast<std::size_t>(kMaxFieldPickups));

    // One more full interval at cap: NO rng draws may occur (FR-009 edge case).
    rng snapshot = r;  // copy of the stream state
    tick = run_one_interval(system, r, tick);
    EXPECT_EQ(alive_count(system), static_cast<std::size_t>(kMaxFieldPickups));
    EXPECT_EQ(r.next_u32(), snapshot.next_u32());  // streams still identical
}

// Spawns one pickup and consumes it with a well owned by `player`; returns the kind
// consumed and forwards any flood request. Uses only the public API: the well is
// teleported onto the pickup's seed-drawn position.
struct consume_outcome {
    powerup_kind kind;
    bool flooded;
    std::uint32_t flood_count;
};

consume_outcome spawn_and_consume(powerup_system& system, rng& r, std::uint64_t& tick,
                                  std::uint8_t player) {
    (void)run_one_interval(system, r, tick);
    tick += kPowerupSpawnIntervalTicks;
    const pickup& p = first_alive(system);
    const powerup_kind kind = p.kind;

    gravity_well wells[4] = {};
    for (gravity_well& w : wells) {
        w.active = false;
    }
    wells[player] = well_at(p.position[0], p.position[1]);
    const auto flood = system.consume_pickups({wells, 4}, r);
    return consume_outcome{kind, flood.has_value(),
                           flood.has_value() ? flood->count : 0u};
}

// Keeps spawning/consuming until `wanted` is consumed (bounded; seed-dependent order).
consume_outcome consume_until(powerup_system& system, rng& r, std::uint64_t& tick,
                              powerup_kind wanted, std::uint8_t player) {
    for (int attempt = 0; attempt < 64; ++attempt) {
        const consume_outcome outcome = spawn_and_consume(system, r, tick, player);
        if (outcome.kind == wanted) {
            return outcome;
        }
    }
    ADD_FAILURE() << "kind never drawn within 64 spawns";
    return consume_outcome{wanted, false, 0};
}

TEST(powerup, strength_surge_doubles_pull_for_exactly_300_ticks_then_reverts) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    powerup_system system{alloc};
    rng r{5};
    std::uint64_t tick = 0;

    (void)consume_until(system, r, tick, powerup_kind::strength_surge, 0);
    EXPECT_TRUE(system.is_active(0, powerup_kind::strength_surge));
    EXPECT_FLOAT_EQ(system.strength_multiplier(0), 2.0f);
    EXPECT_FLOAT_EQ(system.strength_multiplier(1), 1.0f);  // other players unaffected

    for (std::uint32_t i = 0; i < kStrengthSurgeTicks - 1; ++i) {
        system.tick_effects();
    }
    EXPECT_TRUE(system.is_active(0, powerup_kind::strength_surge));  // tick 299: active
    system.tick_effects();                                           // tick 300: expires
    EXPECT_FALSE(system.is_active(0, powerup_kind::strength_surge));
    EXPECT_FLOAT_EQ(system.strength_multiplier(0), 1.0f);
}

TEST(powerup, double_points_doubles_capture_value_for_600_ticks) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    powerup_system system{alloc};
    rng r{6};
    std::uint64_t tick = 0;

    (void)consume_until(system, r, tick, powerup_kind::double_points, 1);
    EXPECT_TRUE(system.is_active(1, powerup_kind::double_points));
    EXPECT_EQ(system.points_multiplier(1), 2u);
    EXPECT_EQ(system.points_multiplier(0), 1u);

    for (std::uint32_t i = 0; i < kDoublePointsTicks; ++i) {
        system.tick_effects();
    }
    EXPECT_FALSE(system.is_active(1, powerup_kind::double_points));
    EXPECT_EQ(system.points_multiplier(1), 1u);
}

TEST(powerup, particle_flood_returns_immediate_burst_request_and_occupies_no_slot) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    powerup_system system{alloc};
    rng r{7};
    std::uint64_t tick = 0;

    const consume_outcome outcome =
        consume_until(system, r, tick, powerup_kind::particle_flood, 0);
    EXPECT_TRUE(outcome.flooded);  // US3 scenario 5: immediate request
    EXPECT_EQ(outcome.flood_count, kFloodBurstCount);
    EXPECT_FALSE(system.is_active(0, powerup_kind::particle_flood));  // instant, no slot
}

TEST(powerup, same_kind_re_pickup_refreshes_duration) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    powerup_system system{alloc};
    rng r{8};
    std::uint64_t tick = 0;

    (void)consume_until(system, r, tick, powerup_kind::strength_surge, 0);
    for (std::uint32_t i = 0; i < 100; ++i) {
        system.tick_effects();  // burn 100 of the 300 ticks
    }
    ASSERT_TRUE(system.is_active(0, powerup_kind::strength_surge));

    (void)consume_until(system, r, tick, powerup_kind::strength_surge, 0);
    // Refreshed to the full duration (data-model.md rule): 299 more ticks still active.
    for (std::uint32_t i = 0; i < kStrengthSurgeTicks - 1; ++i) {
        system.tick_effects();
    }
    EXPECT_TRUE(system.is_active(0, powerup_kind::strength_surge));
    system.tick_effects();
    EXPECT_FALSE(system.is_active(0, powerup_kind::strength_surge));
}

TEST(powerup, consumption_contention_nearest_well_wins_exact_tie_leaves_pickup) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    powerup_system system{alloc};
    rng r{9};

    std::uint64_t tick = 0;
    (void)run_one_interval(system, r, tick);
    const pickup& p = first_alive(system);
    const float px = p.position[0];
    const float py = p.position[1];

    // Exact tie: two wells mirrored around the pickup → nobody consumes (FR-007).
    gravity_well tied[2] = {well_at(px + 0.1f, py), well_at(px - 0.1f, py)};
    EXPECT_FALSE(system.consume_pickups({tied, 2}, r).has_value());
    EXPECT_EQ(alive_count(system), 1u);  // pickup survives (US3 scenario 2)

    // Strict nearest: well 1 closer → player 1 consumes.
    gravity_well contested[2] = {well_at(px + 0.12f, py), well_at(px + 0.05f, py)};
    (void)system.consume_pickups({contested, 2}, r);
    EXPECT_EQ(alive_count(system), 0u);
    if (p.kind != powerup_kind::particle_flood) {
        EXPECT_TRUE(system.is_active(1, p.kind));
        EXPECT_FALSE(system.is_active(0, p.kind));
    }
}

TEST(powerup, clear_all_ends_every_effect_and_removes_pickups) {
    std::array<std::byte, 65536> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    powerup_system system{alloc};
    rng r{10};
    std::uint64_t tick = 0;

    (void)consume_until(system, r, tick, powerup_kind::strength_surge, 0);
    (void)run_one_interval(system, r, tick);  // leave one pickup on the field
    ASSERT_TRUE(system.is_active(0, powerup_kind::strength_surge));
    ASSERT_GE(alive_count(system), 1u);

    system.clear_all();  // game_over: all effects end immediately (FR-011, US3 sc.6)
    EXPECT_FALSE(system.is_active(0, powerup_kind::strength_surge));
    EXPECT_EQ(alive_count(system), 0u);
    for (std::uint8_t player = 0; player < 4; ++player) {
        EXPECT_FLOAT_EQ(system.strength_multiplier(player), 1.0f);
        EXPECT_EQ(system.points_multiplier(player), 1u);
    }
}

}  // namespace
