// GoogleTest replay-determinism + fairness proof for orbital_arena (T018 —
// test-only task, Constitution Articles 7a, 9, 10).
//
//   (a) SC-002 / FR-017: a scripted 1000-frame match recorded at a fixed seed and
//       replayed from (seed, input log) reproduces every 60-frame state hash (17
//       samples), the final scoreboard, and the winner — bit-exact.
//   (b) Article 7a scripted coverage: captures, the win condition, every power-up
//       kind's seed-determined spawn schedule (SC-006), and end-to-end pickup
//       consumption at fixed seeds.
//   (c) Article 9 / SC-003 fairness: the arena's rotationally symmetric well
//       placement AND mirrored particle replenishment make a 2-player world evolve
//       as an exact point reflection. Mirrored input scripts therefore yield
//       bit-identical mirrored outcomes, and swapping the players' (mirrored)
//       scripts swaps the scores exactly: outcome follows the script, never the
//       slot index. All equality assertions are EXPECT_EQ (bit-exact, Article 5).
//
// Buffers are function-local `static` for the same stack-size reason documented in
// test_arena_integration.cpp.

#include "orbital_arena/arena.h"

#include "engine_demo/sim/rng.h"
#include "orbital_arena/snapshot.h"

#include <array>
#include <cstddef>
#include <gtest/gtest.h>

namespace {

using engine_demo::allocator;
using orbital_arena::arena;
using orbital_arena::arena_config;
using orbital_arena::arena_status;
using orbital_arena::input_frame;
using orbital_arena::kCountdownTicks;
using orbital_arena::kPowerupSpawnIntervalTicks;
using orbital_arena::match_snapshot;
using orbital_arena::match_state;
using orbital_arena::pickup;
using orbital_arena::powerup_kind;
using orbital_arena::state_hash;
using orbital_arena::tick_inputs;

constexpr std::size_t kArenaBytes = 384u * 1024u;

[[nodiscard]] input_frame frame(float sx, float sy, float strength) {
    input_frame f{};
    f.steer[0] = sx;
    f.steer[1] = sy;
    f.strength = strength;
    return f;
}

// Steer negation = point reflection of the input script (strength unchanged).
[[nodiscard]] input_frame mirrored(const input_frame& f) {
    input_frame m = f;
    m.steer[0] = -f.steer[0];
    m.steer[1] = -f.steer[1];
    return m;
}

// Two deterministic, distinct autopilot scripts (pure functions of the tick).
[[nodiscard]] input_frame script_patrol(std::uint64_t t) {
    static const input_frame legs[4] = {
        frame(0.25f, 0.0f, 1.0f), frame(0.0f, 0.25f, 1.0f),
        frame(-0.25f, 0.0f, 1.0f), frame(0.0f, -0.25f, 1.0f)};
    return legs[(t / 75u) % 4u];
}

[[nodiscard]] input_frame script_weave(std::uint64_t t) {
    static const input_frame legs[2] = {frame(0.4f, -0.3f, 1.0f),
                                        frame(-0.3f, 0.4f, 0.8f)};
    return legs[(t / 130u) % 2u];
}

[[nodiscard]] arena_config small_config(std::uint64_t seed) {
    arena_config cfg{};
    cfg.seed = seed;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 128;
    return cfg;
}

void start_playing(arena& a) {
    ASSERT_EQ(a.join(0), arena_status::ok);
    ASSERT_EQ(a.join(1), arena_status::ok);
    ASSERT_EQ(a.set_ready(0, true), arena_status::ok);
    ASSERT_EQ(a.set_ready(1, true), arena_status::ok);
    const tick_inputs idle{};
    for (std::uint32_t i = 0;
         i < kCountdownTicks + 2 && a.state() != match_state::playing; ++i) {
        (void)a.tick(idle);
    }
    ASSERT_EQ(a.state(), match_state::playing);
}

// ---------------------------------------------------------------------------
// (a) 1000-frame record + replay (SC-002, FR-017, Article 10)
// ---------------------------------------------------------------------------

TEST(replay_determinism, replaying_1000_frames_from_seed_and_log_is_bit_exact) {
    static std::array<std::byte, kArenaBytes> buffer_a{};
    static std::array<std::byte, kArenaBytes> buffer_b{};
    allocator alloc_a{buffer_a.data(), buffer_a.size()};
    allocator alloc_b{buffer_b.data(), buffer_b.size()};
    const arena_config cfg = small_config(42);

    // Record: scripted match, inputs logged by the arena itself.
    auto maybe_a = arena::create(alloc_a, cfg);
    ASSERT_TRUE(maybe_a.has_value());
    arena& recorded = *maybe_a;
    start_playing(recorded);
    while (recorded.current_tick() < 1000) {
        tick_inputs in{};
        in.players[0] = script_patrol(recorded.current_tick());
        in.players[1] = script_weave(recorded.current_tick());
        ASSERT_EQ(recorded.tick(in), arena_status::ok);
    }
    ASSERT_EQ(recorded.log().size(), 1000u);
    ASSERT_EQ(recorded.hash_history().size(), 17u);  // ticks 0, 60, ..., 960
    EXPECT_GT(recorded.score(0) + recorded.score(1), 0u);  // captures happened (7a)

    // Replay: same config (seed) + same lobby calls + the recorded log verbatim.
    auto maybe_b = arena::create(alloc_b, cfg);
    ASSERT_TRUE(maybe_b.has_value());
    arena& replayed = *maybe_b;
    start_playing(replayed);
    while (replayed.current_tick() < recorded.log().size()) {
        ASSERT_EQ(replayed.tick(recorded.log().at(replayed.current_tick())),
                  arena_status::ok);
    }

    // Bit-exact equivalence (Article 5: EXPECT_EQ, never EXPECT_NEAR).
    ASSERT_EQ(replayed.hash_history().size(), recorded.hash_history().size());
    for (std::size_t i = 0; i < recorded.hash_history().size(); ++i) {
        EXPECT_EQ(replayed.hash_history()[i], recorded.hash_history()[i]) << "sample " << i;
    }
    EXPECT_EQ(replayed.score(0), recorded.score(0));
    EXPECT_EQ(replayed.score(1), recorded.score(1));
    EXPECT_EQ(replayed.winner(), recorded.winner());
    EXPECT_EQ(replayed.state(), recorded.state());
    EXPECT_EQ(state_hash(replayed.capture_snapshot()),
              state_hash(recorded.capture_snapshot()));

    // SC-006: the pickup schedule (spawn ticks/positions/kinds) replayed exactly.
    for (std::uint32_t i = 0; i < orbital_arena::kMaxFieldPickups; ++i) {
        const pickup& p = recorded.pickups()[i];
        const pickup& q = replayed.pickups()[i];
        EXPECT_EQ(q.alive, p.alive);
        EXPECT_EQ(q.kind, p.kind);
        EXPECT_EQ(q.position[0], p.position[0]);
        EXPECT_EQ(q.position[1], p.position[1]);
        EXPECT_EQ(q.spawn_tick, p.spawn_tick);
    }
}

TEST(replay_determinism, different_seed_diverges) {
    static std::array<std::byte, kArenaBytes> buffer_a{};
    static std::array<std::byte, kArenaBytes> buffer_b{};
    allocator alloc_a{buffer_a.data(), buffer_a.size()};
    allocator alloc_b{buffer_b.data(), buffer_b.size()};
    auto maybe_a = arena::create(alloc_a, small_config(1));
    auto maybe_b = arena::create(alloc_b, small_config(2));
    ASSERT_TRUE(maybe_a.has_value());
    ASSERT_TRUE(maybe_b.has_value());

    const tick_inputs idle{};
    for (std::uint32_t i = 0; i < 200; ++i) {
        (void)maybe_a->tick(idle);
        (void)maybe_b->tick(idle);
    }
    // Sanity guard against hashing constants: the seeded particle fields differ.
    ASSERT_GT(maybe_a->particles().live_count(), 0u);
    const auto pa = maybe_a->particles().live_particles();
    const auto pb = maybe_b->particles().live_particles();
    EXPECT_TRUE(pa[0].position[0] != pb[0].position[0] ||
                pa[0].position[1] != pb[0].position[1]);
    EXPECT_NE(maybe_a->hash_history()[1], maybe_b->hash_history()[1]);
}

// ---------------------------------------------------------------------------
// (c) Input-swap fairness (Article 9, US4 scenario 2, SC-003)
// ---------------------------------------------------------------------------

// Runs a 2-player match where slot0 executes `s0(t)` and slot1 executes `s1(t)`,
// for kFairnessTicks gameplay ticks (kept below the first power-up spawn at
// playing-tick 600, since a single seed-placed pickup is intentionally not
// symmetric). Returns the final snapshot.
constexpr std::uint32_t kFairnessTicks = 560;

using script_fn = input_frame (*)(std::uint64_t);

[[nodiscard]] match_snapshot run_fairness_match(allocator& alloc, script_fn s0,
                                                bool mirror0, script_fn s1,
                                                bool mirror1) {
    auto maybe = arena::create(alloc, small_config(9));
    EXPECT_TRUE(maybe.has_value());
    arena& a = *maybe;
    start_playing(a);
    for (std::uint32_t i = 0; i < kFairnessTicks; ++i) {
        tick_inputs in{};
        const input_frame f0 = s0(i);
        const input_frame f1 = s1(i);
        in.players[0] = mirror0 ? mirrored(f0) : f0;
        in.players[1] = mirror1 ? mirrored(f1) : f1;
        (void)a.tick(in);
    }
    return a.capture_snapshot();
}

TEST(replay_determinism, mirrored_inputs_yield_bit_identical_mirrored_world) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    auto maybe = arena::create(alloc, small_config(9));
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;
    start_playing(a);

    for (std::uint32_t i = 0; i < kFairnessTicks; ++i) {
        tick_inputs in{};
        in.players[0] = script_patrol(i);
        in.players[1] = mirrored(script_patrol(i));  // exact point reflection
        (void)a.tick(in);
        if (i % 10u == 0u) {
            // Neither slot may ever be ahead: zero score difference attributable
            // to player index (Article 9) — bit-exact, every sampled tick.
            ASSERT_EQ(a.score(0), a.score(1)) << "tick " << i;
            ASSERT_EQ(a.wells()[1].position[0], -a.wells()[0].position[0]);
            ASSERT_EQ(a.wells()[1].position[1], -a.wells()[0].position[1]);
        }
    }
    EXPECT_GT(a.score(0), 0u);  // the fairness claim is about real captures
    EXPECT_EQ(a.score(0), a.score(1));
}

TEST(replay_determinism, swapping_input_scripts_swaps_outcomes_exactly) {
    static std::array<std::byte, kArenaBytes> buffer_x{};
    static std::array<std::byte, kArenaBytes> buffer_y{};
    allocator alloc_x{buffer_x.data(), buffer_x.size()};
    allocator alloc_y{buffer_y.data(), buffer_y.size()};

    // Run X: slot0 <- patrol, slot1 <- weave.
    // Run Y: slot0 <- mirrored weave, slot1 <- mirrored patrol.
    // Y is the exact point reflection of X with the slots swapped, so the score a
    // script earns is independent of the slot it is assigned to (SC-003): the
    // winner identity follows the script, never the player index.
    const match_snapshot x =
        run_fairness_match(alloc_x, &script_patrol, false, &script_weave, false);
    const match_snapshot y =
        run_fairness_match(alloc_y, &script_weave, true, &script_patrol, true);

    EXPECT_EQ(y.scores[0], x.scores[1]);  // weave's score, regardless of slot
    EXPECT_EQ(y.scores[1], x.scores[0]);  // patrol's score, regardless of slot
    EXPECT_GT(x.scores[0] + x.scores[1], 0u);
    // Well trajectories are exact reflections of the swapped slots.
    EXPECT_EQ(y.wells[0].position[0], -x.wells[1].position[0]);
    EXPECT_EQ(y.wells[0].position[1], -x.wells[1].position[1]);
    EXPECT_EQ(y.wells[1].position[0], -x.wells[0].position[0]);
    EXPECT_EQ(y.wells[1].position[1], -x.wells[0].position[1]);
    EXPECT_EQ(y.state, x.state);
    EXPECT_EQ(y.sudden_death, x.sudden_death);
}

// ---------------------------------------------------------------------------
// (b) Power-up kind coverage + consumption at fixed seeds (Article 7a, SC-006)
// ---------------------------------------------------------------------------

struct spawn_prediction {
    float x{};
    float y{};
    powerup_kind kind{};
};

// Mirrors powerup_system::tick_spawn's documented draw order: x, y, kind (FR-019).
void predict_spawns(std::uint64_t seed, float half_extent, spawn_prediction out[3]) {
    engine_demo::sim::rng probe{seed};
    for (int i = 0; i < 3; ++i) {
        out[i].x = static_cast<float>((probe.next_double_unit() * 2.0 - 1.0) * half_extent);
        out[i].y = static_cast<float>((probe.next_double_unit() * 2.0 - 1.0) * half_extent);
        out[i].kind = static_cast<powerup_kind>(probe.next_u32() % 3u);
    }
}

// True iff (x, y) is at least `margin` away from both symmetric well starts, so a
// parked well can never consume the pickup by accident.
[[nodiscard]] bool clear_of_wells(float x, float y, float well_r, float margin) {
    const float dx0 = x - well_r;
    const float dx1 = x + well_r;
    return (dx0 * dx0 + y * y) > margin * margin && (dx1 * dx1 + y * y) > margin * margin;
}

TEST(replay_determinism, powerup_schedule_covers_every_kind_at_predicted_positions) {
    // Deterministic seed search: first seed whose first three kind draws cover all
    // three kinds and whose positions stay clear of the parked wells.
    const float half_extent = 2.0f;
    const float well_r = half_extent * 0.5f;
    std::uint64_t seed = 0;
    spawn_prediction predicted[3];
    for (std::uint64_t candidate = 1; candidate < 2000; ++candidate) {
        predict_spawns(candidate, half_extent, predicted);
        const std::uint32_t kind_mask = (1u << static_cast<std::uint32_t>(predicted[0].kind)) |
                                        (1u << static_cast<std::uint32_t>(predicted[1].kind)) |
                                        (1u << static_cast<std::uint32_t>(predicted[2].kind));
        bool clear = kind_mask == 0x7u;
        for (int i = 0; clear && i < 3; ++i) {
            clear = clear_of_wells(predicted[i].x, predicted[i].y, well_r, 0.5f);
        }
        if (clear) {
            seed = candidate;
            break;
        }
    }
    ASSERT_NE(seed, 0u) << "no covering seed found in search range";

    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg = small_config(seed);
    cfg.particle_capacity = 64;
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;
    start_playing(a);

    // Park both wells (zero steer/strength) through three full spawn intervals.
    const tick_inputs parked{};
    for (std::uint32_t i = 0; i < 3u * kPowerupSpawnIntervalTicks + 10u; ++i) {
        (void)a.tick(parked);
    }

    // All three pickups alive, at the probe-predicted positions and kinds — the
    // game-rule stream was consumed exactly 3 draws per spawn, nothing else (FR-019).
    std::uint32_t observed_mask = 0;
    std::uint32_t alive = 0;
    for (const pickup& p : a.pickups()) {
        if (!p.alive) {
            continue;
        }
        bool matched = false;
        for (const spawn_prediction& pr : predicted) {
            if (p.position[0] == pr.x && p.position[1] == pr.y && p.kind == pr.kind) {
                matched = true;
            }
        }
        EXPECT_TRUE(matched) << "pickup at unpredicted position/kind";
        observed_mask |= 1u << static_cast<std::uint32_t>(p.kind);
        ++alive;
    }
    EXPECT_EQ(alive, 3u);
    EXPECT_EQ(observed_mask, 0x7u);  // every power-up kind spawned (Article 7a)
}

TEST(replay_determinism, first_arriving_well_consumes_pickup_and_gains_effect) {
    // Deterministic seed search: first timed-kind pickup clear of both wells.
    const float half_extent = 2.0f;
    const float well_r = half_extent * 0.5f;
    std::uint64_t seed = 0;
    spawn_prediction predicted[3];
    for (std::uint64_t candidate = 1; candidate < 2000; ++candidate) {
        predict_spawns(candidate, half_extent, predicted);
        if (predicted[0].kind != powerup_kind::particle_flood &&
            clear_of_wells(predicted[0].x, predicted[0].y, well_r, 0.7f)) {
            seed = candidate;
            break;
        }
    }
    ASSERT_NE(seed, 0u);

    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg = small_config(seed);
    cfg.particle_capacity = 64;
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;
    start_playing(a);

    // Park until the first pickup spawns.
    const tick_inputs parked{};
    std::uint32_t guard = 0;
    while (!a.pickups()[0].alive && guard++ < kPowerupSpawnIntervalTicks + 10u) {
        (void)a.tick(parked);
    }
    ASSERT_TRUE(a.pickups()[0].alive);
    EXPECT_EQ(a.pickups()[0].position[0], predicted[0].x);
    EXPECT_EQ(a.pickups()[0].kind, predicted[0].kind);

    // Autopilot: player 0 chases the pickup (bang-bang steering); player 1 parks.
    guard = 0;
    while (a.pickups()[0].alive && guard++ < 1000) {
        const float dx = a.pickups()[0].position[0] - a.wells()[0].position[0];
        const float dy = a.pickups()[0].position[1] - a.wells()[0].position[1];
        tick_inputs in{};
        in.players[0] = frame(dx > 0.0f ? 1.0f : (dx < 0.0f ? -1.0f : 0.0f),
                              dy > 0.0f ? 1.0f : (dy < 0.0f ? -1.0f : 0.0f), 0.0f);
        (void)a.tick(in);
    }
    ASSERT_FALSE(a.pickups()[0].alive) << "pickup never consumed";

    // First-arrival consumption granted the timed effect to player 0 (US3).
    const match_snapshot s = a.capture_snapshot();
    EXPECT_EQ(s.effects[0].kind, predicted[0].kind);
    EXPECT_GT(s.effects[0].remaining_ticks, 0u);
    EXPECT_EQ(s.effects[1].remaining_ticks, 0u);  // parked player got nothing
}

}  // namespace
