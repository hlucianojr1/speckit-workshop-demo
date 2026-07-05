// Fixed-step game loop with accumulator.
//
// Constitutional articles satisfied:
//   - 5 (determinism: double accumulator)
//   - 6 (real-time)
//
// FIX BUG-002: accumulator is now `double` per Article 5; the former `float`
// accumulator drifted after ~30 s of simulated time.

#pragma once

#include <cstddef>
#include <cstdint>

namespace engine_demo::sim {

struct game_loop_config {
    double fixed_step_seconds{1.0 / 60.0};
    double max_substeps{8.0};
};

class [[nodiscard]] game_loop {
   public:
    explicit game_loop(game_loop_config cfg) noexcept;

    // Advance the simulation by `delta_seconds`. Returns the number of fixed-step
    // substeps taken. Pure function over (cfg, accumulator) — no global state.
    [[nodiscard]] std::uint32_t advance(double delta_seconds) noexcept;

    [[nodiscard]] double accumulator_seconds() const noexcept;

   private:
    game_loop_config m_cfg;
    // Article 5: time accumulators are double, never float.
    double m_accumulator_seconds{0.0};
};

}  // namespace engine_demo::sim
