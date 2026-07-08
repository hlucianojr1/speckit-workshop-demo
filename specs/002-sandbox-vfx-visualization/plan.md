# Implementation Plan: Sandbox VFX Visualization

**Branch**: `002-sandbox-vfx-visualization` | **Date**: 2026-07-05 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/002-sandbox-vfx-visualization/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/plan-template.md` for the execution workflow.

## Summary

Wire Feature 001's `engine_demo::vfx` subsystem into `apps/sandbox`, closing the gap left by
task T022. Right-click bursts and world-bound collision sparks become visible, purely as an
additional, render-only layer: `state_digest()` and the headless golden trace stay
byte-identical. The design requires exactly one small library addition
(`emitter::set_shape`, [research.md](research.md) §1) plus sandbox-only changes (a shared
`particle_pool` + `emitter` owned by `scene`, a seeded-but-isolated VFX rng stream, and a new
raylib-free `scene`-level test target). See [research.md](research.md) for the six design
decisions, [data-model.md](data-model.md) for the concrete field/constant list, and
[contracts/](contracts/) for the two changed/added API surfaces.

## Technical Context

**Language/Version**: C++20, compiled with `-fno-exceptions -fno-rtti` (project-wide)
**Primary Dependencies**: `engine_demo::vfx` (Feature 001 — `particle_pool`, `emitter`,
`sphere_shape`), EASTL, `engine_demo::allocator`, raylib (sandbox renderer only, unchanged
dependency); no new third-party dependencies
**Storage**: N/A — in-memory only; VFX pool/scratch storage is arena-backed, allocated once
at `scene` construction (and once more on explicit reseed/scene-switch, mirroring the
existing `reset_solver()` idiom)
**Testing**: GoogleTest via CTest — one new test case group in the existing
`tests/engine_demo/test_emitter.cpp`, and one new raylib-free test target
`tests/engine_demo/test_scene_vfx.cpp` compiling `apps/sandbox/scene.cpp` directly (same
pattern as the existing `test_sandbox_render` target)
**Target Platform**: Same as the rest of `engine_demo` — the library addition is
cross-platform; the sandbox integration targets the existing `apps/sandbox` desktop app
(Windows/macOS, demonstrated via this workspace's Windows/VS2022/vcpkg build)
**Project Type**: Single C++ library + existing desktop app (Option 1 — no new top-level
project); this feature only extends the existing `engine_demo` library and `ea-sandbox` app
**Performance Goals**: Sandbox holds its fixed 60 FPS step budget (≤16.67 ms) with the VFX
pool at full live capacity (2048) under sustained bursts/sparks (SC-003); zero heap/arena
allocation per burst/spark after construction (Article 6)
**Constraints**: `state_digest()`/golden trace byte-identical with or without VFX activity
(SC-001, Article 5); VFX rng stream isolated from the scene's existing `m_rng` draw order
(FR-005); no new emitter shapes or force applicator types beyond what Feature 001 shipped
(spec scope boundary); every new public function has a GTest (Article 7)
**Scale/Scope**: One library method addition (`emitter::set_shape`); one new `scene` member
pair (`particle_pool` + `emitter`) with ~6 new public/private methods; renderer + HUD
additions in `app.cpp`; no new headers, no new top-level subsystem

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Article | Requirement | How this feature satisfies it |
|---|---|---|
| 1 — No exceptions | No `try`/`catch`/`throw` | `spawn_vfx_burst`/`spawn_vfx_spark` call the existing `vfx_status`-returning `try_emit`; the status is intentionally not surfaced further (cosmetic, best-effort) but no exception path is introduced anywhere. |
| 2 — No RTTI | No `dynamic_cast`/`typeid` | Reuses Feature 001's `eastl::variant`-based `emitter_shape` (`sphere_shape`) unchanged; no new type-discrimination mechanism is introduced. |
| 3 — EASTL-first | EASTL containers only | No new containers are introduced by this feature beyond the existing `particle_pool`/`emitter` internals (already EASTL, from Feature 001). |
| 4 — Allocator-aware | Explicit allocator per container | `m_vfx_pool`/`m_vfx_emitter` are constructed from `scene`'s existing `m_alloc`, exactly like every other `scene` member (`m_solver`, `m_budget`, `m_body_ids`, ...). |
| 5 — Determinism | Seeded RNG, `double` accumulators, unchanged draw order | `m_vfx_emitter`'s seed is `m_seed ^ kVfxSeedSalt` — folded from the scene seed directly, never drawn from `m_rng` (research.md §3) — so the scene's existing rng draw sequence, and therefore `state_digest()`, is provably unaffected by VFX activity. `particle::remaining_lifetime_seconds` (Feature 001) is already `double`. |
| 6 — Real-time | No allocation in inner loops | `m_vfx_pool`/`m_vfx_emitter` are sized once at construction (2048-capacity pool + matching scratch buffer, ~230 KB total — data-model.md's arena headroom table); `set_shape` (research.md §1) is a trivial variant assignment with no allocation, replacing the tempting-but-wrong "construct a fresh emitter per burst" approach that would reallocate every click. |
| 7 — Test-first | GTest per public function | `emitter::set_shape` gets 3 new cases in `test_emitter.cpp` (contracts/emitter-set-shape.md); `scene`'s new VFX surface gets a new raylib-free `test_scene_vfx.cpp` target with 5 cases including the digest-parity assertion (contracts/scene-vfx.md). |
| 8 — HITL gates | Human review between plan→tasks and each implement task | This plan stops here for human approval before `/speckit.tasks`; `/speckit.implement` will pause between each generated task. |

**Result**: PASS — no violations. Complexity Tracking table is empty (see below).

## Project Structure

### Documentation (this feature)

```text
specs/002-sandbox-vfx-visualization/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md         # Phase 1 output (/speckit.plan command)
├── quickstart.md         # Phase 1 output (/speckit.plan command)
├── contracts/            # Phase 1 output (/speckit.plan command)
│   ├── emitter-set-shape.md
│   └── scene-vfx.md
├── checklists/
│   └── requirements.md
└── tasks.md              # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

Single-project layout (extends the existing `engine_demo` library and `apps/sandbox`
executable — no new top-level project or app):

```text
include/engine_demo/vfx/
└── emitter.h              # + set_shape(emitter_shape) noexcept declaration (only change)

src/engine_demo/vfx/
└── emitter.cpp             # + set_shape implementation (only change)

apps/sandbox/
├── scene.h                 # + m_vfx_pool, m_vfx_emitter, VFX constants, public/private VFX methods
├── scene.cpp                # + spawn_vfx_burst/spawn_vfx_spark/reset_vfx; substep() bounce sites call
│                             #   spawn_vfx_spark; substep() calls m_vfx_emitter.tick(dt); rebuild_scene()
│                             #   calls reset_vfx()
└── app.cpp                  # + RMB handler calls spawn_vfx_burst (additive); draw_scene() renders live
                              #   VFX particles with lifetime-based alpha fade; draw_hud() adds a live
                              #   VFX count

tests/engine_demo/
├── test_emitter.cpp         # + 3 new cases for set_shape (happy path, no-allocation, rng-unaffected)
└── test_scene_vfx.cpp       # NEW — raylib-free scene-level tests (5 cases, incl. digest parity)
```

Existing files touched (wiring only, no unrelated behavioral changes):

- `tests/engine_demo/CMakeLists.txt` — add `test_scene_vfx` following the exact
  `test_sandbox_render` pattern (`engine_demo_add_test` + `target_sources` for
  `apps/sandbox/scene.cpp` + `target_include_directories` for `${CMAKE_SOURCE_DIR}`).
- `.github/copilot-instructions.md` — plan-reference marker updated to point at this plan
  (this file); done as part of this `/speckit.plan` invocation, not a task.

**Structure Decision**: Single project (Option 1). The library addition lives alongside
`emitter`'s existing declarations in `include/engine_demo/vfx/emitter.h` /
`src/engine_demo/vfx/emitter.cpp` — no new header. The sandbox integration lives entirely
inside the existing `apps/sandbox/scene.{h,cpp}` and `apps/sandbox/app.cpp` — no new
translation unit at the app level, following the same "one class owns its arena-backed
subsystems" pattern `scene` already uses for `m_solver`/`m_budget`/`m_body_ids`. The only new
translation unit anywhere is the test file `tests/engine_demo/test_scene_vfx.cpp`, following
the exact precedent set by `test_sandbox_render.cpp` for testing sandbox app code without a
raylib link dependency.

## Phase 2: Task Planning Preview

*This is a sizing estimate only — `/speckit.tasks` generates the authoritative
`tasks.md`. Not created by `/speckit.plan`.*

Test-first ordering, grouped by unit, each task sized to stay under ~150 lines of diff:

1. `test_emitter.cpp` additions — 3 new cases for `set_shape` (red): happy path (position
   moves to new shape), no-allocation (`bytes_used()` unchanged across `set_shape` calls),
   rng-sequence-unaffected (identical seeds + differing `set_shape` call counts still
   produce identical spawn sequences).
2. `include/engine_demo/vfx/emitter.h` + `src/engine_demo/vfx/emitter.cpp` — add
   `set_shape(emitter_shape) noexcept`; tests from task 1 green.
3. `test_scene_vfx.cpp` (part 1, red) + CMake wiring — construct a `scene`, call
   `spawn_vfx_burst`, assert `vfx_particle_count()` increases while `particle_count()`
   (existing ad-hoc physics burst) is unaffected; assert VFX particles age/retire across
   `step()` calls.
4. `scene.h`/`scene.cpp` (part 1) — add `m_vfx_pool`, `m_vfx_emitter`, VFX constants,
   `spawn_vfx_burst`, `vfx_particle_count`/`vfx_particle_at`, `reset_vfx()` wired into
   `rebuild_scene()`; task 3 tests green.
5. `test_scene_vfx.cpp` (part 2, red) — bounce-triggers-spark-in-non-storm-scenes and
   no-spark-in-particle_storm assertions; `life_fraction` starts at 1.0 and decreases
   monotonically.
6. `scene.cpp` (part 2) — add private `spawn_vfx_spark`; wire into both non-storm bounce
   sites in `substep()`; add `m_vfx_emitter.tick(dt)` call once per `substep()`; task 5
   tests green.
7. `test_scene_vfx.cpp` (part 3, red) — the digest-parity assertion (SC-001): two
   identically-seeded scenes, one calling `spawn_vfx_burst` periodically, the other never
   calling it; assert identical `state_digest()` at every compared frame.
8. Renderer + HUD — `app.cpp`: RMB handler additionally calls `spawn_vfx_burst` (additive
   to the existing `spawn_particle_burst` call); `draw_scene()` draws live VFX particles
   with `life_fraction`-based alpha; `draw_hud()`'s top-right panel gains a live VFX count.
   Manual/screenshot verification per quickstart.md (no automated test — raylib render
   path).
9. Full-gate validation — `ctest --preset default-debug --output-on-failure` all green;
   headless golden-trace A/B run (`quickstart.md` §2) confirms byte-identical
   `trace_digest` before/after.

Estimated total: **~9 tasks**, each independently buildable/testable and under the
150-line diff cap; tasks 1–2 (library), 3–4 (burst), 5–6 (sparks), and 7 (digest parity)
form four independently-completable test-then-implement slices, consistent with the spec's
prioritized user stories (US1 burst, US2 sparks, US3 HUD folded into task 8).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations — table intentionally left empty.
