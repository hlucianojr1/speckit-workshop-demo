# Implementation Plan: Particle VFX Subsystem

**Branch**: `004-particle-vfx-subsystem` | **Date**: 2026-07-06 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/004-particle-vfx-subsystem/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Add a purely-visual particle VFX subsystem (`engine_demo::vfx`) providing a fixed-capacity
`particle_pool`, a tagged-union `emitter_shape` (point/cone/sphere), composable
`force_applicator`s (gravity/wind/turbulence), and an `emitter` that spawns and ticks
particles from a single explicitly-seeded `engine_demo::sim::rng`. The subsystem never
allocates outside construction, never touches `engine_demo::physics::constraint_solver` or
`engine_demo::ecs::world`, and is ticked by the caller at the same fixed cadence as
`engine_demo::sim::game_loop`. Emit requests are strictly all-or-nothing against the pool's
free capacity (FR-006), while a per-particle non-positive sampled lifetime is a normal,
successful skip rather than a request failure (FR-013) — see [research.md](research.md) §7
for the reasoning. Six other design decisions (storage/recycle algorithm, shape and force
representation, determinism strategy, memory layout, and frame-budget integration) round out
research.md and shape the API in [data-model.md](data-model.md) and [contracts/](contracts/).

## Technical Context

**Language/Version**: C++20, compiled with `-fno-exceptions -fno-rtti` (project-wide)
**Primary Dependencies**: EASTL (`eastl::vector`, `eastl::variant`, `eastl::span`),
`engine_demo::allocator`, `engine_demo::sim::rng` (existing types — no new third-party
dependencies)
**Storage**: N/A — in-memory only; one arena-backed allocation per `particle_pool` at
construction, no persistence
**Testing**: GoogleTest via CTest, wired through `tests/engine_demo/CMakeLists.txt`'s
`engine_demo_add_test` (matches existing subsystems)
**Target Platform**: Same as the rest of `engine_demo` — cross-platform core library
(Windows/macOS desktop demonstrated via `apps/sandbox`); the VFX subsystem itself has no
platform-specific code
**Project Type**: Single C++ library subsystem added to the existing engine (Option 1 — no
new top-level project)
**Performance Goals**: 500 simultaneous live particles update in < 2 ms/frame at 60 FPS
(SC-002); zero heap/arena allocation during `try_emit`/`tick` (Article 6, SC-006)
**Constraints**: No exceptions/RTTI (Articles 1–2); EASTL-only, allocator-aware containers
(Articles 3–4); explicit-seed RNG, `double` time accumulators except render-boundary
`float`s (Article 5); every public function has a GTest (Article 7)
**Scale/Scope**: One new subsystem — 3 headers, 3 source files, 4 test files; fixed pool
capacities in the low hundreds to low thousands of particles per pool

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Article | Requirement | How this feature satisfies it |
|---|---|---|
| 1 — No exceptions | No `try`/`catch`/`throw` | `vfx_status` enum (`ok`, `pool_exhausted`, `invalid_argument`) returned from every fallible call (`try_spawn`, `try_emit`, `add_force`) — FR-016. |
| 2 — No RTTI | No `dynamic_cast`/`typeid` | `emitter_shape` and `force_applicator` are `eastl::variant` tagged unions (research.md §2–3); no inheritance/vtables used for type discrimination — FR-002, FR-007. |
| 3 — EASTL-first | EASTL containers only | `eastl::vector<particle, eastl_allocator_ref>`, `eastl::vector<force_applicator, eastl_allocator_ref>`, `eastl::span`, `eastl::variant`. |
| 4 — Allocator-aware | Explicit allocator per container | `particle_pool` and `emitter` both take `allocator&` at construction; every container is constructed with `eastl_allocator_ref{alloc}`; no default-constructed containers. |
| 5 — Determinism | Seeded RNG, `double` accumulators | One `engine_demo::sim::rng` per emitter, seeded from `emitter_config::seed` (`std::uint64_t`, full width) — FR-009; `remaining_lifetime_seconds` is `double`; dense-array iteration order is deterministic (research.md §1, §4) — FR-010. |
| 6 — Real-time | No allocation in inner loops | `particle_pool` reserves its dense array once at construction; `emitter` reserves its force list and scratch spawn-candidate buffer once; `try_emit`/`tick`/`age_and_retire` never allocate — FR-011; SC-002 benchmarks 500 particles under 2 ms. |
| 7 — Test-first | GTest per public function | Every declaration in `contracts/*.md` has explicit happy-path + edge-case test obligations listed (FR-017); tests are authored before their implementation per the task ordering below. |
| 8 — HITL gates | Human review between plan→tasks and each implement task | This plan stops after Phase 1 for human approval before `/speckit.tasks`; `/speckit.implement` will pause between each generated task. |

**Result**: PASS — no violations. Complexity Tracking table is empty (see below).

## Project Structure

### Documentation (this feature)

```text
specs/004-particle-vfx-subsystem/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md         # Phase 1 output (/speckit.plan command)
├── quickstart.md         # Phase 1 output (/speckit.plan command)
├── contracts/            # Phase 1 output (/speckit.plan command)
│   ├── particle_pool.md
│   ├── force_applicator.md
│   └── emitter.md
├── checklists/
│   └── requirements.md
└── tasks.md              # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

Single-project layout (extends the existing `engine_demo` library — no new top-level
project or app is introduced):

```text
include/engine_demo/vfx/
├── particle.h            # vfx_status, particle, spawn_params, emit_result, particle_pool
├── force_applicator.h    # gravity_force, wind_force, turbulence_force, force_applicator, apply_force
└── emitter.h              # point/cone/sphere_shape, emitter_shape, emitter_config, emitter

src/engine_demo/vfx/
├── particle.cpp
├── force_applicator.cpp
└── emitter.cpp

tests/engine_demo/
├── test_particle_pool.cpp
├── test_force_applicator.cpp
├── test_emitter.cpp
└── test_vfx_determinism.cpp
```

Existing files touched (wiring only, no behavioral changes):

- `src/engine_demo/CMakeLists.txt` — add the three new `vfx/*.cpp` sources.
- `tests/engine_demo/CMakeLists.txt` — add four new `engine_demo_add_test(...)` entries.
- `.github/copilot-instructions.md` — plan-reference marker (updated by this command).

**Structure Decision**: Single project (Option 1). The subsystem lives alongside the
existing `ecs/`, `physics/`, and `sim/` subdirectories under `include/engine_demo/` and
`src/engine_demo/`, following the exact same header/source/test split, CMake wiring
(`engine_demo_add_test`), and naming conventions (`snake_case` types, `[[nodiscard]]` on
factories/status-returning functions, `noexcept` on move ops) already used by
`engine_demo::physics::constraint_solver` and `engine_demo::ecs::world`.

## Phase 2: Task Planning Preview

*This is a sizing estimate only — `/speckit.tasks` generates the authoritative `tasks.md`.
Not created by `/speckit.plan`.*

Test-first ordering, grouped by unit, each task sized to stay under ~150 lines of diff:

1. `test_particle_pool.cpp` — happy-path spawn + all-or-nothing exhaustion edge case
   (written first, red).
2. `include/engine_demo/vfx/particle.h` — `vfx_status`, `particle`, `spawn_params`,
   `emit_result`, `particle_pool` declaration.
3. `src/engine_demo/vfx/particle.cpp` — dense-array spawn (all-or-nothing)/age/retire
   (swap-remove) logic; CMake wiring; tests green.
4. `test_force_applicator.cpp` — gravity/wind pass-through + turbulence bounds/determinism
   (red).
5. `include/engine_demo/vfx/force_applicator.h` — `gravity_force`/`wind_force`/
   `turbulence_force`/`force_applicator`/`apply_force` declaration.
6. `src/engine_demo/vfx/force_applicator.cpp` — `apply_force` implementation; CMake wiring;
   tests green.
7. `test_emitter.cpp` (part 1) — shape sampling (cone-angle/sphere-radius bound checks) +
   `try_emit` pool-exhaustion and lifetime-skip edge cases (red).
8. `include/engine_demo/vfx/emitter.h` — shape structs, `emitter_config`, `emitter`
   declaration.
9. `src/engine_demo/vfx/emitter.cpp` — shape sampling + all-or-nothing `try_emit` (with
   lifetime-skip filtering) + `tick`; CMake wiring; part-1 tests green.
10. `test_emitter.cpp` (part 2) — multi-force velocity accumulation over several ticks,
    `dt == 0` no-op edge case, `add_force` capacity-exhaustion edge case; implementation
    adjustments as needed.
11. `test_vfx_determinism.cpp` — same-seed/same-sequence byte-identical replay across two
    independent emitters + pools over 1,000 frames (User Story 3 / SC-003); implementation
    adjustments (if any non-determinism is found) folded back into task 9.
12. Perf smoke test (appended to `test_emitter.cpp` or its own file) asserting 500-particle
    `tick()` wall time stays comfortably under the 2 ms/frame budget (SC-002) with generous
    CI headroom, per the project's existing perf-test conventions.
13. Documentation/wiring cleanup — `.github/copilot-instructions.md` subsystem list,
    `README.md` mention if the project maintains a subsystem table (small diff).

Estimated total: **~12–13 tasks**, each independently buildable/testable and under the
150-line diff cap; tasks 1–3, 4–6, and 7–11 form three independently-completable
test-then-implement slices (User Stories 1, 2, and 3 respectively), consistent with the
spec's prioritized, independently-testable user stories.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations — table intentionally left empty.
