# C1 — EASTL Design Concepts (Language-Neutral)

**Sources:** `specs/constitution.md` (Articles 3–6), `include/engine_demo/allocator.h`,
EASTL usage across `include/engine_demo/`, `include/orbital_arena/`, `src/`.

**Purpose:** Extract the *concepts* EASTL delivers to this codebase, independent of C++.
A target language must honor these concepts — not reproduce EASTL's API shape.

---

## Concept 1 — Explicit memory ownership

Every data structure has a named, identifiable owner of its backing memory. In this codebase,
`engine_demo::allocator` is constructed over a caller-supplied buffer; nothing owns memory
"ambiently" via a global heap. (`allocator(void* buffer, std::size_t capacity_bytes)`.)

- **Why it matters for games:** memory is a budgeted resource per subsystem (VFX pool, physics
  bodies, input log). Ownership boundaries make budget overruns attributable and make teardown
  order explicit.
- **What breaks without it:** fragmentation and unbounded growth hide until shipping hardware;
  a leak or overrun cannot be traced to the subsystem that caused it.

## Concept 2 — Allocation is visible and budgeted, never hidden

No container grows silently. Every allocation flows through a subsystem allocator with a fixed
byte capacity; exhaustion returns null/status (`alloc_status::out_of_memory`), never throws or
aborts. Fallible creation is a value: `arena::create` returns an optional, `try_spawn` returns
`emit_result{spawned, status}`.

- **Why it matters for games:** the frame budget (Article 6) includes memory; a hidden realloc
  is a hidden latency spike and a hidden budget breach. Failure paths must be handleable at
  60 Hz without unwinding.
- **What breaks without it:** hitches from mid-frame reallocation; out-of-memory becomes a crash
  instead of a designed degradation (e.g. "spawn fewer particles this frame").

## Concept 3 — Fixed capacity decided up front

Capacity is a construction-time decision, not an emergent runtime property. `particle_pool`
sizes its dense array once at construction; `try_spawn` and `age_and_retire` never allocate
afterward. Steady-state (inner-loop) allocation count is zero by design.

- **Why it matters for games:** worst-case behavior is decided at design time, so frame time is
  bounded and testable. Pools convert "how much memory will this use?" into a reviewable number.
- **What breaks without it:** worst-case memory/time is unknowable; the first stress spike in
  the field is the first time anyone learns the real ceiling.

## Concept 4 — Allocation observability

Memory state is queryable at runtime: `bytes_used()`, `capacity()`, and a human-readable
allocator name (`get_name`/`set_name`) for attribution in telemetry and tests. Pools expose
`live_count()` / `free_count()`.

- **Why it matters for games:** budgets are only enforceable if they're measurable — tests can
  assert "zero bytes allocated during 1000 frames," and telemetry can report per-subsystem usage.
- **What breaks without it:** budget rules become aspirational; regressions in allocation
  behavior land silently and are found by profiling after the fact.

## Concept 5 — Deterministic container behavior

Iteration order is a specified, seed/insertion-independent property. Physics bodies live in a
key-sorted map (`vector_map` ordered by body id) and constraints are kept sorted by canonical
key so solver projection order — and therefore convergence — is identical across runs and
construction orders (BUG-004 fix; Article 5). Hash- or pointer-order iteration is forbidden in
sim paths.

- **Why it matters for games:** replay, lockstep networking, and golden-trace testing all require
  bit-identical simulation given the same seed and inputs.
- **What breaks without it:** heisenbugs that reproduce on one machine only; replays desync;
  determinism tests pass locally and fail in CI.

## Concept 6 — Graceful degradation at capacity limits

Hitting a limit is a normal, partial-success outcome, not an error path. `try_spawn` spawns as
many particles as free capacity allows and reports the count; the game keeps running with fewer
particles rather than failing.

- **Why it matters for games:** visual/simulation quality can degrade smoothly under load;
  the player sees fewer sparks, not a crash or a stall.
- **What breaks without it:** capacity limits become crash sites or force over-provisioning
  "just in case," wasting the memory budget.

## Concept 7 — Value-based type discrimination (closed sums, no dynamic dispatch)

Heterogeneous-but-closed sets (emitter shapes, force kinds) are tagged unions (`variant`) held
by value in dense arrays — no inheritance, vtables, or runtime type queries. Dispatch is an
explicit branch over a known, finite set.

- **Why it matters for games:** contiguous, cache-friendly storage of mixed kinds; dispatch cost
  is visible and predictable; the full set of cases is auditable at compile time.
- **What breaks without it:** pointer-chasing heterogeneous object graphs wreck cache behavior
  in inner loops, and open-ended hierarchies hide unhandled cases.

---

## Explicitly excluded (C++ mechanics, not concepts)

- **Allocator template parameters** (`eastl::vector<T, eastl_allocator_ref>`) and the
  `eastl_allocator_ref` adapter, including its forced default-constructibility — C++ plumbing
  to thread ownership through a language without it built in (Concept 1 is the concept).
- **`fixed_vector` overflow-enabled flags / EASTL allocator hook signatures** (`allocate(n,
  alignment, offset, flags)`) — API shape, not design intent.
- **`eastl::move`, `get_if`, deleted copy ctors, `noexcept` decorations** — C++ move/exception
  semantics workarounds; languages with affine ownership or no exceptions get these for free.
