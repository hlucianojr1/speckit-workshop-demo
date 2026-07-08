// Orbital Arena — per-tick player input frames and the append-only replay log.
//
// Constitutional articles satisfied:
//   - 1 (no exceptions: arena_status enum)
//   - 3, 4 (EASTL, allocator-aware: eastl::vector<tick_inputs, eastl_allocator_ref>)
//   - 5, 10 (determinism: log stores post-clamp values; replay feeds them back verbatim)
//   - 6 (real-time: log reserved once at construction, appends never allocate)
//
// Player→well mapping is identity by roster slot, fixed at lobby time (FR-015):
// player i's input_frame always drives well i. No remapping API exists in v1.

#pragma once

#include "engine_demo/allocator.h"
#include "orbital_arena/types.h"

#include <EASTL/vector.h>

#include <cstddef>

namespace orbital_arena {

// One player's steering input for one fixed tick. Values are stored post-clamp
// (steer per-axis in [-1, 1], strength in [0, 1]) — log-what-you-simulate (FR-015).
struct input_frame {
    float steer[2]{};
    float strength{0.0f};
};

// All players' inputs for one tick (identity slot mapping, FR-015).
struct tick_inputs {
    input_frame players[kMaxPlayers]{};
};

// Upper bound on recorded ticks per match (> 1000-frame test matches; Article 6:
// the log is reserved once for this many entries and never reallocates).
inline constexpr std::size_t kMaxRecordedTicks = 4096;

// Clamps a raw frame: steer axes to [-1, 1], strength to [0, 1]. NaN maps to 0
// (deterministic policy — replaying the log must never re-clamp differently,
// Article 10).
[[nodiscard]] input_frame clamp_input(const input_frame& raw) noexcept;

// Append-only per-match input log (FR-016). Records inputs for EVERY tick in EVERY
// match state; gameplay rules honor them only in `playing` (FR-014).
class [[nodiscard]] input_log {
   public:
    input_log(engine_demo::allocator& alloc, std::size_t max_ticks) noexcept;

    input_log(const input_log&) = delete;
    input_log& operator=(const input_log&) = delete;
    input_log(input_log&&) noexcept = default;
    input_log& operator=(input_log&&) noexcept = default;

    // Appends one tick's inputs. Returns log_full (frame dropped, no reallocation)
    // once max_ticks entries are stored (Article 6).
    [[nodiscard]] arena_status append(const tick_inputs& frame) noexcept;

    [[nodiscard]] std::size_t size() const noexcept { return m_frames.size(); }

    // Precondition: tick < size().
    [[nodiscard]] const tick_inputs& at(std::size_t tick) const noexcept {
        return m_frames[tick];
    }

    // Resets for a new match; retains the reserved capacity (no allocation).
    void clear() noexcept;

   private:
    std::size_t m_max_ticks;
    eastl::vector<tick_inputs, engine_demo::eastl_allocator_ref> m_frames;
};

}  // namespace orbital_arena
