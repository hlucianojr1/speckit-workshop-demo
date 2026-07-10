# engine_demo — Rust/Bevy Constitution (Spec-Kit ground truth, target language)

Eleven non-negotiable articles binding every `/speckit.specify`, `/speckit.plan`,
`/speckit.tasks`, and `/speckit.implement` invocation for the Rust/Bevy port. This is the
target-language sibling of [`specs/constitution.md`](../constitution.md) (C++/EASTL):
same behavioral guarantees, Rust-idiomatic mechanisms. Reviewers reject any artifact that
violates them.

> Derived from [`specs/transform/rust-guidelines.md`](rust-guidelines.md) (15 guidelines +
> HITL Resolutions, approved 2026-07-10) and the Phase A behavioral specs
> ([`ecs-world.spec.md`](ecs-world.spec.md), [`physics-constraint.spec.md`](physics-constraint.spec.md),
> [`game-loop.spec.md`](game-loop.spec.md), [`rng.spec.md`](rng.spec.md),
> [`frame-budget.spec.md`](frame-budget.spec.md), [`allocator.spec.md`](allocator.spec.md)).
> Each article is tagged **Inherited** (guarantee and mechanism both carry over),
> **Adapted** (guarantee carries over, C++ mechanism replaced by a Rust idiom), or
> **New** (guarantee did not exist as a standalone article in the C++ constitution).

---

## Article I — No panics in sim/system code (Adapted)

Per-frame simulation and system code never panics. `unwrap`, `expect`, `panic!`, `todo!`,
`unimplemented!`, and slice-indexing (`v[i]`) are forbidden in sim crates; panics are
permitted only at startup/asset-load (fail fast, matching the reference implementations'
one-time init paths) and inside `#[cfg(test)]`. This is the direct heir of Article 1
(No exceptions): a panicking Bevy system unwinds the whole `App`, which is strictly worse
than a C++ exception escaping a frame, so the rule is at least as strict as the original.

- **Derives from:** Guideline 6.
- **Enforcement:** `[lints.clippy]` in each sim crate's `Cargo.toml` denies `unwrap_used`,
  `expect_used`, `panic`, `todo`, `unimplemented`, `indexing_slicing`; `cargo clippy
  --all-targets --all-features -- -D warnings` in CI (Article X).

## Article II — Errors and results are values (Adapted)

Fallible operations return `Result<T, E>` or `Option<T>`, never a thrown/unwound error.
Sim-path error enums are small, fieldless (or `Copy`), and derive `Debug, Clone, Copy,
PartialEq`; `anyhow`/`Box<dyn Error>` are confined to bin targets and tooling. This replaces
Article 1's `eastl::expected<T, status>` / status-enum pattern; Rust upgrades it because
`#[must_use]` makes consumption compiler-enforced and `?` makes propagation free.

- **Derives from:** Guidelines 7–8.
- **Enforcement:** `cargo-deny` bans `anyhow` as a dependency of sim crates; `unused_must_use`
  promoted to `deny`; `#[must_use]` required on factories, guards/handles, and bare status
  returns (`clippy::must_use_candidate`, pedantic, cherry-picked).

## Article III — Ownership-based memory safety; no `unsafe` (Adapted)

Memory lifetime and attribution come from ownership, borrowing, and `Drop` — not from
ported allocator-template plumbing. `#![forbid(unsafe_code)]` at every game-crate root; any
future exemption (SIMD, GPU buffer casts) is isolated in one small, heavily tested module
with `// SAFETY:` comments and a `# Safety` rustdoc section per `unsafe fn`. This replaces
Article 3 (EASTL-first) and Article 2 (No RTTI): both existed to work around C++ manual
memory management and unsafe casting, and ownership + the `forbid` lint deliver the
equivalent guarantee natively.

- **Derives from:** Guidelines 1, 9.
- **Enforcement:** `#![forbid(unsafe_code)]` (hard compile error, cannot be downstream
  `allow`ed); code review rejects `Vec<T, A>`-style allocator generics; `clippy::
  undocumented_unsafe_blocks` + `clippy::missing_safety_doc` for any exempted module.

## Article IV — Zero-allocation steady-state frame loop (Adapted)

All frame-loop collections are sized at startup with `Vec::with_capacity` or fixed-capacity
types (`arrayvec::ArrayVec`, `heapless::Vec`) and reused via `clear()`; no collection grows
during a steady-state frame. `smallvec` is excluded from budget-bearing/`FixedUpdate` paths
(HITL C-2: its silent heap spill defeats the declared budget) and is permitted only as a
non-sim locality optimization. This is the direct Rust-idiomatic replacement for Article 4
(allocator-aware EASTL containers) and the no-allocation half of Article 6: the
`engine_demo::allocator` arena becomes "reserve at startup, prove it with a test" because
Rust has no stable per-container allocator parameter to make the budget visible any other
way (per [`allocator.spec.md`](allocator.spec.md) §5's transformation note).

- **Derives from:** Guidelines 2–4.
- **Enforcement:** `#[global_allocator]` wrapping `System` with `AtomicUsize` counters;
  dedicated test asserts allocation-per-batch is CONSTANT across two equal-length
  post-warm-up batches, not literally zero and not growing (empirically, Bevy 0.16's own
  scheduler/task-pool internals allocate-and-free a small constant amount per frame even
  with zero game systems registered — this is framework overhead outside game-code control,
  not a violation; a *growing* per-batch delta is the actual bug this test exists to catch —
  a leak or unbounded collection growth in game code); `debug_assert!(v.len() < v.capacity())`
  on hard-budget collections; clippy lint table denies bare `push` where `try_push` exists.

## Article V — Capacity exhaustion is graceful degradation (New)

Hitting a budget limit in the frame loop (spawn/emit/allocate-from-pool) is a normal,
partial-success outcome: the API returns a count-and-status value (how much work was done)
and the game continues degraded — it never panics or fails the frame. All other fallible
operations use plain `Result<T, E>` (Article II). This makes explicit, as a standalone rule,
a split that Articles 1 and 4 left implicit in the C++ constitution (status enums covered
both "degraded" and "hard failure" without distinguishing them).

- **Derives from:** Guideline 5; HITL Resolution C-5.
- **Enforcement:** every capacity-limited API (e.g. `ecs-world` §3.1 create-when-exhausted,
  `allocator` §3 allocate-when-full) gets a `#[test]` driving it past its limit and asserting
  degraded-but-alive behavior, not a panic; `#[must_use]` on the returned status (Article II).

## Article VI — Deterministic simulation (Inherited)

Simulation is deterministic across runs at a fixed seed, same platform and build
(HITL Resolution C-3 sets the bar: bitwise run-twice equality, not cross-platform or
lockstep). Concretely:

- RNG is `rand::rngs::StdRng`, seeded explicitly via `seed_from_u64` from a full 64-bit
  seed ([`rng.spec.md`](rng.spec.md) §2); OS entropy (`OsRng`, `thread_rng`) is forbidden in
  sim paths (HITL Resolution C-1 — this workshop's bar is behavioral, same-build
  reproducibility, not cross-version replay-file compatibility).
- Time and score accumulators are `f64` ([`game-loop.spec.md`](game-loop.spec.md) §5,
  [`frame-budget.spec.md`](frame-budget.spec.md) §5); `f32` only at the point of use
  (render/component math).
- Order-sensitive computation ([`physics-constraint.spec.md`](physics-constraint.spec.md)
  §4.1) never relies on `HashMap`/`HashSet` iteration order, raw ECS query order, or
  ambiguous parallel system order; sort explicitly by a stable game-assigned key.

This is a direct carry-over of Article 5; only the mechanism names change (`StdRng` instead
of `std::mt19937`, `BTreeMap`/explicit sort instead of ordered EASTL containers).

- **Derives from:** Guidelines 10–11.
- **Enforcement:** `cargo-deny`/review bans `OsRng`/`thread_rng`/non-`StdRng` engines in sim
  crates; `ScheduleBuildSettings::ambiguity_detection = LogLevel::Error` on sim schedules;
  grep gate bans `std::collections::HashMap` iteration in sim modules; deterministic-replay
  `#[test]` (seed → step N fixed ticks → hash state via `to_bits()` → rerun → assert
  identical) per Article X.

## Article VII — Real-time frame budget and fixed timestep (Inherited)

Frame budgets remain ≤16.67 ms at 60 FPS / ≤8.33 ms at 120 FPS. All gameplay/physics state
mutation lives in Bevy's `FixedUpdate` schedule ([`game-loop.spec.md`](game-loop.spec.md)
§4); input sampling, camera, and interpolation stay in `Update`. Frame-timing telemetry
uses a fixed-size rolling window recorded with zero allocation
([`frame-budget.spec.md`](frame-budget.spec.md) §3–4). Performance regressions are tracked
with `criterion` baselines (or `iai-callgrind` instruction counts) — never `assert!(elapsed
< 16ms)` in a test, since shared CI runners make wall-clock asserts flaky. This carries
Article 6's guarantee forward; the mechanism moves from a hand-rolled ring buffer + manual
scheduling to Bevy's built-in fixed-timestep schedule plus a criterion-based perf gate.

- **Derives from:** Guideline 12, 15 (perf-gate clause).
- **Enforcement:** review checklist: any `Query`-mutating gameplay system registered outside
  `FixedUpdate` needs written justification; headless tests step exact fixed ticks via
  `TimeUpdateStrategy::ManualDuration`; criterion baseline comparison gate in CI (not a
  `#[test]` timing assertion).

## Article VIII — Closed sums over dynamic dispatch in hot paths (New)

Heterogeneous-but-closed sets (emitter shapes, force kinds) are Rust `enum`s stored by
value in contiguous collections; `Box<dyn Trait>` and open-ended dispatch are banned from
inner loops. This names, as its own article, a guarantee that Article 2 (No RTTI) partially
covered by banning `dynamic_cast`/`typeid` — the closed-set-dispatch half of that intent has
no single C++ article of its own, so it is promoted here rather than folded silently into
Article III.

- **Derives from:** Guideline 13.
- **Enforcement:** code review requires justification for any `dyn` in sim-crate hot paths;
  `match` without wildcard arms on closed sim enums (new variants become compile errors);
  Article IV's allocation gate catches boxed-trait storage as a side effect.

## Article IX — One plugin per subsystem, headless-testable (Adapted)

Each subsystem (physics, scoring, VFX, input) is a Bevy `Plugin` owning its registrations,
resources, and schedule placement, addable to a bare/`MinimalPlugins` `App` without
dragging in rendering. Singletons (RNG, frame budget, score) are plain structs in resources,
not forced into the ECS. Every plugin ships at least one headless integration test built by
constructing a minimal `App`, inserting only the resources/messages it needs, and driving it
with `app.update()`. This is the mechanism-level replacement for the "every public function
has a GTest" half of Article 7: the unit of testability moves from a free function to a
plugin's headless `App`.

- **Derives from:** Guideline 14.
- **Enforcement:** CI runs the full test suite windowless; every plugin's headless test is
  required before merge (Article X).

## Article X — CI is the enforcement layer (Adapted)

Every merge passes, in fail-fast order: `cargo fmt --check`, `cargo clippy --all-targets
--all-features -- -D warnings` (lint tables from Articles I–III/VIII live in `Cargo.toml`
so local == CI), the full test suite (including the deterministic-replay test of Article VI
and the zero-allocation frame test of Article IV), and `cargo deny check`. This replaces
Article 7's `ctest` + clang-tidy local gate with the Rust-native equivalent chain; the
guarantee ("tests and lints are the merge gate, not review discretion alone") is unchanged.

- **Derives from:** Guideline 15.
- **Enforcement:** branch protection requires the pipeline green; cargo/target caching in CI
  (cold Bevy builds dominate cost) is an operational note, not a relaxation of the gate.

## Article XI — HITL gates (Inherited)

Spec-Kit pauses for human approval **between** `/speckit.plan` and `/speckit.tasks`, and
**between every `/speckit.implement` task**. Coding-agent handoffs require human review of
the produced PR before merge. Unchanged from Article 8 — this is a process guarantee with
no C++-specific mechanism to replace.

- **Derives from:** C++ constitution Article 8 (carried over unchanged).
- **Enforcement:** Spec-Kit workflow tooling (`/speckit.plan`, `/speckit.tasks`,
  `/speckit.implement` prompts) enforces the pause points; PR review is a required GitHub
  branch-protection check.

---

## Deliberately not promoted

- **Bevy version pin (HITL C-4).** Resolved as a rolling *policy* ("latest stable,
  re-verified per session," currently 0.16.x), not a fixed number — a version pin is an
  implementation detail that will rot inside a constitution article, so it lives in the
  plan/research docs instead.
- **`smallvec`'s exact standing (HITL C-2).** The *decision* (excluded from budget paths) is
  folded into Article IV's body rather than given its own article; it is a single-crate
  policy note, not a standalone guarantee.
- **Specific fixed-capacity crate names (`arrayvec`, `heapless`).** Named as enforcement
  detail under Article IV, not elevated to their own article — they are interchangeable
  implementations of the same "capacity is a construction-time decision" guarantee.
- **Guideline 1 (ownership replaces allocator plumbing) as a standalone article.** Folded
  into Article III's rationale rather than kept separate, since its only normative content
  ("don't invent an `Allocator`-trait port") is a boundary condition on Article III, not an
  independent guarantee.
- **Lint-tier policy detail (deny `correctness`, promote `suspicious`/`complexity`/`perf`/
  `style`, cherry-pick `pedantic`/`restriction`).** This is `Cargo.toml` configuration, not a
  constitutional guarantee; it is enforcement machinery for Articles I, II, III, VIII, and X
  and is referenced there rather than given its own article.
- **Cross-platform / lockstep-grade determinism (HITL C-3, rejected tier).** Explicitly out
  of scope per the HITL resolution; not promoted because promoting it would misstate the
  bar this workshop actually validated.
