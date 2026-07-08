# Contract: arena (integration) + snapshot

**Headers**: `include/orbital_arena/arena.h`, `include/orbital_arena/snapshot.h`
**Impls**: `src/orbital_arena/arena.cpp`, `src/orbital_arena/snapshot.cpp`
**Tests**: `tests/orbital_arena/test_arena_integration.cpp`, `test_replay_determinism.cpp`
**Satisfies**: FR-017, FR-018, FR-019, SC-001..SC-006, Articles 4, 5, 6, 7a, 9, 10, 11.

## snapshot.h

```cpp
namespace orbital_arena {

// Flat POD (Article 11): no pointers, no handles, no containers. See data-model.md
// for the full field list (wells[4], effects[4], pickups[3], particles[512], ...).
struct match_snapshot { /* data-model.md layout */ };

// Field-wise FNV-1a 64 over every field's value in declared order. NEVER hashes raw
// struct bytes (padding is indeterminate). Pure, allocation-free (research D5).
[[nodiscard]] uint64_t state_hash(const match_snapshot& s) noexcept;

}
```

## arena.h

```cpp
namespace orbital_arena {

struct arena_config {
    uint64_t seed{1};                  // shared match seed (FR-019, Article 9)
    uint8_t  player_count{2};          // 2..4 (validated)
    float    half_extent{5.0f};
    std::size_t particle_capacity{kTargetParticleCount};
};

class [[nodiscard]] arena {
   public:
    // Allocates ALL pools/logs up front from `alloc` (Articles 4, 6). Factory returns
    // status for invalid config (Article 1) — [[nodiscard]] per house style.
    [[nodiscard]] static eastl::optional<arena> create(engine_demo::allocator& alloc,
                                                       const arena_config& cfg) noexcept;

    // Lobby passthroughs: join / set_ready / leave / acknowledge_results (see match.md).

    // ONE fixed-step tick. `inputs` = one frame per roster slot. Executes the pipeline
    // in data-model.md §Tick Pipeline order (the determinism backbone, Article 10).
    // Returns log_full if the input log is exhausted; gameplay still advances.
    [[nodiscard]] arena_status tick(const tick_inputs& inputs) noexcept;

    // Articles 10/11:
    [[nodiscard]] match_snapshot capture_snapshot() const noexcept;
    [[nodiscard]] eastl::span<const uint64_t> hash_history() const noexcept; // every 60 ticks
    [[nodiscard]] const input_log& log() const noexcept;

    // Queries for tests + HUD (render boundary):
    [[nodiscard]] match_state state() const noexcept;
    [[nodiscard]] uint32_t score(uint8_t player) const noexcept;
    [[nodiscard]] int8_t winner() const noexcept;
    [[nodiscard]] eastl::span<const gravity_well> wells() const noexcept;
    [[nodiscard]] eastl::span<const pickup> pickups() const noexcept;
    [[nodiscard]] const engine_demo::vfx::particle_pool& particles() const noexcept;
    [[nodiscard]] const engine_demo::frame_budget& budget() const noexcept; // Article 6 self-measure
};

}
```

## Behavioral guarantees

- **Replay (FR-017, Article 10)**: two arenas with equal `arena_config` fed equal input
  sequences produce equal `hash_history()` and equal final scores/winner — bit-exact.
- **Draw order (FR-019)**: game-rule rng consumed only in pipeline steps 3e/3f in fixed
  order; vfx emitter runs on salted sub-seed `seed ^ kParticleSeedSalt` (research D6).
- **Budget (Article 6)**: `tick()` performs zero heap allocation (everything reserved in
  `create`); `frame_budget` wraps the tick for self-measurement.
- **Fairness (Article 9)**: wells placed with rotational symmetry on playing-entry;
  no code path branches on player index except slot addressing.

## Test obligations

- `test_arena_integration.cpp`:
  1. SC-001: scripted 2-player match lobby → winner, first to 100 wins, scores freeze.
  2. Well steering moves only in `playing` (FR-014); boundary clamp end-to-end.
  3. Capture end-to-end: particle enters capture radius → removed + score same tick
     (SC-004: assert score changed on the capturing tick).
  4. Allocation: `allocator::bytes_used()` identical before/after 100 ticks.
  5. Snapshot round-trip: capture → inspect fields == live queries (FR-018).
- `test_replay_determinism.cpp` (Article 7a — 1000 frames, fixed seed):
  6. SC-002: run scripted 1000-tick match twice → identical hash_history (17 hashes),
     winner, and final scores.
  7. SC-003: swap the two players' input scripts → mirror-equivalent outcome (score
     multiset identical, winner identity follows the script not the slot).
  8. Power-up schedule determinism: pickup spawn ticks/positions/kinds identical
     across both runs (SC-006).
  9. Seed sensitivity: different seed → different hash_history (sanity guard against
     hashing constants).
