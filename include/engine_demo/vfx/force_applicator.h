// Force applicators — composable per-particle acceleration contributions.
//
// Constitutional articles satisfied:
//   - 2 (no RTTI: eastl::variant tagged union for type discrimination)
//   - 5 (determinism: turbulence draws a fixed count from the caller-supplied rng)
//   - 6 (real-time: apply_force never allocates)
//
// See specs/004-particle-vfx-subsystem/contracts/force_applicator.md for the full contract.

#pragma once

#include <EASTL/variant.h>

#include "engine_demo/sim/rng.h"

namespace engine_demo::vfx {

struct gravity_force {
    float acceleration[3]{0.0f, -9.81f, 0.0f};
};

struct wind_force {
    float acceleration[3]{};
};

struct turbulence_force {
    float strength{0.0f};
    float frequency{1.0f};
};

using force_applicator = eastl::variant<gravity_force, wind_force, turbulence_force>;

// Returns this applicator's acceleration contribution (m/s^2) for one particle on one
// tick. `rng` is consumed only by turbulence_force, in a fixed draw count (3, one per
// axis) regardless of `strength` — callers MUST invoke this once per live particle per
// tick, in stable dense-array index order, to preserve determinism (Article 5).
void apply_force(const force_applicator& force,
                  engine_demo::sim::rng& rng,
                  float out_acceleration[3]) noexcept;

}  // namespace engine_demo::vfx
