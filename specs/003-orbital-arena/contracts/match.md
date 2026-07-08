# Contract: match

**Header**: `include/orbital_arena/match.h` | **Impl**: `src/orbital_arena/match.cpp`
**Tests**: `tests/orbital_arena/test_match.cpp`
**Satisfies**: FR-012, FR-013, FR-014 (state gate), Articles 1, 2, 5.

## Types

```cpp
namespace orbital_arena {

enum class match_state : uint8_t { lobby, countdown, playing, game_over };

struct player_slot {
    bool joined{false};
    bool ready{false};
    bool departed{false};
};

class [[nodiscard]] match {
   public:
    match() noexcept = default;   // POD-ish; no containers, no allocator needed
};

}
```

## Functions (methods of `match`)

```cpp
// Lobby management (valid only in lobby; otherwise returns wrong_state — FR-012):
[[nodiscard]] arena_status join(uint8_t player) noexcept;        // up to kMaxPlayers
[[nodiscard]] arena_status set_ready(uint8_t player, bool ready) noexcept;

// State machine tick — called once per arena tick, BEFORE gameplay systems:
//   lobby:     all joined (>= kMinPlayers) ready        -> countdown, timer := kCountdownTicks
//   countdown: any unready/departure                    -> lobby (US edge: countdown interruption)
//              timer-- == 0                             -> playing (signals reset via return)
//   playing:   (win handled by arena via enter_game_over)
//   game_over: (waits for acknowledge)
// Returns a transition report so the arena can perform side effects (reset scores,
// place wells symmetrically) exactly once on the transition tick.
struct transition { match_state from, to; };
[[nodiscard]] eastl::optional<transition> tick() noexcept;

// Departure during countdown -> lobby; during playing -> slot.departed = true and the
// arena deactivates the well; if active players drop below kMinPlayers the arena calls
// enter_game_over(last_remaining) (FR-013).
void leave(uint8_t player) noexcept;

// Called by the arena when scoring reports a winner or player count drops below 2.
void enter_game_over() noexcept;

// game_over -> lobby; ready flags cleared (scores reset by arena) (FR-012).
[[nodiscard]] arena_status acknowledge_results() noexcept;

// Queries:
[[nodiscard]] match_state state() const noexcept;
[[nodiscard]] bool gameplay_input_enabled() const noexcept;  // state == playing (FR-014)
[[nodiscard]] uint8_t joined_count() const noexcept;
[[nodiscard]] uint8_t active_count() const noexcept;         // joined && !departed
[[nodiscard]] uint32_t countdown_remaining_ticks() const noexcept;
```

## Behavioral guarantees

- Only the transitions in the FR-012 diagram are reachable; anything else returns
  `wrong_state` without mutating (Article 1 status-enum discipline).
- Countdown is an integer tick counter (research D4) — expires after exactly 180 ticks.
- No allocation, no rng — the state machine is a pure deterministic automaton.

## Test obligations

1. Full happy path: 2 joins → both ready → countdown → 180 ticks → playing.
2. Unready/leave during countdown → back to lobby (US2 edge).
3. Ready/join rejected outside lobby (`wrong_state`).
4. `gameplay_input_enabled` false in lobby/countdown/game_over (FR-014 / US2 scenario 6).
5. Departure in playing keeps state until active < 2 → game_over (FR-013).
6. acknowledge → lobby with ready flags cleared (US2 scenario 5).
