# Research: Particle VFX Subsystem

All unknowns from the Technical Context are resolved below. No `[NEEDS CLARIFICATION]`
markers remain in [spec.md](spec.md); this document records the concrete design decisions
needed to move from requirements to a buildable plan. Where this codebase already contains
a proven, structurally similar subsystem (`engine_demo::ecs::world`, `engine_demo::sim`,
`frame_budget`), that precedent is preferred over a novel pattern.

## 1. Particle storage & recycle algorithm

- **Decision**: A dense, contiguous array of `particle` reserved once to the pool's fixed
  capacity at construction (`eastl::vector<particle, eastl_allocator_ref>`, one-time
  `.reserve(capacity)`, logical size tracked by a separate `live_count`). `try_spawn` appends
  new particles starting at `live_count`. `age_and_retire` scans `[0, live_count)`; an
  expired particle (`remaining_lifetime_seconds <= 0.0`) is replaced in place by the current
  last live particle (swap-remove) and `live_count` is decremented, without advancing the
  scan index past the newly-swapped-in element.
- **Rationale**: Matches this codebase's existing hot-path convention (`ecs::world::m_slots`,
  `frame_budget::m_samples`) of fixed, pre-sized storage with zero allocation after
  construction (Article 6). A dense array keeps `age_and_retire` cache-friendly — every
  scanned element is live, no holes to skip. Swap-remove is O(1) and fully deterministic:
  given an identical sequence of spawn/tick calls, the same swaps occur in the same order
  every run (Article 5, FR-010).
- **Alternatives considered**: A sparse array with an intrusive free-list (rejected — adds
  indirection and holes that hurt cache locality for no benefit, since the spec never
  requires a stable external handle to an individual particle); a linked list (rejected —
  pointer-chasing and non-deterministic layout, and disallowed allocation pattern for
  per-particle nodes).

## 2. Emitter shape representation

- **Decision**: `eastl::variant<point_shape, cone_shape, sphere_shape>` held by
  `emitter_config`, each alternative a small POD struct of shape parameters.
- **Rationale**: Constitution Article 2 names `eastl::variant` as an accepted no-RTTI tagged
  union. It satisfies FR-002's "tagged enum, no runtime type identification" requirement
  while remaining type-safe (`index()`/`get_if<T>` instead of a raw C union that risks
  reading the wrong active member).
- **Alternatives considered**: A hand-rolled `enum class` plus a raw union (rejected — same
  discriminant behavior with none of `variant`'s type safety); a virtual `shape` interface
  (rejected — requires vtables/dynamic dispatch for type discrimination, against the spirit
  of Article 2).

## 3. Force applicator representation & composition

- **Decision**: `eastl::variant<gravity_force, wind_force, turbulence_force>` stored in a
  per-emitter `eastl::vector<force_applicator, eastl_allocator_ref>` reserved once (small
  fixed upper bound, 4) at emitter construction. Each tick, every attached applicator's
  contribution is computed and summed additively into one acceleration before it is
  integrated into a particle's velocity.
- **Rationale**: Same no-RTTI tagged-union reasoning as shapes (FR-007). Additive composition
  is the simplest model satisfying "composable... whose effects combine additively" (FR-007)
  and requires no ordering guarantees beyond a fixed iteration order over the attached-force
  list, which is trivially deterministic.
- **Alternatives considered**: Dedicated optional fields per force type on `emitter`
  (rejected — does not scale past three force kinds and forces the emitter type to change
  whenever a new force is added); an ordered override pipeline (rejected — no requirement
  calls for override semantics, and it would make results order-dependent).

## 4. Determinism strategy

- **Decision**: Each `emitter` owns exactly one `engine_demo::sim::rng`, explicitly seeded at
  construction from a full-width `std::uint64_t`. `try_emit` and `turbulence_force` both draw
  from that single stream, always in a fixed per-particle field order (shape-derived
  position/direction → speed variance → lifetime variance → size variance → per-tick
  turbulence perturbation), and always in dense-array index order.
- **Rationale**: A single, explicitly-ordered consumption sequence is what makes FR-009/
  FR-010/SC-003 achievable: identical seed + identical call sequence necessarily produces an
  identical sequence of `rng` draws, and the deterministic iteration order (§1) means every
  draw lands on the same particle slot on every run.
- **Alternatives considered**: A separate RNG per force applicator (rejected — multiplies the
  surface area for order-of-consumption bugs with no benefit in a single-threaded tick);
  reseeding per-frame from the frame number (rejected — Article 5 requires an explicit seed
  at construction, not per-call reseeding).

## 5. Memory layout & alignment

- **Decision**: `particle` field order: `double remaining_lifetime_seconds;` first (8-byte
  aligned, avoids padding before it), followed by `float position[3]`, `float velocity[3]`,
  `float color[4]`, `float size;`. Natural size is 8 + 12 + 12 + 16 + 4 = 52 bytes, padded to
  56 bytes (a multiple of `alignof(double) == 8`). No manual padding or `alignas` override is
  needed beyond natural struct layout.
- **Rationale**: Article 5 requires time accumulators to be `double`; `remaining_lifetime_seconds`
  is decremented every tick, so it is that one `double` field (FR-003). Position/velocity/
  color/size are render-boundary values, for which Article 5 explicitly permits `float`, and
  match the `float`-based render types used elsewhere (e.g. `apps/sandbox`). Ordering the
  single 8-byte field first avoids inserting mid-struct padding.
- **Alternatives considered**: An all-`double` particle (rejected — doubles the working-set
  size of the hot dense array at 500+ particles for no accuracy benefit on purely visual
  attributes, working against SC-002's 2 ms budget); struct-of-arrays layout (rejected —
  added implementation complexity not justified at this fixed scale; revisit only if
  profiling later shows the AoS layout is cache-bound).

## 6. Frame budget integration

- **Decision**: `particle_pool`/`emitter` do not own or call into `engine_demo::frame_budget`
  directly. The subsystem exposes plain, independently-timeable `try_emit`/`tick` calls; the
  integration point (an app or scene, mirroring the existing pattern of wrapping the physics
  step with a stopwatch and calling `frame_budget::record_sample`) measures the VFX tick and
  records the elapsed milliseconds into its own `frame_budget` instance.
- **Rationale**: Matches the existing decoupled composition style in this codebase —
  `game_loop`, `sim::rng`, and `frame_budget` are sibling members composed by the call site,
  not nested inside one another. Keeping VFX ignorant of `frame_budget` satisfies FR-015
  ("expose a way to record...timing sample compatible with...frame budget") without coupling
  a pure simulation type to a specific telemetry consumer, and keeps determinism tests free
  of timing-infrastructure dependencies (User Story 3 / SC-003).
- **Alternatives considered**: Passing a `frame_budget&` into `emitter::tick()` (rejected —
  couples simulation to a specific telemetry consumer, complicates pure-determinism tests);
  a static/global budget instance (rejected outright — disallowed global mutable state).

## 7. Emit semantics: all-or-nothing exhaustion + zero/negative-lifetime skip

- **Decision**: `particle_pool::try_spawn(span<const spawn_params>)` is strictly
  all-or-nothing: if `params.size() > free_count()`, it spawns nothing and returns
  `vfx_status::pool_exhausted`; otherwise it spawns every entry and returns `vfx_status::ok`.
  `emitter::try_emit(count)` samples `count` candidate `spawn_params` (consuming `m_rng` in
  the fixed order from §4 for every candidate, regardless of outcome, to preserve
  determinism), then discards — without forwarding to the pool — any candidate whose sampled
  lifetime is `<= 0.0`. The remaining candidates are forwarded to `particle_pool::try_spawn`
  in one call. If that call succeeds, `try_emit` reports `{spawned: <forwarded count>,
  status: ok}` even when `spawned < count` due to lifetime skips (FR-013: skipped particles
  are not a failure). If the pool rejects the forwarded batch, `try_emit` reports
  `{spawned: 0, status: pool_exhausted}` — no particle from the request becomes active
  (FR-006: no partial emission, no corruption of existing state).
- **Rationale**: Directly satisfies FR-006's explicit "reject the request... without
  partially emitting particles" wording (a deliberate divergence from a partial-fill design)
  while still satisfying FR-013's distinct requirement that a non-positive sampled lifetime
  is a normal, successful no-op for that one particle rather than a request failure. Keeping
  the two failure modes (pool capacity vs. per-particle lifetime) orthogonal keeps both
  `try_spawn` and `try_emit` simple to reason about and test independently.
- **Scratch buffer bound**: `emitter` reserves one scratch `eastl::vector<spawn_params,
  eastl_allocator_ref>` sized to its referenced pool's `capacity()` at construction, so
  `try_emit` never allocates regardless of burst size. If a caller requests `count` greater
  than the pool's total `capacity()`, the request cannot possibly succeed as a whole batch;
  `try_emit` returns `{0, vfx_status::pool_exhausted}` immediately without consuming `m_rng`,
  preserving the "no partial emission" guarantee and avoiding any resize.
- **Alternatives considered**: Partial-fill semantics (spawn as many as fit) — rejected
  because it directly contradicts FR-006's explicit wording for this feature; treating a
  non-positive lifetime as a full-request failure — rejected because FR-013 explicitly calls
  out this case as a per-particle skip, not an overall failure.
