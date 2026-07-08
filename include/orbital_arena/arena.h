// Orbital Arena — aggregate root header (T001 stub: status enum + shared constants).
//
// Constitutional articles satisfied:
//   - 1 (no exceptions: arena_status enum)
//   - 2 (no RTTI)
//   - 5 (determinism: all gameplay durations are integer tick counts, research.md D4)
//
// The full `arena` integration type lands in a later task (T017); every other
// orbital_arena module includes this header for the shared constants below
// (data-model.md "Constants" table).

#pragma once

#include <cstddef>
#include <cstdint>

namespace orbital_arena {

// Status return for fallible orbital_arena operations (Article 1: no exceptions).
enum class arena_status : std::uint8_t {
    ok = 0,
    invalid_argument,
    wrong_state,
    log_full,
};

// Roster (FR-001).
inline constexpr std::uint8_t kMaxPlayers = 4;
inline constexpr std::uint8_t kMinPlayers = 2;

// Win rule (FR-006).
inline constexpr std::uint32_t kWinScore = 100;

// Fixed-step timing (existing engine step is 1/60 s; research.md D4 — all gameplay
// durations are integer tick counts, never float accumulation).
inline constexpr std::uint32_t kTicksPerSecond = 60;
inline constexpr std::uint32_t kCountdownTicks = 180;              // 3 s (FR-012)
inline constexpr std::uint32_t kPowerupSpawnIntervalTicks = 600;   // 10 s (FR-009)
inline constexpr std::uint32_t kStrengthSurgeTicks = 300;          // 5 s (FR-010)
inline constexpr std::uint32_t kDoublePointsTicks = 600;           // 10 s (FR-010)
inline constexpr std::uint32_t kStateHashIntervalTicks = 60;       // Article 10

// Power-up field cap (FR-009).
inline constexpr std::uint32_t kMaxFieldPickups = 3;

// Particle field sizing (assumption: particle_storm parity; research.md D8).
inline constexpr std::size_t kTargetParticleCount = 500;
inline constexpr std::size_t kMaxSnapshotParticles = 512;

// Fixed bounded arena half extent (US1 scenario 4).
inline constexpr float kArenaHalfExtent = 5.0f;

}  // namespace orbital_arena
