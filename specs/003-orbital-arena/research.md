# Research: Orbital Arena — Foundation Mapping Decisions

**Feature**: 006-orbital-arena | **Date**: 2026-07-08
**Input**: [spec.md](spec.md) Technical Context — the spec shipped with zero
`NEEDS CLARIFICATION` markers, so Phase 0 research resolves *design* unknowns:
how each game mechanic maps onto the existing `engine_demo` foundation, and the
determinism/fairness mechanics mandated by Articles 9–11.

---

## D1 — Gravity well force: apply directly to the vfx particle span, not via `physics::constraint_solver`

- **Decision**: `gravity_well.h` exposes a pure function that computes radial
  acceleration (inverse-square with clamped minimum distance, zero beyond the influence
  radius) and the arena applies it in-place to
  `vfx::particle_pool::live_particles()` each tick, integrating velocity → position the
  same way `vfx::emitter::tick()` does.
- **Rationale**: The training doc (§3.5) suggests integrating with
  `physics::constraint_solver`, but the actual solver API is a verlet
  body-and-constraint system (rope/cloth); free-floating particles in this codebase
  live in `vfx::particle_pool` as dense float-position records. The existing
  particle_storm sandbox scene already implements exactly this pattern — two gravity
  wells pulling 500 free particles — without touching the constraint solver. Reusing
  that pattern keeps the delta minimal and avoids modifying `engine_demo`.
- **Alternatives considered**:
  - *New `radial_attractor` variant alternative added to `vfx::force_applicator`* —
    rejected: requires editing `engine_demo` (violates "reuse as-is" scope) and
    `force_applicator` has no per-particle-position input in its signature (forces are
    position-independent by design there).
  - *Wells as verlet bodies inside `constraint_solver`* — rejected: wells are
    kinematically player-driven (velocity clamped, boundary-clamped), not simulated
    bodies; a constraint formulation adds complexity for zero benefit.

## D2 — Capture removal: lifetime-zeroing through the pool's own retirement path

- **Decision**: Capturing a particle sets its `remaining_lifetime_seconds` to `0.0`;
  the arena's end-of-tick `age_and_retire(dt)` (dt = fixed step > 0) swap-removes it.
  Score is credited at capture time, so SC-004 (same-tick scoring) holds.
- **Rationale**: `particle_pool` has no public "remove index" API and Article/scope
  discipline says don't add one. `age_and_retire` documents `dt == 0.0` as a no-op, but
  the arena always ticks with the fixed 1/60 step, so zeroed lifetimes retire the same
  tick they're marked. No engine change, no allocation, O(live) work already being paid.
- **Alternatives considered**:
  - *Add `particle_pool::retire_at(index)`* — rejected: modifies `engine_demo`.
  - *Maintain a parallel "alive" bitmask in orbital_arena* — rejected: duplicates pool
    state, invites desync between the mask and swap-removed indices.

## D3 — Deterministic capture contention (FR-007, Article 9)

- **Decision**: For each capturable object, iterate wells in fixed index order
  `0..player_count`, compute squared distance (float, same ops every run), track strict
  minimum. If exactly one well holds the strict minimum *and* is within capture radius →
  it captures. Exact tie of squared distances → no capture this tick.
- **Rationale**: Comparison is on distance values only; player index is used solely as
  a stable iteration order and never as a tie-break, which makes Article 9
  index-independence structural (verified by the SC-003 input-swap test). Float
  comparison is exact and deterministic because identical inputs traverse identical
  arithmetic every run (Article 10 requires run-to-run determinism, not cross-platform
  bit-equality).
- **Alternatives considered**:
  - *Tie-break by lowest player index* — rejected: directly violates Article 9 and spec
    US1 scenario 5.
  - *Epsilon-based tie detection* — rejected: introduces an arbitrary constant with its
    own fairness edge cases; exact equality is the honest reading of "exact distance tie".

## D4 — All gameplay durations are integer tick counters

- **Decision**: Game rules never accumulate float/double time. The fixed 60 Hz tick is
  the time unit: countdown = 180 ticks, power-up cadence = 600 ticks, Strength Surge =
  300 ticks, Double Points = 600 ticks, state hashes every 60 ticks. The only `double`
  accumulator is the existing `sim::game_loop` one that converts wall time to fixed
  substeps (reused unchanged).
- **Rationale**: Article 5 requires `double` accumulators where accumulation exists;
  integer tick math is stronger still — exact, overflow-safe for match lengths
  (`uint64_t` ticks), and trivially snapshot-serializable (Article 11). It also makes
  "expires within one tick of stated duration" (SC-006) exact rather than approximate.
- **Alternatives considered**: *`double` seconds accumulators per effect* — rejected:
  works, but invites `0.1 * N` float-residual bugs (a known repo gotcha from the vfx
  features) and hashes less cleanly.

## D5 — State hash: field-wise FNV-1a 64, sampled every 60 ticks

- **Decision**: `snapshot.h` provides `state_hash(const match_snapshot&) noexcept ->
  uint64_t` implemented as FNV-1a 64 folded over each field's value bytes in declared
  order (scalars widened to fixed-width types), never over the raw struct memory.
- **Rationale**: Article 10 mandates 60-frame hashing for drift detection. Raw
  `memcmp`/byte hashing of a struct includes padding bytes with indeterminate values —
  a classic desync false-positive. FNV-1a is 5 lines, allocation-free, and needs no
  dependency.
- **Alternatives considered**: *CRC32 / xxHash* — rejected: xxHash adds a dependency;
  CRC32 tables add code for no verification benefit at this scale.

## D6 — RNG stream discipline (Articles 5, 9, 10 / FR-019)

- **Decision**: The arena owns exactly one game-rule `sim::rng` seeded from the shared
  match seed. Draw order is fixed by the tick pipeline: power-up spawn position/type
  draws happen only on spawn ticks, then particle-replenishment draws. The vfx emitter
  used for target-particle spawning is seeded with `match_seed ^ kParticleSeedSalt`
  (the sandbox's `kVfxSeedSalt` pattern) so its internal draws are on a separate stream
  and render-side effects can never perturb game-rule draw order.
- **Rationale**: A single stream with a documented fixed draw order is the simplest
  auditable determinism story; salted sub-seeds isolate subsystems whose draw *counts*
  vary (emitter sampling draws vary with burst size).
- **Alternatives considered**: *One rng per module* — rejected for game rules: multiple
  streams are harder to reason about in replay disputes; kept only for the emitter
  where draw-count isolation is genuinely needed.

## D7 — Input model and replay log (FR-015..017)

- **Decision**: `input_frame { float steer_x, steer_y; float strength; }` per player per
  tick, clamped on ingest (`|steer| ≤ 1`, `0 ≤ strength ≤ 1`). The arena consumes an
  `eastl::span<const input_frame>` (one entry per roster slot) each tick and appends it
  to an `eastl::vector` input log reserved up front for `kMaxRecordedTicks`
  (1000-frame test matches fit comfortably; long matches wrap via explicit
  `log_full` status rather than silent reallocation — Article 6).
  Replay = construct arena with same seed, feed logged frames back tick-by-tick.
- **Rationale**: Pull-based (caller supplies inputs) keeps orbital_arena free of any
  platform/input dependency, which is what lets the same arena run under GTest, headless
  sandbox, and interactive sandbox unchanged. Inputs outside `playing` are still logged
  (edge case in spec) but ignored by rules (FR-014).
- **Alternatives considered**: *Callback/polling interface into the app* — rejected:
  inverts the dependency and makes replay tests awkward.

## D8 — Snapshot layout (FR-018, Article 11)

- **Decision**: `match_snapshot` is a flat POD: match state enum, tick counter, seed,
  per-player fixed arrays `[4]` (scores, well position/velocity/strength, active flags,
  effect kind + remaining ticks), pickup array `[3]` (kind, position, spawn tick,
  alive flag), particle array `[kMaxSnapshotParticles]` of `{pos, vel}` + live count.
  No pointers, no `entity_handle`s, no EASTL containers inside the snapshot.
- **Rationale**: Fixed maxima are all spec-bounded (4 players, 3 pickups, particle
  population target is a tunable constant), so a flat struct with counts is fully
  self-contained — storable, transmittable, restorable (Article 11), and directly
  hashable by D5.
- **Alternatives considered**: *Serialize to a byte buffer* — rejected for v1: a struct
  copy is already flat; byte-level serialization is a transport concern deferred until
  networking exists.

## D9 — Sandbox visualization scene (workshop screenshots)

- **Decision**: Add `scene_kind::orbital_arena` to `apps/sandbox`, modeled on
  particle_storm: the scene owns one `orbital_arena::arena` (2 players for screenshots,
  4 supported), drives it with deterministic scripted autopilot inputs (seed-derived
  steering functions — no keyboard multiplexing needed), and renders wells as colored
  discs with influence/capture radii rings, particles from the pool, pickups as icons,
  plus a HUD line per player (`P1  score 42  [Double Points 3.2s]`) and the match state
  banner (`LOBBY / COUNTDOWN 2.1 / PLAYING / GAME OVER — P2 WINS`).
- **Rationale**: Needed for workshop screenshots (§3 training materials); the training's
  8-task list omits any visualization task, which this plan corrects as an explicit
  render-only task. HUD/trails stay out of `state_digest` exactly as feature 002 did for
  vfx, so existing headless digests remain stable.
- **Alternatives considered**: *Keyboard-controlled players* — rejected for v1
  screenshots: 4-player local keyboard mapping is real work with no screenshot benefit;
  scripted inputs are deterministic and photogenic. Interactive control can be a later
  feature.

## D10 — Build wiring (training gap)

- **Decision**: New static library target `orbital_arena` (see plan.md "CMake Wiring"):
  new `src/orbital_arena/CMakeLists.txt` + `tests/orbital_arena/CMakeLists.txt`, one-line
  `add_subdirectory` edits in `src/CMakeLists.txt` and `tests/CMakeLists.txt`, no
  top-level change. Test helper clones `engine_demo_add_test()` and deliberately omits
  `GTest::gmock` (documented repo-specific false-positive bug).
- **Rationale**: The training doc names the file layout but never the build wiring; a
  student following it verbatim produces orphaned sources. Making wiring an explicit
  first task also gives every later task a compiling-and-testing skeleton (Article 7).
- **Alternatives considered**: *Fold sources into the `engine_demo` library target* —
  rejected: the Orbital Arena constitution scopes Articles 7a/9–11 to
  `orbital_arena/`-rooted paths; a separate target keeps that boundary enforceable and
  keeps game code out of the engine library.
