// GoogleTest for orbital_arena match state machine (T010 — written before the
// implementation, per Constitution Article 7 test-first discipline).

#include "orbital_arena/match.h"

#include <gtest/gtest.h>

namespace {

using orbital_arena::arena_status;
using orbital_arena::kCountdownTicks;
using orbital_arena::match;
using orbital_arena::match_state;

// Joins `count` players and readies them all; leaves the match in lobby, ready to tick.
match make_ready_lobby(std::uint8_t count) noexcept {
    match m{};
    for (std::uint8_t i = 0; i < count; ++i) {
        (void)m.join(i);
        (void)m.set_ready(i, true);
    }
    return m;
}

// Drives a ready lobby through countdown into playing.
match make_playing(std::uint8_t count) noexcept {
    match m = make_ready_lobby(count);
    (void)m.tick();  // lobby -> countdown
    for (std::uint32_t i = 0; i < kCountdownTicks; ++i) {
        (void)m.tick();
    }
    return m;
}

TEST(match, happy_path_lobby_to_countdown_to_playing_in_exactly_180_ticks) {
    match m = make_ready_lobby(2);
    EXPECT_EQ(m.state(), match_state::lobby);

    // All joined (>= kMinPlayers) ready -> countdown (US2 scenario 1).
    const auto to_countdown = m.tick();
    ASSERT_TRUE(to_countdown.has_value());
    EXPECT_EQ(to_countdown->from, match_state::lobby);
    EXPECT_EQ(to_countdown->to, match_state::countdown);
    EXPECT_EQ(m.countdown_remaining_ticks(), kCountdownTicks);

    // Exactly kCountdownTicks integer ticks (research D4): 179 ticks -> still counting.
    for (std::uint32_t i = 0; i < kCountdownTicks - 1; ++i) {
        EXPECT_FALSE(m.tick().has_value());
    }
    EXPECT_EQ(m.state(), match_state::countdown);
    EXPECT_EQ(m.countdown_remaining_ticks(), 1u);

    // 180th tick -> playing (US2 scenario 2).
    const auto to_playing = m.tick();
    ASSERT_TRUE(to_playing.has_value());
    EXPECT_EQ(to_playing->from, match_state::countdown);
    EXPECT_EQ(to_playing->to, match_state::playing);
    EXPECT_EQ(m.state(), match_state::playing);
}

TEST(match, lobby_does_not_start_below_min_players_or_when_not_all_ready) {
    match solo{};
    ASSERT_EQ(solo.join(0), arena_status::ok);
    ASSERT_EQ(solo.set_ready(0, true), arena_status::ok);
    EXPECT_FALSE(solo.tick().has_value());  // 1 < kMinPlayers
    EXPECT_EQ(solo.state(), match_state::lobby);

    match pair{};
    ASSERT_EQ(pair.join(0), arena_status::ok);
    ASSERT_EQ(pair.join(1), arena_status::ok);
    ASSERT_EQ(pair.set_ready(0, true), arena_status::ok);
    EXPECT_FALSE(pair.tick().has_value());  // player 1 not ready
    EXPECT_EQ(pair.state(), match_state::lobby);
}

TEST(match, unready_during_countdown_returns_to_lobby) {
    match m = make_ready_lobby(2);
    (void)m.tick();
    ASSERT_EQ(m.state(), match_state::countdown);
    ASSERT_EQ(m.set_ready(1, false), arena_status::ok);
    EXPECT_EQ(m.state(), match_state::lobby);  // FR-013 countdown interruption
}

TEST(match, leave_during_countdown_returns_to_lobby) {
    match m = make_ready_lobby(3);
    (void)m.tick();
    ASSERT_EQ(m.state(), match_state::countdown);
    m.leave(2);
    EXPECT_EQ(m.state(), match_state::lobby);
    EXPECT_EQ(m.joined_count(), 2u);
}

TEST(match, join_and_ready_rejected_outside_their_legal_states) {
    match m = make_playing(2);
    ASSERT_EQ(m.state(), match_state::playing);
    EXPECT_EQ(m.join(2), arena_status::wrong_state);
    EXPECT_EQ(m.set_ready(0, false), arena_status::wrong_state);

    m.enter_game_over();
    EXPECT_EQ(m.join(2), arena_status::wrong_state);
    EXPECT_EQ(m.set_ready(0, true), arena_status::wrong_state);

    // Bad player index in lobby -> invalid_argument.
    match lobby{};
    EXPECT_EQ(lobby.join(orbital_arena::kMaxPlayers), arena_status::invalid_argument);
    EXPECT_EQ(lobby.set_ready(0, true), arena_status::invalid_argument);  // never joined
}

TEST(match, gameplay_input_enabled_only_while_playing) {
    match m = make_ready_lobby(2);
    EXPECT_FALSE(m.gameplay_input_enabled());  // lobby (FR-014, US2 scenario 6)
    (void)m.tick();
    EXPECT_FALSE(m.gameplay_input_enabled());  // countdown
    for (std::uint32_t i = 0; i < kCountdownTicks; ++i) {
        (void)m.tick();
    }
    EXPECT_TRUE(m.gameplay_input_enabled());   // playing
    m.enter_game_over();
    EXPECT_FALSE(m.gameplay_input_enabled());  // game_over
}

TEST(match, departure_during_playing_marks_departed_and_match_continues) {
    match m = make_playing(3);
    m.leave(1);
    EXPECT_EQ(m.state(), match_state::playing);  // FR-013: match continues
    EXPECT_EQ(m.active_count(), 2u);
    EXPECT_EQ(m.joined_count(), 3u);  // roster is fixed once playing

    // Second departure -> active < kMinPlayers; the arena then calls enter_game_over.
    m.leave(2);
    EXPECT_EQ(m.active_count(), 1u);
    m.enter_game_over();
    EXPECT_EQ(m.state(), match_state::game_over);
}

TEST(match, acknowledge_returns_to_lobby_with_ready_flags_cleared) {
    match m = make_playing(2);
    m.enter_game_over();
    ASSERT_EQ(m.state(), match_state::game_over);

    // acknowledge only legal in game_over (FR-012).
    ASSERT_EQ(m.acknowledge_results(), arena_status::ok);
    EXPECT_EQ(m.state(), match_state::lobby);
    EXPECT_EQ(m.acknowledge_results(), arena_status::wrong_state);

    // Ready flags cleared (US2 scenario 5): ticking must NOT restart the countdown.
    EXPECT_FALSE(m.tick().has_value());
    EXPECT_EQ(m.state(), match_state::lobby);
    // Re-readying both players restarts normally.
    ASSERT_EQ(m.set_ready(0, true), arena_status::ok);
    ASSERT_EQ(m.set_ready(1, true), arena_status::ok);
    EXPECT_TRUE(m.tick().has_value());
    EXPECT_EQ(m.state(), match_state::countdown);
}

TEST(match, departed_player_is_removed_from_roster_on_acknowledge) {
    match m = make_playing(2);
    m.leave(1);
    m.enter_game_over();
    ASSERT_EQ(m.acknowledge_results(), arena_status::ok);
    EXPECT_EQ(m.joined_count(), 1u);  // the departed player left the roster
    EXPECT_EQ(m.active_count(), 1u);
}

}  // namespace
