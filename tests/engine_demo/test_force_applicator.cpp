// GoogleTest for engine_demo::vfx force applicators.
// See specs/004-particle-vfx-subsystem/contracts/force_applicator.md for the full contract.

#include "engine_demo/vfx/force_applicator.h"

#include <gtest/gtest.h>

namespace {

using engine_demo::sim::rng;
using engine_demo::vfx::apply_force;
using engine_demo::vfx::force_applicator;
using engine_demo::vfx::gravity_force;
using engine_demo::vfx::turbulence_force;
using engine_demo::vfx::wind_force;

TEST(force_applicator, gravity_force_passes_through_and_does_not_draw_rng) {
    rng r{42};
    rng reference{42};

    force_applicator force = gravity_force{{0.0f, -9.81f, 0.0f}};
    float out[3]{};
    apply_force(force, r, out);

    EXPECT_FLOAT_EQ(out[0], 0.0f);
    EXPECT_FLOAT_EQ(out[1], -9.81f);
    EXPECT_FLOAT_EQ(out[2], 0.0f);

    // No draw consumed: r's next draw must match an untouched reference's first draw.
    EXPECT_EQ(r.next_u32(), reference.next_u32());
}

TEST(force_applicator, wind_force_passes_through_and_does_not_draw_rng) {
    rng r{7};
    rng reference{7};

    force_applicator force = wind_force{{3.0f, 0.5f, -1.0f}};
    float out[3]{};
    apply_force(force, r, out);

    EXPECT_FLOAT_EQ(out[0], 3.0f);
    EXPECT_FLOAT_EQ(out[1], 0.5f);
    EXPECT_FLOAT_EQ(out[2], -1.0f);
    EXPECT_EQ(r.next_u32(), reference.next_u32());
}

TEST(force_applicator, turbulence_force_output_bounded_by_strength) {
    rng r{123};
    force_applicator force = turbulence_force{2.5f, 1.0f};
    float out[3]{};
    apply_force(force, r, out);

    for (float component : out) {
        EXPECT_GE(component, -2.5f);
        EXPECT_LE(component, 2.5f);
    }
}

// Edge case: strength == 0 always yields {0,0,0} but still consumes exactly 3 draws.
TEST(force_applicator, turbulence_force_zero_strength_yields_zero_but_still_draws_three) {
    rng r{99};
    rng reference{99};

    force_applicator force = turbulence_force{0.0f, 1.0f};
    float out[3]{};
    apply_force(force, r, out);

    EXPECT_FLOAT_EQ(out[0], 0.0f);
    EXPECT_FLOAT_EQ(out[1], 0.0f);
    EXPECT_FLOAT_EQ(out[2], 0.0f);

    // Exactly 3 draws consumed: skip 3 on the reference, then both must agree again.
    [[maybe_unused]] double const skip1 = reference.next_double_unit();
    [[maybe_unused]] double const skip2 = reference.next_double_unit();
    [[maybe_unused]] double const skip3 = reference.next_double_unit();
    EXPECT_EQ(r.next_u32(), reference.next_u32());
}

// Determinism: two identically-seeded rng instances produce identical turbulence output.
TEST(force_applicator, turbulence_force_is_deterministic_given_same_seed) {
    rng r_a{55};
    rng r_b{55};
    force_applicator force = turbulence_force{1.0f, 2.0f};

    float out_a[3]{};
    float out_b[3]{};
    apply_force(force, r_a, out_a);
    apply_force(force, r_b, out_b);

    EXPECT_FLOAT_EQ(out_a[0], out_b[0]);
    EXPECT_FLOAT_EQ(out_a[1], out_b[1]);
    EXPECT_FLOAT_EQ(out_a[2], out_b[2]);
}

}  // namespace
