# Phase 1 Data Model: Sandbox VFX Visualization

No new persistent entities are introduced at the `engine_demo::vfx` library level beyond
Feature 001's `particle`, `particle_pool`, and `emitter` (reused as-is, plus one additive
method — see [contracts/emitter-set-shape.md](contracts/emitter-set-shape.md)). This
feature's data model is entirely within `apps/sandbox::scene`.

## Sandbox VFX State (new members of `ea_sandbox::scene`)

| Field | Type | Notes |
|---|---|---|
| `m_vfx_pool` | `engine_demo::vfx::particle_pool` | Fixed capacity `kVfxPoolCapacity` (2048), constructed once from `m_alloc`. Shared by every burst and spark. Never included in `state_digest()`. |
| `m_vfx_emitter` | `engine_demo::vfx::emitter` | One shared emitter; shape repositioned via `set_shape(sphere_shape{...})` before each `try_emit`. Seeded from `m_seed ^ kVfxSeedSalt` (never from `m_rng`). No force applicators attached (spec explicitly excludes new force types; a floaty un-forced spark/burst is visually sufficient and keeps this feature's diff minimal). |

Declaration order in `scene.h` matters: `m_vfx_pool` **must** be declared (and thus
constructed) before `m_vfx_emitter`, because `emitter`'s constructor takes
`particle_pool&`.

### Constants (new, `scene.h`)

| Constant | Value | Purpose |
|---|---|---|
| `kVfxPoolCapacity` | `2048` | Pool capacity; see arena headroom math below. |
| `kVfxBurstCount` | `24` | Particles spawned per right-click VFX burst. |
| `kVfxSparkCount` | `6` | Particles spawned per collision spark. |
| `kVfxSpawnRadius` | `0.03f` | `sphere_shape` radius for both burst and spark (small positional jitter; see research.md §2). |
| `kVfxLifetimeSeconds` | `0.6` | Fixed `lifetime_min_seconds == lifetime_max_seconds`; also the fade denominator (research.md §5). |
| `kVfxSeedSalt` | `0x564658u` (`"VFX"`-derived constant) | XOR-folded into `m_seed` for `emitter_config::seed` (research.md §3). |

### Public surface added to `scene`

```cpp
// Spawns a right-click VFX burst at (wx, wy). Purely additive to the existing
// spawn_particle_burst — never removes or alters that call's behavior. Render-only:
// never affects state_digest().
void spawn_vfx_burst(double wx, double wy) noexcept;

// Read-only view of one live VFX particle, for the renderer only.
struct vfx_particle_view {
    float x{0.0f};
    float y{0.0f};
    float size{1.0f};
    float life_fraction{0.0f};  // remaining_lifetime_seconds / kVfxLifetimeSeconds, clamped [0,1]
};

[[nodiscard]] std::size_t vfx_particle_count() const noexcept;
[[nodiscard]] vfx_particle_view vfx_particle_at(std::size_t index) const noexcept;
```

### Private surface added to `scene`

```cpp
// Emits a small spark burst at the bounce contact point (wx, wy). Called only from the
// non-storm branch of substep()'s free-particle bounce handling.
void spawn_vfx_spark(double wx, double wy) noexcept;

// Destroys and placement-new reconstructs m_vfx_pool + m_vfx_emitter in place. Called
// from rebuild_scene(), immediately after reset_solver(). See research.md §4.
void reset_vfx() noexcept;
```

## Behavioral Contract (scene-level)

1. **Construction**: `scene`'s constructor initializer list constructs `m_vfx_pool{m_alloc,
   kVfxPoolCapacity}` then `m_vfx_emitter{m_alloc, m_vfx_pool, /* config with seed = m_seed ^
   kVfxSeedSalt, shape = sphere_shape{}, speed/lifetime/size ranges, no forces */}`, exactly
   once. `rebuild_scene()` (called from the constructor body, `reseed()`, and
   `switch_scene()`) additionally calls `reset_vfx()`, mirroring `reset_solver()`.
2. **Burst**: `spawn_vfx_burst(wx, wy)` calls `m_vfx_emitter.set_shape(sphere_shape{{wx, wy,
   0}, kVfxSpawnRadius})` then `m_vfx_emitter.try_emit(kVfxBurstCount)`, discarding/ignoring a
   `pool_exhausted` result other than not crashing (FR-009) — no error surfaces to the
   caller because VFX is cosmetic and best-effort.
3. **Spark**: Identical to Burst but with `kVfxSparkCount`, called once per bounce, only from
   the non-storm branch of `substep()`, at the already-clamped contact position.
4. **Tick**: `substep(double dt)` calls `m_vfx_emitter.tick(dt)` exactly once per fixed step,
   after all bounce/spark emission for that step, mirroring how existing free particles
   don't move until the *next* substep after spawning.
5. **Digest exclusion**: `state_digest()` is unmodified by this feature — it reads only
   `m_body_ids`/`rope_node_position` and `m_particles`. `m_vfx_pool`/`m_vfx_emitter` state is
   never read by `state_digest()`, guaranteeing SC-001 by construction rather than by
   convention.
6. **Rendering**: `vfx_particle_count()`/`vfx_particle_at()` read directly from
   `m_vfx_pool.live_particles()` (no copying/allocation); `life_fraction` is computed from
   `remaining_lifetime_seconds` at read time.

## Arena Headroom (Article 6 / research.md §1 justification)

`scene::kArenaBytes` is 4 MiB (4,194,304 bytes). New allocations, once at construction:

| Allocation | Approx. size | Approx. bytes |
|---|---|---|
| `m_vfx_pool` dense `particle` array | `2048 * sizeof(particle)` (~56 B aligned) | ~115 KB |
| `m_vfx_emitter` scratch `spawn_params` array | `2048 * sizeof(spawn_params)` (~56 B aligned) | ~115 KB |
| `m_vfx_emitter` forces vector | `4 * sizeof(force_applicator)` (unused, capacity only) | < 1 KB |

Total new steady-state usage ≈ 230 KB, comfortably inside the existing 4 MiB arena
alongside the largest existing scene (`particle_storm`: 500 physics particles + trails,
well under 100 KB). No capacity increase to `kArenaBytes` is needed.
