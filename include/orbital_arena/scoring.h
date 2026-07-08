// Orbital Arena — deterministic capture contention and per-player scoring.
//
// Constitutional articles satisfied:
//   - 1 (no exceptions), 2 (no RTTI)
//   - 5, 10 (determinism: pure functions over inputs; no rng, no allocation)
//   - 6 (real-time: span iteration only)
//   - 9 (fairness: contention compares distance values ONLY; player index is never
//        a tie-break — index-independence is structural, research.md D3)
//
// Well index == player index (identity roster mapping, FR-015).

#pragma once

#include "engine_demo/vfx/particle.h"
#include "orbital_arena/arena.h"
#include "orbital_arena/gravity_well.h"

#include <EASTL/span.h>

#include <cstdint>

namespace orbital_arena {

// Outcome of one contention resolution (FR-007).
struct capture_result {
    std::int8_t winner_index{-1};  // -1: no capture this tick (no candidate or exact tie)
    bool        tie{false};        // true iff >=2 wells tied at the strict minimum distance
};

// Per-player scores plus match-outcome latches (data-model.md).
struct score_table {
    std::uint32_t scores[kMaxPlayers]{};
    std::int8_t   winner{-1};        // set once on game_over; scores freeze afterwards
    bool          sudden_death{false};
};

// FR-007 contention: among wells whose capture radius contains `pos`, returns the
// index of the STRICTLY nearest (squared distance); exact tie -> {-1, tie=true}.
// Iterates wells[0..count) in index order but compares distance values ONLY.
[[nodiscard]] capture_result resolve_capture(eastl::span<const gravity_well> wells,
                                             const float pos[2]) noexcept;

// FR-005: +base_value (x2 if double_points) to scores[player]. No-op once a winner
// is latched (score freeze on the winning tick — US2 scenario 3); state gating to
// `playing` is enforced at the arena level.
void award_capture(score_table& t, std::uint8_t player,
                   std::uint32_t base_value, bool double_points) noexcept;

// Applies FR-007 contention to every live particle in one pass (fixed particle index
// order): the winning well's player is awarded base value 1 (x2 if double_points[player])
// and the particle's remaining_lifetime_seconds is zeroed so the pool's own
// age_and_retire() swap-removes it at end of tick (research.md D2 — zero engine changes).
// Returns a bitmask: bit i set iff player i captured >= 1 particle this tick (feeds
// evaluate_win's scored_this_tick).
[[nodiscard]] std::uint32_t resolve_captures(eastl::span<const gravity_well> wells,
                                             eastl::span<engine_demo::vfx::particle> particles,
                                             score_table& table,
                                             const bool double_points[kMaxPlayers]) noexcept;

}  // namespace orbital_arena
