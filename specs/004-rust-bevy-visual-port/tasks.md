---

description: "Task list for Rust/Bevy Visual Port of the Orbital Arena Simulation"
---

# Tasks: Rust/Bevy Visual Port of the Orbital Arena Simulation

**Input**: Design documents from `specs/004-rust-bevy-visual-port/`
**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/cli.md](contracts/cli.md), [contracts/plugins.md](contracts/plugins.md), [quickstart.md](quickstart.md)

**Tests**: Tests ARE included — FR-012/FR-013/FR-014 explicitly require automated headless tests
(determinism, constraint solver, zero-allocation), and User Story 4 exists specifically to cover
them. Per-plugin tests from [contracts/plugins.md](contracts/plugins.md) are also included.

**Organization**: Tasks are grouped by user story (US1–US4, priorities P1/P1/P2/P3 per spec.md)
to enable independent implementation and testing. All paths are relative to
`rust-port/orbital-arena-rs/` unless stated otherwise.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

Single standalone Cargo crate at `rust-port/orbital-arena-rs/` (per plan.md's Project Structure),
NOT wired into the top-level CMake build:

```text
rust-port/orbital-arena-rs/
├── Cargo.toml, Cargo.lock, deny.toml
├── src/{main,lib,config,rng,body,constraint,frame_budget,visuals,screenshot,alloc}.rs
└── tests/{determinism,constraint_solver,zero_alloc}.rs
```

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Scaffold the standalone Cargo crate, independent of the CMake build

- [ ] T001 Create `rust-port/orbital-arena-rs/` directory with `cargo init --lib` bin+lib split (`src/main.rs` + `src/lib.rs`), confirm it is a sibling of `src/`/`include/`/`tests/` at repo root
- [ ] T002 Write `rust-port/orbital-arena-rs/Cargo.toml` with `bevy` 0.16.x (default features), `rand` 0.9, `clap` 4 (derive feature), and a `[lints.clippy]` deny table (Article I / plan.md Constitution Check row I)
- [ ] T003 [P] Add `#![forbid(unsafe_code)]` to `rust-port/orbital-arena-rs/src/lib.rs` (Article III)
- [ ] T004 [P] Write `rust-port/orbital-arena-rs/deny.toml` banning non-`StdRng` RNG engines and `anyhow` in sim-path dependencies (Article II/VI, quickstart.md Lint/format/dependency gates)
- [ ] T005 [P] Verify FR-001 CMake-exclusion: confirm no target in `CMakeLists.txt` or `cmake/*.cmake` references `rust-port/` (quickstart.md's `Select-String` check); no file changes expected, just validation
- [ ] T006 [P] Add `.gitignore` entry for `rust-port/orbital-arena-rs/target/` if not already covered by repo root `.gitignore`

**Checkpoint**: Crate compiles empty (`cargo build` with a stub `main`/`lib`) before any subsystem code is added.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core resources, components, and CLI plumbing every user story's plugins depend on

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T007 Create `SimConfig` resource and `RunMode` enum in `rust-port/orbital-arena-rs/src/config.rs` (data-model.md §Resources `SimConfig`: `seed: u64` default 42, `run_mode: RunMode` closed enum `Windowed` / `Screenshot { warmup_frames: u32, output_path: PathBuf }`)
- [ ] T008 Implement CLI argument parsing with `clap` derive in `rust-port/orbital-arena-rs/src/main.rs` per [contracts/cli.md](contracts/cli.md): default windowed mode with `--seed`, `screenshot --warmup-frames <u32> --out <path> [--seed <u64>]` subcommand, mapping into `SimConfig`
- [ ] T009 Define `Body` component (`position`, `velocity`: `DVec2`; `inverse_mass: f64`) and `Anchor` zero-sized marker component in `rust-port/orbital-arena-rs/src/body.rs` (data-model.md §Components)
- [ ] T010 Define `ConstraintLink` component (`a`, `b`: `Entity`; `rest_length: f64`) in `rust-port/orbital-arena-rs/src/constraint.rs` (data-model.md §Components `ConstraintLink`)
- [ ] T011 [P] Implement `#[global_allocator]` `CountingAllocator` wrapper in `rust-port/orbital-arena-rs/src/alloc.rs` (research.md R4, data-model.md §Zero-allocation design) — test-only instrumentation, feature-gated or always-present per research.md's chosen approach
- [ ] T012 Wire `src/lib.rs` to re-export `config`, `rng` (added in US2/T014), `body`, `constraint`, `frame_budget` (added in US4/T028), `visuals` (added in US1/T018), `screenshot` (added in US2/T022), and `alloc` modules so `tests/*.rs` and `main.rs` can both consume them

**Checkpoint**: Foundation ready — `SimConfig`, CLI parsing, and core components exist; user story plugin work can now begin.

---

## Phase 3: User Story 1 - Watch the Orbital Arena Simulate Live (Priority: P1) 🎯 MVP

**Goal**: A windowed run renders one immovable anchor, orbiting bodies, constraint links as
lines, and an arena boundary circle, animating continuously with no input.

**Independent Test**: `cargo run` with no arguments opens a window showing the anchor, orbiting
bodies, links, and boundary; scene animates without exiting or freezing (spec.md US1 Independent
Test).

### Implementation for User Story 1

- [ ] T013 [US1] Implement `RngPlugin` in `rust-port/orbital-arena-rs/src/rng.rs`: `DeterministicRng` resource wrapping `StdRng::seed_from_u64(SimConfig.seed)`, inserted in `Startup` before any consuming system (contracts/plugins.md `RngPlugin`, data-model.md §Resources `DeterministicRng`)
- [ ] T014 [US1] Implement `PhysicsPlugin`'s `Startup` spawn system in `rust-port/orbital-arena-rs/src/constraint.rs` (or `body.rs`): spawn 1 anchor (`inverse_mass = 0.0`) + 3–8 orbiting bodies (spec.md Assumptions) with RNG-jittered initial placement, plus `ConstraintLink` entities connecting them
- [ ] T015 [US1] Implement the sorted-order distance-constraint solver as a `FixedUpdate` system in `rust-port/orbital-arena-rs/src/constraint.rs`: explicit sort by canonical `(min(a,b), max(a,b))` key via `Entity`'s `Ord` (FR-007/FR-012 determinism), pre-sized `Vec::with_capacity(link_count)` sort buffer populated at `Startup` and `clear()`+repopulated each tick (data-model.md §Zero-allocation design, research.md R6)
- [ ] T016 [US1] Configure fixed-timestep loop via `Time::<Fixed>::from_hz(60.0)` in `rust-port/orbital-arena-rs/src/main.rs` (FR-006; data-model.md §Fixed-step loop — no custom accumulator, Bevy's own `FixedUpdate` used as-is)
- [ ] T017 [US1] Implement `VisualsPlugin` in `rust-port/orbital-arena-rs/src/visuals.rs`: `Startup` system spawning `Camera2d`; `Update` system drawing bodies as circles, links as lines, and the arena boundary as a circle via `Gizmos` (FR-004; contracts/plugins.md `VisualsPlugin`)
- [ ] T018 [US1] Assemble `main.rs`: build `App` with `DefaultPlugins` + `RngPlugin` + `PhysicsPlugin` + `FrameBudgetPlugin` (stub until US4) + `VisualsPlugin`, parse CLI via T008, run indefinitely for windowed mode (FR-009; contracts/plugins.md Composition)
- [ ] T019 [US1] [P] Add `Cargo.toml` `[lints.clippy]`-clean pass and manual smoke run (`cargo run`) confirming: anchor never moves, orbiting bodies + links + boundary all render, no crash over an extended run (spec.md US1 Acceptance Scenarios 1–5)

**Checkpoint**: User Story 1 fully functional and independently testable — `cargo run` shows a live, animating orbital scene.

---

## Phase 4: User Story 2 - Capture Repeatable Evidence via Screenshot Mode (Priority: P1)

**Goal**: A screenshot-mode invocation runs warm-up frames off-screen/on-screen and writes a
single image file to a caller-specified path, then exits successfully.

**Independent Test**: Launch with the screenshot-mode flag and warm-up frame count; verify an
image file is written after warm-up completes and the process exits 0 (spec.md US2 Independent
Test).

### Implementation for User Story 2

- [ ] T020 [P] [US2] Define `ScreenshotState` resource (`fixed_ticks_elapsed: u32`, `triggered: bool`) in `rust-port/orbital-arena-rs/src/screenshot.rs` (data-model.md §Resources `ScreenshotState`), inserted only when `SimConfig.run_mode` is `Screenshot`
- [ ] T021 [US2] Implement `ScreenshotPlugin`'s `FixedUpdate` system incrementing `fixed_ticks_elapsed` in `rust-port/orbital-arena-rs/src/screenshot.rs` (contracts/plugins.md `ScreenshotPlugin`)
- [ ] T022 [US2] Implement `ScreenshotPlugin`'s `Update` system: once `fixed_ticks_elapsed >= warmup_frames` and `!triggered`, spawn `bevy::render::view::screenshot::Screenshot` + `save_to_disk` observer targeting `SimConfig`'s `output_path`, set `triggered = true` (FR-010, FR-011; edge case: `warmup_frames = 0` captures at first post-setup render)
- [ ] T023 [US2] Handle screenshot write failure as a non-panicking `Result`-based path: log a clear error via `error!` and exit non-zero on failure, per Article I/II and contracts/cli.md's exit-code table (no file written before warm-up elapses; FR-011)
- [ ] T024 [US2] Wire `screenshot` subcommand into `main.rs`: when `RunMode::Screenshot`, add `ScreenshotPlugin` to the `App` built in T018 and ensure the process exits 0 only after the file is confirmed written (contracts/cli.md Exit codes; spec.md US2 Acceptance Scenario 4)
- [ ] T025 [US2] [P] Manual verification run: `cargo run -- screenshot --warmup-frames 300 --out ../../docs/screenshots/part4-rust-arena.png` produces a valid, non-empty, readable image (quickstart.md; spec.md US2 Acceptance Scenarios 1–3, SC-002)

**Checkpoint**: User Stories 1 AND 2 both work independently — live window and automated screenshot capture.

---

## Phase 5: User Story 3 - Trust the Simulation Is Deterministic (Priority: P2)

**Goal**: Two runs from the same seed produce identical recorded state; a different seed produces
differing state; default seed is 42 when unspecified.

**Independent Test**: Run the automated determinism check twice at the same seed and confirm
identical state summaries; rerun at a different seed and confirm the summary differs (spec.md US3
Independent Test).

### Tests for User Story 3

- [ ] T026 [P] [US3] Write `rust-port/orbital-arena-rs/tests/determinism.rs`: build two headless `MinimalPlugins` `App`s with `RngPlugin` + `PhysicsPlugin` at seed 42, step both the same fixed-tick count, assert identical `SimulationSnapshot`; repeat with a different seed and assert the snapshot differs (FR-012; spec.md US3 Acceptance Scenarios 1–2)
- [ ] T027 [P] [US3] Add a case in `tests/determinism.rs` (or a dedicated `#[test]`) asserting that omitting `--seed` yields `SimConfig.seed == 42` (FR-007; spec.md US3 Acceptance Scenario 3)

### Implementation for User Story 3

- [ ] T028 [US3] Implement `capture_snapshot(app: &App) -> SimulationSnapshot` helper in `rust-port/orbital-arena-rs/src/lib.rs`: collect `(position.x, position.y, velocity.x, velocity.y)` per body ordered by a stable spawn-order index, not `Entity` (data-model.md §Simulation State Snapshot)
- [ ] T029 [US3] Add a stable spawn-order index component/field to bodies spawned in T014 so `capture_snapshot` can order deterministically independent of `Entity` allocation details (data-model.md's robustness note)

**Checkpoint**: Determinism is automatically verifiable via `cargo test` without any visual inspection.

---

## Phase 6: User Story 4 - Trust the Constraint Solver and Frame Budget Behave as Specified (Priority: P3)

**Goal**: Automated tests confirm the constraint solver converges bodies toward rest length,
never moves the anchor, and that the simulation allocates zero memory post-warm-up.

**Independent Test**: Run the constraint-solver test cases and the allocation test independently
of the windowed app, using a headless test harness (spec.md US4 Independent Test).

### Tests for User Story 4

- [ ] T030 [P] [US4] Write `rust-port/orbital-arena-rs/tests/constraint_solver.rs`: two bodies connected by a distance constraint, displaced beyond rest length; assert separation moves measurably closer to rest length after solver steps; assert the anchor's position never changes across any number of connected constraints (FR-013; spec.md US4 Acceptance Scenarios 1–2)
- [ ] T031 [P] [US4] Write `rust-port/orbital-arena-rs/tests/zero_alloc.rs`: run N post-warm-up `FixedUpdate` frames headless, read `CountingAllocator` deltas before/after, assert zero allocation delta (FR-014; spec.md US4 Acceptance Scenario 3)
- [ ] T032 [P] [US4] Write a `FrameBudgetPlugin`-focused test (in `tests/` or as a module `#[cfg(test)]` block) recording N synthetic frame-duration samples and asserting the expected rolling average, including the warm-up-counted-once rule (contracts/plugins.md `FrameBudgetPlugin` Test; frame-budget.spec.md §4)

### Implementation for User Story 4

- [ ] T033 [US4] Implement `FrameBudget` resource (`samples: [f64; 64]`, `cursor: usize`, `count: usize`) in `rust-port/orbital-arena-rs/src/frame_budget.rs` (data-model.md §Resources `FrameBudget`) — fixed array, never heap-allocated
- [ ] T034 [US4] Implement `record_sample`/`rolling_average` functions and the `FrameBudgetPlugin`'s `Update` system recording each frame's duration in `rust-port/orbital-arena-rs/src/frame_budget.rs`, matching frame-budget.spec.md §3–4 exactly (contracts/plugins.md `FrameBudgetPlugin`)
- [ ] T035 [US4] Wire `FrameBudgetPlugin` into the `App` composition in `main.rs` (replacing the T018 stub) and into headless test `App`s used in T030–T032 where relevant (contracts/plugins.md Composition: headless tests use `MinimalPlugins` + `RngPlugin` + `PhysicsPlugin` [+ `FrameBudgetPlugin` where relevant])

**Checkpoint**: All four user stories independently functional and covered by automated tests where required.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Gates and documentation that span all user stories (Article X enforcement)

- [ ] T036 [P] Run `cargo fmt --check` across `rust-port/orbital-arena-rs/` and fix formatting (quickstart.md Lint/format/dependency gates)
- [ ] T037 [P] Run `cargo clippy --all-targets --all-features -- -D warnings` and resolve all lints (Article I / X)
- [ ] T038 [P] Run `cargo deny check` against `deny.toml` from T004 and resolve any banned-dependency findings (Article II/VI)
- [ ] T039 Run `cargo test` end-to-end (all of tests/determinism.rs, constraint_solver.rs, zero_alloc.rs) and confirm 10 consecutive clean runs with no flakes (SC-004)
- [ ] T040 Execute the full [quickstart.md](quickstart.md) walkthrough (build, windowed run, screenshot run, test, lint gates, FR-001 CMake-exclusion check) end-to-end as a final acceptance pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational; no dependency on other stories
- **User Story 2 (Phase 4)**: Depends on Foundational; reuses US1's `PhysicsPlugin`/`main.rs` App (T018) as an integration point but is independently testable once its own screenshot code exists
- **User Story 3 (Phase 5)**: Depends on Foundational + US1's `RngPlugin`/`PhysicsPlugin` (T013–T014) existing to snapshot against; does not depend on US2
- **User Story 4 (Phase 6)**: Depends on Foundational + US1's `PhysicsPlugin`/solver (T014–T015) existing to test against; does not depend on US2/US3
- **Polish (Phase 7)**: Depends on all four user stories being complete

### User Story Dependencies (ordering concern)

Unlike a typical CRUD feature, US2–US4 are not fully decoupled from US1: all three consume the
`Body`/`ConstraintLink`/`RngPlugin` state that US1 establishes (T013–T015). Recommended order is
strictly **US1 → US2 → US3 → US4** even though US1/US2 share priority P1, because:

- US2 (`ScreenshotPlugin`) needs a rendered scene (US1's `VisualsPlugin`) to have something to
  screenshot.
- US3's determinism test (T026) snapshots the body/constraint state US1's `PhysicsPlugin` spawns.
- US4's constraint-solver and zero-alloc tests (T030–T031) exercise the exact solver system US1
  implements in T015.

Teams could parallelize US3 and US4 against each other once US1 (Phase 3) is done, since both
only need `RngPlugin`+`PhysicsPlugin`, not US2's `ScreenshotPlugin`.

### Within Each User Story

- RNG/component/resource definitions before the systems that consume them
- Solver/spawn systems before the plugin composition step that wires them into `main.rs`
- Tests (US3, US4) can be written in parallel with each other but read state produced by US1

### Parallel Opportunities

- Setup tasks T003–T006 can run in parallel (different files)
- Foundational task T011 (`alloc.rs`) can run in parallel with T007–T010 (different files)
- US3's T026/T027 (tests) can run in parallel with each other; both depend on US1 being done
- US4's T030/T031/T032 (tests) can all run in parallel with each other; all depend on US1's T015
- US3 and US4 phases can be worked in parallel by different contributors once US1 is complete
- Polish tasks T036–T038 can run in parallel with each other

---

## Parallel Example: User Story 4

```bash
# Launch all three US4 tests together (all depend on US1's solver, not on each other):
Task: "Write tests/constraint_solver.rs asserting convergence + anchor immobility"
Task: "Write tests/zero_alloc.rs asserting zero allocation delta post-warmup"
Task: "Write FrameBudgetPlugin rolling-average test"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: `cargo run` shows a live, animating orbital scene (spec.md US1
   Acceptance Scenarios); this alone satisfies SC-001 and SC-005 evidence gathering
5. Deploy/demo if ready — this is the minimum "visible, watchable proof" the spec calls for

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → validate live window → screenshot-ready evidence baseline (MVP!)
3. Add User Story 2 → automated screenshot capture → SC-002 satisfied
4. Add User Story 3 → determinism proof → SC-003 satisfied
5. Add User Story 4 → solver + zero-alloc proof → SC-004 satisfied
6. Polish phase → Article X gates green (fmt/clippy/deny), full quickstart re-run

### Risks / Flags Carried Forward from plan.md and research.md

- **Rendering feasibility on the workshop VM**: plan.md notes DX12 WARP was confirmed working
  this session for `DefaultPlugins`; if a different VM lacks WARP, US1/US2 (both require
  rendering) are blocked — no headless fallback is designed for `VisualsPlugin`/`ScreenshotPlugin`
  per contracts/plugins.md.
- **`criterion` perf-gate deferred**: plan.md's Article VII row flags this as deferred, not
  covered by any task above — Article VII's frame-budget *tracking* is implemented (T033–T034)
  but no automated perf-regression gate exists yet.
- **No CI workflow file**: Article X's gates (T036–T038) are local-only per plan.md; wiring them
  into CI is explicitly out of scope for this feature and not tracked as a task here.
- **Snapshot ordering robustness (T029)**: data-model.md explicitly calls out that ordering by
  `Entity` would be fragile; T029 must land before/alongside T028, not after, or T026's
  determinism test could pass for the wrong reason (comparing `Entity`-ordered snapshots that
  happen to match by coincidence of spawn order).
