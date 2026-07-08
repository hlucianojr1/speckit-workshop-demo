# Orbital Arena — Constitution

Game-specific constitution for **Orbital Arena**, a competitive 2D physics game built on
the `engine_demo` foundation. Extends the engine constitution
([`specs/constitution.md`](../constitution.md)) — Articles 1–6 apply unchanged, Article 7
is modified, Articles 9–11 are new.

> Machine-readable copy merged into [`.specify/memory/constitution.md`](../../.specify/memory/constitution.md)
> so `/speckit.*` invocations targeting `orbital_arena` code are bound by all 11 articles.

## Inherited Articles (from engine_demo)

Articles 1–6 apply unchanged:

1. **No exceptions** — `-fno-exceptions` / `/EHs-c-`; status enums for fallible operations
2. **No RTTI** — `-fno-rtti` / `/GR-`; tagged unions / `eastl::variant`
3. **EASTL-first** — `std::` containers only at labeled interop boundaries
4. **Allocator-aware** — every container takes an explicit `engine_demo::allocator`
5. **Determinism** — seeded RNG, `double` accumulators, deterministic iteration order
6. **Real-time budgets** — no allocation in inner loops; pool/arena up front

## Article 7 — Test-First (Modified)

Every public function has at least one GTest. Additionally, every **game rule** has a
deterministic replay test that validates correct scoring across 1000 simulated frames.

## Article 8 — HITL Gates (Unchanged)

Spec-Kit pauses for human approval between stages.

## Article 9 — Competitive Fairness (NEW)

No game mechanic may provide asymmetric advantage based on player index. All players
have identical starting conditions. Randomization (power-up spawning) uses a shared
seed visible to all players for verifiability.

## Article 10 — Lockstep Replay (NEW)

The game state must be fully deterministic from initial seed + input sequence. Any
desync between two clients replaying the same input log is a critical bug. State
hashing every 60 frames for drift detection.

## Article 11 — Spectator-Safe State (NEW)

All game state is representable as a flat serializable snapshot (no pointers, no handles
crossing frame boundaries). This enables replay, spectating, and rollback netcode.

## Enforcement

Same local gate as the engine constitution: green
`cmake --build --preset default-debug` + `ctest --preset default-debug --output-on-failure`.
Orbital Arena code lives under `include/orbital_arena/`, `src/orbital_arena/`,
`tests/orbital_arena/`.
