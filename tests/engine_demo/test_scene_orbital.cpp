// GoogleTest for the Orbital Arena sandbox scene (Feature 003, T019).
//
// Compiles apps/sandbox/scene.cpp directly (raylib-free by design), the same pattern
// as test_scene_vfx.cpp. Covers the new public surface: scene_kind::orbital_arena,
// scene::orbital(), orbital_match_state_name(), the orbital digest contribution, and
// the CRITICAL invariant that existing scene kinds' digests are untouched.

#include "apps/sandbox/scene.h"

#include <gtest/gtest.h>

#include <cstdint>

namespace {

using ea_sandbox::scene;
using ea_sandbox::scene_kind;

// scene is neither copyable nor movable — switch an existing instance in place.
void to_orbital(scene& s) {
    s.switch_scene(scene_kind::orbital_arena);
}

void step_n(scene& s, int n) {
    for (int i = 0; i < n; ++i) {
        (void)s.step(scene::kFixedStepSeconds);
    }
}

// ---------------------------------------------------------------------------------------
// Construction + accessor surface
// ---------------------------------------------------------------------------------------

TEST(scene_orbital, orbital_accessor_is_null_for_every_other_scene_kind) {
    scene s{42};
    EXPECT_EQ(s.orbital(), nullptr);  // rope (default)
    s.switch_scene(scene_kind::pendulum_tower);
    EXPECT_EQ(s.orbital(), nullptr);
    s.switch_scene(scene_kind::cloth);
    EXPECT_EQ(s.orbital(), nullptr);
    s.switch_scene(scene_kind::particle_storm);
    EXPECT_EQ(s.orbital(), nullptr);
}

TEST(scene_orbital, switching_to_orbital_constructs_arena_within_scene_allocator) {
    scene s{42};
    s.switch_scene(scene_kind::orbital_arena);
    ASSERT_NE(s.orbital(), nullptr);
    EXPECT_EQ(s.kind(), scene_kind::orbital_arena);
    EXPECT_STREQ(ea_sandbox::scene_kind_name(s.kind()), "orbital arena");
    // Article 6 at the scene level: everything fit in the arena buffer.
    EXPECT_LT(s.arena_bytes_used(), scene::kArenaBytes);
    // No rope bodies / free particles — the embedded arena owns the world.
    EXPECT_EQ(s.rope_node_count(), 0u);
    EXPECT_EQ(s.particle_count(), 0u);
}

TEST(scene_orbital, match_state_names_cover_all_states) {
    EXPECT_STREQ(ea_sandbox::orbital_match_state_name(orbital_arena::match_state::lobby),
                 "lobby");
    EXPECT_STREQ(ea_sandbox::orbital_match_state_name(orbital_arena::match_state::countdown),
                 "countdown");
    EXPECT_STREQ(ea_sandbox::orbital_match_state_name(orbital_arena::match_state::playing),
                 "playing");
    EXPECT_STREQ(ea_sandbox::orbital_match_state_name(orbital_arena::match_state::game_over),
                 "game over");
}

// ---------------------------------------------------------------------------------------
// Scripted match lifecycle
// ---------------------------------------------------------------------------------------

TEST(scene_orbital, scripted_lobby_reaches_playing_after_countdown) {
    scene s{42};
    to_orbital(s);
    ASSERT_NE(s.orbital(), nullptr);
    // Both scripted players joined + ready at build time; the very first tick moves
    // lobby -> countdown, then exactly kCountdownTicks ticks reach playing.
    step_n(s, 1);
    EXPECT_EQ(s.orbital()->state(), orbital_arena::match_state::countdown);
    step_n(s, static_cast<int>(orbital_arena::kCountdownTicks) + 1);
    EXPECT_EQ(s.orbital()->state(), orbital_arena::match_state::playing);
}

TEST(scene_orbital, autopilot_wells_move_and_scores_eventually_accrue) {
    scene s{42};
    to_orbital(s);
    ASSERT_NE(s.orbital(), nullptr);

    // Enter playing, record well placement.
    step_n(s, static_cast<int>(orbital_arena::kCountdownTicks) + 2);
    ASSERT_EQ(s.orbital()->state(), orbital_arena::match_state::playing);
    const eastl::span<const orbital_arena::gravity_well> wells = s.orbital()->wells();
    ASSERT_EQ(wells.size(), static_cast<std::size_t>(scene::kOrbitalPlayers));
    const float w0_start[2] = {wells[0].position[0], wells[0].position[1]};

    // ~30 s of scripted play: wells must have moved and captures must have scored.
    step_n(s, 1800);
    EXPECT_TRUE(wells[0].position[0] != w0_start[0] || wells[0].position[1] != w0_start[1]);
    std::uint32_t total = 0;
    for (std::uint8_t p = 0; p < scene::kOrbitalPlayers; ++p) {
        total += s.orbital()->score(p);
    }
    EXPECT_GT(total, 0u);
    // Field replenishment keeps the particle world populated.
    EXPECT_GT(s.orbital()->particles().live_count(), 0u);
}

// ---------------------------------------------------------------------------------------
// Determinism (Article 10 at the scene layer)
// ---------------------------------------------------------------------------------------

TEST(scene_orbital, same_seed_produces_identical_digests_across_instances) {
    scene a{42};
    to_orbital(a);
    scene b{42};
    to_orbital(b);
    step_n(a, 400);
    step_n(b, 400);
    EXPECT_EQ(a.state_digest(), b.state_digest());
    ASSERT_NE(a.orbital(), nullptr);
    ASSERT_NE(b.orbital(), nullptr);
    EXPECT_EQ(a.orbital()->score(0), b.orbital()->score(0));
    EXPECT_EQ(a.orbital()->score(1), b.orbital()->score(1));
}

TEST(scene_orbital, different_seeds_produce_different_digests) {
    scene a{42};
    to_orbital(a);
    scene b{1337};
    to_orbital(b);
    step_n(a, 300);
    step_n(b, 300);
    EXPECT_NE(a.state_digest(), b.state_digest());
}

TEST(scene_orbital, reseed_replays_the_same_match) {
    scene s{7};
    to_orbital(s);
    step_n(s, 250);
    const std::uint64_t first_run = s.state_digest();
    s.reseed(7);
    step_n(s, 250);
    EXPECT_EQ(s.state_digest(), first_run);
}

// CRITICAL T019 invariant: adding the orbital scene must not perturb existing scenes.
// A rope scene that transits through orbital_arena and back must produce the exact
// digest of a rope scene that never saw the orbital kind (same rng draw order).
TEST(scene_orbital, existing_scene_digests_are_unchanged_by_orbital_round_trip) {
    scene pristine{99};
    step_n(pristine, 120);
    const std::uint64_t golden = pristine.state_digest();

    scene tourist{99};
    tourist.switch_scene(scene_kind::orbital_arena);
    step_n(tourist, 60);
    tourist.switch_scene(scene_kind::rope);
    step_n(tourist, 120);
    EXPECT_EQ(tourist.state_digest(), golden);
}

TEST(scene_orbital, switching_away_tears_down_the_embedded_arena) {
    scene s{11};
    to_orbital(s);
    ASSERT_NE(s.orbital(), nullptr);
    s.switch_scene(scene_kind::particle_storm);
    EXPECT_EQ(s.orbital(), nullptr);
}

}  // namespace
