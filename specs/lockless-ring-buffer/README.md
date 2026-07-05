# Lockless ring buffer — Part 2 demo feature (§2.3)

> **Stage:** skeleton only. The facilitator runs `/speckit.specify`, `/speckit.plan`,
> `/speckit.tasks`, `/speckit.implement` live; this directory exists so the spec-kit CLI
> has a destination.

## Brief (one-paragraph problem statement)

Provide a single-producer / single-consumer lockless ring buffer for cross-thread message
hand-off in `engine_demo`. Capacity is fixed at construction; storage comes from an
`engine_demo::allocator&`. Backpressure policy is **drop-oldest** when full (configurable
in the stretch goal). Must satisfy constitutional articles 1–6 (no exceptions, no
RTTI, EASTL containers, allocator-aware, deterministic ordering, real-time / no
allocation in inner loops).

## Demo arc

1. `/speckit.constitution` — facilitator reviews the eight articles aloud (5 min).
2. `/speckit.specify` — capture the brief above as `spec.md` (10 min).
3. `/speckit.plan` — Copilot proposes a memory layout, atomics protocol, and tests-first
   ordering (10 min).
4. `/speckit.tasks` — Copilot decomposes into 4–6 task entries (10 min).
5. `/speckit.implement` — facilitator approves one task at a time; HITL between each (15 min).
