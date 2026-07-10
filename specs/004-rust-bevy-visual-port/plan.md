# Implementation Plan: Rust/Bevy Visual Port of the Orbital Arena Simulation

**Branch**: `part4-rust-transformation` | **Date**: 2026-07-10 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/004-rust-bevy-visual-port/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Port the six behavioral specs under `specs/transform/*.spec.md` (ecs-world, physics-constraint,
game-loop, rng, frame-budget, allocator) into a standalone Rust/Bevy 0.16.x binary crate at
`rust-port/orbital-arena-rs/`, structurally isolated from the CMake build. The crate renders a
2D scene (one immovable anchor body, several orbiting bodies, distance-constraint links drawn as
lines, an arena boundary circle) driven by a deterministic 60 Hz `FixedUpdate` simulation seeded
by a `StdRng`. It supports a windowed run mode (human observation) and a screenshot-capture mode
(automated evidence via `bevy::render::view::screenshot`), and ships headless tests proving
determinism (Article VI), constraint-solver correctness, and zero steady-state allocation
(Article IV). Bevy's own `Entity`/ECS world already delivers the generational-safety guarantee
of `ecs-world.spec.md`, so no custom entity-slot table is built — this is documented as an
intentional "guarantee already delivered by the framework" case in research.md, not a gap.

## Technical Context

**Language/Version**: Rust 1.75+ (stable toolchain; workshop VM has 1.96.1 installed)
**Primary Dependencies**: `bevy` 0.16.x (default features — `render`+`winit`+`bevy_gizmos`, needed
for both windowed and screenshot-capture modes), `rand` 0.9 (`rngs::StdRng`), `clap` 4 (derive,
CLI argument parsing for run-mode/seed/warmup-frames/output-path)
**Storage**: N/A (no persistence; screenshot mode writes one PNG file to a caller-specified path)
**Testing**: `cargo test` — headless integration tests built on `MinimalPlugins` +
`TimeUpdateStrategy::ManualDuration` (determinism, constraint solver, zero-allocation); no GPU/
window required for any test
**Target Platform**: Windows desktop (workshop VM, no hardware GPU — DX12 WARP software adapter,
verified this session); crate itself is not Windows-specific (wgpu backend selection is
automatic per-platform)
**Project Type**: Standalone Rust binary+lib crate (single project), intentionally NOT wired into
the top-level CMake build
**Performance Goals**: 60 Hz fixed-timestep sim, ≤16.67 ms frame budget (Article VII); no hard
FPS requirement on rendering since this is a visualization, not a twitch game
**Constraints**: Zero heap allocation in steady-state `FixedUpdate`/`Update` systems after
warm-up (Article IV); no `unsafe` (Article III); no panics in sim/system code (Article I); `f64`
for all physics state, `f32` only at render boundary (Article VI / FR-008)
**Scale/Scope**: 1 anchor + 3–8 orbiting bodies (per spec Assumptions), 1 arena boundary, N
distance-constraint links (small, single-digit count) — no networking, no save/load, no input

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Constitution: [specs/transform/rust-constitution.md](../transform/rust-constitution.md) (11
articles). Per-article gate for this feature:

| Article | Gate | Status |
|---|---|---|
| I — No panics in sim/system code | `FixedUpdate`/`Update` systems must not `unwrap`/`expect`/index-panic; `#[lints.clippy]` deny table in `Cargo.toml` | PASS (planned: deny table from task 1; screenshot I/O failure path (FR-011) returns `Result`, logged via `error!`, not panicked) |
| II — Errors/results are values | Screenshot write failure, CLI parse failure are `Result`/`Option`, not thrown | PASS (clap already returns `Result`; screenshot failure handled via observer + `AppExit::error`) |
| III — Ownership-based memory safety; no `unsafe` | `#![forbid(unsafe_code)]` at crate root | PASS (no SIMD/GPU-buffer casts needed for this feature) |
| IV — Zero-allocation steady-state frame loop | Body/constraint `Vec`s sized at startup via `with_capacity`; frame-budget ring buffer fixed-size array; `#[global_allocator]` counting wrapper + dedicated test (FR-014) | PASS (design honors this; see data-model.md §Zero-allocation design) |
| V — Capacity exhaustion is graceful degradation | N/A for this feature — body count is fixed at startup from CLI/RNG-driven config, no runtime spawn/despawn under budget pressure | N/A (documented, not a gap) |
| VI — Deterministic simulation | `StdRng::seed_from_u64`, `f64` accumulators, explicit sort by canonical `(min(a,b), max(a,b))` key using `Entity`'s `Ord` impl for constraint iteration order (FR-007, FR-008, FR-012) | PASS |
| VII — Real-time frame budget and fixed timestep | `Time::<Fixed>::from_hz(60.0)`; sim mutation confined to `FixedUpdate`; frame-budget resource records `Update` frame durations in a fixed window (FR-006) | PASS (criterion perf-gate deferred — see research.md Open Question) |
| VIII — Closed sums over dynamic dispatch | No heterogeneous shape/force enums needed for this feature (bodies are homogeneous); no `dyn Trait` in hot paths | N/A (nothing to violate) |
| IX — One plugin per subsystem, headless-testable | `RngPlugin`, `PhysicsPlugin`, `FrameBudgetPlugin`, `VisualsPlugin`, `ScreenshotPlugin` each own their resources/systems; sim plugins addable to `MinimalPlugins` for tests | PASS |
| X — CI is the enforcement layer | `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, `cargo deny check` — documented in quickstart.md; no CI workflow file created by this feature (out of scope per FR-001's CMake-exclusion intent, but noted as follow-up) | PASS (local gate only for this feature; CI wiring is a future task, flagged in research.md) |
| XI — HITL gates | This plan stops for human review before `/speckit.tasks`; every `/speckit.implement` task pauses | PASS (process gate, not code) |

No violations requiring Complexity Tracking justification.

## Project Structure

### Documentation (this feature)

```text
specs/004-rust-bevy-visual-port/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── cli.md           # Run-mode / argument / exit-code contract
│   └── plugins.md        # Per-plugin resource/component/system contract
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
rust-port/orbital-arena-rs/        # standalone Cargo project; NOT referenced by CMakeLists.txt
├── Cargo.toml                     # [lints.clippy] deny table (Article I), bevy/rand/clap deps
├── Cargo.lock
├── deny.toml                      # cargo-deny config (Articles II/VI dependency bans)
├── src/
│   ├── main.rs                    # thin entry point: parse CLI, build App, pick run mode, run()
│   ├── lib.rs                     # re-exports plugins for tests + main.rs
│   ├── config.rs                  # SimConfig resource: seed, RunMode, warmup_frames, output_path
│   ├── rng.rs                     # RngPlugin: DeterministicRng resource wrapping StdRng
│   ├── body.rs                    # Body/Anchor components, spawn_bodies startup system
│   ├── constraint.rs              # PhysicsPlugin: ConstraintLink component, solver FixedUpdate system
│   ├── frame_budget.rs            # FrameBudgetPlugin: fixed-window rolling-average resource
│   ├── visuals.rs                 # VisualsPlugin: camera + body/link/boundary Gizmos drawing
│   ├── screenshot.rs              # ScreenshotPlugin: warm-up counter, triggers Screenshot+save_to_disk
│   └── alloc.rs                   # #[global_allocator] counting allocator (test-only instrumentation)
└── tests/
    ├── determinism.rs             # FR-012: same seed twice -> identical snapshot; different seed -> differs
    ├── constraint_solver.rs       # FR-013: separation converges toward rest_length; anchor never moves
    └── zero_alloc.rs              # FR-014: N post-warmup FixedUpdate frames -> zero allocation delta
```

**Structure Decision**: Single standalone Cargo project under `rust-port/orbital-arena-rs/`
(Rust "binary + lib" split within one crate — `src/lib.rs` exposes plugins so `tests/*.rs`
integration tests can build headless `App`s without spawning a window, while `src/main.rs`
stays a thin CLI-parsing + `App::run()` entry point). This directory is a sibling of `src/`,
`include/`, and `tests/` at the repo root, is not listed in the top-level `CMakeLists.txt` or
`cmake/`, and is validated exclusively via `cargo` commands (documented in quickstart.md), per
FR-001 and the spec's Assumptions section.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations — table intentionally empty.
