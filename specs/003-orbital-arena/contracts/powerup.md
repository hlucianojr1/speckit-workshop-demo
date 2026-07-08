# Contract: powerup

**Header**: `include/orbital_arena/powerup.h` | **Impl**: `src/orbital_arena/powerup.cpp`
**Tests**: `tests/orbital_arena/test_powerup.cpp`
**Satisfies**: FR-009, FR-010, FR-011, FR-019, Articles 1, 2, 4, 5, 6, 9.

## Types

```cpp
namespace orbital_arena {

enum class powerup_kind : uint8_t { strength_surge, double_points, particle_flood };

struct pickup {
    powerup_kind kind{powerup_kind::strength_surge};
    float        position[2]{};
    uint64_t     spawn_tick{0};
    bool         alive{false};
};

struct active_effect {
    powerup_kind kind{powerup_kind::strength_surge};
    uint32_t     remaining_ticks{0};   // 0 == inactive
};

// Fixed-slot manager: no allocation after construction (Article 6).
class [[nodiscard]] powerup_system {
   public:
    explicit powerup_system(engine_demo::allocator& alloc) noexcept;
    // ... non-copyable, movable (noexcept)
};

}
```

## Functions (methods of `powerup_system`)

```cpp
// Called once per playing tick. Decrements the spawn timer; when it hits 0:
// resets to kPowerupSpawnIntervalTicks and, iff alive_pickup_count < kMaxFieldPickups,
// spawns one pickup with position AND kind drawn from `rng` (exactly 3 draws: x, y,
// kind — fixed order, FR-019). When at cap: timer still resets, spawn skipped, and
// NO rng draws occur (documented: draw-on-spawn-only keeps the stream auditable).
void tick_spawn(engine_demo::sim::rng& rng, uint64_t current_tick,
                float half_extent) noexcept;

// FR-011 consumption: contention over alive pickups using scoring's resolve_capture
// rule (strict nearest well whose capture zone reaches the pickup; exact tie -> no
// consume). On consume: pickup.alive = false and the effect applies:
//   strength_surge -> effects[player] = {strength_surge, kStrengthSurgeTicks}
//   double_points  -> effects[player] = {double_points,  kDoublePointsTicks}
//   particle_flood -> returns flood request (burst count + seed-derived positions
//                     drawn from `rng`) for the arena to forward to the vfx emitter
// Same-kind re-pickup refreshes the duration (documented v1 rule, research D-note).
struct flood_request { uint32_t count; /* positions drawn by emitter sub-seed */ };
[[nodiscard]] eastl::optional<flood_request>
consume_pickups(eastl::span<const gravity_well> wells,
                engine_demo::sim::rng& rng) noexcept;

// Decrement all nonzero effect timers by one tick; expire at 0 (FR-011 exact duration).
void tick_effects() noexcept;

// game_over: all effects end immediately (US3 scenario 6); pickups cleared.
void clear_all() noexcept;

// Queries (used by arena + HUD):
[[nodiscard]] bool is_active(uint8_t player, powerup_kind kind) const noexcept;
[[nodiscard]] eastl::span<const pickup> field_pickups() const noexcept;   // slots, check .alive
[[nodiscard]] eastl::span<const active_effect> effects() const noexcept;  // per player
```

## Behavioral guarantees

- All randomness from the injected shared-seed `rng`, drawn in a fixed documented order
  (FR-019, Article 9 verifiability).
- Fixed arrays `pickup[kMaxFieldPickups]`, `active_effect[kMaxPlayers]` — zero
  allocation after construction (Article 6).
- Effects expire on the exact tick their counter reaches 0 (SC-006 exact).

## Test obligations

1. Spawn exactly every 600 ticks at seed-determined positions (fixed seed → known
   sequence reproduced twice).
2. Cap: 3 alive pickups → spawn skipped, timer continues, rng stream NOT consumed.
3. Each kind applies its effect: surge 300 ticks, double 600 ticks, flood returns a
   burst request immediately.
4. Consumption contention: nearest well wins; exact tie → pickup survives.
5. `clear_all` on game_over zeroes every effect (US3 scenario 6).
6. Timer-refresh rule on same-kind re-pickup.
