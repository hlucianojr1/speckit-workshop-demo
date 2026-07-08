// Orbital Arena input module — see include/orbital_arena/input.h for the contract.

#include "orbital_arena/input.h"

namespace orbital_arena {

namespace {

// NaN → 0, then clamp to [lo, hi]. NaN comparisons are all false, so the explicit
// self-inequality test keeps the policy deterministic (FR-015, Article 10).
[[nodiscard]] float clamp_component(float value, float lo, float hi) noexcept {
    if (value != value) {  // NaN
        return 0.0f;
    }
    if (value < lo) {
        return lo;
    }
    if (value > hi) {
        return hi;
    }
    return value;
}

}  // namespace

input_frame clamp_input(const input_frame& raw) noexcept {
    input_frame clamped{};
    clamped.steer[0] = clamp_component(raw.steer[0], -1.0f, 1.0f);
    clamped.steer[1] = clamp_component(raw.steer[1], -1.0f, 1.0f);
    clamped.strength = clamp_component(raw.strength, 0.0f, 1.0f);
    return clamped;
}

input_log::input_log(engine_demo::allocator& alloc, std::size_t max_ticks) noexcept
    : m_max_ticks{max_ticks}, m_frames{engine_demo::eastl_allocator_ref{alloc}} {
    // Article 6: single up-front reservation; append never reallocates.
    m_frames.reserve(m_max_ticks);
}

arena_status input_log::append(const tick_inputs& frame) noexcept {
    if (m_frames.size() >= m_max_ticks) {
        return arena_status::log_full;
    }
    m_frames.push_back(frame);
    return arena_status::ok;
}

void input_log::clear() noexcept {
    m_frames.clear();  // capacity retained — no allocation on subsequent appends
}

}  // namespace orbital_arena
