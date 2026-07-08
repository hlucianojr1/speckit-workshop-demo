// Orbital Arena — seed-driven power-up pickups and timed per-player effects.
//
// Constitutional articles satisfied:
//   - 1 (no exceptions), 2 (no RTTI: powerup_kind is a plain enum class)
//   - 4 (allocator accepted at construction for interface uniformity; v1 storage is
//        fixed slot arrays only, so nothing allocates)
//   - 5, 9, 10 (determinism/fairness: ALL randomness comes from the injected shared-seed
//        rng, drawn in a fixed documented order — x, y, kind — and only on spawn ticks,
//        FR-019; consumption contention reuses scoring's distance-only rule)
//   - 6 (real-time: fixed arrays pickup[kMaxFieldPickups], active_effect[kMaxPlayers];
//        zero allocation after construction)
//
// Effect slot rule (data-model.md): ONE active_effect slot per player; the latest
// consumed timed pickup occupies it, and a same-kind re-pickup refreshes the duration.
// particle_flood is instant and never occupies a slot.

#pragma once

#include "engine_demo/allocator.h"
#include "engine_demo/sim/rng.h"
#include "orbital_arena/gravity_well.h"
#include "orbital_arena/types.h"

#include <EASTL/optional.h>
#include <EASTL/span.h>

#include <cstdint>

namespace orbital_arena {

enum class powerup_kind : std::uint8_t { strength_surge, double_points, particle_flood };

// Neutral burst size requested by a consumed Particle Flood (US3 scenario 5). Spawn
// positions are drawn by the arena's salted emitter sub-seed (research.md D6), never
// from the game-rule stream.
inline constexpr std::uint32_t kFloodBurstCount = 100;

// Field pickup slot (data-model.md). Slot array of kMaxFieldPickups — no allocation.
struct pickup {
    powerup_kind kind{powerup_kind::strength_surge};
    float        position[2]{};
    std::uint64_t spawn_tick{0};
    bool         alive{false};
};

// Per-player timed effect slot; remaining_ticks == 0 means inactive (research.md D4).
struct active_effect {
    powerup_kind kind{powerup_kind::strength_surge};
    std::uint32_t remaining_ticks{0};
};

// Burst request surfaced as data for the arena to forward to the vfx emitter.
struct flood_request {
    std::uint32_t count{0};
};

// Fixed-slot power-up manager: no allocation after construction (Article 6).
class [[nodiscard]] powerup_system {
   public:
    // The allocator is accepted for constructor-signature uniformity with the other
    // allocator-aware modules (Article 4); v1 storage is entirely fixed slot arrays.
    explicit powerup_system(engine_demo::allocator& alloc) noexcept;

    powerup_system(const powerup_system&) = delete;
    powerup_system& operator=(const powerup_system&) = delete;
    powerup_system(powerup_system&&) noexcept = default;
    powerup_system& operator=(powerup_system&&) noexcept = default;

    // Called once per playing tick. Decrements the spawn timer; when it hits 0:
    // resets to kPowerupSpawnIntervalTicks and, iff fewer than kMaxFieldPickups are
    // alive, spawns one pickup with position AND kind drawn from `rng` (exactly 3
    // draws: x, y, kind — fixed order, FR-019). At cap: timer still resets, spawn is
    // skipped, and NO rng draws occur (draw-on-spawn-only keeps the stream auditable).
    void tick_spawn(engine_demo::sim::rng& rng, std::uint64_t current_tick,
                    float half_extent) noexcept;

    // FR-011 consumption: contention over alive pickups using scoring's resolve_capture
    // rule (strict nearest well whose capture zone reaches the pickup; exact tie -> no
    // consume). Timed kinds occupy the winner's effect slot; particle_flood returns a
    // flood request (merged if several floods are consumed the same tick). The rng
    // reference is part of the contract surface but unused in v1 (flood positions are
    // drawn by the emitter sub-seed).
    [[nodiscard]] eastl::optional<flood_request>
    consume_pickups(eastl::span<const gravity_well> wells,
                    engine_demo::sim::rng& rng) noexcept;

    // Decrements all nonzero effect timers by one tick; expires at 0 (FR-011 exact).
    void tick_effects() noexcept;

    // game_over: all effects end immediately, pickups cleared, spawn timer reset for
    // the next match (US3 scenario 6, FR-011).
    void clear_all() noexcept;

    // Snapshot support (T017, Article 11 — additive beyond contracts/powerup.md):
    // remaining spawn-timer ticks for match_snapshot::powerup_timer_ticks, and a
    // restore that installs pickup/effect slots + timer verbatim from a snapshot.
    [[nodiscard]] std::uint32_t spawn_timer_ticks() const noexcept { return m_spawn_timer; }
    void restore(eastl::span<const pickup> pickups,
                 eastl::span<const active_effect> effects,
                 std::uint32_t spawn_timer) noexcept;

    [[nodiscard]] bool is_active(std::uint8_t player, powerup_kind kind) const noexcept;
    [[nodiscard]] float strength_multiplier(std::uint8_t player) const noexcept;
    [[nodiscard]] std::uint32_t points_multiplier(std::uint8_t player) const noexcept;

    // Slot views (check .alive / .remaining_ticks — used by the arena and the HUD).
    [[nodiscard]] eastl::span<const pickup> field_pickups() const noexcept {
        return {m_pickups, kMaxFieldPickups};
    }
    [[nodiscard]] eastl::span<const active_effect> effects() const noexcept {
        return {m_effects, kMaxPlayers};
    }

   private:
    pickup        m_pickups[kMaxFieldPickups]{};
    active_effect m_effects[kMaxPlayers]{};
    std::uint32_t m_spawn_timer{kPowerupSpawnIntervalTicks};
};

}  // namespace orbital_arena
