# Contract: input

**Header**: `include/orbital_arena/input.h` | **Impl**: `src/orbital_arena/input.cpp`
**Tests**: `tests/orbital_arena/test_input.cpp`
**Satisfies**: FR-015, FR-016, Articles 1, 3, 4, 5, 6, 10.

## Types

```cpp
namespace orbital_arena {

struct input_frame {
    float steer[2]{};     // clamped to [-1, 1] per axis on ingest
    float strength{0.0f}; // clamped to [0, 1] on ingest
};

struct tick_inputs {
    input_frame players[kMaxPlayers]{};
};

// Append-only per-match input log (FR-016). Reserved once at construction for
// kMaxRecordedTicks entries (Article 6: no reallocation, ever).
class [[nodiscard]] input_log {
   public:
    input_log(engine_demo::allocator& alloc, std::size_t max_ticks) noexcept;
    // non-copyable, movable (noexcept)
};

inline constexpr std::size_t kMaxRecordedTicks = 4096;  // > 1000-frame test matches

}
```

## Functions

```cpp
// Clamp helper — applied by the arena on ingest so the LOG stores the clamped values
// (replaying the log must not re-clamp differently; log-what-you-simulate, Article 10).
[[nodiscard]] input_frame clamp_input(const input_frame& raw) noexcept;

// Methods of input_log:
[[nodiscard]] arena_status append(const tick_inputs& frame) noexcept; // log_full when at capacity
[[nodiscard]] std::size_t size() const noexcept;
[[nodiscard]] const tick_inputs& at(std::size_t tick) const noexcept; // precondition: tick < size()
void clear() noexcept;                                                // new match
```

## Behavioral guarantees

- Player→well mapping is identity by roster slot, fixed at lobby time (FR-015): player
  `i`'s `input_frame` always drives well `i`. No remapping API exists in v1.
- Inputs are recorded for EVERY tick in EVERY match state (spec edge case: ignored for
  gameplay outside `playing`, but logged for faithful replay).
- Log stores post-clamp values; replay feeds them back verbatim (Article 10).
- `append` past capacity returns `log_full` and drops the frame — never reallocates
  (Article 6); the arena surfaces this status to the caller.

## Test obligations

1. `clamp_input`: out-of-range steer/strength clamped; in-range untouched; NaN policy
   documented (NaN → 0, deterministic).
2. Log round-trip: N appended frames read back identically in order.
3. `log_full` at capacity; `size()` stops growing; no crash.
4. Allocation check: `allocator::bytes_used()` stable across appends after construction.
