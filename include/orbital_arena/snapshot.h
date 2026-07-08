// Orbital Arena — flat POD match snapshot + field-wise FNV-1a 64 state hash.
//
// Constitutional articles satisfied:
//   - 1 (no exceptions), 2 (no RTTI)
//   - 5, 10 (determinism: pure hash over deterministic state, sampled every
//        kStateHashIntervalTicks by the arena)
//   - 6 (real-time: no allocation anywhere in this module)
//   - 11 (snapshot-serializable: no pointers, no handles, no containers — a struct
//        copy IS the serialization; see research.md D8)
//
// FR-018: the snapshot captures everything needed to reproduce the deterministic
// state: match phase, tick, seed, scores, wells, effects, pickups, particles.
// NOTE (documented limitation): rng engine internals (std::mt19937 state) are not
// flat-POD representable and are NOT part of the snapshot; full replay is defined
// as (seed, input log) per FR-017, not as snapshot resume.

#pragma once

#include "orbital_arena/match.h"    // match_state
#include "orbital_arena/powerup.h"  // powerup_kind

#include <cstdint>

namespace orbital_arena {

// Per-player well state (data-model.md match_snapshot layout).
struct well_record {
    float position[2]{};
    float velocity[2]{};
    float strength{0.0f};
    bool  active{false};
};

// Per-player timed-effect slot; remaining_ticks == 0 means inactive.
struct effect_record {
    powerup_kind  kind{powerup_kind::strength_surge};
    std::uint32_t remaining_ticks{0};
};

// Field pickup slot mirror.
struct pickup_record {
    powerup_kind  kind{powerup_kind::strength_surge};
    float         position[2]{};
    std::uint64_t spawn_tick{0};
    bool          alive{false};
};

// Live particle state relevant to game rules (2D projection of the vfx particle).
struct particle_record {
    float position[2]{};
    float velocity[2]{};
};

// Flat POD (Article 11 / FR-018): no pointers, no handles, no containers.
// Field order below IS the hash order (research.md D5).
struct match_snapshot {
    match_state   state{match_state::lobby};
    std::uint64_t tick{0};
    std::uint64_t seed{0};
    std::uint8_t  player_count{0};
    std::uint32_t scores[kMaxPlayers]{};
    std::int8_t   winner{-1};
    bool          sudden_death{false};
    well_record   wells[kMaxPlayers]{};
    effect_record effects[kMaxPlayers]{};
    pickup_record pickups[kMaxFieldPickups]{};
    std::uint32_t powerup_timer_ticks{0};
    std::uint32_t live_particle_count{0};
    particle_record particles[kMaxSnapshotParticles]{};
};

static_assert(kMaxSnapshotParticles >= kTargetParticleCount,
              "snapshot must hold every live particle (research.md D8)");

// Field-wise FNV-1a 64 over every field's value in declared order — NEVER over raw
// struct bytes (padding is indeterminate, research.md D5). Particle records beyond
// live_particle_count are slack, not state, and are excluded. Pure, allocation-free.
[[nodiscard]] std::uint64_t state_hash(const match_snapshot& s) noexcept;

}  // namespace orbital_arena
