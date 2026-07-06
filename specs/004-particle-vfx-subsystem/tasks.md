# Tasks: Particle VFX Subsystem

**Input**: Design documents from `/specs/004-particle-vfx-subsystem/`
**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/)

**Tests**: Constitution Article 7 ("Every public function has at least one GTest") and this
feature's FR-017 / Success Criteria (SC-003, SC-005) make tests mandatory for this feature,
not optional. Every implementation task below is preceded by its test task, written first
and expected to fail (RED) until the corresponding implementation task lands (GREEN), per
[.github/instructions/tests.instructions.md](../../.github/instructions/tests.instructions.md).

**Organization**: Tasks are grouped by user story (US1/US2/US3, matching [spec.md](spec.md)
priorities P1/P2/P3) after a Foundational phase that every story's `emitter` code depends
on. This refines [plan.md](plan.md)'s Phase 2 file-level task preview (~12–13 tasks) into
the standard test+header+impl+wire granularity below (~23 tasks).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Paths are relative to the repository root

## Path Conventions

Single project (existing `engine_demo` library — see [plan.md](plan.md) Project Structure):

- Headers: `include/engine_demo/vfx/`
- Sources: `src/engine_demo/vfx/`
- Tests: `tests/engine_demo/`

---

## Phase 1: Setup

**Purpose**: Confirm a clean baseline before adding the new subsystem

- [X] T001 Confirm the existing build/test baseline is green: `cmake --preset default-debug`,
      `cmake --build --preset default-debug`, `ctest --preset default-debug
      --output-on-failure`. Fix or report any pre-existing failure before proceeding — no
      new `vfx` code should land on top of a red baseline.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: `particle_pool` and `force_applicator` are structural dependencies of
`emitter` for *every* user story (the `emitter` contract embeds an
`eastl::vector<force_applicator, ...>` member per [contracts/emitter.md](contracts/emitter.md)),
so both must exist and be tested before any user-story-specific emitter work begins.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [X] T002 [P] Write `tests/engine_demo/test_particle_pool.cpp` per
      [contracts/particle_pool.md](contracts/particle_pool.md) test obligations:
      happy-path spawn (N <= capacity in one call, fields match `spawn_params`), the
      **all-or-nothing** edge case (a `try_spawn` request larger than `free_count()`
      returns `{0, vfx_status::pool_exhausted}` and leaves every existing live particle
      untouched — snapshot `live_particles()` before/after to prove it, per FR-006 /
      research.md §7), happy-path `age_and_retire` decrementing lifetime and retiring at
      `<= 0.0`, edge case `age_and_retire(0.0)` no-op, and a determinism check (two pools
      driven through an identical call sequence produce identical `live_particles()` via
      `EXPECT_EQ`). Expected to fail to compile/link until T004 lands.
- [X] T003 [P] Create `include/engine_demo/vfx/particle.h` per
      [contracts/particle_pool.md](contracts/particle_pool.md): `vfx_status`, `particle`
      (field order: `double remaining_lifetime_seconds` first, then `float position[3]`,
      `float velocity[3]`, `float color[4]`, `float size` — [research.md](research.md) §5),
      `spawn_params`, `emit_result`, and the `particle_pool` class declaration
      (constructor takes `allocator&` and `capacity`; `try_spawn`, `age_and_retire`,
      `live_particles` (const/non-const), `capacity`, `live_count`, `free_count`).
- [X] T004 Create `src/engine_demo/vfx/particle.cpp` implementing `particle_pool` per
      [research.md](research.md) §1 and §7 (dense array reserved once at construction;
      `try_spawn` is strictly all-or-nothing — if `params.size() > free_count()` it spawns
      nothing and returns `pool_exhausted`, otherwise it writes every entry starting at
      `live_count` and increments `live_count` by `params.size()`; `age_and_retire`
      decrements every live particle's lifetime and swap-removes any that reach `<= 0.0`,
      without allocating). Depends on T003.
- [X] T005 Wire `particle.cpp` into `src/engine_demo/CMakeLists.txt` and
      `test_particle_pool.cpp` into `tests/engine_demo/CMakeLists.txt` (new
      `engine_demo_add_test(test_particle_pool test_particle_pool.cpp)` entry). Build and
      confirm all T002 tests pass (GREEN). Depends on T004.
- [X] T006 [P] Write `tests/engine_demo/test_force_applicator.cpp` per
      [contracts/force_applicator.md](contracts/force_applicator.md) test obligations:
      `gravity_force`/`wind_force` pass-through with no `rng` draw (compare
      `rng.next_u32()` before/after against an independently-seeded reference `rng`),
      `turbulence_force` output bounded within `[-strength, strength]` per axis, the
      `strength == 0.0f` edge case (output is always `{0,0,0}` but still consumes exactly 3
      draws), and a determinism check (two identically-seeded `rng` instances feed the same
      `turbulence_force` config to identical output via `EXPECT_EQ`). Expected to fail to
      compile/link until T008 lands.
- [X] T007 [P] Create `include/engine_demo/vfx/force_applicator.h` per
      [contracts/force_applicator.md](contracts/force_applicator.md): `gravity_force`,
      `wind_force`, `turbulence_force`, `force_applicator` (`eastl::variant` of the three),
      and the `apply_force(const force_applicator&, engine_demo::sim::rng&, float
      out_acceleration[3])` free function declaration.
- [X] T008 Create `src/engine_demo/vfx/force_applicator.cpp` implementing `apply_force`:
      gravity/wind copy their configured `acceleration` verbatim without touching `rng`;
      turbulence draws exactly 3 values from `rng.next_double_unit()` (one per axis), maps
      `[0,1)` -> `[-1,1)`, and scales by `strength`. Depends on T007.
- [X] T009 Wire `force_applicator.cpp` into `src/engine_demo/CMakeLists.txt` and
      `test_force_applicator.cpp` into `tests/engine_demo/CMakeLists.txt`. Build and confirm
      all T006 tests pass (GREEN). Depends on T008.

**Checkpoint**: `particle_pool` and `force_applicator` exist, are tested, and are wired into
the build. `emitter` work (all user stories) can now begin.

---

## Phase 3: User Story 1 - Emit a Visual Effect Burst (Priority: P1) 🎯 MVP

**Goal**: Spawn a burst of particles from a point emitter, watch them move and age each
tick, and see them retired once their lifetime elapses — all without invoking the physics
constraint solver, and with pool exhaustion reported as a clean status rather than a
partial/corrupt burst.

**Independent Test**: Construct an `allocator`, a `particle_pool`, and a single
`point_shape` `emitter`; call `try_emit(N)`, then `tick(dt)` several times; verify N
particles become active with initialized fields, position/lifetime evolve correctly each
tick, and particles are retired at end-of-life — no `engine_demo::physics::constraint_solver`
is constructed anywhere in the test.

### Tests for User Story 1 ⚠️

- [X] T010 [P] [US1] Write `tests/engine_demo/test_emitter.cpp` covering
      [contracts/emitter.md](contracts/emitter.md)'s User Story 1 test obligations:
      burst-emit N particles from a `point_shape` emitter and verify initialized
      position/velocity/lifetime/color/size; tick once and verify position advances by
      `velocity * dt` and lifetime decreases by `dt`; a `try_emit` request larger than the
      pool's free capacity returns `{0, vfx_status::pool_exhausted}` with **no** particle
      from that request becoming active (FR-006, all-or-nothing — no partial burst); a
      `try_emit` request larger than the pool's total `capacity()` is rejected the same way
      without consuming `m_rng` (research.md §7); a candidate sampled with
      `lifetime_seconds <= 0.0` (via `lifetime_min_seconds <= 0.0`) is silently skipped —
      `try_emit` still returns `vfx_status::ok` with `spawned < count`, and the skipped
      particle never appears in `pool().live_particles()` (FR-013); `tick(0.0)` leaves all
      particle state unchanged; `add_force` succeeds up to the emitter's fixed
      force-applicator capacity and returns `vfx_status::invalid_argument` (without
      disturbing previously attached forces) once that capacity is exhausted; and a "no
      `constraint_solver`/`ecs::world` constructed" independence check (FR-005, FR-012) for
      every test in this file. Expected to fail to compile/link until T012 lands.

### Implementation for User Story 1

- [X] T011 [US1] Create `include/engine_demo/vfx/emitter.h` per
      [contracts/emitter.md](contracts/emitter.md): `point_shape`, `cone_shape`,
      `sphere_shape`, `emitter_shape` (`eastl::variant` of the three), `emitter_config`, and
      the `emitter` class declaration (constructor takes `allocator&`, `particle_pool&`,
      `emitter_config`; `add_force`, `try_emit`, `tick`, `pool()`). Depends on T005, T009.
      Declare the full shape/force surface now even though this task's implementation
      (T012) only makes the `point_shape` path work — User Story 2 fills in `cone_shape` /
      `sphere_shape` sampling without changing this header.
- [X] T012 [US1] Create `src/engine_demo/vfx/emitter.cpp` implementing: construction (owns
      an `engine_demo::sim::rng` seeded from `emitter_config::seed`; reserves a scratch
      `eastl::vector<spawn_params, eastl_allocator_ref>` sized to `pool.capacity()` so
      `try_emit` never allocates — research.md §7); `try_emit` for the `point_shape` case
      only (immediately returns `{0, pool_exhausted}` without consuming `m_rng` if
      `count > pool.capacity()`; otherwise samples `count` candidates — position is always
      `shape.position`; draws speed, lifetime, size per the fixed order in
      [research.md](research.md) §4 — discards any candidate with `lifetime_seconds <= 0.0`
      (FR-013), then forwards the remainder to `particle_pool::try_spawn` in one
      all-or-nothing call and returns its result, with `spawned` equal to the forwarded
      count on success); `tick` (sum any attached force accelerations — empty list is a
      valid no-op sum — integrate `velocity += acceleration * dt` then
      `position += velocity * dt` for every live particle, then delegate lifetime
      aging/retirement to `particle_pool::age_and_retire(dt)`; `dt == 0.0` is a no-op);
      `add_force` (push onto the pre-reserved force list, `vfx_status::invalid_argument` if
      full). `cone_shape`/`sphere_shape` sampling in `try_emit` may assert/unreachable for
      now — User Story 2 (T015) fills them in. Depends on T011.
- [X] T013 [US1] Wire `emitter.cpp` into `src/engine_demo/CMakeLists.txt` and
      `test_emitter.cpp` into `tests/engine_demo/CMakeLists.txt`. Build and confirm all T010
      (User Story 1) tests pass (GREEN). Depends on T012.

**Checkpoint**: User Story 1 is fully functional and independently testable — this is the
MVP slice.

---

## Phase 4: User Story 2 - Shape Particle Motion with Composable Forces (Priority: P2)

**Goal**: Emit from `cone_shape`/`sphere_shape` emitters and see attached `gravity_force`/
`wind_force`/`turbulence_force` applicators change particle velocity over multiple ticks.

**Independent Test**: Configure a `cone_shape` emitter and a `sphere_shape` emitter (each
with a different force-applicator combination), emit from each, and verify initial
placement/direction matches the configured shape and that velocity changes over subsequent
ticks according to the attached forces — independent of the `point_shape` emitter used by
User Story 1's tests.

### Tests for User Story 2 ⚠️

- [X] T014 [P] [US2] Extend `tests/engine_demo/test_emitter.cpp` with User Story 2 cases from
      [contracts/emitter.md](contracts/emitter.md): `cone_shape` emitted particle directions
      all lie within `half_angle_radians` of the configured direction (dot-product check);
      `sphere_shape` emitted particle positions all lie within `radius` of `center`
      (distance check); an emitter with `gravity_force` + `wind_force` attached shows
      velocity changing by the summed acceleration across multiple ticks (FR-007, FR-008).
      Expected to fail (new cases RED) until T015–T016 land.

### Implementation for User Story 2

- [X] T015 [US2] Implement `cone_shape` and `sphere_shape` sampling in `emitter.cpp`'s
      `try_emit` per [contracts/emitter.md](contracts/emitter.md)'s fixed-order shape
      sampling (cone: 2 `rng` draws for direction within the cone, position always
      `shape.origin`; sphere: 3 `rng` draws for a uniformly-sampled position within the
      volume, direction outward from `shape.center`). Depends on T012.
- [X] T016 [US2] Implement force summation in `emitter.cpp`'s `tick`: for every live
      particle, call `apply_force` for each attached `force_applicator` (consuming `rng` for
      any `turbulence_force`, per-particle, in stable dense-array index order per
      [research.md](research.md) §4), sum the resulting accelerations, then integrate as in
      T012. Depends on T015.
- [X] T017 [US2] Build and confirm all T014 (User Story 2) tests pass (GREEN), and that all
      User Story 1 tests (T010) still pass. Depends on T016.

**Checkpoint**: User Stories 1 and 2 both work independently.

---

## Phase 5: User Story 3 - Replay-Safe, Real-Time Particle Simulation at Scale (Priority: P3)

**Goal**: Prove that identical seed + identical call sequence yields byte-identical particle
state over an extended run, and that 500 simultaneous particles stay well within the 2 ms
frame budget.

**Independent Test**: Construct two `allocator`/`particle_pool`/`emitter` triples with
identical `emitter_config` (same `seed`), drive both through an identical sequence of
`try_emit`/`tick` calls for 1,000 frames, and compare `pool().live_particles()` for exact
equality. Separately, measure a single `tick()` at 500 live particles and confirm it
completes in under 2 ms.

### Tests for User Story 3 ⚠️

- [X] T018 [P] [US3] Write `tests/engine_demo/test_vfx_determinism.cpp` per
      [contracts/emitter.md](contracts/emitter.md)'s determinism test obligations: two
      emitters built with identical `emitter_config` (same `seed`, same shape, same attached
      forces) and driven through an identical sequence of `try_emit`/`tick` calls over 1,000
      frames produce byte-identical `live_particles()` snapshots (`EXPECT_EQ` on every
      field, not `EXPECT_NEAR`, per
      [.github/instructions/tests.instructions.md](../../.github/instructions/tests.instructions.md));
      and replaying the same burst from a freshly-constructed, identically-seeded emitter
      reproduces identical initial particle attributes (FR-010, SC-003). Expected to fail to
      compile/link until wired in T019 (should already pass functionally, since T012/T016
      built determinism in from the start — this phase is a validation gate, not new
      production code).

### Implementation for User Story 3

- [X] T019 [US3] Wire `test_vfx_determinism.cpp` into `tests/engine_demo/CMakeLists.txt`.
      Build and confirm T018 passes. If any non-determinism surfaces (e.g., an `rng`
      draw-order dependency on iteration order), fix it in `emitter.cpp` and/or
      `force_applicator.cpp` and re-verify T010/T014 still pass. Depends on T018, T016.
- [X] T020 [P] [US3] Add an SC-002 perf smoke test to `tests/engine_demo/test_emitter.cpp`:
      construct a 500-capacity pool/emitter with gravity + wind + turbulence attached, fully
      populate it, and assert `tick()` wall-clock time stays comfortably under the 2 ms/frame
      budget (use a generous CI-safe multiple, e.g. assert under a bound well above 2 ms but
      far below a value that would mask a real regression, matching this codebase's existing
      perf-test conventions). Depends on T016.

**Checkpoint**: All three user stories are independently functional and tested.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Validate the feature's remaining Success Criteria and tidy up documentation.

- [X] T021 [P] Add an SC-006 sustained-run test to `tests/engine_demo/test_particle_pool.cpp`:
      drive a small-capacity pool through >= 10,000 frames of continuous `try_spawn`/
      `age_and_retire` calls once capacity is reached, asserting `vfx_status::pool_exhausted`
      is reported on every over-capacity attempt with zero crashes, and confirm
      `allocator::bytes_used()` does not grow after the pool's initial construction.
- [X] T022 [P] Update [.github/copilot-instructions.md](../../.github/copilot-instructions.md)'s
      subsystem list (`allocator, ecs, physics, sim, frame_budget` -> add `vfx`). Leave
      [README.md](../../README.md)'s sandbox-scene dependency line untouched: `apps/sandbox`
      does not construct an `engine_demo::vfx::emitter` yet, so listing it there would be
      inaccurate until a follow-up feature wires VFX into the sandbox scene.
- [X] T023 Run the full local gate end-to-end: `cmake --preset default-debug`,
      `cmake --build --preset default-debug`, `ctest --preset default-debug
      --output-on-failure`. Confirm every `vfx`-related test is green, then walk through
      [quickstart.md](quickstart.md)'s usage example against the final API and correct any
      drift found. Depends on all prior tasks.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup. BLOCKS all user stories (`emitter.h` embeds a
  `force_applicator` vector, so both `particle_pool` and `force_applicator` must exist
  first).
- **User Story 1 (Phase 3)**: Depends on Foundational (T005, T009). No dependency on US2/US3.
- **User Story 2 (Phase 4)**: Depends on Foundational and on User Story 1's `emitter.h`/
  `emitter.cpp` existing (T011, T012) — it extends the same files rather than duplicating
  them, per [research.md](research.md)'s incremental shape/force design.
- **User Story 3 (Phase 5)**: Depends on Foundational and on T016 (force summation) so the
  determinism and perf checks exercise the full tick path, including forces.
- **Polish (Phase 6)**: Depends on all desired user stories being complete.

### Parallel Opportunities

- T002 (test) and T003 (header) can run in parallel — different files, no code dependency
  between writing a test and writing the header it will eventually link against.
- T006 and T007 can run in parallel (same reasoning).
- T010 can start as soon as T005/T009 land, in parallel with implementation work on T011 if
  staffed by a different person (though T011 does not depend on T010 being written first —
  T010 only needs to be written and RED before T012 is considered done).
- T018 and T020 both depend on T016 but touch different files/assertions and can run in
  parallel.
- T021 and T022 are mutually independent (different files) and can run in parallel.
- User Story 2 and User Story 3 test-writing (T014, T018) can be drafted in parallel by
  different people once Foundational is done, even though their implementation tasks
  (T015–T016 vs. T019) share `emitter.cpp` and must land sequentially.

---

## Parallel Example: Foundational Phase

```bash
# Launch T002 and T003 together (different files, no dependency):
Task: "Write tests/engine_demo/test_particle_pool.cpp per contracts/particle_pool.md"
Task: "Create include/engine_demo/vfx/particle.h per contracts/particle_pool.md"

# Later, launch T006 and T007 together:
Task: "Write tests/engine_demo/test_force_applicator.cpp per contracts/force_applicator.md"
Task: "Create include/engine_demo/vfx/force_applicator.h per contracts/force_applicator.md"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: Setup (T001).
2. Phase 2: Foundational (T002–T009) — **CRITICAL**, blocks everything else.
3. Phase 3: User Story 1 (T010–T013).
4. **STOP and VALIDATE**: run `ctest --preset default-debug -R test_emitter
   --output-on-failure` and confirm User Story 1's acceptance scenarios all pass
   independently of physics, including the all-or-nothing pool-exhaustion and
   lifetime-skip edge cases.

### Incremental Delivery

1. Setup + Foundational → shared `particle_pool` + `force_applicator` ready.
2. Add User Story 1 (T010–T013) → point-emitter spawn/tick/retire works → MVP demo-able.
3. Add User Story 2 (T014–T017) → cone/sphere shapes + composable forces → richer effects.
4. Add User Story 3 (T018–T020) → determinism and perf proven → safe for replay-based QA
   tooling and real-time use at scale.
5. Polish (T021–T023) → allocation-free-at-scale validated, docs updated.

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (T001–T009).
2. Once Foundational is done:
   - Developer A: User Story 1 (T010–T013)
   - Developer B: waits on Developer A's `emitter.h`/`emitter.cpp` (T011/T012) before
     starting User Story 2 (T014–T017), since both extend the same files
   - Developer C: drafts User Story 3's test file (T018) and perf test (T020) in parallel,
     wiring/verifying them (T019) once T016 lands
3. Stories complete and integrate independently; Polish (T021–T023) closes out the feature.
