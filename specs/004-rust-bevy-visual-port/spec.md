# Feature Specification: Rust/Bevy Visual Port of the Orbital Arena Simulation

**Feature Branch**: `part4-rust-transformation`
**Created**: 2026-07-10
**Status**: Draft
**Input**: User description: "Rust/Bevy visual port of the orbital arena simulation — a standalone Rust/Bevy binary crate at rust-port/orbital-arena-rs/ (NOT under src/ or include/, excluded from the CMake build) that is a language-agnostic-spec-driven port of the C++ engine_demo subsystems, rendered so it can be screenshotted."

**Constitution**: Bound by the Rust/Bevy port constitution (Articles I–XI,
[specs/transform/rust-constitution.md](../transform/rust-constitution.md)). This spec covers
the **behavior**, not the C++ syntax, of the six subsystems already captured as
language-agnostic specs: [ecs-world.spec.md](../transform/ecs-world.spec.md),
[physics-constraint.spec.md](../transform/physics-constraint.spec.md),
[game-loop.spec.md](../transform/game-loop.spec.md), [rng.spec.md](../transform/rng.spec.md),
[frame-budget.spec.md](../transform/frame-budget.spec.md), and
[allocator.spec.md](../transform/allocator.spec.md). Those five documents plus the
constitution are the source of truth for every requirement below; the original C++ headers
under `include/engine_demo/` are not authoritative for this feature.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Watch the Orbital Arena Simulate Live (Priority: P1)

A workshop attendee or reviewer launches the port in its normal windowed mode and watches a
central heavy body hold several orbiting bodies in place via visible springy links, inside a
clearly marked arena boundary, running smoothly with no input required.

**Why this priority**: This is the core deliverable — a visible, watchable proof that the
behavioral port is faithful. Without a running visual scene there is nothing to evaluate or
screenshot.

**Independent Test**: Launch the binary with no arguments; a window opens showing one
central body, multiple orbiting bodies connected to it (directly or via each other) by
visible links, and a boundary circle; the scene animates continuously without the process
exiting or freezing.

**Acceptance Scenarios**:

1. **Given** the binary is launched with no arguments, **When** the window opens, **Then** one visually distinct central (anchor) body is rendered at the arena center and does not move for the lifetime of the run.
2. **Given** the simulation is running, **When** any frame renders, **Then** every orbiting body is drawn as a circle and every distance constraint connecting it to another body is drawn as a line between the two body centers.
3. **Given** the simulation is running, **When** any frame renders, **Then** a boundary circle marking the arena's outer edge is visible and does not move.
4. **Given** the window is open, **When** the user provides no input (no mouse/keyboard interaction with the simulation), **Then** the bodies continue to orbit and the scene keeps animating — the simulation never waits on or requires player input to proceed.
5. **Given** the simulation has been running for an extended period, **When** the user observes the window, **Then** the process has not crashed, frozen, or visibly stuttered.

---

### User Story 2 - Capture Repeatable Evidence via Screenshot Mode (Priority: P1)

An automated test harness (or a workshop facilitator scripting a demo) launches the port in
a screenshot-capture mode, lets it run a fixed number of warm-up frames off-screen or
on-screen, and receives a single image file on disk depicting the arena mid-simulation,
without needing a human to manually take the screenshot.

**Why this priority**: Automated evidence capture is the stated purpose of the feature — the
visual scene only counts as a deliverable if it can be captured unattended, repeatably, as
part of a test run.

**Independent Test**: Launch the binary with the screenshot-mode flag/argument and a warm-up
frame count; verify the process runs, writes an image file to the specified path after the
warm-up frames complete, and exits (or continues) without manual interaction.

**Acceptance Scenarios**:

1. **Given** the binary is launched in screenshot-capture mode with a warm-up frame count and an output path, **When** that many fixed-timestep frames have elapsed, **Then** an image file is written to the output path depicting the current arena scene (central body, orbiting bodies, links, boundary).
2. **Given** screenshot-capture mode is requested, **When** the warm-up frame count has not yet elapsed, **Then** no image file is written.
3. **Given** the image file has been written, **When** the harness inspects it, **Then** the file is a valid, non-empty image readable by standard image tooling.
4. **Given** screenshot-capture mode completes successfully, **When** the process finishes its run, **Then** it exits with a success status (so it can be used in an automated test pipeline).

---

### User Story 3 - Trust the Simulation Is Deterministic (Priority: P2)

A reviewer runs the simulation twice from the same seed (default 42, or an explicitly
provided seed) and confirms — via an automated check, not visual inspection — that both runs
produce identical simulation state at the same tick, proving the port is a faithful,
reproducible transformation rather than a one-off visual demo.

**Why this priority**: Determinism is a constitutional guarantee (Article VI) carried over
from the C++ engine; without an automated check, a visually plausible but non-deterministic
port would silently violate the spec it claims to implement.

**Independent Test**: Run the same automated determinism check twice at the same seed and
confirm the two recorded state summaries are identical; run it once more at a different seed
and confirm the summary differs from the seed-42 baseline.

**Acceptance Scenarios**:

1. **Given** a fixed seed and a fixed tick count, **When** the simulation is run twice from a cold start, **Then** the recorded state (body positions/velocities at that tick) is identical between the two runs.
2. **Given** a different seed than the baseline, **When** the simulation runs the same tick count, **Then** the recorded state differs from the seed-42 baseline (confirming the seed actually influences initial placement).
3. **Given** no seed is explicitly provided, **When** the simulation starts, **Then** it uses the default seed (42).

---

### User Story 4 - Trust the Constraint Solver and Frame Budget Behave as Specified (Priority: P3)

A reviewer runs the automated test suite and confirms the distance-constraint solver
behaves as described in [physics-constraint.spec.md](../transform/physics-constraint.spec.md)
(links pull bodies toward the target distance, the anchor body never moves) and that the
simulation allocates no memory once past its warm-up frames, per the constitution's
zero-allocation guarantee (Article IV).

**Why this priority**: These are the two behavioral guarantees most likely to silently
regress during implementation; automated coverage is required before this feature can be
considered done, but the scene is already watchable/screenshotable without it (P1/P2 stand
alone).

**Independent Test**: Run the constraint-solver test cases and the allocation test
independently of the windowed app, using a headless test harness.

**Acceptance Scenarios**:

1. **Given** two bodies connected by a distance constraint and displaced beyond its rest length, **When** the solver runs one or more fixed-timestep steps, **Then** their separation moves measurably closer to the constraint's target distance each step.
2. **Given** the central anchor body is connected to any number of constraints, **When** the solver runs, **Then** the anchor body's position never changes.
3. **Given** the simulation has completed its warm-up period, **When** a fixed number of additional frames are stepped in a headless test, **Then** the measured memory allocation delta across those frames is zero.

---

### Edge Cases

- What happens if the requested warm-up frame count for screenshot mode is zero? The scene should still be captured at its initial (post-setup, pre-first-fixed-step or first-fixed-step) state rather than erroring.
- What happens if the output path for the screenshot cannot be written (e.g., invalid/unwritable directory)? The process must report a clear failure rather than silently producing no file or crashing with an unhandled panic (per constitutional Article I).
- What happens if a body's orbit decays enough that it would visually overlap the anchor or leave the arena boundary? The simulation continues running without crashing; visual overlap is an accepted outcome of the physics, not an error condition.
- What happens when the simulation is run with an explicit seed of 0? It is treated as a valid, ordinary seed like any other (no special-casing).
- What happens if two determinism runs are executed on different machines/OSes? Only same-build, same-platform reproducibility is guaranteed (per Article VI); cross-platform bitwise equality is explicitly out of scope.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The port MUST exist as a standalone Rust binary crate located at `rust-port/orbital-arena-rs/`, structurally independent of `src/` and `include/`, and MUST NOT be built or referenced by the CMake build (`CMakeLists.txt` / `cmake/`).
- **FR-002**: The port's simulation behavior (entity/component model, constraint solving, fixed-timestep game loop, RNG usage, frame-budget tracking, and memory-allocation discipline) MUST match the behavior described in the six language-agnostic specs under `specs/transform/*.spec.md`, not the C++ implementation's syntax or API shapes.
- **FR-003**: Every design and implementation decision for this feature MUST be consistent with the eleven articles of `specs/transform/rust-constitution.md`; where a requirement in this spec and the constitution appear to conflict, the constitution governs.
- **FR-004**: The simulation MUST render a 2D scene containing exactly one central heavy body that never moves (the anchor), one or more orbiting bodies, a visible line for every active distance constraint linking two bodies, and one visible circle marking the arena's outer boundary.
- **FR-005**: The simulation MUST require no player input of any kind to run, animate, or complete its behavior — it is a passive, deterministic visualization, not an interactive game.
- **FR-006**: The simulation's gameplay/physics state MUST advance on a fixed-timestep update running at 60 Hz, independent of rendering frame rate.
- **FR-007**: Initial placement jitter for orbiting bodies MUST be derived from a seeded random number generator; the seed MUST be configurable at launch and MUST default to 42 when not specified.
- **FR-008**: All simulation physics state (positions, velocities, accumulated time) MUST be tracked in double precision (`f64`); single precision is permitted only at the point of rendering.
- **FR-009**: The binary MUST support a normal windowed run mode, intended for a human to watch, with no time limit or exit condition tied to frame count.
- **FR-010**: The binary MUST support a screenshot-capture mode, selected via a launch argument, that runs the simulation for a caller-specified number of warm-up frames and then writes a single image file to a caller-specified path depicting the current scene.
- **FR-011**: In screenshot-capture mode, the port MUST NOT write an image file before the specified warm-up frame count has elapsed, and MUST report a clear, non-crashing failure if the image cannot be written to the specified path.
- **FR-012**: The port MUST include automated, windowless (headless) tests that run the simulation twice from the same seed and fixed tick count and assert the resulting state is identical between runs.
- **FR-013**: The port MUST include automated, windowless tests that verify the distance-constraint solver moves connected bodies toward their target separation and never moves the anchor body.
- **FR-014**: The port MUST include an automated test that measures memory-allocation activity across a run of fixed-timestep frames after warm-up and asserts zero allocation during that measured window.
- **FR-015**: Determinism guarantees (FR-012) apply to same-build, same-platform reproducibility only; cross-platform or cross-build bitwise equality is explicitly out of scope for this feature.

### Key Entities

- **Body**: A simulated object with a position, velocity, and mass in the arena; distinguishes the single immovable anchor (central heavy body) from orbiting bodies.
- **Constraint Link**: A distance constraint connecting exactly two bodies, with a target (rest) distance; rendered as a line between the two connected body positions.
- **Arena**: The bounded 2D space the simulation occupies, visualized as a boundary circle; not itself a physics participant.
- **Simulation Configuration**: The set of launch-time parameters — RNG seed (default 42), run mode (windowed vs. screenshot-capture), warm-up frame count and output path (screenshot mode only).
- **Simulation State Snapshot**: The recorded positions/velocities of all bodies at a given tick, used by the determinism tests to compare two runs.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A reviewer launching the port in windowed mode can visually identify the anchor body, all orbiting bodies, their constraint links, and the arena boundary within 5 seconds of the window opening, with no prior explanation needed.
- **SC-002**: An automated test harness can obtain a valid screenshot image of the running simulation from a single command invocation, with zero manual steps, 100% of the time across repeated runs.
- **SC-003**: Two determinism runs at the same seed produce identical recorded state 100% of the time; runs at differing seeds produce differing recorded state 100% of the time.
- **SC-004**: The constraint-solver and zero-allocation automated tests pass consistently (no flaky failures) across at least 10 consecutive runs.
- **SC-005**: The simulation sustains continuous animation (no visible freeze or crash) for a run of at least 5 minutes in windowed mode.

## Assumptions

- The six existing documents under `specs/transform/*.spec.md` and `specs/transform/rust-constitution.md` are treated as complete and correct behavioral inputs for this feature; this spec does not re-derive or re-validate them.
- "Several orbiting bodies" without a further-specified exact count is left to planning/implementation as a reasonable small number (e.g., 3–8) sufficient to make the constraint links visually legible in a screenshot; the exact count is not a user-facing behavioral requirement.
- The screenshot image format (e.g., PNG) is an implementation detail left to planning, so long as it produces a standard, widely-readable image file.
- "Excluded from the CMake build" means no CMake target references `rust-port/`; it does not preclude a separate CI job or script invoking `cargo` directly for this crate.
- The feature branch already in use (`part4-rust-transformation`) is reused for this spec's directory per the user's explicit instruction; no new branch or worktree is created by this command.
