# R2 — Memory Management Best Practices for Real-Time Rust Games

Research note for the C++ (EASTL) → Rust transformation. Companion to R1 (architecture/ECS).
Scope: pre-allocation, per-frame allocation detection, fixed-capacity crates, and when
custom allocators are (un)necessary. No code prescriptions here — guidelines land in C2.

## 1. Pre-allocation patterns

- **`Vec::with_capacity(n)`** — the baseline. Allocate once at startup/load, then `push`/`clear`
  within capacity; `clear()` keeps the buffer, so a per-frame scratch `Vec` reused across frames
  never reallocates after warm-up. Guard growth with `debug_assert!(v.len() < v.capacity())`
  where capacity is a hard budget. Source: std `Vec` docs (docs.rs/std) [live: docs.rs].
- **Object pools** — a `Vec<T>` of pre-constructed objects plus a free list (indices or
  generational handles). In ECS designs this is often unnecessary: the ECS storage *is* the pool
  (entity despawn/respawn reuses archetype storage in Bevy). Explicit pools remain useful for
  non-ECS resources (audio voices, network buffers). `heapless::pool` provides lock-free
  `ObjectPool`/`BoxPool`/`ArcPool` singletons ([docs.rs/heapless](https://docs.rs/heapless/latest/heapless/pool/index.html)) [live].
- **Arenas (bump allocation)** — [`bumpalo`](https://docs.rs/bumpalo/latest/bumpalo/) [live]:
  pointer-bump allocation, no per-object free; reset the whole arena with `Bump::reset()`.
  Explicitly "well-suited for phase-oriented allocations" — e.g. a per-frame scratch arena reset
  at frame end. Caveats: `Drop` is **not** run on plain `bump.alloc` values (use
  `bumpalo::boxed::Box` when destructors matter); `Bump` is `!Sync` (use `bumpalo-herd` for
  rayon-style parallelism); when a chunk fills, it *does* hit the global allocator, so size the
  arena for worst case. `bumpalo::collections::Vec/String` give arena-backed collections on stable.
- **Alternatives**: `typed-arena` (single-type, runs `Drop`), and the stable-Rust
  `allocator-api2` bridge that lets `bumpalo::Bump` back any allocator-generic collection
  [live: docs.rs/bumpalo, "Using the Allocator API on Stable Rust"].

## 2. Detecting and preventing per-frame heap allocation

- **Counting global allocator** — wrap the system allocator in a `#[global_allocator]` that
  increments atomics on alloc/dealloc; assert zero deltas across a frame in tests. Crates:
  `assert_no_alloc` (panics/aborts on alloc inside a scope; used widely in real-time audio) and
  `dhat` (heap profiling + testing mode with allocation-count assertions) [from training knowledge].
- **Test-time gates** — a headless "run 60 frames, assert allocation counter delta == 0 after
  warm-up frame" integration test is the Rust analogue of this repo's C++ allocator-observability
  articles. Feeds R5 (testing).
- **Profiling** — `dhat`, heaptrack (Linux), or Tracy's memory zones via the `tracy-client`
  crate (Bevy has built-in Tracy support: `trace_tracy` feature) [from training knowledge].
- **Prevention idioms** — hoist collections out of loops and `clear()` instead of re-creating;
  avoid `format!`/`String` and iterator `.collect()` in hot systems; prefer `&str`, fixed
  buffers, and pre-sized `Vec`s; beware hidden allocators (`Box<dyn Fn>`, `Rc`, `HashMap`
  growth, logging).
- **Clippy support is limited** — no lint reliably flags "allocation in hot path"; runtime
  counting is the enforceable mechanism [from training knowledge].

## 3. Fixed-capacity collection crates

| Crate | Storage | On overflow | Best for |
|---|---|---|---|
| [`arrayvec`](https://docs.rs/arrayvec/latest/arrayvec/) [live] | Inline `[T; CAP]`, capacity in the type | `push` panics; `try_push` → `CapacityError` | Hard per-entity budgets (e.g. "max 8 contacts"); `ArrayString` for no-alloc text |
| [`smallvec`](https://docs.rs/smallvec/latest/smallvec/) [live] | Inline up to N, **spills to heap** beyond | Silently reallocates to heap | "Usually small" data where occasional spill is acceptable — *not* a no-alloc guarantee |
| [`heapless`](https://docs.rs/heapless/latest/heapless/) [live] | Inline/static, `no_std` | Fallible API: `Result` on `push` | Truly heapless targets; richest set: `Vec`, `String`, `IndexMap`, `Deque`, SPSC/MPMC queues, `HistoryBuf`, pools |

Trade-offs:

- `arrayvec`/`heapless` give **true constant-time push** (no amortized reallocation), which
  heapless explicitly calls out as required for hard-real-time; the cost is moving/copying the
  full inline buffer when the struct moves, and capacity baked into the type signature.
- `smallvec` optimizes the common case but its heap spill defeats "no per-frame allocation"
  guarantees — treat it as a locality optimization, not a budget-enforcement tool.
- `heapless` structures return `Result` on capacity exhaustion, matching this project's
  no-exceptions/status-return philosophy; `arrayvec::try_push` does the same.
- A fixed-size ring/history buffer (`heapless::HistoryBuf` or a hand-rolled `[T; N]` + head
  index) covers the frame-budget rolling-window use case without any allocation.

## 4. When ownership makes a custom allocator unnecessary — and when it doesn't

**Usually unnecessary.** EASTL-style allocator plumbing in C++ exists largely to (a) make
allocation visible/attributable, (b) prevent hidden copies and lifetime bugs, and (c) route
containers to arenas/pools. In Rust:

- Ownership + `Drop` make lifetimes deterministic without allocator bookkeeping; leaks and
  double-frees are prevented by the type system, not by allocator discipline.
- Moves are memcpy-cheap and explicit; no hidden copy-constructor allocations to police.
- Visibility is achieved with a counting `#[global_allocator]` + tests (§2) rather than
  per-container allocator parameters.
- Capacity budgeting is achieved with `with_capacity` + fixed-capacity types (§3) rather than
  fixed-size allocator adapters.
- The `Allocator` trait (parameterized `Vec<T, A>` etc.) is still **nightly-unstable**
  (rust-lang/rust#42774; noted in bumpalo's docs [live]), so per-container allocators are not
  idiomatic stable Rust anyway.

**Rare cases where custom allocation still earns its keep:**

1. Per-frame scratch arenas (bumpalo) where thousands of short-lived allocations must be O(1)
   and freed en masse — cheaper than even a well-tuned malloc.
2. `no_std` / console-like targets with no OS allocator: `heapless` + static pools.
3. Replacing the global allocator wholesale (`mimalloc`, `jemalloc`) for throughput/fragmentation,
   or with a counting wrapper for observability — a one-line swap, not EASTL-style plumbing.
4. Lock-free cross-thread queues with bounded memory (`heapless::spsc/mpmc`) where allocator
   nondeterminism would break latency budgets.

**Bottom line for C2:** preserve the *guarantees* (no per-frame allocation, visible budgets,
bounded worst case) via `with_capacity` + fixed-capacity types + a counting-allocator test gate;
do **not** reproduce EASTL's allocator-parameter mechanism — stable Rust neither needs nor
supports it idiomatically.

## Sources

- bumpalo — https://docs.rs/bumpalo/latest/bumpalo/ [live]
- heapless — https://docs.rs/heapless/latest/heapless/ [live]
- arrayvec — https://docs.rs/arrayvec/latest/arrayvec/ [live]
- smallvec — https://docs.rs/smallvec/latest/smallvec/ (README via docs.rs) [live]
- std `Vec::with_capacity`, `#[global_allocator]`, allocator_api tracking issue rust-lang/rust#42774 [live via bumpalo docs; details from training knowledge]
- assert_no_alloc, dhat, Tracy/tracy-client, typed-arena, bumpalo-herd [from training knowledge]
