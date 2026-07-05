// Force applicators — apply_force implementation.
//
// FIX: none yet (initial implementation).

#include "engine_demo/vfx/force_applicator.h"

namespace engine_demo::vfx {

namespace {

void apply_constant(const float acceleration[3], float out_acceleration[3]) noexcept {
    out_acceleration[0] = acceleration[0];
    out_acceleration[1] = acceleration[1];
    out_acceleration[2] = acceleration[2];
}

void apply_turbulence(const turbulence_force& t,
                      engine_demo::sim::rng& rng,
                      float out_acceleration[3]) noexcept {
    // Exactly 3 draws (one per axis), regardless of `strength`, so downstream determinism
    // never depends on the configured strength value.
    for (int axis = 0; axis < 3; ++axis) {
        const double unit = rng.next_double_unit();      // [0, 1)
        const double signed_unit = unit * 2.0 - 1.0;      // [-1, 1)
        out_acceleration[axis] = static_cast<float>(signed_unit) * t.strength;
    }
}

}  // namespace

void apply_force(const force_applicator& force,
                  engine_demo::sim::rng& rng,
                  float out_acceleration[3]) noexcept {
    if (const auto* gravity = eastl::get_if<gravity_force>(&force)) {
        apply_constant(gravity->acceleration, out_acceleration);
        return;
    }
    if (const auto* wind = eastl::get_if<wind_force>(&force)) {
        apply_constant(wind->acceleration, out_acceleration);
        return;
    }
    if (const auto* turbulence = eastl::get_if<turbulence_force>(&force)) {
        apply_turbulence(*turbulence, rng, out_acceleration);
        return;
    }
}

}  // namespace engine_demo::vfx
