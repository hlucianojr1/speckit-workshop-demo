# Data Model: Particle VFX Subsystem

Entities correspond to the Key Entities in [spec.md](spec.md). Types are named as they will
appear in `include/engine_demo/vfx/`; see [research.md](research.md) for the rationale
behind each representation choice.

## `particle`

Visual-only state for a single particle. Never read or written by
`engine_demo::physics::constraint_solver`; never registered with `engine_demo::ecs::world`.

| Field                         | Type       | Notes                                                                          |
| ------------------------------ | ---------- | ------------------------------------------------------------------------------- |
| `remaining_lifetime_seconds`   | `double`   | Time accumulator (Article 5). Particle is retired when this is `<= 0.0`.       |
| `position`                     | `float[3]` | Render-boundary value (Article 5).                                            |
| `velocity`                     | `float[3]` | Render-boundary value; updated by force applicators each tick.                |
| `color`                        | `float[4]` | RGBA, normalized `[0, 1]`.                                                     |
| `size`                         | `float`    | Uniform world-space scale.                                                    |

**Validation rules**: A candidate whose sampled `lifetime_seconds` is `<= 0.0` never becomes
a `particle` in the pool — it is discarded during `emitter::try_emit` (FR-013), not written
by `particle_pool::try_spawn`.

## `vfx_status`

Status enum for fallible VFX operations (Article 1: no exceptions; FR-016).

| Value               | Meaning                                                                                             |
| -------------------- | ----------------------------------------------------------------------------------------------------- |
| `ok`                  | Operation fully succeeded (including a `try_emit` where some candidates were skipped for lifetime `<= 0.0` — FR-013). |
| `pool_exhausted`      | The pool did not have enough free capacity for the whole forwarded batch; nothing was spawned (FR-006). |
| `invalid_argument`    | Caller supplied an invalid configuration to an API that validates it — currently only `emitter::add_force` when the emitter's fixed-capacity force-applicator list is already full. |

## `particle_pool`

Fixed-capacity store of `particle` (FR-001), owning the dense array described in
research.md §1.

- **Fields (conceptual)**: `capacity` (`std::size_t`, fixed at construction), a dense
  `eastl::vector<particle, eastl_allocator_ref>` reserved once to `capacity`, and a
  `live_count`.
- **Relationships**: One or more `emitter` instances may reference (do not own) a
  `particle_pool` and draw new particles from it via `try_spawn`.
- **State transitions**: `free (capacity - live_count > 0) -> live` on a successful
  `try_spawn`; `live -> free` when a particle's `remaining_lifetime_seconds` reaches `<= 0.0`
  during `age_and_retire` (FR-014), via swap-remove.

## `emitter_shape` (variant)

Tagged union over `{ point, cone, sphere }` (FR-002, research.md §2):
`eastl::variant<point_shape, cone_shape, sphere_shape>`.

| Shape          | Parameters                                                              | Spawn behavior                                                             |
| -------------- | -------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `point_shape`   | `position: float[3]`                                                    | All particles spawn at this exact point; no `rng` draw for position.       |
| `cone_shape`    | `origin: float[3]`, `direction: float[3]` (unit), `half_angle_radians: float` | Spawn position is `origin`; initial direction drawn within the cone.        |
| `sphere_shape`  | `center: float[3]`, `radius: float`                                     | Spawn position drawn uniformly within the sphere volume.                   |

## `force_applicator` (variant)

Tagged union over `{ gravity_force, wind_force, turbulence_force }` (FR-007, research.md §3).

| Force               | Parameters                             | Contribution                                                                 |
| -------------------- | ----------------------------------------- | --------------------------------------------------------------------------------- |
| `gravity_force`      | `acceleration: float[3]`                 | Constant acceleration added every tick.                                    |
| `wind_force`         | `acceleration: float[3]`                 | Constant acceleration added every tick (distinct from gravity by config only). |
| `turbulence_force`   | `strength: float`, `frequency: float`    | Per-particle, per-tick pseudo-random perturbation drawn from the owning emitter's seeded `rng`, in fixed order (research.md §4). |

## `emitter_config`

Construction-time configuration for an `emitter`.

| Field                                              | Type                                                     | Notes                                                    |
| ---------------------------------------------------- | ----------------------------------------------------------- | ----------------------------------------------------------- |
| `shape`                                              | `emitter_shape`                                          | Selects spawn geometry.                                  |
| `seed`                                               | `std::uint64_t`                                          | Full-width seed for the emitter's owned `rng` (Article 5, FR-009). |
| `speed_min`, `speed_max`                             | `float`                                                   | Initial speed variance range along the shape-derived direction. |
| `lifetime_min_seconds`, `lifetime_max_seconds`       | `double`                                                  | Spawn lifetime variance range; may include `<= 0.0` values to exercise FR-013. |
| `color`                                               | `float[4]`                                                | Spawn color (constant per emitter for v1).               |
| `size_min`, `size_max`                                | `float`                                                   | Spawn size variance range.                               |

## `emitter`

- **Fields (conceptual)**: `emitter_config`, an owned `engine_demo::sim::rng`, a reference to
  a `particle_pool`, a fixed-capacity `eastl::vector<force_applicator, eastl_allocator_ref>`,
  and a scratch `eastl::vector<spawn_params, eastl_allocator_ref>` sized to the referenced
  pool's `capacity()` at construction (research.md §7).
- **Relationships**: Draws particles from exactly one `particle_pool`; owns zero or more
  `force_applicator` instances. One emitter's forces never affect another emitter's
  particles (FR-008).
- **Behavior**: `try_emit(count)` samples up to `count` candidates from `shape` and the
  config's variance ranges (consuming `m_rng` in the fixed order from research.md §4),
  discards non-positive-lifetime candidates (FR-013), and forwards the remainder to the pool
  in one all-or-nothing `try_spawn` call (research.md §7). `tick(dt_seconds)` (FR-005: callable
  independently of the physics solver) sums attached force contributions for every live
  particle in the referenced pool, integrates velocity into position, then delegates lifetime
  aging/retirement to `particle_pool::age_and_retire(dt_seconds)`.
- **Sharing rule (v1 scoping)**: Each `emitter` owns exactly one `particle_pool` for the
  lifetime of this feature (per spec Assumptions: "Each emitter owns... exactly one particle
  pool sized at construction"). Multi-emitter pool sharing is explicitly out of scope.

## `emit_result`

Return value of `try_emit` (and, at the pool level, `try_spawn`).

| Field       | Type            | Notes                                                                    |
| ----------- | ---------------- | --------------------------------------------------------------------------- |
| `spawned`    | `std::uint32_t` | Number of particles that actually became active (`0 <= spawned <= count`). |
| `status`     | `vfx_status`    | `ok` unless the forwarded batch was rejected for insufficient pool capacity, in which case `pool_exhausted` and `spawned == 0` (research.md §7). |
