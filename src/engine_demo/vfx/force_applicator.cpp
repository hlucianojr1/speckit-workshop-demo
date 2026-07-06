// engine_demo::vfx::apply_force implementation.
// See specs/004-particle-vfx-subsystem/contracts/force_applicator.md.

#include "engine_demo/vfx/force_applicator.h"

namespace engine_demo::vfx {

void apply_force(const force_applicator& force,
                  engine_demo::sim::rng& rng,
                  float out_acceleration[3]) noexcept {
    if (const gravity_force* g = eastl::get_if<gravity_force>(&force)) {
        out_acceleration[0] = g->acceleration[0];
        out_acceleration[1] = g->acceleration[1];
        out_acceleration[2] = g->acceleration[2];
        return;
    }

    if (const wind_force* w = eastl::get_if<wind_force>(&force)) {
        out_acceleration[0] = w->acceleration[0];
        out_acceleration[1] = w->acceleration[1];
        out_acceleration[2] = w->acceleration[2];
        return;
    }

    const turbulence_force* t = eastl::get_if<turbulence_force>(&force);
    // Fixed draw count (3, one per axis) regardless of `strength` so downstream
    // determinism never depends on the configured strength value (research.md §4).
    for (int axis = 0; axis < 3; ++axis) {
        double const unit = rng.next_double_unit();  // [0, 1)
        double const bipolar = unit * 2.0 - 1.0;      // [-1, 1)
        out_acceleration[axis] = static_cast<float>(bipolar) * t->strength;
    }
}

}  // namespace engine_demo::vfx
