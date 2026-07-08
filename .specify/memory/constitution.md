# engine_demo Constitution

<!-- Synced with specs/constitution.md — the human-facing ground truth. If you edit one,
     edit both. The speckit.* prompts and agents read THIS file. -->

Eight non-negotiable articles that bind every `/speckit.specify`, `/speckit.plan`,
`/speckit.tasks`, and `/speckit.implement` invocation in this workspace.

## Core Principles

### Article 1 — No exceptions

Code compiles with `-fno-exceptions` / `/EHs-c-`. No `try`, `catch`, `throw`,
`noexcept(false)`, or `std::exception` derivations. Fallible operations return a status
enum (preferred) or `eastl::expected<T, status>` where the EASTL revision supports it.

### Article 2 — No RTTI

Code compiles with `-fno-rtti` / `/GR-`. No `dynamic_cast` or `typeid`. Type discrimination
uses tagged unions, `eastl::variant`, or typed-handle patterns.

### Article 3 — EASTL-first

EASTL containers and smart pointers are the default in committed code (`eastl::vector`,
`eastl::string`, `eastl::unique_ptr`, `eastl::shared_ptr`, `eastl::function`, `eastl::span`,
`eastl::optional`, `eastl::variant`). `std::` containers are permitted only in interop layers
explicitly labeled `// interop boundary` in code comments.

### Article 4 — Allocator-aware containers

Every EASTL container takes an explicit allocator at construction. The project's
`engine_demo::allocator` (in `include/engine_demo/allocator.h`) is the source of truth.
Default-constructed containers are forbidden in committed code.

### Article 5 — Determinism

Simulation paths are deterministic across runs at fixed seeds:

- RNG engines are seeded explicitly at construction from the full seed width;
  `std::random_device` is forbidden in sim paths. `std::mt19937` is permitted as an
  interop boundary because EASTL ships no Mersenne Twister engine.
- Time accumulators are `double`. `float` is permitted only at the render boundary.
- Iteration order over containers is deterministic; never hash-keyed containers keyed on
  pointer identity in sim paths.

### Article 6 — Real-time budgets

Frame budgets: ≤16.67 ms at 60 FPS, ≤8.33 ms at 120 FPS. No allocation in inner loops; pool
or arena up front. Branch-prediction-friendly idioms preferred. No locks in render thread;
use lockless ring buffers for cross-thread handoff.

### Article 7 — Test-first

Every public function has at least one GTest covering happy + edge cases.
`/speckit.implement` tasks emit the test before the implementation. The local gate is
green tests (`ctest --preset default-debug`) plus clang-tidy clean.

### Article 8 — HITL gates

Spec-Kit pauses for human approval **between** `/speckit.plan` and `/speckit.tasks`, and
**between every `/speckit.implement` task**. Coding Agent handoffs require human review of
the produced PR before merge.

## Orbital Arena Extension (applies to `orbital_arena` code only)

Human-facing ground truth: `specs/orbital-arena/constitution.md`. For any feature whose
code lives under `include/orbital_arena/`, `src/orbital_arena/`, or `tests/orbital_arena/`,
the following additional articles bind every `/speckit.*` invocation:

### Article 7a — Game-rule replay tests (modifies Article 7)

Every **game rule** (scoring, capture, win condition, power-up effect) has a deterministic
replay test that validates correct behavior across 1000 simulated frames at a fixed seed.

### Article 9 — Competitive Fairness

No game mechanic may provide asymmetric advantage based on player index. All players have
identical starting conditions. Randomization (power-up spawning) uses a shared seed
visible to all players for verifiability.

### Article 10 — Lockstep Replay

The game state must be fully deterministic from initial seed + input sequence. Any desync
between two clients replaying the same input log is a critical bug. State hashing every
60 frames for drift detection.

### Article 11 — Spectator-Safe State

All game state is representable as a flat serializable snapshot (no pointers, no handles
crossing frame boundaries). This enables replay, spectating, and rollback netcode.

## Enforcement

- **clang-tidy** (`.clang-tidy`) flags `bugprone-*`, `cert-*`, `cppcoreguidelines-*`,
  `modernize-*`, `performance-*`, `portability-*`, `readability-*`. WarningsAsErrors mirrors.
- **Local gate**: `cmake --preset default-debug && cmake --build --preset default-debug &&
  ctest --preset default-debug --output-on-failure` must be green before any task is
  approved.

## Governance

The constitution supersedes all other practices in this workspace. Amendments require a PR
that updates BOTH this file and `specs/constitution.md`, plus the approval of the workshop
maintainer. If a spec-kit task requires violating an article, the spec is wrong — fix the
spec, not the article.

**Version**: 1.1.0 | **Ratified**: 2026-07-04 | **Last Amended**: 2026-07-04

