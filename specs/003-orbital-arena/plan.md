# Implementation Plan: Orbital Arena — Competitive Gravity-Well Game

**Branch**: `006-orbital-arena` | **Date**: 2026-07-08 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/003-orbital-arena/spec.md`
**Constitution**: `.specify/memory/constitution.md` v1.1.0 (Articles 1–8 + Orbital Arena extension 7a, 9, 10, 11; human copy [specs/orbital-arena/constitution.md](../orbital-arena/constitution.md))

## Summary

Orbital Arena is a 2–4 player competitive gravity-well game built as a **new static
library `orbital_arena`** that links the existing `engine_demo` library and re-plans
*nothing*. All new code lives under `include/orbital_arena/`, `src/orbital_arena/`, and
`tests/orbital_arena/`. Existing subsystems — allocator, ecs::world, physics,
sim::game_loop, sim::rng, frame_budget, and the vfx particle pipeline — are consumed
as-is through their public headers. The delta is: a radial-attraction gravity well, a
deterministic nearest-well capture/scoring module, a seed-driven power-up module, a
match state machine, a per-tick input log, an integration `arena` that ties one match
tick together, a flat snapshot + 60-frame state hash for lockstep replay, and one
render-only sandbox scene (`scene_kind::orbital_arena`) with a score/state HUD for
workshop screenshots.

## Technical Context

**Language/Version**: C++20, MSVC (`/EHs-c-`, `/GR-`) and GCC/Clang (`-fno-exceptions -fno-rtti`)
**Primary Dependencies**: `engine_demo` static library (this repo), EASTL (vcpkg), GoogleTest (tests only), raylib (sandbox app only, optional)
**Storage**: N/A (in-memory; match snapshot is a flat POD struct per Article 11)
**Testing**: GoogleTest via CTest — new `tests/orbital_arena/` directory, preset `default-debug`
**Target Platform**: Windows x64 (MSVC) primary; Linux/macOS per existing presets
**Project Type**: Static library + tests + one sandbox scene (game-rules library, render-free)
**Performance Goals**: Full match tick (4 wells + full particle field + scoring + power-ups) inside the 60 FPS budget (≤16.67 ms, Article 6); vfx history shows ~0.1 ms for 500-particle ticks on this VM, so ample headroom
**Constraints**: No exceptions, no RTTI, EASTL-only containers with explicit `engine_demo::allocator`, zero heap allocation after construction (pools sized up front), fully deterministic from `(seed, input log)` (Articles 5, 10)
**Scale/Scope**: 2–4 players, ≈500 target particles, ≤3 concurrent pickups, 1000-frame replay tests; ~7 new header/source pairs + 7 test files + 1 sandbox scene

No `NEEDS CLARIFICATION` items — the spec resolved all open questions (checklist passed 2026-07-08).

## Constitution Check

*GATE evaluated pre-Phase-0 and re-evaluated post-Phase-1: **PASS** (no violations, Complexity Tracking empty).*

| Article | Requirement | How this plan satisfies it |
|---|---|---|
| 1 — No exceptions | No `try/catch/throw` | All fallible ops return `arena_status` enum (mirrors `vfx_status` pattern); nothing here can fail in ways needing unwinding |
| 2 — No RTTI | No `dynamic_cast`/`typeid` | Power-up kinds and match states are plain `enum class` tags; no polymorphism anywhere in the design |
| 3 — EASTL-first | EASTL containers only | `eastl::vector`, `eastl::span`, `eastl::array` throughout; `std::mt19937` reuse stays inside existing `sim::rng` (already a labeled interop boundary) |
| 4 — Allocator-aware | Explicit allocator on every container | Every `orbital_arena` type takes `engine_demo::allocator&` at construction (same pattern as `vfx::particle_pool`); no default-constructed containers |
| 5 — Determinism | Seeded RNG, `double` accumulators | Single match seed feeds `sim::rng`; **all gameplay timers are integer tick counters** (countdown = 180 ticks, etc.), sidestepping float accumulation entirely; the only `double` accumulator is the existing `game_loop` one (reused). See research.md D4 |
| 6 — Real-time budgets | No allocation in inner loops | All pools/logs `reserve()`d at match construction (input log sized for max match length, snapshot arrays fixed-size); per-tick work is span iteration only; `frame_budget` wraps the arena tick |
| 7 — Test-first | GTest per public function | Every module ships `tests/orbital_arena/test_*.cpp` written before implementation; wired into CTest (see CMake Wiring) |
| 7a — Game-rule replay tests | 1000-frame deterministic replay per rule | `test_replay_determinism.cpp` drives scripted 1000-frame matches covering capture, scoring, win condition, and each power-up effect at fixed seeds |
| 8 — HITL gates | Pause between plan → tasks → each task | This plan stops here; `/speckit.tasks` runs only after human approval |
| 9 — Competitive fairness | No player-index advantage | Capture contention resolves by *distance value only* (exact tie → no capture, FR-007); wells start rotationally symmetric; power-up positions derive from the shared seed; `test_scoring.cpp` + SC-003 input-swap test verify index-independence |
| 10 — Lockstep replay | State = f(seed, input log); 60-frame hashes | `arena::tick()` consumes exactly one `input_frame` per player per tick; `snapshot.h` provides field-wise FNV-1a `state_hash()` sampled every 60 ticks; replay test compares hash sequences |
| 11 — Spectator-safe state | Flat serializable snapshot | `match_snapshot` is a POD struct of fixed-size arrays (no pointers, no `entity_handle`s escaping); `arena::capture_snapshot()`/`restore()` round-trips it |

## Module Mapping

The core planning artifact (training §3.5): which existing `engine_demo` modules map
directly to each new feature, and which need new code. Existing modules are **reused
unmodified** — zero edits under `include/engine_demo/` or `src/engine_demo/`.

| New Feature | Builds On (existing, unchanged) | New Code |
|---|---|---|
| Gravity Well | `vfx::particle_pool::live_particles()` span; radial-pull math pattern from `apps/sandbox` particle_storm + `vfx::force_applicator` | `gravity_well.h/cpp` — well state (position, velocity, strength, radii, active) + `radial_attraction()` force + `is_within_capture()` predicate |
| Capture & Scoring | `ecs::world` (well/pickup entity handles + liveness queries) | `scoring.h/cpp` — nearest-well contention resolution (FR-007), per-player score table, capture events |
| Power-ups | `sim::rng` (shared match seed) + `ecs::world` | `powerup.h/cpp` — 10 s spawn schedule, 3 kinds, cap of 3, timed `active_effect` list |
| Match State | *new — no engine equivalent* | `match.h/cpp` — `lobby → countdown → playing → game_over` state machine + roster/ready flags |
| Input | *new — sandbox used direct mouse* | `input.h/cpp` — per-tick `input_frame` (2-axis steer + strength), fixed player→well mapping, append-only input log (FR-016) |
| Target Particles | `vfx::emitter` + `vfx::particle_pool` (feature 001/005) — spawn, replenish, Particle Flood bursts | thin emitter configs only; capture-removal via lifetime-zeroing (research.md D2) |
| Match Tick Integration | `sim::game_loop` (fixed step), `frame_budget` (Article 6 self-measure) | `arena.h/cpp` — owns all modules, one `tick(inputs)` = forces → integrate → captures → scoring → power-ups → state machine |
| Replay & Snapshot | — | `snapshot.h/cpp` — flat `match_snapshot` POD + FNV-1a `state_hash()` every 60 ticks (Articles 10–11) |
| Sandbox Visualization | `apps/sandbox` scene/HUD infrastructure (particle_storm as template) | `scene_kind::orbital_arena` + HUD (per-player scores, match state); **render-only, excluded from `state_digest`** — mirrors how feature 002 kept vfx out of the digest |

**Reuse score**: 6 of 7 engine subsystems (allocator, ecs, physics/game_loop, rng,
frame_budget, vfx) consumed without modification; only genuinely-new game logic is written.

## Dependency Graph

```text
                     ┌────────────────┐
                     │  arena (tick)  │  ← game_loop + frame_budget (existing)
                     └──────┬─────────┘
            ┌────────┬──────┴───┬──────────┐
            ▼        ▼          ▼          ▼
       ┌───────┐ ┌───────┐ ┌─────────┐ ┌──────────┐
       │ input │ │ match │ │ scoring │ │ powerup  │
       └───┬───┘ └───────┘ └────┬────┘ └────┬─────┘
           │                    ▼           │
           │            ┌──────────────┐    │
           └───────────▶│ gravity_well │◀───┘
                        └──────┬───────┘
                               ▼
              ┌────────────────────────────────┐
              │ vfx::particle_pool + emitter   │  (existing, unchanged)
              │ ecs::world + sim::rng          │  (existing, unchanged)
              └────────────────────────────────┘
       snapshot.h/cpp hashes/serializes across all of the above (Articles 10–11)
```

## Project Structure

### Documentation (this feature)

```text
specs/003-orbital-arena/
├── spec.md              # /speckit.specify output (done)
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── gravity_well.md
│   ├── scoring.md
│   ├── powerup.md
│   ├── match.md
│   ├── input.md
│   └── arena.md         # integration tick + snapshot/hash contract
├── checklists/          # /speckit.specify output (done)
└── tasks.md             # /speckit.tasks output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
include/orbital_arena/           # NEW — public headers, one per module
├── gravity_well.h
├── scoring.h
├── powerup.h
├── match.h
├── input.h
├── snapshot.h
└── arena.h

src/orbital_arena/               # NEW — static library target `orbital_arena`
├── CMakeLists.txt
├── gravity_well.cpp
├── scoring.cpp
├── powerup.cpp
├── match.cpp
├── input.cpp
├── snapshot.cpp
└── arena.cpp

tests/orbital_arena/             # NEW — GTest targets wired into CTest
├── CMakeLists.txt
├── test_gravity_well.cpp
├── test_scoring.cpp
├── test_powerup.cpp
├── test_match.cpp
├── test_input.cpp
├── test_arena_integration.cpp
└── test_replay_determinism.cpp  # Article 7a / 10: 1000-frame fixed-seed replays

apps/sandbox/                    # MODIFIED — one new scene, render-only
├── scene.h / scene.cpp          # + scene_kind::orbital_arena (particle_storm template)
├── app.cpp                      # + HUD: per-player scores + match state
└── CMakeLists.txt               # + link orbital_arena
```

**Structure Decision**: New sibling library `orbital_arena` beside `engine_demo`,
matching the layout mandated by the Orbital Arena constitution preamble ("code lives
under `include/orbital_arena/`, `src/orbital_arena/`, `tests/orbital_arena/`"). The
sandbox scene is the only edit outside those three roots, and it is render-only.

## CMake Wiring (explicit — training §3.5 omits this)

The training's 8-task table never mentions build wiring; a student following it verbatim
gets code that never compiles into any target. The wiring delta is four files:

1. **`src/orbital_arena/CMakeLists.txt`** (new):

   ```cmake
   add_library(orbital_arena STATIC
       gravity_well.cpp
       scoring.cpp
       powerup.cpp
       match.cpp
       input.cpp
       snapshot.cpp
       arena.cpp
   )
   target_include_directories(orbital_arena PUBLIC ${CMAKE_SOURCE_DIR}/include)
   target_link_libraries(orbital_arena PUBLIC engine_demo)   # transitively: EASTL
   engine_demo_set_target_options(orbital_arena)             # -fno-exceptions -fno-rtti etc.
   ```

2. **`src/CMakeLists.txt`** (edit): `add_subdirectory(orbital_arena)` after
   `add_subdirectory(engine_demo)` (link-dependency order).

3. **`tests/orbital_arena/CMakeLists.txt`** (new): local `orbital_arena_add_test()`
   helper cloned from `engine_demo_add_test()`, linking `orbital_arena GTest::gtest
   GTest::gtest_main` — **never `GTest::gmock`** (known repo bug: linking gmock.dll
   silently voids the whole test registry and produces false-positive CTest passes).

4. **`tests/CMakeLists.txt`** (edit): `add_subdirectory(orbital_arena)`.

**No top-level `CMakeLists.txt` change** — it already does `add_subdirectory(src)`,
`add_subdirectory(tests)`, and `enable_testing()`; the new subdirectories are picked up
through the two one-line edits above.

**Sandbox knock-on** (visualization task only): `apps/sandbox/CMakeLists.txt` adds
`orbital_arena` to `target_link_libraries(ea-sandbox …)`, and — because
`tests/engine_demo/CMakeLists.txt`'s `test_scene_vfx` target compiles
`apps/sandbox/scene.cpp` directly — that one existing test target must also link
`orbital_arena` once `scene.cpp` includes its headers.

## Design Decisions (summary — full rationale in research.md)

- **D1**: Radial attraction applies directly to `vfx::particle_pool::live_particles()`
  spans, *not* through `physics::constraint_solver` (the training doc's suggestion). The
  solver is a verlet body/constraint API; free particles live in the vfx pool, exactly
  as the existing particle_storm scene already demonstrates.
- **D2**: Capture removes a particle by zeroing `remaining_lifetime_seconds`, letting the
  pool's own `age_and_retire(dt)` swap-remove it at end-of-tick — zero engine changes.
- **D3**: Contention (FR-007) resolves on squared distances in fixed well-index iteration
  order; winner is the strict minimum, exact tie → no capture. Distance-only comparison
  makes index-independence structural (Article 9).
- **D4**: All gameplay durations are integer tick counts at the fixed 60 Hz step
  (countdown 180, power-up cadence 600, Strength Surge 300, Double Points 600). No float
  time accumulation in game rules at all (Article 5, stronger than required).
- **D5**: `state_hash()` is field-wise FNV-1a 64 (never `memcmp`/raw-struct hashing —
  padding bytes are indeterminate), sampled every 60 ticks (Article 10).
- **D6**: RNG discipline: one `sim::rng` owned by the arena, seeded from the match seed,
  consumed *only* by power-up scheduling and particle replenishment in a fixed order;
  the vfx emitter gets a salted sub-seed (same XOR-salt pattern as the sandbox's
  `kVfxSeedSalt`), so render-side effects never perturb game-rule draw order.
- **D7**: Sandbox scene drives the arena with deterministic scripted autopilot inputs
  (screenshot-friendly, no 4-keyboard problem); HUD text and trails are render-only and
  never feed `state_digest`, mirroring feature 002's digest isolation.

## Task Decomposition Shape (estimate for /speckit.tasks)

The training's 8-task table maps onto this plan almost 1:1, plus three tasks the
training omits (CMake scaffolding, snapshot/hash, sandbox scene):

| # | Task | ≈Lines | Deps | Training equivalent |
|---|---|---|---|---|
| 1 | CMake scaffolding: lib target + test dir + empty smoke test | ~60 | — | *(omitted by training)* |
| 2 | Gravity well: state + radial force + capture predicate (test-first) | ~120 | 1 | Task 1 |
| 3 | Capture detection + FR-007 contention resolution | ~90 | 2 | Task 2 |
| 4 | Scoring state + win/sudden-death rules | ~100 | 3 | Task 3 |
| 5 | Power-up types + seed-driven spawn/effect lifecycle | ~130 | 1 | Task 4 |
| 6 | Match state machine (lobby/countdown/playing/game_over) | ~110 | 1 | Task 5 |
| 7 | Input frames, player→well mapping, input log | ~80 | 1 | Task 6 |
| 8 | Snapshot struct + FNV-1a state hash (Articles 10–11) | ~90 | 4,5,6 | *(omitted)* |
| 9 | Arena integration: full match tick loop | ~140 | 2–8 | Task 7 |
| 10 | Replay determinism + input-swap fairness tests (1000 frames) | ~100 | 9 | Task 8 |
| 11 | Sandbox scene `orbital_arena` + HUD (render-only) + screenshots | ~150 | 9 | *(omitted)* |

≈11 tasks, each ≤150 lines (training §3.8 budget). Tasks 5, 6, 7 are parallelizable
after task 1; the P1→P4 user-story priority order is preserved (US1 = tasks 2–3,
US2 = tasks 4+6, US3 = task 5, US4 = tasks 8–10).

## Complexity Tracking

> Constitution Check passed with no violations — table intentionally empty.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| *(none)* | | |
