# Contract: `engine_demo::vfx::emitter`

Header: `include/engine_demo/vfx/emitter.h`
Source: `src/engine_demo/vfx/emitter.cpp`
Test: `tests/engine_demo/test_emitter.cpp`, `tests/engine_demo/test_vfx_determinism.cpp`

## Declarations (contract surface)

```cpp
namespace engine_demo::vfx {

struct point_shape {
    float position[3]{};
};

struct cone_shape {
    float origin[3]{};
    float direction[3]{0.0f, 1.0f, 0.0f};
    float half_angle_radians{0.5f};
};

struct sphere_shape {
    float center[3]{};
    float radius{1.0f};
};

using emitter_shape = eastl::variant<point_shape, cone_shape, sphere_shape>;

struct emitter_config {
    emitter_shape shape{point_shape{}};
    std::uint64_t seed{1};
    float speed_min{0.0f};
    float speed_max{1.0f};
    double lifetime_min_seconds{1.0};
    double lifetime_max_seconds{1.0};
    float color[4]{1.0f, 1.0f, 1.0f, 1.0f};
    float size_min{1.0f};
    float size_max{1.0f};
};

class [[nodiscard]] emitter {
   public:
    emitter(allocator& alloc, particle_pool& pool, emitter_config cfg) noexcept;

    emitter(const emitter&) = delete;
    emitter& operator=(const emitter&) = delete;
    emitter(emitter&&) noexcept = default;
    emitter& operator=(emitter&&) noexcept = default;

    // Attaches a force applicator (up to a small fixed capacity reserved at construction).
    // Returns invalid_argument if the emitter's force-applicator capacity is exhausted.
    [[nodiscard]] vfx_status add_force(force_applicator force) noexcept;

    // Samples `count` particles' initial state from `cfg.shape` and variance ranges,
    // consuming `m_rng` in the fixed order from research.md §4, and forwards them to the
    // referenced particle_pool. See particle_pool.md for exhaustion semantics.
    [[nodiscard]] emit_result try_emit(std::uint32_t count) noexcept;

    // Applies the sum of all attached force applicators to every live particle in the
    // referenced pool, integrates velocity into position, then ages and retires expired
    // particles. Must not be called concurrently with another emitter's tick() against the
    // same particle_pool within the same frame (see data-model.md "Sharing rule").
    void tick(double dt_seconds) noexcept;

    [[nodiscard]] const particle_pool& pool() const noexcept;
};

}  // namespace engine_demo::vfx
```

## Behavior / pre- and post-conditions

- **Shape sampling** (consumed once per particle, in this fixed order, from `m_rng`):
  1. `point_shape`: position is always `shape.position` (no `rng` draw for position).
  2. `cone_shape`: direction is drawn as a random unit vector within `half_angle_radians` of
     `shape.direction` (2 draws: azimuth, polar offset); position is always `shape.origin`.
  3. `sphere_shape`: position is drawn uniformly within the sphere volume (3 draws: one per
     axis via rejection or direct volumetric sampling — implementation detail, not part of
     the contract); initial direction is outward from `shape.center` through the sampled
     position.
  4. Then, regardless of shape: 1 draw for speed within `[speed_min, speed_max]`, 1 draw for
     lifetime within `[lifetime_min_seconds, lifetime_max_seconds]`, 1 draw for size within
     `[size_min, size_max]`.
- **`try_emit`**: Builds `count` `spawn_params` (per above) and forwards them in one call to
  `particle_pool::try_spawn`; returns that call's `emit_result` unchanged. `count == 0`
  returns `{0, vfx_status::ok}` without consuming `m_rng`.
- **`tick`**: For every live particle in the pool (in dense-array index order): sums each
  attached force applicator's contribution via `apply_force` (turbulence consumes `m_rng`
  here, per-particle, in the same index order), applies `velocity += acceleration * dt`,
  then `position += velocity * dt`, then delegates lifetime aging/retirement to
  `particle_pool::age_and_retire(dt_seconds)` for the whole pool in one pass.
  `dt_seconds == 0.0` is a no-op (no force applied, no `rng` consumed, no aging).
- **`add_force`**: `invalid_argument` when the (small, fixed) force-applicator list is full;
  never allocates on success (list capacity reserved at construction). A failed `add_force`
  call never modifies the existing attached-force list.

## Test obligations (Article 7)

- Happy path (User Story 1): burst-emit N particles from a `point_shape` emitter, tick once,
  verify position advanced by `velocity * dt` and lifetime decreased by `dt`.
- Edge case (User Story 1): emit against a pool at full capacity returns
  `vfx_status::pool_exhausted`; a subsequent `tick()` still runs correctly over the particles
  that did get spawned.
- Happy path (User Story 1): `add_force` attaches applicators one at a time up to the
  emitter's fixed force-applicator capacity, returning `vfx_status::ok` each time.
- Edge case (User Story 1): calling `add_force` once the force-applicator list is already at
  capacity returns `vfx_status::invalid_argument` and leaves the previously attached forces
  unchanged (verified by checking `tick()` still applies exactly the same accelerations as
  before the failed call).
- Happy path (User Story 2): `cone_shape` emitted particle directions all lie within
  `half_angle_radians` of the configured direction (dot-product check).
- Happy path (User Story 2): `sphere_shape` emitted particle positions all lie within
  `radius` of `center` (distance check).
- Happy path (User Story 2): an emitter with `gravity_force` + `wind_force` attached shows
  velocity changing by the summed acceleration each tick across multiple ticks.
- Edge case: `tick(0.0)` leaves all particle state unchanged.
- Determinism (User Story 3): two emitters built with identical `emitter_config` (same
  `seed`) and driven through an identical sequence of `try_emit`/`tick` calls produce
  byte-identical `pool().live_particles()` snapshots (`EXPECT_EQ`).
- Determinism (User Story 3): the same emitter's `try_emit` burst, replayed from a freshly
  constructed emitter with the same seed, spawns particles with identical initial attributes.
- Independence (SC-003): every emitter test above constructs only `allocator`,
  `particle_pool`, and `emitter` — no `engine_demo::physics::constraint_solver` is
  constructed or invoked in any emitter test.
