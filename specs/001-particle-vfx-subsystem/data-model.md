# Data Model: Particle VFX Subsystem

Entities correspond to the Key Entities in [spec.md](spec.md). Types are named as they will
appear in `include/engine_demo/vfx/`; see [research.md](research.md) for the rationale
behind each representation choice.

## `particle`

Visual-only state for a single particle. Never read or written by
`engine_demo::physics::constraint_solver`.

| Field                       | Type          | Notes                                             |
|-----------------------------|---------------|----------------------------------------------------|
| `remaining_lifetime_seconds` | `double`      | Time accumulator (Article 5). Particle is retired when this reaches `<= 0.0`. |
| `position`                   | `float[3]`    | Render-boundary value (Article 5).                |
| `velocity`                   | `float[3]`    | Render-boundary value; updated by force applicators each tick. |
| `color`                      | `float[4]`    | RGBA, normalized `[0, 1]`.                         |
| `size`                       | `float`       | Uniform world-space scale.                         |

**Validation rules**: `remaining_lifetime_seconds` is set to a positive value at spawn time;
a spawn request with a non-positive lifetime is rejected by the emitter before it reaches
the pool (caller error, not a pool-exhaustion condition).

## `vfx_status`

Status enum for fallible VFX operations (Article 1: no exceptions).

| Value             | Meaning                                                        |
|-------------------|-----------------------------------------------------------------|
| `ok`               | Operation fully succeeded.                                     |
| `pool_exhausted`   | Fewer particles were spawned than requested (0 or more); pool had insufficient free capacity. |
| `invalid_argument` | Caller supplied an invalid configuration to an API that validates it — currently only `emitter::add_force` when the emitter's fixed-capacity force-applicator list is already full. Constructors (`particle_pool`, `emitter`) are `noexcept` and perform no argument validation: a zero-capacity `particle_pool` is valid and simply reports `pool_exhausted` on every subsequent spawn attempt. |

## `particle_pool`

Fixed-capacity store of `particle`, owning the dense array described in research.md §1.

- **Fields (conceptual)**: capacity (`std::size_t`, fixed at construction), a dense
  `eastl::vector<particle, eastl_allocator_ref>` reserved once to capacity, and a
  `live_count`.
- **Relationships**: Zero or more `emitter` instances reference (do not own) a
  `particle_pool` and draw new particles from it via `try_spawn`.
- **State transitions**: `free (capacity - live_count > 0) → live` on successful spawn;
  `live → free` when a particle's `remaining_lifetime_seconds` reaches zero during `tick`,
  via swap-remove.

## `emitter_shape_kind` (variant discriminant)

Conceptually a tagged enum over `{ point, cone, sphere }`, implemented as
`eastl::variant<point_shape, cone_shape, sphere_shape>` (research.md §2).

| Shape         | Parameters                                                             |
|---------------|-------------------------------------------------------------------------|
| `point_shape`  | `position: float[3]` — all particles spawn at this exact point.        |
| `cone_shape`   | `origin: float[3]`, `direction: float[3]` (unit), `half_angle_radians: float` — spawn position at origin, initial velocity direction within the cone. |
| `sphere_shape` | `center: float[3]`, `radius: float` — spawn position uniformly within the sphere volume. |

## `force_applicator` (variant)

Tagged union over `{ gravity_force, wind_force, turbulence_force }` (research.md §3).

| Force               | Parameters                                   | Contribution                                              |
|---------------------|-----------------------------------------------|-------------------------------------------------------------|
| `gravity_force`      | `acceleration: float[3]`                     | Constant acceleration added every tick.                    |
| `wind_force`         | `acceleration: float[3]`                     | Constant acceleration added every tick (directionally distinct from gravity by configuration only). |
| `turbulence_force`   | `strength: float`, `frequency: float`        | Per-particle, per-tick pseudo-random perturbation drawn from the owning emitter's seeded `rng` in fixed order (research.md §4). |

## `emitter_config`

Construction-time configuration for an `emitter`.

| Field                | Type                                    | Notes                                             |
|----------------------|-------------------------------------------|------------------------------------------------------|
| `shape`               | `eastl::variant<point_shape, cone_shape, sphere_shape>` | Selects spawn geometry.               |
| `seed`                | `std::uint64_t`                          | Full-width seed for the emitter's owned `rng` (Article 5). |
| `speed_min`, `speed_max` | `float`                                | Initial speed variance range along the shape-derived direction. |
| `lifetime_min_seconds`, `lifetime_max_seconds` | `double`         | Spawn lifetime variance range.                       |
| `color`               | `float[4]`                                | Spawn color (constant per emitter for v1).            |
| `size_min`, `size_max` | `float`                                   | Spawn size variance range.                            |

## `emitter`

- **Fields (conceptual)**: `emitter_config`, an owned `engine_demo::sim::rng`, a reference to
  a `particle_pool`, and a fixed-capacity `eastl::vector<force_applicator, eastl_allocator_ref>`.
- **Relationships**: Draws particles from exactly one `particle_pool`; owns zero or more
  `force_applicator` instances.
- **Behavior**: `try_emit(count)` samples `count` particles' initial attributes from `shape`
  and the config's variance ranges (consuming `rng` in the fixed order from research.md §4)
  and forwards them to the `particle_pool`; `tick(dt_seconds)` applies the summed force
  contributions to every currently-live particle in the referenced pool, then advances
  position and decrements lifetime for all of them.
- **Sharing rule (v1 scoping)**: Multiple emitters MAY reference (i.e. call `try_emit`
  against) the same `particle_pool`, which is how the shared-capacity edge case in
  [spec.md](spec.md#edge-cases) arises. However, exactly one emitter may be designated to
  call `tick()` against a given `particle_pool` per frame — `tick()` ages and force-integrates
  *every* live particle in the pool regardless of which emitter spawned it, so a second,
  concurrent `tick()` call against the same pool in the same frame would double-integrate
  particles. This restriction is documented on `particle_pool`/`emitter` and validated by a
  dedicated test in [contracts/emitter.md](contracts/emitter.md); it is not enforced at
  runtime (no cross-emitter locking), matching the single-threaded, caller-disciplined style
  used elsewhere in this codebase (e.g. `constraint_solver`, `game_loop`).

## `emit_result`

Return value of `try_emit`.

| Field      | Type          | Notes                                                     |
|------------|----------------|--------------------------------------------------------------|
| `spawned`   | `std::uint32_t` | Number of particles actually spawned (`0 <= spawned <= count`). |
| `status`    | `vfx_status`   | `ok` if `spawned == count`, else `pool_exhausted`.            |
