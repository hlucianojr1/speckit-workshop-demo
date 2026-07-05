# Research: Particle VFX Subsystem

All unknowns from the Technical Context are resolved below. No `[NEEDS CLARIFICATION]`
markers remain in [spec.md](spec.md); this document instead records the concrete design
decisions needed to move from requirements to a buildable plan.

## 1. Particle storage & recycle algorithm

- **Decision**: A dense, contiguous array of `particle` sized to the pool's fixed capacity
  and reserved once at construction (`eastl::vector<particle, eastl_allocator_ref>` with a
  one-time `.reserve(capacity)` / fixed logical size). A `live_count` tracks how many of the
  leading elements are active. `emit()` appends at `live_count` and increments it. `tick()`
  scans `[0, live_count)`; an expired particle is replaced with the particle at
  `live_count - 1` (swap-remove) and `live_count` is decremented, without advancing the scan
  index.
- **Rationale**: Matches the codebase's existing hot-path convention (`ecs::world::m_slots`,
  `frame_budget::m_samples`) of fixed, pre-sized storage with zero allocation after
  construction (Article 6). A dense array keeps `tick()` branch-prediction-friendly and
  cache-friendly (Article 6) since every scanned element is live. Swap-remove is O(1) and
  fully deterministic: given the same emit/tick call sequence, the same swaps occur in the
  same order every run (Article 5).
- **Alternatives considered**: A sparse array with an intrusive free-list (rejected — adds
  a level of indirection and skips holes during `tick()`, hurting cache locality, and the
  spec does not require stable external handles to individual particles); a linked list of
  particles (rejected — pointer-chasing, non-deterministic layout, disallowed allocation
  pattern).

## 2. Emitter shape representation

- **Decision**: `eastl::variant<point_shape, cone_shape, sphere_shape>` held by the emitter
  configuration, with each alternative a small POD struct of shape parameters.
- **Rationale**: Constitution Article 2 explicitly names `eastl::variant` as an accepted
  no-RTTI, no-inheritance discriminated-union pattern. A variant *is* a tagged enum in
  effect — `index()` gives the discriminant — while remaining type-safe (no raw union,
  no risk of reading the wrong active member).
- **Alternatives considered**: A hand-rolled `enum class emitter_shape` plus a raw C
  union of parameter structs (rejected — same tagging behavior as `eastl::variant` but with
  none of its type safety, and the constitution prefers `eastl::variant` where available);
  a virtual `shape` interface with `point_shape : shape`, etc. (rejected outright — requires
  vtables, which pulls in RTTI-adjacent machinery and violates Article 2's spirit of
  avoiding dynamic dispatch for type discrimination).

## 3. Force applicator representation & composition

- **Decision**: `eastl::variant<gravity_force, wind_force, turbulence_force>` stored in a
  per-emitter `eastl::vector<force_applicator, eastl_allocator_ref>` reserved once (small
  fixed upper bound, e.g. 4) at emitter construction. Each tick, every attached applicator's
  contribution is computed and summed additively into one acceleration before it is
  integrated into particle velocity.
- **Rationale**: Same no-RTTI tagged-union reasoning as emitter shapes. Additive composition
  is the simplest model that satisfies FR-004 and matches the spec's assumption that force
  applicators combine via linear superposition with no ordering dependence.
- **Alternatives considered**: Per-force dedicated emitter fields (`eastl::optional<gravity>`,
  `eastl::optional<wind>`, ...) (rejected — does not scale past 3 force types and forces the
  emitter type to change whenever a new force is added); an ordered pipeline where forces
  can override rather than sum (rejected — no requirement calls for it, and it would make
  results order-dependent, complicating determinism reasoning).

## 4. Determinism strategy

- **Decision**: Each `emitter` owns one `engine_demo::sim::rng`, explicitly seeded at
  construction. `emit()` and `turbulence_force` both draw from that single stream, always in
  a fixed field order per particle (position-in-shape → direction → speed variance →
  color/size variance → turbulence perturbation), and always in dense-array index order.
- **Rationale**: A single, explicitly-ordered consumption sequence is what makes FR-010/
  SC-004 achievable: identical seed + identical call sequence necessarily produces an
  identical sequence of `rng` draws, and the deterministic dense-array iteration order (§1)
  means every draw is applied to the same particle slot on every run.
- **Alternatives considered**: A separate RNG per force applicator (rejected — multiplies
  the surface area for order-of-consumption bugs with no benefit, since draws are already
  totally ordered by the single-threaded tick); reseeding per tick from a hash of frame
  number (rejected — the constitution requires seeding "explicitly at construction from the
  full seed width", not per-call reseeding).

## 5. Memory layout & alignment

- **Decision**: `particle` field order: `double remaining_lifetime_seconds;` first (8-byte
  aligned, avoids padding before it), followed by `float position[3]`, `float velocity[3]`,
  `float color[4]`, `float size;`. Natural size is 8 + 12 + 12 + 16 + 4 = 52 bytes, padded to
  56 bytes (multiple of `alignof(double) == 8`). No manual padding or `alignas` overrides are
  needed beyond natural struct layout.
- **Rationale**: Article 5 requires time accumulators to be `double`; `remaining_lifetime_seconds`
  is exactly such an accumulator (decremented every tick), so it is the one `double` field.
  Position/velocity/color/size are render-boundary values (Article 5 permits `float` there)
  consumed directly by a rendering step, matching `float`-based render types (e.g. raylib
  `Vector3`) used elsewhere in `apps/sandbox`. Ordering the single 8-byte field first avoids
  inserting padding in the middle of the struct.
- **Alternatives considered**: All-`double` particle (rejected — doubles the working-set
  size of the hot dense array for 500+ particles with no accuracy benefit for purely visual
  attributes, hurting the 2 ms/frame budget in SC-001); struct-of-arrays layout (rejected —
  higher implementation complexity for a fixed 500-particle scale that is not shown to be
  cache-bound at the AoS granularity; can be revisited if profiling later shows it's needed).

## 6. Frame budget integration

- **Decision**: `particle_pool`/`emitter` do not own or call into `engine_demo::frame_budget`
  directly. The subsystem exposes plain, independently-timeable `emit()`/`tick()` calls; the
  integration point (e.g. `apps/sandbox/scene.cpp`, mirroring its existing
  `m_budget.record_sample(delta_seconds * 1000.0)` call for the physics step) wraps the VFX
  tick with a stopwatch and records the elapsed milliseconds into its own `frame_budget`
  instance.
- **Rationale**: Matches the existing decoupled pattern in this codebase — `game_loop`,
  `rng`, and `frame_budget` are sibling members composed by the call site (`scene.h`), not
  nested inside one another. Keeping VFX ignorant of `frame_budget` keeps its unit tests
  independent of timing infrastructure (satisfies User Story 3's "independent of the physics
  solver" framing extended to independent of the app's timing harness too).
- **Alternatives considered**: Passing a `frame_budget&` into `emitter::tick()` (rejected —
  couples a pure simulation type to a specific telemetry consumer and complicates
  determinism tests that don't care about timing); a static/global budget instance
  (rejected outright — global mutable state, disallowed by the codebase's composition style).

## 7. Pool exhaustion / partial-emit semantics

- **Decision**: `emit(count)` spawns as many particles as the pool has free capacity for, up
  to `count`, and returns a small result `{ std::uint32_t spawned; vfx_status status; }`.
  `status` is `vfx_status::ok` when `spawned == count` and `vfx_status::pool_exhausted` when
  `spawned < count` (including `spawned == 0`).
- **Rationale**: Directly satisfies FR-006 ("MUST report a distinguishable status ... MUST
  leave existing particle state unaffected by the failed portion") while maximizing visual
  continuity under load — a partially-satisfied burst still shows some particles rather than
  none. This also matches the spec's edge case: multiple emitters sharing one pool where an
  earlier emitter's request can partially or fully succeed before a later one is exhausted.
- **Alternatives considered**: All-or-nothing burst semantics (rejected — under sustained
  load a single greedy emitter could starve visual feedback entirely for itself and others,
  and the spec's edge-case wording implies partial success across emitters is expected).
