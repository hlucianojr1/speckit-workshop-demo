// GoogleTest for the orbital_arena aggregate root (T016 — written before the arena
// implementation, per Constitution Article 7 test-first discipline; replaces the
// T001 scaffolding smoke test).
//
// Covers contracts/arena.md test obligations 1–5 plus tasks.md T016:
//   - create() config validation
//   - lobby → countdown (exactly 180 ticks) → playing lifecycle
//   - rotationally symmetric well placement on playing entry (FR-001)
//   - inputs outside `playing` have no game effect but are logged (FR-014/FR-016)
//   - steering + boundary clamp end-to-end; SC-001 first-to-100 scripted match
//   - same-tick capture scoring (SC-004); deterministic replenishment
//   - zero allocation across ticks (Article 6); frame_budget self-measure
//   - snapshot round-trip (FR-018) and 60-tick hash history (Article 10)
//
// NOTE: arena buffers are function-local `static` (not plain locals) because a
// 384 KiB arena (input log 4096*48 B dominates) times two arenas per test would
// overflow MSVC's default 1 MiB stack. Each test still owns its own buffer.

#include "orbital_arena/arena.h"

#include "orbital_arena/snapshot.h"

#include <array>
#include <cstddef>
#include <cstdio>
#include <gtest/gtest.h>

namespace {

using engine_demo::allocator;
using orbital_arena::arena;
using orbital_arena::arena_config;
using orbital_arena::arena_status;
using orbital_arena::input_frame;
using orbital_arena::kCountdownTicks;
using orbital_arena::kMaxRecordedTicks;
using orbital_arena::kMaxSnapshotParticles;
using orbital_arena::kWinScore;
using orbital_arena::match_snapshot;
using orbital_arena::match_state;
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

[[nodiscard]] tick_inputs two_player(input_frame p0, input_frame p1) {
    tick_inputs in{};
    in.players[0] = p0;
    in.players[1] = p1;
    return in;
}

void run_ticks(arena& a, std::uint32_t count, const tick_inputs& in) {
    for (std::uint32_t i = 0; i < count; ++i) {
        (void)a.tick(in);  // log_full is legal late in long runs; gameplay advances
    }
}

// Joins + readies players 0..count-1 and drives the arena into `playing`.
void start_playing(arena& a, std::uint8_t player_count) {
    for (std::uint8_t p = 0; p < player_count; ++p) {
        ASSERT_EQ(a.join(p), arena_status::ok);
        ASSERT_EQ(a.set_ready(p, true), arena_status::ok);
    }
    const tick_inputs idle{};
    for (std::uint32_t i = 0; i < kCountdownTicks + 2 && a.state() != match_state::playing;
         ++i) {
        (void)a.tick(idle);
    }
    ASSERT_EQ(a.state(), match_state::playing);
}

TEST(arena_integration, create_validates_config) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};

    arena_config bad_players{};
    bad_players.player_count = 1;
    EXPECT_FALSE(arena::create(alloc, bad_players).has_value());
    bad_players.player_count = 5;
    EXPECT_FALSE(arena::create(alloc, bad_players).has_value());

    arena_config bad_capacity{};
    bad_capacity.particle_capacity = 0;
    EXPECT_FALSE(arena::create(alloc, bad_capacity).has_value());
    bad_capacity.particle_capacity = kMaxSnapshotParticles + 1;
    EXPECT_FALSE(arena::create(alloc, bad_capacity).has_value());

    arena_config bad_extent{};
    bad_extent.half_extent = -1.0f;
    EXPECT_FALSE(arena::create(alloc, bad_extent).has_value());

    arena_config good{};
    good.seed = 42;
    good.player_count = 2;
    good.half_extent = 2.0f;
    good.particle_capacity = 64;
    EXPECT_TRUE(arena::create(alloc, good).has_value());
}

TEST(arena_integration, countdown_is_exactly_180_ticks_then_playing) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg{};
    cfg.seed = 42;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 32;
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;

    EXPECT_EQ(a.state(), match_state::lobby);
    ASSERT_EQ(a.join(0), arena_status::ok);
    ASSERT_EQ(a.join(1), arena_status::ok);
    ASSERT_EQ(a.set_ready(0, true), arena_status::ok);
    ASSERT_EQ(a.set_ready(1, true), arena_status::ok);

    const tick_inputs idle{};
    ASSERT_EQ(a.tick(idle), arena_status::ok);  // lobby -> countdown transition tick
    EXPECT_EQ(a.state(), match_state::countdown);

    run_ticks(a, kCountdownTicks - 1, idle);  // 179 countdown ticks: still counting
    EXPECT_EQ(a.state(), match_state::countdown);

    ASSERT_EQ(a.tick(idle), arena_status::ok);  // exactly the 180th (research D4)
    EXPECT_EQ(a.state(), match_state::playing);
    EXPECT_EQ(a.score(0), 0u);
    EXPECT_EQ(a.score(1), 0u);
}

TEST(arena_integration, wells_start_rotationally_symmetric) {
    // 2 players: exact point reflection. 4 players: exact 90-degree index rotations.
    {
        static std::array<std::byte, kArenaBytes> buffer{};
        allocator alloc{buffer.data(), buffer.size()};
        arena_config cfg{};
        cfg.seed = 1;
        cfg.player_count = 2;
        cfg.half_extent = 2.0f;
        cfg.particle_capacity = 32;
        auto maybe = arena::create(alloc, cfg);
        ASSERT_TRUE(maybe.has_value());
        start_playing(*maybe, 2);

        const auto wells = maybe->wells();
        ASSERT_EQ(wells.size(), 2u);
        EXPECT_EQ(wells[0].position[0], 1.0f);  // half_extent * 0.5
        EXPECT_EQ(wells[0].position[1], 0.0f);
        EXPECT_EQ(wells[1].position[0], -wells[0].position[0]);  // exact negation
        EXPECT_EQ(wells[1].position[1], -wells[0].position[1]);
        EXPECT_TRUE(wells[0].active);
        EXPECT_TRUE(wells[1].active);
        EXPECT_EQ(wells[0].influence_radius, wells[1].influence_radius);  // FR-001
        EXPECT_EQ(wells[0].capture_radius, wells[1].capture_radius);
    }
    {
        static std::array<std::byte, kArenaBytes> buffer{};
        allocator alloc{buffer.data(), buffer.size()};
        arena_config cfg{};
        cfg.seed = 1;
        cfg.player_count = 4;
        cfg.half_extent = 2.0f;
        cfg.particle_capacity = 32;
        auto maybe = arena::create(alloc, cfg);
        ASSERT_TRUE(maybe.has_value());
        start_playing(*maybe, 4);

        const auto wells = maybe->wells();
        ASSERT_EQ(wells.size(), 4u);
        const float r = wells[0].position[0];
        EXPECT_GT(r, 0.0f);
        // (r,0) -> (0,r) -> (-r,0) -> (0,-r): exact quarter-turn index rotations.
        EXPECT_EQ(wells[1].position[0], 0.0f);
        EXPECT_EQ(wells[1].position[1], r);
        EXPECT_EQ(wells[2].position[0], -r);
        EXPECT_EQ(wells[2].position[1], 0.0f);
        EXPECT_EQ(wells[3].position[0], 0.0f);
        EXPECT_EQ(wells[3].position[1], -r);
    }
}

TEST(arena_integration, inputs_outside_playing_are_logged_but_have_no_game_effect) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg{};
    cfg.seed = 3;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 32;
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;

    const tick_inputs shove = two_player(frame(1.0f, 1.0f, 1.0f), frame(-1.0f, 0.0f, 1.0f));
    run_ticks(a, 25, shove);  // lobby: nobody joined, full steer
    EXPECT_EQ(a.state(), match_state::lobby);
    EXPECT_EQ(a.log().size(), 25u);  // FR-016: every tick in every state is logged
    EXPECT_EQ(a.wells()[0].position[0], 0.0f);  // FR-014: no game effect
    EXPECT_EQ(a.wells()[0].position[1], 0.0f);
    EXPECT_EQ(a.score(0), 0u);

    // Logged values are stored post-clamp (FR-015).
    const tick_inputs wild = two_player(frame(5.0f, -7.0f, 42.0f), frame(0.0f, 0.0f, 0.0f));
    (void)a.tick(wild);
    const tick_inputs& logged = a.log().at(a.log().size() - 1);
    EXPECT_EQ(logged.players[0].steer[0], 1.0f);
    EXPECT_EQ(logged.players[0].steer[1], -1.0f);
    EXPECT_EQ(logged.players[0].strength, 1.0f);
}

TEST(arena_integration, steering_moves_well_only_in_playing_and_clamps_to_bounds) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg{};
    cfg.seed = 4;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 32;
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;
    start_playing(a, 2);

    const float start_x = a.wells()[0].position[0];
    const tick_inputs steer_east = two_player(frame(1.0f, 0.0f, 0.0f), frame(0.0f, 0.0f, 0.0f));
    (void)a.tick(steer_east);
    EXPECT_GT(a.wells()[0].position[0], start_x);  // moves in playing

    // 2 u/s for 5 s crosses the whole 4x4 box: must clamp exactly at +half_extent.
    run_ticks(a, 300, steer_east);
    EXPECT_EQ(a.wells()[0].position[0], cfg.half_extent);
    EXPECT_LE(a.wells()[0].position[1], cfg.half_extent);
}

TEST(arena_integration, replenishment_keeps_field_populated_deterministically) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg{};
    cfg.seed = 5;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 128;
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;

    const tick_inputs idle{};
    (void)a.tick(idle);  // first tick fills the empty arena (edge case)
    EXPECT_EQ(a.particles().live_count(), cfg.particle_capacity);

    run_ticks(a, 61, idle);  // no captures in lobby: stays fully populated
    EXPECT_EQ(a.particles().live_count(), cfg.particle_capacity);

    // Particles live inside the arena bounds, on the 2D game plane.
    for (const engine_demo::vfx::particle& p : a.particles().live_particles()) {
        EXPECT_LE(p.position[0], cfg.half_extent);
        EXPECT_GE(p.position[0], -cfg.half_extent);
        EXPECT_LE(p.position[1], cfg.half_extent);
        EXPECT_GE(p.position[1], -cfg.half_extent);
        EXPECT_EQ(p.position[2], 0.0f);
    }
}

TEST(arena_integration, captures_score_and_remove_particles_same_tick) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg{};
    cfg.seed = 6;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 128;
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;
    start_playing(a, 2);

    // Player 0 vacuums at full strength; player 1 idles at zero strength.
    const tick_inputs vacuum = two_player(frame(0.0f, 0.0f, 1.0f), frame(0.0f, 0.0f, 0.0f));

    std::uint32_t total_captured = 0;
    for (std::uint32_t i = 0; i < 550; ++i) {
        // On replenish ticks the pool is topped up, so live-count deltas are only
        // attributable to captures on off-schedule ticks.
        const bool replenish_tick = (a.current_tick() % 60u) == 0u;
        const std::size_t live_before = a.particles().live_count();
        const std::uint32_t score_before = a.score(0) + a.score(1);
        (void)a.tick(vacuum);
        const std::uint32_t captured = (a.score(0) + a.score(1)) - score_before;
        total_captured += captured;
        if (!replenish_tick) {
            // SC-004: every captured particle is removed the same tick it scores.
            EXPECT_EQ(live_before - a.particles().live_count(), captured);
        }
    }
    EXPECT_GT(total_captured, 0u);      // the vacuum actually captured something
    EXPECT_GT(a.score(0), a.score(1));  // full-strength well out-captures idle well
}

TEST(arena_integration, first_to_100_wins_and_scores_freeze) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg{};
    cfg.seed = 42;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 128;
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;
    start_playing(a, 2);

    // Scripted patrol: player 0 sweeps a slow square at full strength (SC-001).
    const input_frame legs[4] = {frame(0.25f, 0.0f, 1.0f), frame(0.0f, 0.25f, 1.0f),
                                 frame(-0.25f, 0.0f, 1.0f), frame(0.0f, -0.25f, 1.0f)};
    std::uint32_t ticks = 0;
    const std::uint32_t kMaxTicks = 60000;
    while (a.state() == match_state::playing && ticks < kMaxTicks) {
        const tick_inputs in =
            two_player(legs[(ticks / 90u) % 4u], frame(0.0f, 0.0f, 0.0f));
        (void)a.tick(in);
        ++ticks;
    }
    ASSERT_EQ(a.state(), match_state::game_over) << "no winner after " << kMaxTicks;
    EXPECT_EQ(a.winner(), 0);
    EXPECT_GE(a.score(0), kWinScore);

    // Scores freeze on the winning tick (US2 scenario 3).
    const std::uint32_t frozen0 = a.score(0);
    const std::uint32_t frozen1 = a.score(1);
    run_ticks(a, 20, two_player(frame(1.0f, 0.0f, 1.0f), frame(-1.0f, 0.0f, 1.0f)));
    EXPECT_EQ(a.state(), match_state::game_over);
    EXPECT_EQ(a.score(0), frozen0);
    EXPECT_EQ(a.score(1), frozen1);

    // Acknowledge returns to lobby with scores reset (FR-012).
    ASSERT_EQ(a.acknowledge_results(), arena_status::ok);
    EXPECT_EQ(a.state(), match_state::lobby);
    EXPECT_EQ(a.score(0), 0u);
    EXPECT_EQ(a.winner(), -1);
}

TEST(arena_integration, tick_never_allocates_after_create) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg{};
    cfg.seed = 8;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 500;  // full-size field (Article 6 at scale)
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;
    start_playing(a, 2);

    const tick_inputs busy = two_player(frame(0.5f, 0.25f, 1.0f), frame(-0.5f, 0.0f, 0.7f));
    const std::size_t bytes_before = alloc.bytes_used();
    run_ticks(a, 100, busy);
    EXPECT_EQ(alloc.bytes_used(), bytes_before);  // contracts/arena.md obligation 4
}

TEST(arena_integration, snapshot_round_trip_mid_match) {
    static std::array<std::byte, kArenaBytes> buffer_a{};
    static std::array<std::byte, kArenaBytes> buffer_b{};
    allocator alloc_a{buffer_a.data(), buffer_a.size()};
    allocator alloc_b{buffer_b.data(), buffer_b.size()};
    arena_config cfg{};
    cfg.seed = 7;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 64;
    auto maybe_a = arena::create(alloc_a, cfg);
    ASSERT_TRUE(maybe_a.has_value());
    arena& a = *maybe_a;
    start_playing(a, 2);
    run_ticks(a, 250, two_player(frame(0.3f, 0.1f, 1.0f), frame(-0.2f, 0.4f, 0.8f)));

    // Obligation 5 (FR-018): snapshot fields mirror the live queries.
    const match_snapshot s = a.capture_snapshot();
    EXPECT_EQ(s.state, match_state::playing);
    EXPECT_EQ(s.tick, a.current_tick());
    EXPECT_EQ(s.seed, cfg.seed);
    EXPECT_EQ(s.player_count, cfg.player_count);
    EXPECT_EQ(s.scores[0], a.score(0));
    EXPECT_EQ(s.scores[1], a.score(1));
    EXPECT_EQ(s.winner, a.winner());
    for (std::uint8_t i = 0; i < cfg.player_count; ++i) {
        EXPECT_EQ(s.wells[i].position[0], a.wells()[i].position[0]);
        EXPECT_EQ(s.wells[i].position[1], a.wells()[i].position[1]);
        EXPECT_EQ(s.wells[i].strength, a.wells()[i].strength);
        EXPECT_EQ(s.wells[i].active, a.wells()[i].active);
    }
    EXPECT_EQ(s.live_particle_count, a.particles().live_count());

    // Restore into a FRESH arena: capture -> restore -> capture is hash-identity.
    auto maybe_b = arena::create(alloc_b, cfg);
    ASSERT_TRUE(maybe_b.has_value());
    arena& b = *maybe_b;
    ASSERT_EQ(b.restore(s), arena_status::ok);
    EXPECT_EQ(b.state(), match_state::playing);
    EXPECT_EQ(b.score(0), a.score(0));
    EXPECT_EQ(b.current_tick(), a.current_tick());
    EXPECT_EQ(state_hash(b.capture_snapshot()), state_hash(s));

    // Mismatched identity is rejected (Article 1 status discipline).
    match_snapshot wrong_seed = s;
    wrong_seed.seed = cfg.seed + 1;
    EXPECT_EQ(b.restore(wrong_seed), arena_status::invalid_argument);
    match_snapshot wrong_count = s;
    wrong_count.player_count = 3;
    EXPECT_EQ(b.restore(wrong_count), arena_status::invalid_argument);
}

TEST(arena_integration, hash_history_sampled_every_60_ticks) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg{};
    cfg.seed = 9;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 32;
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;

    const tick_inputs idle{};
    run_ticks(a, 130, idle);
    // Sampled at ticks 0, 60, 120 (Article 10).
    EXPECT_EQ(a.hash_history().size(), 3u);
    run_ticks(a, 60, idle);
    EXPECT_EQ(a.hash_history().size(), 4u);
}

TEST(arena_integration, input_log_saturates_with_log_full_but_gameplay_advances) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg{};
    cfg.seed = 10;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 16;
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;

    const tick_inputs idle{};
    for (std::size_t i = 0; i < kMaxRecordedTicks; ++i) {
        ASSERT_EQ(a.tick(idle), arena_status::ok);
    }
    EXPECT_EQ(a.log().size(), kMaxRecordedTicks);
    EXPECT_EQ(a.tick(idle), arena_status::log_full);  // frame dropped, no reallocation
    EXPECT_EQ(a.log().size(), kMaxRecordedTicks);
    EXPECT_EQ(a.current_tick(), kMaxRecordedTicks + 1);  // gameplay still advances
}

TEST(arena_integration, tick_stays_within_frame_budget) {
    static std::array<std::byte, kArenaBytes> buffer{};
    allocator alloc{buffer.data(), buffer.size()};
    arena_config cfg{};
    cfg.seed = 11;
    cfg.player_count = 2;
    cfg.half_extent = 2.0f;
    cfg.particle_capacity = 500;  // full-size field
    auto maybe = arena::create(alloc, cfg);
    ASSERT_TRUE(maybe.has_value());
    arena& a = *maybe;
    start_playing(a, 2);
    run_ticks(a, 300, two_player(frame(0.4f, 0.2f, 1.0f), frame(-0.3f, 0.5f, 1.0f)));

    ASSERT_GT(a.budget().sample_count(), 0u);
    const double avg_ms = a.budget().rolling_average();
    std::printf("[ INFO     ] arena::tick rolling average: %.4f ms (budget 16.67 ms)\n",
                avg_ms);
    EXPECT_LT(avg_ms, 16.67);  // Article 6: 60 Hz frame budget, self-measured
}

}  // namespace
