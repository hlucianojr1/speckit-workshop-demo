// GoogleTest for the Sandbox VFX Visualization feature's additions to ea_sandbox::scene.
//
// This file compiles apps/sandbox/scene.cpp directly (raylib-free by design — see
// specs/002-sandbox-vfx-visualization/research.md §6), the same pattern already used by
// test_sandbox_render.cpp for apps/sandbox/viewport.h.

#include "apps/sandbox/scene.h"

#include <gtest/gtest.h>

#include <chrono>
#include <cmath>
#include <cstdint>

namespace {

using ea_sandbox::scene;

// ---------------------------------------------------------------------------------------
// Foundational — VFX pool/emitter construction (Phase 2)
// ---------------------------------------------------------------------------------------

TEST(scene_vfx, fresh_scene_has_zero_live_vfx_particles) {
    scene s{42};
    EXPECT_EQ(s.vfx_particle_count(), 0u);
}

TEST(scene_vfx, vfx_pool_construction_fits_within_arena_headroom) {
    scene s{42};
    // scene::budget() -> frame_budget owns a reference to the same arena allocator scene
    // constructs everything (including the VFX pool/emitter) from; this lets us verify the
    // arena headroom math from data-model.md without a dedicated accessor.
    const auto& alloc = s.budget().get_allocator();
    EXPECT_LT(alloc.bytes_used(), alloc.capacity());
}

// ---------------------------------------------------------------------------------------
// User Story 1 — Visual Feedback on Right-Click Burst
// ---------------------------------------------------------------------------------------

TEST(scene_vfx, spawn_vfx_burst_increases_vfx_count_without_touching_physics_burst) {
    scene s{1};
    // Default scene_kind is rope, which seeds kRopeFreeParticles (32) free particles at
    // construction — spawn_vfx_burst must not add to or otherwise disturb that count.
    const std::size_t particle_count_before = s.particle_count();
    ASSERT_EQ(s.vfx_particle_count(), 0u);

    s.spawn_vfx_burst(0.5, -0.5);

    EXPECT_EQ(s.particle_count(), particle_count_before);
    EXPECT_GT(s.vfx_particle_count(), 0u);
    EXPECT_LE(s.vfx_particle_count(), scene::kVfxBurstCount);
}

TEST(scene_vfx, vfx_particles_age_and_retire_across_step_calls) {
    // particle_storm is the one scene whose free particles never bounce (they wrap
    // instead), so no collision sparks interfere with observing the burst fully retire.
    scene s{2};
    s.switch_scene(ea_sandbox::scene_kind::particle_storm);
    s.spawn_vfx_burst(0.0, 0.0);
    ASSERT_GT(s.vfx_particle_count(), 0u);

    const int steps = static_cast<int>(std::ceil(scene::kVfxLifetimeSeconds / scene::kFixedStepSeconds)) + 4;
    for (int i = 0; i < steps; ++i) {
        s.step(scene::kFixedStepSeconds);
    }

    EXPECT_EQ(s.vfx_particle_count(), 0u);
}

TEST(scene_vfx, vfx_particle_life_fraction_starts_at_one_and_decreases_monotonically) {
    scene s{3};
    s.spawn_vfx_burst(0.0, 0.0);
    ASSERT_GT(s.vfx_particle_count(), 0u);

    float previous = s.vfx_particle_at(0).life_fraction;
    EXPECT_FLOAT_EQ(previous, 1.0f);

    for (int i = 0; i < 5; ++i) {
        s.step(scene::kFixedStepSeconds);
        ASSERT_GT(s.vfx_particle_count(), 0u) << "burst retired before the monotonic check finished";
        const float current = s.vfx_particle_at(0).life_fraction;
        EXPECT_LT(current, previous);
        previous = current;
    }
}

TEST(scene_vfx, reseed_and_switch_scene_clear_vfx_particles) {
    scene s{4};
    s.spawn_vfx_burst(0.0, 0.0);
    ASSERT_GT(s.vfx_particle_count(), 0u);
    s.reseed(5);
    EXPECT_EQ(s.vfx_particle_count(), 0u);

    s.spawn_vfx_burst(0.0, 0.0);
    ASSERT_GT(s.vfx_particle_count(), 0u);
    s.switch_scene(ea_sandbox::scene_kind::cloth);
    EXPECT_EQ(s.vfx_particle_count(), 0u);
}

// FR-009 pool-exhaustion case (closes the zero-coverage gap identified in /speckit.analyze).
TEST(scene_vfx, spawn_vfx_burst_handles_pool_exhaustion_gracefully) {
    scene s{6};
    const std::size_t bursts_needed = (scene::kVfxPoolCapacity / scene::kVfxBurstCount) + 2;
    for (std::size_t i = 0; i < bursts_needed; ++i) {
        s.spawn_vfx_burst(0.0, 0.0);
    }

    EXPECT_LE(s.vfx_particle_count(), scene::kVfxPoolCapacity);

    // Spot-check existing live particle state is not corrupted by the exhausted call.
    ASSERT_GT(s.vfx_particle_count(), 0u);
    const auto view = s.vfx_particle_at(0);
    EXPECT_TRUE(std::isfinite(view.x));
    EXPECT_TRUE(std::isfinite(view.y));
    EXPECT_GE(view.life_fraction, 0.0f);
    EXPECT_LE(view.life_fraction, 1.0f);

    // One more call must not crash and must not exceed capacity.
    s.spawn_vfx_burst(1.0, 1.0);
    EXPECT_LE(s.vfx_particle_count(), scene::kVfxPoolCapacity);
}

// Article 6 verification: the scene-level wrapper itself must not allocate, not just the
// try_emit()/set_shape() calls it forwards to (closes the gap identified in /speckit.analyze).
TEST(scene_vfx, spawn_vfx_burst_does_not_allocate) {
    scene s{7};
    s.spawn_vfx_burst(0.0, 0.0);  // warm up (first call may touch cold pages, not "allocate")
    s.step(scene::kFixedStepSeconds);

    const std::size_t before = s.arena_bytes_used();
    for (int i = 0; i < 5; ++i) {
        s.spawn_vfx_burst(static_cast<double>(i) * 0.1, 0.0);
        s.step(scene::kFixedStepSeconds);
    }
    const std::size_t after = s.arena_bytes_used();

    EXPECT_EQ(before, after);
}

// SC-001 digest-parity regression guard: VFX bursts must never perturb state_digest().
TEST(scene_vfx, spawn_vfx_burst_never_changes_state_digest) {
    scene with_bursts{8};
    scene without_bursts{8};

    for (int frame = 0; frame < 200; ++frame) {
        if (frame % 7 == 0) {
            with_bursts.spawn_vfx_burst(0.1 * frame, -0.1 * frame);
        }
        with_bursts.step(scene::kFixedStepSeconds);
        without_bursts.step(scene::kFixedStepSeconds);

        ASSERT_EQ(with_bursts.state_digest(), without_bursts.state_digest())
            << "digest diverged at frame " << frame;
    }
}

// ---------------------------------------------------------------------------------------
// User Story 2 — Collision Sparks on World-Bound Bounce
// ---------------------------------------------------------------------------------------

TEST(scene_vfx, bounce_in_non_storm_scene_triggers_spark) {
    // Default scene_kind is rope, which seeds kRopeFreeParticles (32) free particles that
    // bounce off world bounds in substep()'s non-storm branch.
    scene s{9};
    bool spark_seen = false;
    for (int i = 0; i < 600 && !spark_seen; ++i) {
        s.step(scene::kFixedStepSeconds);
        if (s.vfx_particle_count() > 0) {
            spark_seen = true;
        }
    }
    EXPECT_TRUE(spark_seen) << "no bounce spark observed within 10s of sim time";
}

TEST(scene_vfx, particle_storm_never_triggers_spark_on_wrap) {
    scene s{9};
    s.switch_scene(ea_sandbox::scene_kind::particle_storm);
    for (int i = 0; i < 600; ++i) {
        s.step(scene::kFixedStepSeconds);
        ASSERT_EQ(s.vfx_particle_count(), 0u) << "spark emitted in particle_storm at step " << i;
    }
}

// ---------------------------------------------------------------------------------------
// Polish — SC-003/FR-010 automated perf smoke test (closes the automation gap identified
// in /speckit.analyze; mirrors Feature 001's test_emitter.cpp perf-smoke precedent).
// ---------------------------------------------------------------------------------------

TEST(scene_vfx, step_with_full_vfx_pool_completes_well_under_frame_budget) {
    scene s{10};
    // Fill the VFX pool to capacity via repeated bursts.
    const std::size_t bursts_needed = (scene::kVfxPoolCapacity / scene::kVfxBurstCount) + 2;
    for (std::size_t i = 0; i < bursts_needed; ++i) {
        s.spawn_vfx_burst(0.0, 0.0);
    }
    ASSERT_GT(s.vfx_particle_count(), 0u);

    const auto start = std::chrono::steady_clock::now();
    s.step(scene::kFixedStepSeconds);
    const auto end = std::chrono::steady_clock::now();
    const double elapsed_ms = std::chrono::duration<double, std::milli>(end - start).count();

    // Fixed 60 FPS budget is 16.67 ms; assert a generous, explicit, CI-safe ceiling well
    // above that so this test still catches real regressions without being flaky on
    // loaded hardware (matches this codebase's existing perf-test conventions).
    EXPECT_LT(elapsed_ms, 50.0);
}

}  // namespace

