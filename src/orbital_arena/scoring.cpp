// Orbital Arena scoring — see include/orbital_arena/scoring.h for the contract.

#include "orbital_arena/scoring.h"

namespace orbital_arena {

capture_result resolve_capture(eastl::span<const gravity_well> wells,
                               const float pos[2]) noexcept {
    capture_result result{};
    float best_dist_sq = 0.0f;
    bool have_candidate = false;
    bool tied_at_best = false;

    // Fixed well-index iteration order; comparison on squared distance ONLY (D3).
    for (std::size_t i = 0; i < wells.size(); ++i) {
        const gravity_well& well = wells[i];
        if (!is_within_capture(well, pos)) {
            continue;
        }
        const float dx = well.position[0] - pos[0];
        const float dy = well.position[1] - pos[1];
        const float dist_sq = dx * dx + dy * dy;
        if (!have_candidate || dist_sq < best_dist_sq) {
            best_dist_sq = dist_sq;
            result.winner_index = static_cast<std::int8_t>(i);
            have_candidate = true;
            tied_at_best = false;
        } else if (dist_sq == best_dist_sq) {
            tied_at_best = true;  // exact tie of squared distances (US1 scenario 5)
        }
    }

    if (tied_at_best) {
        result.winner_index = -1;
        result.tie = true;
    }
    return result;
}

void award_capture(score_table& t, std::uint8_t player,
                   std::uint32_t base_value, bool double_points) noexcept {
    if (t.winner != -1) {
        return;  // scores freeze once a winner is latched (US2 scenario 3)
    }
    if (player >= kMaxPlayers) {
        return;
    }
    t.scores[player] += double_points ? base_value * 2u : base_value;
}

std::uint32_t resolve_captures(eastl::span<const gravity_well> wells,
                               eastl::span<engine_demo::vfx::particle> particles,
                               score_table& table,
                               const bool double_points[kMaxPlayers]) noexcept {
    std::uint32_t scored_mask = 0;
    for (engine_demo::vfx::particle& p : particles) {
        const float pos[2] = {p.position[0], p.position[1]};
        const capture_result result = resolve_capture(wells, pos);
        if (result.winner_index < 0) {
            continue;  // no candidate, or exact tie -> nobody captures this tick
        }
        const std::uint8_t player = static_cast<std::uint8_t>(result.winner_index);
        award_capture(table, player, 1u, double_points[player]);
        // Mark for removal: the pool's age_and_retire() swap-removes it (research D2);
        // the score is credited the same tick (SC-004).
        p.remaining_lifetime_seconds = 0.0;
        scored_mask |= 1u << player;
    }
    return scored_mask;
}

std::int8_t evaluate_win(score_table& t, std::uint8_t player_count,
                         std::uint32_t scored_this_tick) noexcept {
    if (t.winner != -1) {
        return t.winner;  // already decided; scores are frozen
    }

    const std::uint8_t count = player_count < kMaxPlayers ? player_count : kMaxPlayers;

    if (t.sudden_death) {
        // FR-008: the next tick where exactly one contender captures decides the match.
        std::int8_t sole_scorer = -1;
        std::uint8_t scoring_contenders = 0;
        for (std::uint8_t i = 0; i < count; ++i) {
            const bool contender = t.scores[i] >= kWinScore;
            const bool scored = (scored_this_tick & (1u << i)) != 0;
            if (contender && scored) {
                ++scoring_contenders;
                sole_scorer = static_cast<std::int8_t>(i);
            }
        }
        if (scoring_contenders == 1) {
            t.winner = sole_scorer;
            return t.winner;
        }
        return -1;  // zero or multiple contenders scored: sudden death continues
    }

    std::int8_t crosser = -1;
    std::uint8_t crossers = 0;
    for (std::uint8_t i = 0; i < count; ++i) {
        if (t.scores[i] >= kWinScore) {
            ++crossers;
            crosser = static_cast<std::int8_t>(i);
        }
    }
    if (crossers == 1) {
        t.winner = crosser;  // FR-006: match ends this tick
        return t.winner;
    }
    if (crossers >= 2) {
        t.sudden_death = true;  // FR-008: simultaneous crossing
    }
    return -1;
}

void reset_scores(score_table& t) noexcept {
    for (std::uint32_t& score : t.scores) {
        score = 0;
    }
    t.winner = -1;
    t.sudden_death = false;
}

}  // namespace orbital_arena
