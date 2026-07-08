// GoogleTest for orbital_arena scoring: capture contention (T006) and win rules (T008).
// Written before the implementation, per Constitution Article 7 test-first discipline.

#include "orbital_arena/scoring.h"

#include <gtest/gtest.h>

namespace {

using engine_demo::vfx::particle;
using orbital_arena::award_capture;
using orbital_arena::capture_result;
using orbital_arena::evaluate_win;
using orbital_arena::gravity_well;
using orbital_arena::kMaxPlayers;
using orbital_arena::reset_scores;
using orbital_arena::resolve_capture;
using orbital_arena::resolve_captures;
using orbital_arena::score_table;

gravity_well make_well(float x, float y, float capture_radius) noexcept {
    gravity_well well{};
    well.position[0] = x;
    well.position[1] = y;
    well.capture_radius = capture_radius;
    return well;
}

particle make_particle(float x, float y) noexcept {
    particle p{};
    p.position[0] = x;
    p.position[1] = y;
    p.remaining_lifetime_seconds = 10.0;
    return p;
}

// --- T006: capture contention (FR-007, Article 9) ---

TEST(scoring, nearest_well_wins_contested_capture) {
    const gravity_well wells[] = {make_well(0.0f, 0.0f, 1.0f), make_well(1.0f, 0.0f, 1.0f)};
    const float pos[2] = {0.3f, 0.0f};  // dist_sq: 0.09 vs 0.49 — well 0 strictly nearest
    const capture_result result = resolve_capture({wells, 2}, pos);
    EXPECT_EQ(result.winner_index, 0);
    EXPECT_FALSE(result.tie);
}

TEST(scoring, well_outside_capture_radius_never_wins) {
    // Well 1 is nearer but its capture radius does not reach the particle.
    const gravity_well wells[] = {make_well(0.0f, 0.0f, 2.0f), make_well(1.0f, 0.0f, 0.1f)};
    const float pos[2] = {0.8f, 0.0f};  // 0.2 from well 1 (outside 0.1), 0.8 from well 0
    const capture_result result = resolve_capture({wells, 2}, pos);
    EXPECT_EQ(result.winner_index, 0);
}

TEST(scoring, exact_distance_tie_yields_no_capture) {
    const gravity_well wells[] = {make_well(-1.0f, 0.0f, 2.0f), make_well(1.0f, 0.0f, 2.0f)};
    const float pos[2] = {0.0f, 0.0f};  // dist_sq exactly 1.0 to both (US1 scenario 5)
    const capture_result result = resolve_capture({wells, 2}, pos);
    EXPECT_EQ(result.winner_index, -1);
    EXPECT_TRUE(result.tie);
}

TEST(scoring, inactive_well_is_not_a_capture_candidate) {
    gravity_well near_well = make_well(0.1f, 0.0f, 1.0f);
    near_well.active = false;
    const gravity_well wells[] = {near_well, make_well(1.0f, 0.0f, 2.0f)};
    const float pos[2] = {0.2f, 0.0f};
    const capture_result result = resolve_capture({wells, 2}, pos);
    EXPECT_EQ(result.winner_index, 1);  // only the active well competes (FR-013)
}

TEST(scoring, no_candidate_yields_no_capture_without_tie) {
    const gravity_well wells[] = {make_well(5.0f, 5.0f, 0.1f)};
    const float pos[2] = {0.0f, 0.0f};
    const capture_result result = resolve_capture({wells, 1}, pos);
    EXPECT_EQ(result.winner_index, -1);
    EXPECT_FALSE(result.tie);
}

TEST(scoring, resolution_is_identical_under_permuted_well_order) {
    // Article 9 index-independence: permuting the span permutes indices consistently,
    // never outcomes — the same physical well wins in both orders.
    const gravity_well a = make_well(0.0f, 0.0f, 1.0f);
    const gravity_well b = make_well(1.0f, 0.0f, 1.0f);
    const float pos[2] = {0.3f, 0.0f};

    const gravity_well order_ab[] = {a, b};
    const gravity_well order_ba[] = {b, a};
    const capture_result r_ab = resolve_capture({order_ab, 2}, pos);
    const capture_result r_ba = resolve_capture({order_ba, 2}, pos);
    ASSERT_NE(r_ab.winner_index, -1);
    ASSERT_NE(r_ba.winner_index, -1);
    // Winner in AB order is index 0 (well a); in BA order the same well sits at index 1.
    EXPECT_EQ(r_ab.winner_index, 0);
    EXPECT_EQ(r_ba.winner_index, 1);
}

TEST(scoring, award_capture_adds_base_value_and_doubles_under_flag) {
    score_table table{};
    award_capture(table, 2, 1, false);
    EXPECT_EQ(table.scores[2], 1u);
    award_capture(table, 2, 1, true);  // Double Points active: +2 (US3 scenario 4)
    EXPECT_EQ(table.scores[2], 3u);
    EXPECT_EQ(table.scores[0], 0u);
}

TEST(scoring, resolve_captures_awards_and_zeroes_lifetime_same_tick) {
    // SC-004: the capturing tick both credits the score and marks the particle for
    // removal (lifetime zeroed → pool's age_and_retire swap-removes it, research D2).
    const gravity_well wells[] = {make_well(0.0f, 0.0f, 1.0f), make_well(3.0f, 0.0f, 1.0f)};
    particle particles[] = {
        make_particle(0.2f, 0.0f),  // captured by well 0
        make_particle(3.1f, 0.0f),  // captured by well 1
        make_particle(1.5f, 0.0f),  // outside both capture radii — survives
    };
    score_table table{};
    const bool double_points[kMaxPlayers] = {false, false, false, false};

    const std::uint32_t scored_mask =
        resolve_captures({wells, 2}, {particles, 3}, table, double_points);

    EXPECT_EQ(table.scores[0], 1u);
    EXPECT_EQ(table.scores[1], 1u);
    EXPECT_EQ(scored_mask, 0b11u);
    EXPECT_EQ(particles[0].remaining_lifetime_seconds, 0.0);
    EXPECT_EQ(particles[1].remaining_lifetime_seconds, 0.0);
    EXPECT_GT(particles[2].remaining_lifetime_seconds, 0.0);
}

// --- T008: win rules (FR-006, FR-008) ---

TEST(scoring, reaching_exactly_win_score_ends_match_that_tick) {
    score_table table{};
    table.scores[1] = 99;
    award_capture(table, 1, 1, false);  // 99 + 1 = exactly 100 (US2 scenario 3)
    const std::int8_t winner = evaluate_win(table, 2, 0b10u);
    EXPECT_EQ(winner, 1);
    EXPECT_EQ(table.winner, 1);
    EXPECT_FALSE(table.sudden_death);
}

TEST(scoring, overshooting_past_win_score_also_wins) {
    score_table table{};
    table.scores[0] = 99;
    award_capture(table, 0, 1, true);  // 99 + 2 = 101 under Double Points
    const std::int8_t winner = evaluate_win(table, 2, 0b01u);
    EXPECT_EQ(winner, 0);
    EXPECT_EQ(table.scores[0], 101u);
}

TEST(scoring, simultaneous_crossers_enter_sudden_death_then_sole_scorer_wins) {
    score_table table{};
    table.scores[0] = 100;
    table.scores[1] = 102;
    // Both crossed on the same evaluation → sudden death, no winner yet (FR-008).
    std::int8_t winner = evaluate_win(table, 2, 0b11u);
    EXPECT_EQ(winner, -1);
    EXPECT_TRUE(table.sudden_death);
    EXPECT_EQ(table.winner, -1);

    // Tick with no captures: still no winner.
    winner = evaluate_win(table, 2, 0u);
    EXPECT_EQ(winner, -1);

    // Both contenders capture the same tick: sudden death continues.
    winner = evaluate_win(table, 2, 0b11u);
    EXPECT_EQ(winner, -1);
    EXPECT_TRUE(table.sudden_death);

    // Next sole-capture tick decides (US2 scenario 4).
    table.scores[1] += 1;
    winner = evaluate_win(table, 2, 0b10u);
    EXPECT_EQ(winner, 1);
    EXPECT_EQ(table.winner, 1);
}

TEST(scoring, no_scoring_after_winner_latched) {
    score_table table{};
    table.scores[3] = 100;
    ASSERT_EQ(evaluate_win(table, 4, 0b1000u), 3);
    // Scores freeze on the winning tick (US2 scenario 3).
    award_capture(table, 0, 1, false);
    award_capture(table, 3, 1, true);
    EXPECT_EQ(table.scores[0], 0u);
    EXPECT_EQ(table.scores[3], 100u);
    // Re-evaluation keeps the latched winner.
    EXPECT_EQ(evaluate_win(table, 4, 0b0001u), 3);
    EXPECT_EQ(table.winner, 3);
}

TEST(scoring, reset_scores_clears_table_for_new_match) {
    score_table table{};
    table.scores[0] = 100;
    table.scores[1] = 55;
    ASSERT_EQ(evaluate_win(table, 2, 0b01u), 0);
    table.sudden_death = true;  // force-set to prove reset clears it
    reset_scores(table);
    EXPECT_EQ(table.scores[0], 0u);
    EXPECT_EQ(table.scores[1], 0u);
    EXPECT_EQ(table.winner, -1);
    EXPECT_FALSE(table.sudden_death);
}

}  // namespace
