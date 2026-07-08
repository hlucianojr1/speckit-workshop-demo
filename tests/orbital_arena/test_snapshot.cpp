// GoogleTest for orbital_arena match_snapshot + state_hash (T014 — written before
// the implementation, per Constitution Article 7 test-first discipline).
//
// Article 11: the snapshot is a flat POD (no pointers/handles/containers).
// Research D5: state_hash is FIELD-WISE FNV-1a 64 — never raw-struct byte hashing,
// because padding bytes are indeterminate. The padding test below memsets two
// snapshots with different garbage, assigns identical field values, and demands
// identical hashes.

#include "orbital_arena/snapshot.h"

#include <cstdint>
#include <cstring>
#include <gtest/gtest.h>
#include <type_traits>

namespace {

using orbital_arena::kMaxFieldPickups;
using orbital_arena::kMaxPlayers;
using orbital_arena::kMaxSnapshotParticles;
using orbital_arena::match_snapshot;
using orbital_arena::match_state;
using orbital_arena::powerup_kind;
using orbital_arena::state_hash;

// Compile-time Article 11 / FR-018 guarantees.
static_assert(std::is_trivially_copyable_v<match_snapshot>,
              "match_snapshot must be a flat POD (Article 11)");
static_assert(std::is_standard_layout_v<match_snapshot>,
              "match_snapshot must be standard-layout (Article 11)");

// Assigns EVERY field of the snapshot a deterministic non-default value. Used by the
// padding test: two objects prefilled with different garbage get identical fields.
void fill_snapshot(match_snapshot& s) {
    s.state = match_state::playing;
    s.tick = 4242;
    s.seed = 0xDEADBEEFCAFEF00Dull;
    s.player_count = 3;
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        s.scores[i] = 10u * (i + 1u);
    }
    s.winner = -1;
    s.sudden_death = true;
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        s.wells[i].position[0] = 1.5f * static_cast<float>(i);
        s.wells[i].position[1] = -0.5f * static_cast<float>(i);
        s.wells[i].velocity[0] = 0.25f;
        s.wells[i].velocity[1] = -0.125f;
        s.wells[i].strength = 0.75f;
        s.wells[i].active = (i % 2u) == 0u;
        s.effects[i].kind = powerup_kind::double_points;
        s.effects[i].remaining_ticks = 100u + i;
    }
    for (std::uint32_t i = 0; i < kMaxFieldPickups; ++i) {
        s.pickups[i].kind = powerup_kind::particle_flood;
        s.pickups[i].position[0] = -2.0f + static_cast<float>(i);
        s.pickups[i].position[1] = 3.0f - static_cast<float>(i);
        s.pickups[i].spawn_tick = 600ull * (i + 1ull);
        s.pickups[i].alive = true;
    }
    s.powerup_timer_ticks = 359;
    s.live_particle_count = 8;
    for (std::uint32_t i = 0; i < kMaxSnapshotParticles; ++i) {
        s.particles[i].position[0] = 0.01f * static_cast<float>(i);
        s.particles[i].position[1] = -0.01f * static_cast<float>(i);
        s.particles[i].velocity[0] = 0.5f;
        s.particles[i].velocity[1] = -0.5f;
    }
}

TEST(snapshot, is_flat_trivially_copyable_pod) {
    EXPECT_TRUE(std::is_trivially_copyable_v<match_snapshot>);
    EXPECT_TRUE(std::is_standard_layout_v<match_snapshot>);
    // Holds every live particle the arena can produce (D8).
    EXPECT_GE(kMaxSnapshotParticles, orbital_arena::kTargetParticleCount);
}

TEST(snapshot, copy_round_trip_hashes_identically) {
    match_snapshot original{};
    fill_snapshot(original);

    const match_snapshot restored = original;  // trivial copy IS the restore path
    EXPECT_EQ(state_hash(original), state_hash(restored));
}

TEST(snapshot, padding_bytes_never_affect_hash) {
    // Trivially-copyable: memset-ing the whole object is legal. Fill one with 0xAB
    // garbage and one with 0x00, then assign identical values to every field. If the
    // hash were computed over raw struct bytes, the indeterminate padding would leak
    // in and the hashes would differ (research D5 desync false-positive).
    match_snapshot a;
    match_snapshot b;
    std::memset(&a, 0xAB, sizeof a);
    std::memset(&b, 0x00, sizeof b);
    fill_snapshot(a);
    fill_snapshot(b);

    EXPECT_EQ(state_hash(a), state_hash(b));
}

TEST(snapshot, single_field_difference_changes_hash) {
    match_snapshot base{};
    fill_snapshot(base);
    const std::uint64_t base_hash = state_hash(base);

    match_snapshot mutated = base;
    mutated.tick += 1;
    EXPECT_NE(state_hash(mutated), base_hash);

    mutated = base;
    mutated.scores[2] += 1;
    EXPECT_NE(state_hash(mutated), base_hash);

    mutated = base;
    mutated.sudden_death = false;
    EXPECT_NE(state_hash(mutated), base_hash);

    mutated = base;
    mutated.wells[1].position[0] += 0.001f;
    EXPECT_NE(state_hash(mutated), base_hash);

    mutated = base;
    mutated.pickups[0].alive = false;
    EXPECT_NE(state_hash(mutated), base_hash);

    mutated = base;
    mutated.particles[0].velocity[1] += 0.5f;  // index 0 < live_particle_count
    EXPECT_NE(state_hash(mutated), base_hash);

    mutated = base;
    mutated.powerup_timer_ticks -= 1;
    EXPECT_NE(state_hash(mutated), base_hash);
}

TEST(snapshot, particle_slack_beyond_live_count_is_not_hashed) {
    // Only the first live_particle_count records are state; slack slots must be
    // hash-transparent so a restore-then-capture round trip is hash-stable even if
    // slack contents differ (they are value-initialized in practice).
    match_snapshot base{};
    fill_snapshot(base);
    ASSERT_LT(base.live_particle_count, kMaxSnapshotParticles);

    match_snapshot mutated = base;
    mutated.particles[base.live_particle_count].position[0] = 999.0f;
    EXPECT_EQ(state_hash(mutated), state_hash(base));
}

}  // namespace
