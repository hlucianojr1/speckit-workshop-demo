// Orbital Arena — match lifecycle state machine: lobby → countdown → playing →
// game_over → lobby (FR-012).
//
// Constitutional articles satisfied:
//   - 1 (no exceptions: illegal calls return arena_status::wrong_state, no mutation)
//   - 2 (no RTTI: plain enum states, no polymorphism)
//   - 5, 10 (determinism: pure automaton — integer tick countdown, no rng, no floats)
//   - 6 (real-time: no containers, no allocation)

#pragma once

#include "orbital_arena/arena.h"

#include <EASTL/optional.h>

#include <cstdint>

namespace orbital_arena {

enum class match_state : std::uint8_t { lobby, countdown, playing, game_over };

// Roster entry (data-model.md). Roster is fixed once countdown completes (FR-015).
struct player_slot {
    bool joined{false};
    bool ready{false};
    bool departed{false};
};

class [[nodiscard]] match {
   public:
    match() noexcept = default;  // POD-ish; no containers, no allocator needed

    // Reported by tick() so the arena performs transition side effects (reset scores,
    // place wells symmetrically) exactly once on the transition tick.
    struct transition {
        match_state from;
        match_state to;
    };

    // Lobby management (valid only in lobby; otherwise wrong_state — FR-012).
    // join: player must be < kMaxPlayers and not already joined (invalid_argument).
    [[nodiscard]] arena_status join(std::uint8_t player) noexcept;

    // set_ready is additionally legal during countdown, where un-readying interrupts
    // the countdown and returns the match to lobby immediately (FR-013).
    [[nodiscard]] arena_status set_ready(std::uint8_t player, bool ready) noexcept;

    // State machine tick — called once per arena tick, BEFORE gameplay systems:
    //   lobby:     all joined (>= kMinPlayers) ready -> countdown (timer := kCountdownTicks)
    //   countdown: timer-- reaches 0                 -> playing (exactly 180 ticks, D4)
    //   playing / game_over: no self-driven transitions (win + acknowledge are calls).
    [[nodiscard]] eastl::optional<transition> tick() noexcept;

    // Departure: lobby/countdown -> removed from roster (countdown interrupts to lobby);
    // playing/game_over -> slot flagged departed, roster size unchanged (FR-013). The
    // arena deactivates the well and calls enter_game_over() when active_count() < kMinPlayers.
    void leave(std::uint8_t player) noexcept;

    // Called by the arena when scoring latches a winner or active players drop below 2.
    void enter_game_over() noexcept;

    // game_over -> lobby; ready flags cleared, departed players removed (FR-012).
    [[nodiscard]] arena_status acknowledge_results() noexcept;

    [[nodiscard]] match_state state() const noexcept { return m_state; }
    [[nodiscard]] bool gameplay_input_enabled() const noexcept {
        return m_state == match_state::playing;  // FR-014
    }
    [[nodiscard]] std::uint8_t joined_count() const noexcept;
    [[nodiscard]] std::uint8_t active_count() const noexcept;  // joined && !departed
    [[nodiscard]] std::uint32_t countdown_remaining_ticks() const noexcept {
        return m_countdown_ticks;
    }

   private:
    [[nodiscard]] bool all_joined_ready() const noexcept;

    match_state m_state{match_state::lobby};
    player_slot m_slots[kMaxPlayers]{};
    std::uint32_t m_countdown_ticks{0};
};

}  // namespace orbital_arena
