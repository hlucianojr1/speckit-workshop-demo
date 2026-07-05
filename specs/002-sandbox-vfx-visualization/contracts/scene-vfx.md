# Contract: `ea_sandbox::scene` VFX surface (sandbox-side addition)

Header: `apps/sandbox/scene.h`
Source: `apps/sandbox/scene.cpp`
Test: `tests/engine_demo/test_scene_vfx.cpp` (new file + new CMake test target)

See [data-model.md](../data-model.md) for the full field/constant list. This contract
covers only the new public/private member functions and their pre/post-conditions.

## Declarations (contract surface)

```cpp
namespace ea_sandbox {

class scene {
   public:
    // ... existing declarations unchanged ...

    void spawn_vfx_burst(double wx, double wy) noexcept;

    struct vfx_particle_view {
        float x{0.0f};
        float y{0.0f};
        float size{1.0f};
        float life_fraction{0.0f};
    };

    [[nodiscard]] std::size_t vfx_particle_count() const noexcept;
    [[nodiscard]] vfx_particle_view vfx_particle_at(std::size_t index) const noexcept;

   private:
    void spawn_vfx_spark(double wx, double wy) noexcept;
    void reset_vfx() noexcept;
};

}  // namespace ea_sandbox
```

## Behavior / pre- and post-conditions

### `spawn_vfx_burst(wx, wy)`

- **Pre-condition**: none; callable in any scene kind, any frame, any number of times.
- **Post-condition**: up to `kVfxBurstCount` new VFX particles become live in `m_vfx_pool`,
  positioned within `kVfxSpawnRadius` of `(wx, wy)`. The existing 12-particle
  `spawn_particle_burst` call is entirely unaffected — this method never touches
  `m_particles`/`m_trails`.
- **Digest**: `state_digest()` is bit-for-bit unchanged by calling this method, in isolation
  or interleaved with any sequence of other `scene` calls.
- **Allocation**: none (Article 6) — `m_vfx_pool`/`m_vfx_emitter` are pre-sized at
  construction.
- **Exhaustion**: if `m_vfx_pool` has fewer than `kVfxBurstCount` free slots, `try_emit`
  spawns as many as fit and reports `pool_exhausted`; this method does not crash, throw, or
  corrupt existing particle state, and the return status is intentionally not surfaced to
  the caller (cosmetic, best-effort).

### `spawn_vfx_spark(wx, wy)` (private)

- **Pre-condition**: called only from the non-storm branch of `substep()`'s free-particle
  bounce handling, at the contact position (already clamped to the world bound).
- **Post-condition / digest / allocation / exhaustion**: identical guarantees to
  `spawn_vfx_burst`, with `kVfxSparkCount` particles instead of `kVfxBurstCount`.
- **Scope**: never called from the `particle_storm` branch of `substep()` (storm particles
  wrap, they do not bounce — FR-003).

### `vfx_particle_count()` / `vfx_particle_at(index)`

- **Pre-condition**: `index < vfx_particle_count()`.
- **Post-condition**: returns a read-only snapshot (position, size, and `life_fraction ==
  clamp(remaining_lifetime_seconds / kVfxLifetimeSeconds, 0, 1)`) of the `index`-th live VFX
  particle. No mutation of `scene` state; no allocation.
- **Renderer usage**: `life_fraction` maps directly to alpha (`alpha = life_fraction`,
  1.0 = just spawned, 0.0 = about to retire).

### `reset_vfx()` (private)

- **Pre-condition**: called only from `rebuild_scene()`, immediately after
  `reset_solver()`.
- **Post-condition**: `m_vfx_pool` and `m_vfx_emitter` are destroyed and placement-new
  reconstructed in place (mirrors `reset_solver()`); `m_vfx_pool.live_count() == 0`
  afterward; `m_vfx_emitter`'s rng is re-seeded from the (possibly new) `m_seed ^
  kVfxSeedSalt`.

### `substep(dt)` (existing method, modified)

- **Addition 1**: in the non-storm (`else`) branch, at each of the two bounce sites (x-bound,
  y-bound), call `spawn_vfx_spark(p.x, p.y)` after clamping `p.x`/`p.y` to the bound.
- **Addition 2**: exactly once per call, after all physics/spark work for this step, call
  `m_vfx_emitter.tick(dt)`.
- **Non-goal**: no change to any existing physics math, rng draw, or digest-contributing
  state in this method.

## Test obligations (Article 7 / FR-011)

New file `tests/engine_demo/test_scene_vfx.cpp`, compiling `apps/sandbox/scene.cpp`
directly (raylib-free, per research.md §6):

1. `spawn_vfx_burst` increases `vfx_particle_count()` by up to `kVfxBurstCount` and leaves
   `particle_count()` (the existing ad-hoc physics particles) unchanged.
2. VFX particles age and retire across repeated `step()` calls: `vfx_particle_count()`
   eventually returns to 0 after `kVfxLifetimeSeconds` of sim time with no further bursts.
3. A bounce in a non-storm scene (e.g. `rope`) increases `vfx_particle_count()`; an
   equivalent crossing in `particle_storm` does not.
4. **Digest parity (SC-001)**: run two identically-seeded scenes for N frames, one calling
   `spawn_vfx_burst`/triggering bounces, one not (or with VFX calls elided) — actually,
   since bounces occur naturally from physics regardless, the real assertion is: run one
   scene for N frames while calling `spawn_vfx_burst` every few frames, and compare its
   `state_digest()` sequence against a second identically-seeded scene that never calls
   `spawn_vfx_burst` — both must produce identical digests at every frame, proving VFX
   bursts never perturb `state_digest()`.
5. `vfx_particle_at(i).life_fraction` starts at 1.0 immediately after spawn and decreases
   monotonically toward 0.0 across subsequent `step()` calls.
