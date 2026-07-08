# Contract: scoring

**Header**: `include/orbital_arena/scoring.h` | **Impl**: `src/orbital_arena/scoring.cpp`
**Tests**: `tests/orbital_arena/test_scoring.cpp`
**Satisfies**: FR-004, FR-005, FR-006, FR-007, FR-008, Articles 1, 5, 9, 10.

## Types

```cpp
namespace orbital_arena {

struct capture_result {
    int8_t winner_index{-1};   // -1: no capture this tick (no candidate or exact tie)
    bool   tie{false};         // true iff >=2 wells tied at the strict minimum distance
};

struct score_table {
    uint32_t scores[kMaxPlayers]{};
    int8_t   winner{-1};       // set on game_over
    bool     sudden_death{false};
};

}
```

## Functions

```cpp
// FR-007 contention: among wells whose capture radius contains `pos`, returns the
// index of the STRICTLY nearest (squared distance); exact tie -> {-1, tie=true}.
// Iterates wells[0..count) in index order but compares distance values ONLY —
// player index is never a tie-break (Article 9).
[[nodiscard]] capture_result resolve_capture(eastl::span<const gravity_well> wells,
                                             const float pos[2]) noexcept;

// FR-005: +base_value (x2 if double_points) to scores[player]. Caller guarantees
// match state == playing (enforced at arena level, asserted here in debug).
void award_capture(score_table& t, uint8_t player,
                   uint32_t base_value, bool double_points) noexcept;

// FR-006/FR-008 evaluated once per tick AFTER all captures resolve:
// - exactly one player >= kWinScore            -> returns that index (match ends)
// - >=2 players >= kWinScore this evaluation   -> sets t.sudden_death, returns -1
// - sudden_death && exactly one contender scored this tick -> returns that index
// `scored_this_tick` bitmask: bit i set iff player i captured >=1 particle this tick.
[[nodiscard]] int8_t evaluate_win(score_table& t, uint8_t player_count,
                                  uint32_t scored_this_tick) noexcept;

void reset_scores(score_table& t) noexcept;  // game_over -> lobby (FR-012)
```

## Behavioral guarantees

- Deterministic: pure functions over inputs; no rng, no allocation (Articles 5, 6, 10).
- Index-independence: relabeling wells (permuting the span) permutes the result
  identically — verified by test (Article 9 / SC-003).
- Same-tick scoring: `award_capture` is called inside the capturing tick (SC-004).

## Test obligations

1. Nearest-well wins among 2 wells; farther well never scores.
2. Exact-tie squared distances → no capture, `tie == true` (US1 scenario 5).
3. Permuting well order flips indices consistently, never outcomes (Article 9).
4. 99 + 1-point capture → win on same evaluation (US2 scenario 3).
5. Two players cross 100 same tick → sudden_death, no winner; next sole scorer wins
   (US2 scenario 4, FR-008).
6. Double Points doubles exactly, and only while flagged (US3 scenario 4).
