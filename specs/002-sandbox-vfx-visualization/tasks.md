# Tasks: Sandbox VFX Visualization

**Input**: Design documents from `/specs/002-sandbox-vfx-visualization/`
**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/](contracts/)

**Tests**: Constitution Article 7 ("Every public function has at least one GTest") and the
feature's Success Criteria (SC-001, SC-002) make tests mandatory for this feature, not
optional. Every implementation task below is preceded by its test task, written first and
expected to fail (RED) until the corresponding implementation task lands (GREEN), per
[.github/instructions/tests.instructions.md](../../.github/instructions/tests.instructions.md).
The one exception is the raylib renderer/HUD wiring (`app.cpp`), which has no automated
test — verified manually per [quickstart.md](quickstart.md), matching Feature 001's
precedent that graphical render code is outside the GTest surface.

**Organization**: Tasks are grouped by user story (US1/US2/US3, matching [spec.md](spec.md)
priorities P1/P2/P3) after a Foundational phase that all three stories depend on (the
library `set_shape` addition, plus the shared `m_vfx_pool`/`m_vfx_emitter` scene members).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Paths are relative to the repository root

## Path Conventions

Single project (existing `engine_demo` library + `apps/sandbox` app — see [plan.md](plan.md)
Project Structure):

- Library header/source: `include/engine_demo/vfx/emitter.h`, `src/engine_demo/vfx/emitter.cpp`
- Sandbox: `apps/sandbox/scene.h`, `apps/sandbox/scene.cpp`, `apps/sandbox/app.cpp`
- Tests: `tests/engine_demo/test_emitter.cpp` (existing), `tests/engine_demo/test_scene_vfx.cpp` (new)

---

## Phase 1: Setup

**Purpose**: Confirm a clean baseline before touching the library or the sandbox

- [ ] T001 Confirm the existing build/test baseline is green: `cmake --preset default-debug`,
      `cmake --build --preset default-debug`, `ctest --preset default-debug
      --output-on-failure`. Also re-run the headless golden trace at seed 42/600 frames and
      confirm its `trace_digest` matches this session's already-recorded
      `baseline-trace.csv` — this is the number every later digest-parity task compares
      against. Fix or report any pre-existing failure before proceeding.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: `emitter::set_shape` and `scene`'s VFX pool/emitter members are structural
dependencies of every user story — US1 (burst) and US2 (spark) both call `set_shape` +
`try_emit` on the same shared emitter, and US3 (HUD) reads the same pool via
`vfx_particle_count()`. All three must exist and be tested before any user-story-specific
work begins.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T002 [P] Add 3 new cases to `tests/engine_demo/test_emitter.cpp` per
      [contracts/emitter-set-shape.md](contracts/emitter-set-shape.md) test obligations:
      happy path (`set_shape` to a `sphere_shape` moves subsequent spawn positions off the
      old shape), no-reallocation (`allocator::bytes_used()` unchanged across several
      `set_shape` calls interleaved with `try_emit`/`tick`), and rng-sequence-unaffected
      (two identically-seeded emitters, differing `set_shape` call counts between identical
      `try_emit` calls, still produce identical spawned-particle sequences). Expected to
      fail to compile/link until T003 lands.
- [ ] T003 Add `void set_shape(emitter_shape shape) noexcept;` to
      `include/engine_demo/vfx/emitter.h` and implement it in
      `src/engine_demo/vfx/emitter.cpp` (overwrites `m_cfg.shape` only; no allocation; does
      not touch `m_rng`/`m_pool`/`m_forces`/`m_scratch`) per
      [contracts/emitter-set-shape.md](contracts/emitter-set-shape.md). Build and confirm
      all T002 cases pass (GREEN), and that every pre-existing `test_emitter` case still
      passes. Depends on T002.
- [ ] T004 [P] Create `tests/engine_demo/test_scene_vfx.cpp` (new file, raylib-free —
      compiles `apps/sandbox/scene.cpp` directly per [research.md](research.md) §6) and wire
      it into `tests/engine_demo/CMakeLists.txt` following the exact `test_sandbox_render`
      pattern (`engine_demo_add_test` + `target_sources` for `apps/sandbox/scene.cpp` +
      `target_include_directories(${CMAKE_SOURCE_DIR})`). Add foundational assertions per
      [data-model.md](data-model.md): a freshly constructed `scene` reports
      `vfx_particle_count() == 0`; the VFX pool's construction fits comfortably within the
      existing 4 MiB arena (`m_alloc`'s `bytes_used() < capacity()` after construction,
      matching the arena headroom table). Expected to fail to compile/link until T005 lands.
- [ ] T005 Add `m_vfx_pool` (`engine_demo::vfx::particle_pool`) and `m_vfx_emitter`
      (`engine_demo::vfx::emitter`) members, the VFX constants (`kVfxPoolCapacity`,
      `kVfxBurstCount`, `kVfxSparkCount`, `kVfxSpawnRadius`, `kVfxLifetimeSeconds`,
      `kVfxSeedSalt`), the `vfx_particle_view` struct, and the
      `vfx_particle_count()`/`vfx_particle_at()` accessors to `apps/sandbox/scene.h`. In
      `apps/sandbox/scene.cpp`: construct `m_vfx_pool`/`m_vfx_emitter` in `scene`'s
      constructor initializer list (seed = `m_seed ^ kVfxSeedSalt`, never drawn from
      `m_rng` — [research.md](research.md) §3), add the private `reset_vfx()` that destroys
      and placement-new reconstructs both members in place (mirrors `reset_solver()`), and
      call `reset_vfx()` from `rebuild_scene()` immediately after `reset_solver()`. Build and
      confirm all T004 assertions pass (GREEN). Depends on T003, T004.

**Checkpoint**: `set_shape` exists and is tested; `scene` owns a constructed, tested,
zero-population VFX pool/emitter with working accessors. User-story-specific work (burst,
spark, HUD) can now begin.

---

## Phase 3: User Story 1 - Visual Feedback on Right-Click Burst (Priority: P1) 🎯 MVP

**Goal**: Right-clicking spawns a fading radial VFX burst at the cursor, layered on top of
the existing (unmodified) 12-particle physics burst, with zero effect on `state_digest()`.

**Independent Test**: Launch the sandbox, right-click in the world, and observe a fading VFX
burst at the click location alongside the existing physics burst; independently confirm via
the headless golden trace that triggering this burst never changes `state_digest()` for an
identical seed/frame sequence.

### Tests for User Story 1 ⚠️

- [ ] T006 [P] [US1] Extend `tests/engine_demo/test_scene_vfx.cpp` with User Story 1 cases
      per [contracts/scene-vfx.md](contracts/scene-vfx.md): `spawn_vfx_burst` increases
      `vfx_particle_count()` by up to `kVfxBurstCount` while `particle_count()` (the
      existing ad-hoc physics burst) is unaffected; VFX particles age and retire across
      repeated `step()` calls, returning `vfx_particle_count()` to 0 after
      `kVfxLifetimeSeconds` of sim time with no further bursts; `vfx_particle_at(i)
      .life_fraction` starts at 1.0 immediately after spawn and decreases monotonically;
      and (extending the Foundational reset check now that a spawn method exists) calling
      `reseed()`/`switch_scene()` after a burst leaves `vfx_particle_count() == 0`. Expected
      to fail to compile/link until T010 lands.
- [ ] T007 [P] [US1] Extend `tests/engine_demo/test_scene_vfx.cpp` with the FR-009
      pool-exhaustion case per [contracts/scene-vfx.md](contracts/scene-vfx.md): drive
      `m_vfx_pool` to full capacity via repeated `spawn_vfx_burst` calls (enough bursts to
      reach `kVfxPoolCapacity`, rounding up), then call `spawn_vfx_burst`/`spawn_vfx_spark`
      once more and assert: no crash, `vfx_particle_count()` never exceeds
      `kVfxPoolCapacity`, and pre-existing live VFX particle state (spot-checked via
      `vfx_particle_at(i)`) is unchanged by the exhausted call. Closes the zero-coverage gap
      on FR-009 identified in `/speckit.analyze`. Expected to fail to compile/link until
      T010 lands.
- [ ] T008 [P] [US1] Add a minimal `[[nodiscard]] std::size_t arena_bytes_used() const
      noexcept` accessor to `apps/sandbox/scene.h`/`scene.cpp` (one-line forwarding getter
      to `m_alloc.bytes_used()`), then extend `tests/engine_demo/test_scene_vfx.cpp` with an
      Article 6 case: record `arena_bytes_used()` immediately before and after several
      `spawn_vfx_burst`/`spawn_vfx_spark` calls and assert it is unchanged — proving the
      scene-level wrapper methods themselves (not just the `try_emit`/`set_shape` calls they
      invoke) never allocate. Closes the Article 6 verification gap identified in
      `/speckit.analyze` (previously only checked at construction-time in T004 and at the
      emitter level in T002). Expected to fail to compile/link until T010 (and the
      accessor) lands.
- [ ] T009 [P] [US1] Add the digest-parity case (SC-001) to `test_scene_vfx.cpp`: construct
      two identically-seeded `scene` instances, call `spawn_vfx_burst` periodically on one
      (never on the other), step both through an identical number of frames, and assert
      `state_digest()` is identical between the two at every compared frame. Expected to
      pass immediately once T010 lands (this is a regression guard, not new production
      behavior — `state_digest()` never reads VFX state by construction).

### Implementation for User Story 1

- [ ] T010 [US1] Add public `spawn_vfx_burst(double wx, double wy) noexcept` to
      `apps/sandbox/scene.h`/`scene.cpp`: calls `m_vfx_emitter.set_shape(sphere_shape{{wx,
      wy, 0}, kVfxSpawnRadius})` then `m_vfx_emitter.try_emit(kVfxBurstCount)`, per
      [contracts/scene-vfx.md](contracts/scene-vfx.md) (pool exhaustion handled gracefully,
      status not surfaced). Add the `m_vfx_emitter.tick(dt)` call once per `substep(double
      dt)`, after all per-step physics/spark work. Depends on T005.
- [ ] T011 [US1] Build and confirm all T006–T009 tests pass (GREEN), and that T002/T004
      foundational tests still pass. Depends on T010.
- [ ] T012 [US1] In `apps/sandbox/app.cpp`: the existing `MOUSE_BUTTON_RIGHT` handler
      additionally calls `s.spawn_vfx_burst(mwx, mwy)` right after the existing
      `s.spawn_particle_burst(mwx, mwy, 12)` call (purely additive — that call is otherwise
      untouched). In `draw_scene()`, add a render pass iterating
      `s.vfx_particle_count()`/`s.vfx_particle_at(i)` that draws each live VFX particle with
      alpha derived from `life_fraction`. No automated test (raylib render path) — verified
      manually per [quickstart.md](quickstart.md) §3. Depends on T010.
- [ ] T013 [US1] Manual verification per [quickstart.md](quickstart.md) §2 (golden digest
      A/B: headless run at seed 42/600 frames, `trace_digest` must equal T001's baseline)
      and §3 (visual smoke test: right-click produces a fading burst layered over the
      existing physics burst). Depends on T011, T009, T012.

**Checkpoint**: User Story 1 is fully functional and independently testable — this is the
MVP slice.

---

## Phase 4: User Story 2 - Collision Sparks on World-Bound Bounce (Priority: P2)

**Goal**: A free particle bouncing off a world bound in a non-storm scene emits a small
spark burst at the contact point; the particle storm scene (which wraps, not bounces) never
sparks.

**Independent Test**: Run a non-storm scene, let a free particle bounce off a world bound,
and observe a spark burst at the contact point (rendered via the same draw pass US1 already
built); confirm via the headless golden trace that these automatically-triggered sparks
never change `state_digest()`.

### Tests for User Story 2 ⚠️

- [ ] T014 [P] [US2] Extend `tests/engine_demo/test_scene_vfx.cpp` with User Story 2 cases
      per [contracts/scene-vfx.md](contracts/scene-vfx.md): a bounce in a non-storm scene
      (e.g. `rope`) increases `vfx_particle_count()`; an equivalent world-bound crossing in
      `particle_storm` (which wraps) does not. Expected to fail (new cases RED) until T015
      lands.

### Implementation for User Story 2

- [ ] T015 [US2] Add private `spawn_vfx_spark(double wx, double wy) noexcept` to
      `apps/sandbox/scene.cpp` (identical to `spawn_vfx_burst` but with
      `kVfxSparkCount`), and call it from both non-storm bounce sites (x-bound, y-bound) in
      `substep()`'s free-particle handling, at the already-clamped contact position. Never
      called from the `particle_storm` branch (FR-003). Depends on T014, T010.
- [ ] T016 [US2] Build and confirm all T014 tests pass (GREEN), and that all User Story 1
      tests (T006, T007, T008, T009) still pass. Depends on T015.
- [ ] T017 [P] [US2] Re-run the headless golden trace (seed 42, 600 frames — bounces occur
      naturally during normal physics, so this run already exercises spark emission) and
      confirm `trace_digest` still equals T001's baseline. Depends on T015.
- [ ] T018 [US2] Manual verification per [quickstart.md](quickstart.md) §3: bounces in
      `rope`/`pendulum_tower`/`cloth` (keys `1`/`2`/`3`) show a spark at the contact point;
      `particle_storm` (key `4`) shows no sparks on wrap. Depends on T015.

**Checkpoint**: User Stories 1 and 2 both work independently.

---

## Phase 5: User Story 3 - Observe Live VFX Load via HUD (Priority: P3)

**Goal**: The HUD shows a live count of currently-alive VFX particles, rising on
burst/spark and falling as particles expire.

**Independent Test**: Trigger right-click bursts and/or collision sparks repeatedly and
confirm the HUD's live-VFX-count reading increases as particles spawn and decreases as they
expire.

### Implementation for User Story 3

- [ ] T019 [US3] In `apps/sandbox/app.cpp`'s `draw_hud()`, extend the existing top-right
      counts panel (`bodies=... edges=... particles=...`) to also print
      `vfx=%zu` from `s.vfx_particle_count()`. No automated test (raylib render/HUD path) —
      verified manually per [quickstart.md](quickstart.md) §3 (HUD count rises on
      burst/spark, falls back toward 0 as particles expire). Depends on T005 (accessor),
      T012 (panel code already touched by US1's renderer wiring).

**Checkpoint**: All three user stories are independently functional and tested.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Validate the feature's remaining Success Criteria and confirm no regressions.

- [ ] T020 [P] Manual SC-003 frame-budget check per [quickstart.md](quickstart.md) §4: with
      the VFX pool near full capacity (sustained right-click bursts) and a physics
      perf-bomb (`P` key) active, confirm the HUD's `frame_avg=...ms` reading stays close to
      the pre-feature baseline and the sandbox holds its fixed 60 FPS step.
- [ ] T021 [P] Add an automated SC-003/FR-010 perf smoke test to
      `tests/engine_demo/test_scene_vfx.cpp`, mirroring Feature 001's `test_emitter.cpp`
      perf-smoke precedent: populate `m_vfx_pool` to `kVfxPoolCapacity`, call `step()`
      repeatedly, and assert wall-clock time per `step()` stays comfortably under the fixed
      60 FPS step budget (a generous CI-safe multiple of 16.67 ms, matching this codebase's
      existing perf-test conventions). Complements T020's manual check with a repeatable
      regression guard that needs no graphical window. Closes the automation gap identified
      in `/speckit.analyze`.
- [ ] T022 [P] Final full-gate golden-trace re-check: headless run at seed 42/600 frames,
      confirm `trace_digest` still equals T001's baseline (last-mile regression guard after
      all tasks land, independent of T017's mid-feature check).
- [ ] T023 Run the full local gate end-to-end: `cmake --preset default-debug`,
      `cmake --build --preset default-debug`, `ctest --preset default-debug
      --output-on-failure`. Confirm every `vfx`/`scene_vfx`-related test is green, then walk
      through [quickstart.md](quickstart.md) end-to-end and correct any drift found against
      the final API. Depends on all prior tasks.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup. BLOCKS all user stories (`scene`'s
  constructor must own a constructed `m_vfx_pool`/`m_vfx_emitter` before any burst/spark
  method can be added, and `set_shape` must exist before either method can reposition the
  shared emitter).
- **User Story 1 (Phase 3)**: Depends on Foundational (T003, T005). No dependency on
  US2/US3.
- **User Story 2 (Phase 4)**: Depends on Foundational and on User Story 1's
  `spawn_vfx_burst`/`tick()` wiring (T010) — it reuses the same shared emitter and the same
  `substep()` tick call rather than duplicating either.
- **User Story 3 (Phase 5)**: Depends on Foundational (T005's accessor) and on User Story
  1's HUD panel edit point (T012) already existing in `app.cpp`.
- **Polish (Phase 6)**: Depends on all desired user stories being complete.

### Parallel Opportunities

- T002 (library test) and T004 (scene test skeleton) can run in parallel — different files,
  no dependency between them.
- T006, T007, and T008 can run in parallel — all extend the same new test file with
  independent cases (burst behavior, pool exhaustion, and no-allocation, respectively), all
  depending only on T010 to go green.
- T009 (digest-parity test) can be drafted in parallel with T006–T008, though it also
  depends on T010 to go green.
- T014 (US2 tests) can be drafted in parallel with US1 implementation work (T010–T012) by a
  different person, though T015 cannot land until T010 exists (shared emitter/tick wiring).
- T020 and T022 are mutually independent manual/verification checks and can run in
  parallel; T021 (automated perf test) can be drafted independently of both.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 1: Setup (T001).
2. Phase 2: Foundational (T002–T005) — **CRITICAL**, blocks everything else.
3. Phase 3: User Story 1 (T006–T013).
4. **STOP and VALIDATE**: run `ctest --preset default-debug -R "test_emitter|test_scene_vfx"
   --output-on-failure`, then run the headless golden-trace A/B check
   ([quickstart.md](quickstart.md) §2) and confirm the right-click burst is visible and the
   digest is unchanged — User Story 1's acceptance scenarios all pass independently of
   User Stories 2/3.

### Incremental Delivery

1. Setup + Foundational → shared `set_shape` + VFX pool/emitter ready, zero population.
2. Add User Story 1 (T006–T013) → right-click burst visible, digest-safe, pool-exhaustion
   and zero-allocation guarantees verified → MVP demo-able.
3. Add User Story 2 (T014–T018) → collision sparks in non-storm scenes → richer feedback.
4. Add User Story 3 (T019) → HUD live-VFX-count readout → observability polish.
5. Polish (T020–T023) → frame budget (manual + automated), and final digest parity
   confirmed, quickstart validated.
