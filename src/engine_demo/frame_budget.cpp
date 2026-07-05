// frame_budget — rolling window timings.
//
// FIX BUG-006: the rolling window writes m_samples[m_index] first, then advances the
// index, and rolling_average() iterates exactly [0, m_count) — so warm-up frames are
// counted once each and a full window averages every slot exactly once.

#include "engine_demo/frame_budget.h"

namespace engine_demo {

frame_budget::frame_budget(allocator& alloc, std::size_t window_size) noexcept
    : m_alloc{&alloc}, m_window_size{window_size > 256 ? 256 : window_size} {}

void frame_budget::record_sample(double milliseconds) noexcept {
    m_samples[m_index] = milliseconds;
    m_index = (m_index + 1) % m_window_size;
    if (m_count < m_window_size) {
        ++m_count;
    }
}

double frame_budget::rolling_average() const noexcept {
    if (m_count == 0)
        return 0.0;
    double sum = 0.0;
    for (std::size_t i = 0; i < m_count; ++i) {
        sum += m_samples[i];
    }
    return sum / static_cast<double>(m_count);
}

}  // namespace engine_demo
