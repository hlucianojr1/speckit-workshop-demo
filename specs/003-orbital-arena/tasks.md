# Tasks: Orbital Arena — Competitive Gravity-Well Game

**Input**: Design documents from `specs/003-orbital-arena/`
**Prerequisites**: plan.md, spec.md, research.md (D1–D10), data-model.md, contracts/ (gravity_well, scoring, powerup, match, input, arena)
**Branch**: `006-orbital-arena` (feature dir pinned to `specs/003-orbital-arena/` via `.specify/feature.json`)

**Tests**: REQUIRED and test-first (Constitution Article 7 + 7a). Every module's GTest task
precedes its implementation task; the new test MUST be written, compiled, and observed to
FAIL before the paired implementation task begins.

**Sizing**: Every task ≤150 lines of new code (training §3.8 budget). The plan's 11-unit
decomposition is preserved; test-first splitting turns most units into a (test, impl) pair.

**Organization**: Tasks are grouped by user story (spec.md P1–P4) so each story is an
independently testable increment.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: US1–US4 from spec.md; Setup/Foundational/Polish tasks carry no story label
- Every task lists exact file paths

## Path Conventions

Per plan.md Project Structure: new library at `include/orbital_arena/` +
`src/orbital_arena/`, tests at `tests/orbital_arena/`, sandbox edits only in
`apps/sandbox/`. **Zero edits** under `include/engine_demo/` or `src/engine_demo/`.

Build/verify commands for every checkpoint (see AGENTS.md; run `.specify` scripts via
Git-for-Windows bash):

```bash
cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Wire the new `orbital_arena` static library and its test directory into the
build so every later task compiles into a real target (plan.md "CMake Wiring" — the
training's 8-task table omits this entirely).

- [x] T001 Create `orbital_arena` CMake scaffolding (~60 lines): new
      `src/orbital_arena/CMakeLists.txt` (STATIC lib, `target_link_libraries(... PUBLIC engine_demo)`,
      `engine_demo_set_target_options(orbital_arena)`, include dir `${CMAKE_SOURCE_DIR}/include`);
      new `tests/orbital_arena/CMakeLists.txt` with `orbital_arena_add_test()` helper cloned
      from `engine_demo_add_test()` linking `orbital_arena GTest::gtest GTest::gtest_main`
      (NEVER `GTest::gmock` — known false-positive bug); one-line edits to
      `src/CMakeLists.txt` and `tests/CMakeLists.txt` (`add_subdirectory(orbital_arena)`);
      minimal `include/orbital_arena/arena.h` stub with `arena_status` + constants from
      data-model.md, `src/orbital_arena/arena.cpp` stub, and a smoke test
      `tests/orbital_arena/test_arena_integration.cpp` asserting `arena_status::ok == arena_status::ok`.
      Verify: configure + build + `ctest` shows the new test running (`-V` shows `[ RUN ]`).

**Checkpoint**: `orbital_arena` library + test target exist and are green in CTest.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The input module (plan task 7) — `input_frame` is consumed by the match
roster, the arena tick, and the replay log; no story's integration path works without it.

**⚠️ CRITICAL**: Complete before any user story integration work.

- [x] T002 [P] Write failing GTests for input in `tests/orbital_arena/test_input.cpp`
      (~70 lines): `input_frame` steer clamped to unit box and strength clamped 0..1 on
      ingest (FR-015); fixed player→well mapping set at lobby time; append-only input log
      records inputs in every match state (FR-016), returns `arena_status::log_full` when
      the reserved capacity is exhausted; log is `reserve()`d once (Article 6). Register
      in `tests/orbital_arena/CMakeLists.txt`. MUST fail to compile/link before T003.
- [x] T003 Implement input module in `include/orbital_arena/input.h` +
      `src/orbital_arena/input.cpp` (~80 lines): `input_frame { float steer[2]; float strength; }`,
      `tick_inputs = input_frame[kMaxPlayers]`, log as
      `eastl::vector<tick_inputs>` with explicit `engine_demo::allocator&` (Articles 3–4),
      reserved for `kMaxRecordedTicks` at construction, clamping on ingest per contract
      `contracts/input.md`. All T002 tests green.

**Checkpoint**: Input log foundation green — user stories can start.

---

## Phase 3: User Story 1 — Control a Gravity Well and Capture Particles (Priority: P1) 🎯 MVP

**Goal**: The core loop — steer a well, modulate strength, attract vfx particles, capture
them deterministically, and see the owning player's score increment (plan tasks 2–3).

**Independent Test**: One active well + particle field; move well near a cluster at full
strength → particles accelerate toward it each tick and are removed & counted on entering
the capture radius (spec.md US1 Independent Test).

### Tests for User Story 1 (write first, MUST fail)

- [x] T004 [P] [US1] Write failing GTests for gravity well in
      `tests/orbital_arena/test_gravity_well.cpp` (~80 lines): `radial_acceleration()`
      inverse-square with clamped min distance, zero beyond `influence_radius`, scales
      linearly with `strength`, exactly zero at `strength == 0` (FR-003, US1 sc.1/3);
      `is_within_capture()` squared-distance predicate, no sqrt (FR-004); well position
      clamped to `±kArenaHalfExtent` (US1 sc.4); velocity magnitude capped at
      `kMaxWellSpeed` (FR-002); inactive well exerts zero pull (FR-013). Register in
      `tests/orbital_arena/CMakeLists.txt`.
- [x] T006 [P] [US1] Write failing GTests for capture contention in
      `tests/orbital_arena/test_scoring.cpp` (~70 lines): nearest well wins a contested
      particle by strict minimum squared distance; exact distance tie → no capture that
      tick (FR-007, US1 sc.5); resolution identical under permuted well iteration order
      (Article 9 index-independence); `award_capture()` adds base value 1 per standard
      particle, only mutates scores during `playing` (FR-005); capture reflected same tick
      (SC-004). Register in `tests/orbital_arena/CMakeLists.txt`.

### Implementation for User Story 1

- [x] T005 [US1] Implement gravity well in `include/orbital_arena/gravity_well.h` +
      `src/orbital_arena/gravity_well.cpp` (~110 lines): POD `gravity_well` per
      data-model.md (position, velocity, strength, influence_radius, capture_radius,
      active); free functions `radial_acceleration()` + `is_within_capture()` applied to
      `vfx::particle_pool::live_particles()` spans (research.md D1 — NOT via
      `physics::constraint_solver`); boundary clamp + speed cap helpers. Per contract
      `contracts/gravity_well.md`. All T004 tests green. (depends on T004)
- [x] T007 [US1] Implement capture & scoring core in `include/orbital_arena/scoring.h` +
      `src/orbital_arena/scoring.cpp` (~90 lines): `score_table`
      (`uint32_t scores[kMaxPlayers]`, `winner = -1`, `sudden_death = false`),
      `resolve_captures()` — squared-distance nearest-well contention in fixed well-index
      iteration order with distance-only comparison (research.md D3), capture removes the
      particle by zeroing `remaining_lifetime_seconds` so `age_and_retire()` swap-removes
      it (research.md D2 — zero engine changes), `award_capture(player, base_value, double_points_active)`.
      Per contract `contracts/scoring.md`. All T006 tests green. (depends on T005, T006)

**Checkpoint**: US1 fully functional — a single well can chase, attract, and capture
particles with deterministic contention and same-tick scoring. MVP demonstrable via test.

---

## Phase 4: User Story 2 — Win a Match (Scoring & Match Flow) (Priority: P2)

**Goal**: Lobby → countdown → playing → game_over lifecycle with the 100-point win rule
and sudden-death tie handling (plan tasks 4 + 6).

**Independent Test**: Scripted deterministic match runs lobby → countdown → playing →
game_over; first player to 100 is winner; scores freeze on the winning tick (spec.md US2
Independent Test).

### Tests for User Story 2 (write first, MUST fail)

- [x] T008 [P] [US2] Extend `tests/orbital_arena/test_scoring.cpp` with failing win-rule
      GTests (~50 lines): score reaching exactly 100 and overshooting past 100 both end
      the match that tick with that player as winner (FR-006, US2 sc.3); two players
      reaching ≥100 on the same tick → `sudden_death = true`, next sole-capture tick
      decides (FR-008, US2 sc.4); no scoring after `game_over` (US2 sc.3).
- [x] T010 [P] [US2] Write failing GTests for the match state machine in
      `tests/orbital_arena/test_match.cpp` (~80 lines): only legal transitions
      `lobby → countdown → playing → game_over → lobby` (FR-012); all joined (2–4) players
      ready → countdown (US2 sc.1); countdown is exactly `kCountdownTicks = 180` integer
      ticks (research.md D4, no float accumulation) then `playing` with zeroed scores
      (US2 sc.2); un-ready/leave during countdown → back to `lobby` (FR-013); departure
      during `playing` deactivates the well, match continues; <2 active players →
      `game_over` with last remaining player as winner (FR-013, edge case); inputs outside
      `playing` have no game effect but are still logged (FR-014, US2 sc.6); acknowledge
      in `game_over` → `lobby` with scores reset (US2 sc.5). Register in
      `tests/orbital_arena/CMakeLists.txt`.

### Implementation for User Story 2

- [x] T009 [US2] Extend `include/orbital_arena/scoring.h` + `src/orbital_arena/scoring.cpp`
      with win detection (~60 lines): `check_win_condition()` evaluating FR-006/FR-008
      after each tick's captures, sudden-death entry/resolution, winner latching, score
      freeze. All T008 tests green. (depends on T007, T008)
- [x] T011 [US2] Implement match state machine in `include/orbital_arena/match.h` +
      `src/orbital_arena/match.cpp` (~110 lines): `match_state` enum, `player_slot`
      roster (joined/ready/departed per data-model.md), integer tick countdown, transition
      guards returning `arena_status::wrong_state` for illegal calls (Article 1 — no
      exceptions), effects-cleared hook signature for `game_over` (consumed by US3). Per
      contract `contracts/match.md`. All T010 tests green. (depends on T010)

**Checkpoint**: US1 + US2 — a full scripted match runs lobby to declared winner (SC-001).

---

## Phase 5: User Story 3 — Power-Ups Shake Up the Match (Priority: P3)

**Goal**: Seed-driven pickup spawning, first-arrival consumption, and the three timed/
instant effects (plan task 5).

**Independent Test**: Fixed-seed deterministic match: pickups spawn on the 600-tick
schedule at seed-determined positions, first well to touch consumes, effects last exactly
their stated tick durations, all effects end at `game_over` (spec.md US3 Independent Test).

### Tests for User Story 3 (write first, MUST fail)

- [x] T012 [P] [US3] Write failing GTests for power-ups in
      `tests/orbital_arena/test_powerup.cpp` (~90 lines): spawn every
      `kPowerupSpawnIntervalTicks = 600` during `playing` only, position and kind drawn
      from the arena-owned `sim::rng` in fixed order (FR-009/FR-019, research.md D6);
      spawn skipped but timer continues when 3 pickups alive (FR-009, edge case);
      consumption by first capture-zone arrival with FR-007 tie → nobody (US3 sc.2);
      Strength Surge doubles pull for exactly `kStrengthSurgeTicks = 300` then reverts
      (US3 sc.3); Double Points doubles capture value for `kDoublePointsTicks = 600`
      (US3 sc.4); Particle Flood requests an immediate seed-determined neutral burst
      (US3 sc.5); same-kind pickup refreshes duration (data-model.md rule); all effects
      cleared on `game_over` (FR-011, US3 sc.6). Register in
      `tests/orbital_arena/CMakeLists.txt`.

### Implementation for User Story 3

- [x] T013 [US3] Implement power-up module in `include/orbital_arena/powerup.h` +
      `src/orbital_arena/powerup.cpp` (~130 lines): `powerup_kind` enum, fixed
      `pickup[kMaxFieldPickups]` slot array (no allocation, Article 6), `active_effect`
      slots per player with integer `remaining_ticks` (research.md D4), spawn scheduler
      consuming `sim::rng` in deterministic order, effect query helpers
      (`strength_multiplier(player)`, `points_multiplier(player)`), flood-burst request
      surfaced as data for the arena to forward to `vfx::emitter`. Per contract
      `contracts/powerup.md`. All T012 tests green. (depends on T012; needs T005 for
      capture-zone predicate)

**Checkpoint**: US1–US3 all independently green; power-ups verifiably deterministic (SC-006).

---

## Phase 6: User Story 4 — Fair, Replayable Matches (Priority: P4)

**Goal**: Flat snapshot + FNV-1a state hash, the arena integration tick that composes all
modules, and the Article 7a/10 replay proof (plan tasks 8–10).

**Independent Test**: Record a 1000-frame scripted match's input log; replay from the same
seed; every 60-frame state hash and the final scoreboard match exactly (spec.md US4
Independent Test).

### Tests for User Story 4 (write first, MUST fail)

- [x] T014 [P] [US4] Write failing GTests for snapshot + hash in
      `tests/orbital_arena/test_snapshot.cpp` (~70 lines): `match_snapshot` is a flat POD
      (static_asserts: `is_trivially_copyable`, no pointers) per FR-018/Article 11;
      capture → restore round-trip reproduces identical subsequent hashes; `state_hash()`
      is field-wise FNV-1a 64 — NOT raw-struct `memcmp`/byte hashing (research.md D5,
      padding bytes are indeterminate) — and two snapshots differing in exactly one field
      hash differently. Register in `tests/orbital_arena/CMakeLists.txt`. *(Note: adds an
      8th test file beyond plan.md's list of 7 — snapshot has standalone public functions
      and Article 7 requires direct coverage.)*
- [x] T016 [US4] Replace the T001 smoke test with failing integration GTests in
      `tests/orbital_arena/test_arena_integration.cpp` (~100 lines): `arena::tick(inputs)`
      executes the fixed pipeline forces → integrate → captures → scoring → power-ups →
      state machine (contract `contracts/arena.md`); consumes exactly one `input_frame`
      per player per tick and appends to the log (FR-016); wells start rotationally
      symmetric (FR-001); particle replenishment keeps the field populated on the
      deterministic schedule (edge case "empty arena"); `capture_snapshot()`/`restore()`
      round-trip mid-match; hash sampled every `kStateHashIntervalTicks = 60`; tick stays
      within `frame_budget` (Article 6). (depends on T014 for snapshot API surface)

### Implementation for User Story 4

- [x] T015 [US4] Implement snapshot module in `include/orbital_arena/snapshot.h` +
      `src/orbital_arena/snapshot.cpp` (~90 lines): `match_snapshot` POD exactly per
      data-model.md (state, tick, seed, player_count, scores[4], winner, sudden_death,
      well_record[4], effect_record[4], pickup_record[3]); field-wise FNV-1a 64
      `state_hash(const match_snapshot&)`. All T014 tests green. (depends on T014)
- [x] T017 [US4] Implement arena integration in `include/orbital_arena/arena.h` +
      `src/orbital_arena/arena.cpp` (~140 lines): `arena` owns `match`, `score_table`,
      power-up module, input log, wells, one `sim::rng` seeded from the match seed
      (game-rule draws only, fixed order — research.md D6), a `vfx::emitter` +
      `vfx::particle_pool` with XOR-salted sub-seed for replenishment/flood; all pools
      `reserve()`d at construction (Article 6); `tick(inputs)` composes the full pipeline
      inside a `frame_budget` measure; `capture_snapshot()`/`restore()`. All T016 tests
      green. (depends on T003, T005, T007, T009, T011, T013, T015, T016)
- [x] T018 [US4] Write 1000-frame replay determinism + fairness GTests in
      `tests/orbital_arena/test_replay_determinism.cpp` (~100 lines, test-only — Article
      7a/10): (a) scripted 1000-frame match at fixed seed recorded, then replayed from
      `(seed, input log)` — every 60-frame hash and the final scoreboard identical
      (FR-017, SC-002); (b) scripted coverage of capture, win condition, and each power-up
      kind at fixed seeds (Article 7a); (c) input-swap fairness: the same input scripts
      assigned to different player slots produce mirror-equivalent outcomes, zero score
      difference attributable to player index (Article 9, US4 sc.2, SC-003). Register in
      `tests/orbital_arena/CMakeLists.txt`. Must pass against T017 with no implementation
      changes; any failure is an implementation bug. (depends on T017)

**Checkpoint**: All four stories green; `ctest --preset default-debug` fully passes with
`-V` spot-check showing real `[ RUN ]` lines.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Workshop-visible visualization plus proof that the render layer changed
nothing (plan task 11; render-only per spec.md scope note and research.md D7).

- [x] T019 Add sandbox scene `scene_kind::orbital_arena` + HUD (~150 lines, render-only):
      extend `apps/sandbox/scene.h` / `apps/sandbox/scene.cpp` with a new scene (template:
      `particle_storm`) that owns an `orbital_arena::arena` driven by deterministic
      scripted autopilot inputs (research.md D7 — no 4-keyboard problem); HUD in
      `apps/sandbox/app.cpp` showing per-player scores + match state; distinct well/player
      colors. **Digest-excluded**: arena state is never read by `scene::state_digest()`
      (mirrors feature 002's vfx isolation). CMake knock-ons: add `orbital_arena` to
      `target_link_libraries` in `apps/sandbox/CMakeLists.txt` AND to the existing
      `test_scene_vfx` target in `tests/engine_demo/CMakeLists.txt` (it compiles
      `apps/sandbox/scene.cpp` directly). (depends on T017)
- [x] T020 Headless digest A/B verification + screenshots (verification only, ~0 new
      lines): (a) re-run the feature-002 golden-trace check — headless rope scene, seed
      42, 600 frames, `--trace` CSV twice (before/after T019 build) and diff byte-identical,
      proving the new scene left `state_digest()` untouched; (b) run the orbital_arena
      scene headless twice at the same seed and diff identical digests (Article 10 at the
      app layer); (c) capture screenshots (relative `--screenshot` path, Mesa llvmpipe
      DLLs in `build/apps/sandbox/`, warmup ≥400 frames) to `docs/screenshots/orbital-arena-*.png`
      for the workshop doc. Full `ctest --preset default-debug --output-on-failure` green.
      (depends on T019)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: T001 — no dependencies; everything else depends on it.
- **Phase 2 (Foundational)**: T002→T003 — blocks arena integration (Phase 6), not US1–US3 module work.
- **Phase 3 (US1)**: T004→T005, T006→T007 (T007 also needs T005). After T001.
- **Phase 4 (US2)**: T008→T009 (needs T007); T010→T011. Match pair after T001 only.
- **Phase 5 (US3)**: T012→T013 (T013 needs T005's capture predicate). After T001.
- **Phase 6 (US4)**: T014→T015; T016→T017 (T017 needs everything T003–T015); T018 last.
- **Phase 7 (Polish)**: T019 after T017; T020 after T019.

### User Story Dependencies

- **US1 (P1)**: independent — MVP.
- **US2 (P2)**: scoring win rules (T008/T009) build on US1's score_table; match machine (T010/T011) independent.
- **US3 (P3)**: needs only T005's capture predicate; otherwise independent.
- **US4 (P4)**: integrates all prior stories; snapshot pair (T014/T015) is independent.

### Within Each Story

Test task MUST be written and observed failing before its paired implementation task.
Commit after each green (test, impl) pair (HITL gate, Article 8).

### Parallel Opportunities

- After T001: **T002, T004, T006, T010, T012, T014** (six test files, all different files) can be written in parallel.
- After their tests fail: **T003, T005, T011, T015** are mutually parallel (different modules).
- Story-level: US3 (T012–T013) and the match half of US2 (T010–T011) can proceed in parallel with US1.

---

## Parallel Example: post-Setup test wave

```bash
# After T001 is green, write all module tests concurrently (all MUST fail):
Task: "T002 input tests in tests/orbital_arena/test_input.cpp"
Task: "T004 gravity well tests in tests/orbital_arena/test_gravity_well.cpp"
Task: "T006 capture/contention tests in tests/orbital_arena/test_scoring.cpp"
Task: "T010 match state machine tests in tests/orbital_arena/test_match.cpp"
Task: "T012 power-up tests in tests/orbital_arena/test_powerup.cpp"
Task: "T014 snapshot/hash tests in tests/orbital_arena/test_snapshot.cpp"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. T001 (Setup) → T002–T003 (Foundational, optional for MVP demo) → T004–T007 (US1).
2. **STOP and VALIDATE**: `ctest` green; a well attracts and captures particles with
   deterministic contention — demonstrable core loop.

### Incremental Delivery

1. T001 → scaffold green.
2. US1 (T004–T007) → capture loop → MVP.
3. US2 (T008–T011) → full match lifecycle with winner.
4. US3 (T012–T013) → power-ups.
5. US4 (T014–T018) → snapshot + integration + 1000-frame replay proof (Articles 7a/10/11).
6. Polish (T019–T020) → visible game + digest A/B proof + screenshots.

Each story lands as a green, committed increment; the plan's 11-unit shape maps 1:1 onto
these tasks (unit → (test, impl) pair where applicable).

---

## Notes

- Constitution v1.1.0 (Articles 1–8 + 7a, 9, 10, 11) binds every task; if a task appears
  to require `std::` containers, exceptions, RTTI, or unseeded randomness, the task is wrong.
- Never link `GTest::gmock` (silently voids the test registry → false-positive CTest passes).
- All gameplay timers are integer tick counts (D4); the only `double` accumulator is the
  existing `game_loop` one.
- `state_hash` is field-wise FNV-1a — never hash raw struct bytes (padding, D5).
- Sandbox scene is render-only and digest-excluded (D7); T020 is the proof.
