// GoogleTest for engine_demo::vfx force applicators.

#include "engine_demo/vfx/force_applicator.h"

#include <gtest/gtest.h>

namespace {

using engine_demo::sim::rng;
using engine_demo::vfx::apply_force;
using engine_demo::vfx::force_applicator;
using engine_demo::vfx::gravity_force;
using engine_demo::vfx::turbulence_force;
using engine_demo::vfx::wind_force;

TEST(force_applicator, gravity_force_passes_through_configured_acceleration_without_consuming_rng) {
    const force_applicator force = gravity_force{{0.0f, -9.81f, 0.0f}};
    rng live{42u};
    rng reference{42u};

    float out[3]{};
    apply_force(force, live, out);

    EXPECT_FLOAT_EQ(out[0], 0.0f);
    EXPECT_FLOAT_EQ(out[1], -9.81f);
    EXPECT_FLOAT_EQ(out[2], 0.0f);
    EXPECT_EQ(live.next_u32(), reference.next_u32());
}

TEST(force_applicator, wind_force_passes_through_configured_acceleration_without_consuming_rng) {
    const force_applicator force = wind_force{{3.0f, 0.0f, -1.5f}};
    rng live{7u};
    rng reference{7u};

    float out[3]{};
    apply_force(force, live, out);

    EXPECT_FLOAT_EQ(out[0], 3.0f);
    EXPECT_FLOAT_EQ(out[1], 0.0f);
    EXPECT_FLOAT_EQ(out[2], -1.5f);
    EXPECT_EQ(live.next_u32(), reference.next_u32());
}

TEST(force_applicator, turbulence_force_output_is_bounded_by_strength) {
    const force_applicator force = turbulence_force{2.5f, 1.0f};
    rng r{123u};

    for (int i = 0; i < 50; ++i) {
        float out[3]{};
        apply_force(force, r, out);
        for (float axis : out) {
            EXPECT_LE(axis, 2.5f);
            EXPECT_GE(axis, -2.5f);
        }
    }
}

TEST(force_applicator, turbulence_force_with_zero_strength_yields_zero_but_still_consumes_three_draws) {
    const force_applicator force = turbulence_force{0.0f, 1.0f};
    rng with_turbulence{99u};
    rng reference{99u};

    float out[3]{};
    apply_force(force, with_turbulence, out);

    EXPECT_FLOAT_EQ(out[0], 0.0f);
    EXPECT_FLOAT_EQ(out[1], 0.0f);
    EXPECT_FLOAT_EQ(out[2], 0.0f);

    // Exactly 3 draws should have been consumed from `with_turbulence`: advance the
    // reference stream by 3 draws and expect both streams to agree again.
    (void)reference.next_double_unit();
    (void)reference.next_double_unit();
    (void)reference.next_double_unit();
    EXPECT_EQ(with_turbulence.next_u32(), reference.next_u32());
}

TEST(force_applicator, turbulence_force_is_deterministic_for_identical_seeds) {
    const force_applicator force = turbulence_force{1.0f, 1.0f};
    rng a{555u};
    rng b{555u};

    float out_a[3]{};
    float out_b[3]{};
    apply_force(force, a, out_a);
    apply_force(force, b, out_b);

    EXPECT_EQ(out_a[0], out_b[0]);
    EXPECT_EQ(out_a[1], out_b[1]);
    EXPECT_EQ(out_a[2], out_b[2]);
}

}  // namespace
