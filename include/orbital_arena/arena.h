// Orbital Arena — aggregate root: composes match, scoring, power-ups, wells, input
// log, and the vfx particle field into one deterministic fixed-step tick (T017).
//
// Constitutional articles satisfied:
//   - 1 (no exceptions: create() -> eastl::optional, tick() -> arena_status)
//   - 2 (no RTTI), 3 (EASTL), 4 (every container takes engine_demo::allocator)
//   - 5, 10 (determinism: one game-rule sim::rng in fixed pipeline draw order,
//        salted sub-seeds for the emitter + field replenishment — research.md D6;
//        60-tick FNV-1a hash history for drift detection)
//   - 6 (real-time: ALL pools/logs reserved in create(); tick() never allocates;
//        frame_budget self-measure wraps the tick)
//   - 9 (fairness: rotationally symmetric well placement AND rotationally
//        symmetric particle replenishment — for 2 players the entire world evolves
//        as an exact point reflection under mirrored inputs, bit-for-bit)
//   - 11 (capture_snapshot()/restore() over the flat POD match_snapshot)
//
// Deviation from contracts/arena.md (documented): particle replenishment does NOT
// go through the vfx::emitter. The arena draws one (position, velocity) per group
// from a salted field rng and spawns `player_count` rotationally symmetric copies
// directly into the pool (exact negation for 2 players, exact quarter-turn index
// rotations for 4). This is what makes Article 9 mirror-fairness bit-exact. The
// emitter (salted sub-seed, research.md D6) serves Particle Flood bursts only.

#pragma once

#include "orbital_arena/types.h"

#include "orbital_arena/gravity_well.h"
#include "orbital_arena/input.h"
#include "orbital_arena/match.h"
#include "orbital_arena/powerup.h"
#include "orbital_arena/scoring.h"
#include "orbital_arena/snapshot.h"

#include "engine_demo/allocator.h"
#include "engine_demo/frame_budget.h"
#include "engine_demo/sim/rng.h"
#include "engine_demo/vfx/emitter.h"
#include "engine_demo/vfx/particle.h"

#include <EASTL/optional.h>
#include <EASTL/span.h>
#include <EASTL/vector.h>

#include <cstddef>
#include <cstdint>

namespace orbital_arena {

// Field particles never expire naturally: capture (lifetime zeroed, research.md D2)
// is the only removal path, so snapshot {pos, vel} fully characterizes a particle.
inline constexpr double kFieldParticleLifetimeSeconds = 1.0e9;

// Hash history capacity: one sample per kStateHashIntervalTicks across the longest
// recorded match, reserved once (Article 6); sampling stops silently when full.
inline constexpr std::size_t kMaxHashHistory =
    kMaxRecordedTicks / kStateHashIntervalTicks + 1;

struct arena_config {
    std::uint64_t seed{1};         // shared match seed (FR-019, Article 9)
    std::uint8_t player_count{2};  // kMinPlayers..kMaxPlayers (validated)
    float half_extent{kArenaHalfExtent};
    std::size_t particle_capacity{kTargetParticleCount};  // <= kMaxSnapshotParticles
};

class [[nodiscard]] arena {
   public:
    // Allocates ALL pools/logs up front from `alloc` (Articles 4, 6). Returns
    // nullopt for invalid config or allocator exhaustion (Article 1).
    [[nodiscard]] static eastl::optional<arena> create(engine_demo::allocator& alloc,
                                                       const arena_config& cfg) noexcept;

    arena(const arena&) = delete;
    arena& operator=(const arena&) = delete;
    arena(arena&& other) noexcept;
    arena& operator=(arena&& other) noexcept;
    ~arena();

    // Lobby passthroughs (see contracts/match.md). Player indices are restricted to
    // [0, cfg.player_count) so the identity player->well mapping stays dense.
    [[nodiscard]] arena_status join(std::uint8_t player) noexcept;
    [[nodiscard]] arena_status set_ready(std::uint8_t player, bool ready) noexcept;
    void leave(std::uint8_t player) noexcept;
    [[nodiscard]] arena_status acknowledge_results() noexcept;

    // ONE fixed-step tick. `inputs` = one frame per roster slot. Executes the
    // pipeline in data-model.md §Tick Pipeline order (the determinism backbone,
    // Article 10). Returns log_full once the input log is exhausted; gameplay
    // still advances.
    [[nodiscard]] arena_status tick(const tick_inputs& inputs) noexcept;

    // Articles 10/11. restore() installs every snapshot field; the rng streams are
    // NOT snapshot state (see snapshot.h header note) — replay is (seed, input log).
    [[nodiscard]] match_snapshot capture_snapshot() const noexcept;
    [[nodiscard]] arena_status restore(const match_snapshot& s) noexcept;
    [[nodiscard]] eastl::span<const std::uint64_t> hash_history() const noexcept {
        return {m_hashes.data(), m_hashes.size()};
    }
    [[nodiscard]] const input_log& log() const noexcept { return m_log; }

    // Queries for tests + HUD (render boundary).
    [[nodiscard]] match_state state() const noexcept { return m_match.state(); }
    [[nodiscard]] std::uint32_t score(std::uint8_t player) const noexcept {
        return player < kMaxPlayers ? m_scores.scores[player] : 0u;
    }
    [[nodiscard]] std::int8_t winner() const noexcept { return m_scores.winner; }
    [[nodiscard]] std::uint64_t current_tick() const noexcept { return m_tick; }
    [[nodiscard]] eastl::span<const gravity_well> wells() const noexcept {
        return {m_wells, m_cfg.player_count};
    }
    [[nodiscard]] eastl::span<const pickup> pickups() const noexcept {
        return m_powerups.field_pickups();
    }
    [[nodiscard]] const engine_demo::vfx::particle_pool& particles() const noexcept {
        return *m_pool;
    }
    // Article 6 self-measure: rolling average of tick() cost in milliseconds.
    [[nodiscard]] const engine_demo::frame_budget& budget() const noexcept {
        return m_budget;
    }

   private:
    arena(engine_demo::allocator& alloc, const arena_config& cfg,
          engine_demo::vfx::particle_pool* pool) noexcept;

    void destroy_pool() noexcept;
    void on_playing_entry() noexcept;
    void run_playing_systems(const tick_inputs& in) noexcept;
    void replenish_field() noexcept;
    void spawn_flood(std::uint32_t count) noexcept;
    void clear_particles() noexcept;

    engine_demo::allocator* m_alloc;
    arena_config m_cfg;
    // Placement-new'd from m_alloc so its address is stable across arena moves:
    // m_emitter holds an internal pointer to this pool (see create()).
    engine_demo::vfx::particle_pool* m_pool;
    engine_demo::vfx::emitter m_emitter;  // flood bursts only (salted sub-seed, D6)
    engine_demo::sim::rng m_rng;          // game-rule stream (FR-019 fixed draw order)
    engine_demo::sim::rng m_field_rng;    // replenishment stream (salted sub-seed)
    engine_demo::frame_budget m_budget;
    match m_match;
    score_table m_scores;
    powerup_system m_powerups;
    input_log m_log;
    eastl::vector<std::uint64_t, engine_demo::eastl_allocator_ref> m_hashes;
    gravity_well m_wells[kMaxPlayers]{};
    std::uint64_t m_tick{0};
};

}  // namespace orbital_arena
