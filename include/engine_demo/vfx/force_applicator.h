// Force applicators — composable acceleration contributors for engine_demo::vfx::emitter.
//
// Constitutional articles satisfied:
//   - 2 (no RTTI: eastl::variant tagged union, no inheritance/vtables)
//   - 5 (determinism: turbulence draws are the only rng consumers here, in a fixed order)

#pragma once

#include "engine_demo/sim/rng.h"

#include <EASTL/variant.h>

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

// Tagged union (Article 2) over the supported force applicator kinds.
using force_applicator = eastl::variant<gravity_force, wind_force, turbulence_force>;

// Writes this applicator's acceleration contribution (m/s^2) into out_acceleration.
// `rng` is consumed only by turbulence_force (exactly 3 draws, one per axis); callers
// MUST invoke this once per live particle per tick, in stable dense-array index order,
// to preserve determinism (Article 5).
void apply_force(const force_applicator& force,
                  engine_demo::sim::rng& rng,
                  float out_acceleration[3]) noexcept;

}  // namespace engine_demo::vfx
