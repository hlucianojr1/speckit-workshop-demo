# Rust Game-Development Guidelines

> **Status:** APPROVED (HITL review 2026-07-10) — feeds `/speckit.constitution` (§4.3.1).
> All 5 conflicts flagged during consolidation are resolved in the **HITL Resolutions**
> section at the end of this document; guideline bodies above are left as originally
> drafted (with their conflict flags still visible) so students can see what a real
> review looks like, not a silently-cleaned-up final answer.
> **Consolidates:** `specs/transform/research/r1-architecture.md`, `r2-memory.md`,
> `r3-determinism.md`, `r4-errors-api.md`, `r5-testing.md`, `c1-eastl-concepts.md`.
> **Principle:** honor the EASTL *concepts* (C1), not EASTL's mechanisms. Where Rust's
> ownership model already delivers a concept, the guideline says so and stops. Where Rust's
> defaults would silently break a guarantee (e.g. hidden `Vec` growth mid-frame), the
> guideline names the idiom that restores it and how to enforce it.
> **Conflicts between research notes are flagged inline and collected at the end — they are
> HITL review items, not resolved here.**

---

## 1. Trust ownership for memory lifetime — do not rebuild allocator plumbing

- **Guideline:** Rely on ownership, borrowing, and `Drop` for memory lifetime and attribution; do not port EASTL-style per-container allocator parameters.
- **Derives from:** C1 Concept 1 (explicit memory ownership) + C1's exclusion list (allocator template params are C++ mechanics); R2 §4.
- **Rust idiom:** *Ownership already delivers this concept — stop.* Every value has exactly one named owner; leaks/double-frees are type-system errors, not allocator-discipline errors. The `Allocator` trait is nightly-unstable and not idiomatic stable Rust (R2 §4). Subsystem "ownership boundaries" become module/plugin boundaries owning their own collections.
- **Enforcement:** Code review: any `Vec<T, A>`-style allocator generics or hand-rolled allocator adapters are rejected. `#![forbid(unsafe_code)]` (Guideline 9) closes the raw-pointer escape hatch.

## 2. Allocate capacity up front; the steady-state frame loop allocates zero bytes

- **Guideline:** All frame-loop collections are sized at startup/load with `Vec::with_capacity` (or fixed-capacity types, Guideline 3) and reused via `clear()`; no collection may grow during steady-state frames.
- **Derives from:** C1 Concept 2 (allocation visible and budgeted, never hidden) + Concept 3 (fixed capacity decided up front); R2 §1–§2. This is the headline case where Rust does **not** deliver the guarantee for free: `Vec::push`, `format!`, `.collect()`, `HashMap` growth, and `Box<dyn Fn>` all hit the global allocator silently.
- **Rust idiom:** `with_capacity(worst_case)` at startup; hoist scratch buffers out of loops and `clear()` them; per-frame scratch phases may use a pre-sized `bumpalo::Bump` reset at frame end (R2 §1 caveats: no `Drop` on plain alloc, `!Sync`, size for worst case). Avoid `String`/`format!`/`collect()` in hot systems; prefer `&str` and pre-sized buffers.
- **Enforcement:** the counting-allocator test gate of Guideline 4; `debug_assert!(v.len() < v.capacity())` where capacity is a hard budget; review checklist for allocation-prone APIs in systems added to `FixedUpdate`.

## 3. Use fixed-capacity collections with fallible push for hard budgets

- **Guideline:** Where a capacity is a design-time budget (max contacts, max particles, ring buffers), encode it in the type with `arrayvec`/`heapless`, whose `try_push` returns a value on exhaustion.
- **Derives from:** C1 Concept 3 (capacity is a construction-time decision) + Concept 2 (exhaustion returns status, never aborts); R2 §3.
- **Rust idiom:** `arrayvec::ArrayVec<T, CAP>` / `heapless::Vec<T, CAP>` (true constant-time push, capacity in the type signature — the budget becomes a reviewable number in code); `heapless::HistoryBuf` or `[T; N]` + head index for rolling windows (frame-budget analogue). `smallvec` is a locality optimization only — its silent heap spill defeats the budget guarantee (R2 §3). ⚠️ *Conflict C-2 below: R5 lists smallvec alongside the budget tools.*
- **Enforcement:** clippy lint table denies bare `push` on budget-bearing types where `try_push` exists (review-enforced); `cargo-deny` bans direct `smallvec` use in sim crates if HITL adopts the strict reading; allocation test gate (Guideline 4) catches spills.

## 4. Make allocation observable: counting global allocator + zero-alloc frame tests

- **Guideline:** Memory behavior must be measurable at runtime and asserted in tests: wrap the global allocator in a counting `#[global_allocator]` and CI-gate "N warm frames ⇒ zero allocation delta."
- **Derives from:** C1 Concept 4 (allocation observability — `bytes_used()`, `capacity()`, named allocators); R2 §2; R5 §5. Rust has no per-subsystem `bytes_used()` without allocator plumbing (rejected by Guideline 1), so observability moves from *per-container* to *per-process + per-test-scope*.
- **Rust idiom:** `#[global_allocator]` wrapping `System` with `AtomicUsize` counters; `assert_no_alloc` (in `warn_debug` mode under `cargo test`) or `dhat-rs` for locating offenders. Headless integration test: build app, run warm-up frames (first ticks legitimately grow archetype storage — R5 §5), snapshot counter, run N frames via `app.update()`, assert delta == 0.
- **Enforcement:** dedicated CI test job, run `--test-threads=1` (global-instrument noise, R5 §5); regression = red build.

## 5. Capacity exhaustion is a normal, partial-success outcome — degrade, don't fail

- **Guideline:** Hitting a budget limit in the frame loop returns how much work *was* done (e.g. "spawned k of n requested") and the game continues degraded; it never panics, errors the frame, or aborts.
- **Derives from:** C1 Concept 6 (graceful degradation at capacity limits — `try_spawn` semantics); R4 §2 (fallible-but-ignorable per-frame operations).
- **Rust idiom:** return a small `#[must_use]` result struct/enum carrying the achieved count and a status (the honest analogue of C++ `emit_result{spawned, status}`), or `Result<(), E>` where "all-or-nothing" is genuinely the contract. ⚠️ *Conflict C-5 below: R4 leans `Result` by default; C1 Concept 6 is inherently partial-success — which surfaces this needs is a design decision per API.*
- **Enforcement:** `unused_must_use` denied (Guideline 8); GTest-analogue: every capacity-limited API gets a test driving it past its limit and asserting the game-visible degradation (fewer particles, not a crash).

## 6. No panics in sim or system code; tiered panic policy

- **Guideline:** `unwrap`/`expect`/`panic!`/`todo!`/slice-indexing panics are forbidden in per-frame sim and system code; panics are permitted at startup/asset-load (fail fast) and in tests (assertion mechanism).
- **Derives from:** C1 Concept 2 ("never throws or aborts; failure handleable at 60 Hz without unwinding") — the direct heir of the C++ no-exceptions rule; R4 §1 (a panicking Bevy system takes down the whole `App`; `panic = "abort"` makes strays worse, not softer).
- **Rust idiom:** `Result`/`Option` with `?`, `Option` defaults, clamping, entity-skip + logged diagnostic; `get()` instead of `[]` in hot paths; document any residual panic in a `# Panics` rustdoc section (API Guidelines C-FAILURE).
- **Enforcement:** `[lints.clippy]` in `Cargo.toml`: deny `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `indexing_slicing` on sim crates, `#[allow]`ed in `#[cfg(test)]`; `cargo clippy --all-targets -- -D warnings` in CI.

## 7. Errors are values: `Result`/`Option` with small `Copy` error enums — never port status codes verbatim, never `anyhow` in sim

- **Guideline:** Map C++ status enums to Rust error enums used as the `E` of `Result<T, E>`, and C++ null/sentinel returns to `Option<T>`; keep sim-path error types small, fieldless, and `Copy`.
- **Derives from:** C1 Concept 2 (fallible creation is a value); R4 §2. Rust *upgrades* the C++ pattern: consumption is compiler-enforced (`unused_must_use`), propagation is `?` — this part of the concept comes free.
- **Rust idiom:** `#[derive(Debug, Clone, Copy, PartialEq)] enum XError` for engine-internal hot paths (zero-cost, no `Box<dyn Error>`); `thiserror` for derive convenience at library boundaries; `anyhow` confined to bin targets/tooling (allocation + type erasure — R4 §2). `Option` = absence is normal; `Result` = absence is a failure the caller must handle.
- **Enforcement:** `cargo-deny` bans `anyhow` as a dependency of sim crates; review: error enums in hot paths must be `Copy`.

## 8. `#[must_use]` on factories, guards, and bare status returns — `Result` gives it free

- **Guideline:** Every factory, builder, guard/handle type, and bare status return carries `#[must_use]` (with a message where dropping is a real bug); intentional discard is spelled `let _ = f();`.
- **Derives from:** the C++ constitution's `[[nodiscard]]`-on-factories rule via C1; R4 §3. `Result` and `Option` are already `#[must_use]` in std — for `Result`-returning APIs the concept is *automatic; stop*. The explicit annotation is only needed for the remaining cases.
- **Rust idiom:** `#[must_use = "handle leaks pool slot if dropped"]` on the *type* for guards/handles; on the *function* for pure computations and bare status enums (Guideline 5).
- **Enforcement:** `unused_must_use` promoted to deny (`-D warnings` / lint table); `clippy::must_use_candidate` (pedantic, cherry-picked) suggests missing annotations on public APIs.

## 9. Forbid `unsafe`; where truly unavoidable, isolate it with `// SAFETY:` discipline

- **Guideline:** Game crates declare `#![forbid(unsafe_code)]`; any future exemption (SIMD, GPU buffer casts) lives in one small, heavily tested module with a `// SAFETY:` comment per block and a `# Safety` rustdoc section per `unsafe fn`.
- **Derives from:** the memory-safety intent behind C1 Concepts 1–2; R4 §4. *Ownership + slices already replace the pointer arithmetic that motivated EASTL-era discipline — for a no-FFI game port, this concept is delivered by the language; stop.*
- **Rust idiom:** `#![forbid(unsafe_code)]` at crate root; std convention for exemptions (`unsafe_op_in_unsafe_fn` enabled).
- **Enforcement:** the `forbid` attribute itself (compile error, cannot be `allow`ed downstream); `clippy::undocumented_unsafe_blocks` + `clippy::missing_safety_doc` for any exempted module; optional `cargo-geiger` for dependency auditing.

## 10. RNG: explicitly seeded, pinned-algorithm, one stream per subsystem, held as a resource

- **Guideline:** All sim randomness comes from an explicitly seeded RNG of a *named, version-pinned algorithm* stored as an ECS resource, with an independent stream per subsystem (physics, VFX, AI); OS entropy is banned in sim paths.
- **Derives from:** C1 Concept 5 / constitution Article 5 (seeded determinism; the C++ side uses `engine_demo::sim::rng`); R3 §1.
- **Rust idiom:** `rand_chacha::ChaCha8Rng`/`ChaCha12Rng` (or `rand_xoshiro`) with the crate version pinned — **not** `StdRng`, which is documented non-portable and exempt from the Rand reproducibility policy (R3 §1). Seed via `SeedableRng::from_seed`/`seed_from_u64`. Avoid sampling `usize`/`isize` (width varies per target). ⚠️ *Conflict C-1 below: R5's replay-test pattern uses `StdRng::seed_from_u64` — contradicts R3.*
- **Enforcement:** `cargo-deny`/review bans `rand::rngs::OsRng`, `thread_rng`, and `StdRng` in sim crates; exact rand-family versions pinned in `Cargo.toml`; the determinism replay test (Guideline 15) catches regressions.

## 11. Deterministic iteration and ordering: sort explicitly, order systems explicitly, ban hash-order in sim

- **Guideline:** Any order-sensitive sim computation iterates in an explicitly established order — never `HashMap`/`HashSet` iteration order, never raw ECS query order, never ambiguous parallel system order.
- **Derives from:** C1 Concept 5 (deterministic container behavior; the BUG-004 sorted-constraint fix); R3 §3; R1 §2. This is the second headline case where Rust/Bevy defaults **break** the guarantee: `RandomState` hashing is per-process random, query order follows archetype history, and the parallel executor's order "could even change every frame."
- **Rust idiom:** `BTreeMap` or `Vec` + sort-by-stable-key (game-assigned ID, not `Entity` bits alone) before order-sensitive math; `.chain()`/`.before()`/`.after()` on every pair of sim systems with conflicting access; `ScheduleBuildSettings::ambiguity_detection = LogLevel::Error` on sim schedules (default is `Ignore` — must be opted into); serial iteration (no `par_iter`) on determinism-critical reductions.
- **Enforcement:** ambiguity detection set to `Error` (turns silent hazards into build/test failures); review rule: no `std::collections::HashMap` iteration in sim modules (grep-able); replay test (Guideline 15).

## 12. Simulation runs in `FixedUpdate` with `f64` accumulators; `f32` only at the point of use

- **Guideline:** All gameplay/physics state mutation lives in the fixed-timestep schedule; time, energy, and score accumulators are `f64`, converted to `f32` only where component math requires it.
- **Derives from:** constitution Article 5 (`double` accumulators) via C1; R3 §2; R1 §2 (`FixedUpdate` is the documented home for physics/AI/rules; `Update` is variable-rate). R3's clarification carries over: `f64` is about *drift*, ordering is about *determinism* — both rules are needed.
- **Rust idiom:** systems added to `FixedUpdate` (input sampling, camera, interpolation, UI stay in `Update` and forward via messages/resources); `Time::delta_secs_f64()`; `Time<Fixed>` configured once (e.g. 60 Hz); no gameplay in `PreUpdate`/`PostUpdate` engine phases.
- **Enforcement:** review checklist: any `Query`-mutating gameplay system registered outside `FixedUpdate` needs written justification; headless tests step exact fixed ticks via `TimeUpdateStrategy::ManualDuration` (R5 §1), so misplaced logic simply fails the deterministic-replay test.

## 13. Closed sums stay closed: enums by value in dense storage, no trait objects in hot paths

- **Guideline:** Heterogeneous-but-closed sets (emitter shapes, force kinds) are Rust `enum`s stored by value in contiguous collections; `Box<dyn Trait>` and open-ended dispatch are banned from inner loops.
- **Derives from:** C1 Concept 7 (value-based closed sums, no dynamic dispatch).
- **Rust idiom:** *Rust largely delivers this concept natively — `enum` + `match` is the language's default sum type, and exhaustiveness checking is stronger than C++ `variant` visitation; mostly stop.* The residual rule is negative: don't reach for `Box<dyn Trait>` out of OO habit — it reintroduces pointer-chasing and heap allocation (violating Guideline 2) exactly where C++ banned vtables.
- **Enforcement:** review: `dyn` in sim-crate hot paths requires justification; `match` without wildcard arms on closed sim enums so new variants are compile errors; allocation gate (Guideline 4) catches boxed-trait storage.

## 14. One plugin per subsystem, headless-testable by construction

- **Guideline:** Each subsystem (physics, scoring, VFX, input) is a Bevy plugin owning all of its registrations, resources, and schedule placement, and must be addable to a `MinimalPlugins`/bare `App` test without dragging in rendering.
- **Derives from:** R1 §3 (plugin decomposition; cross-plugin contracts = public components, events/messages, resources, named system sets); R5 §1; the subsystem-budget attribution intent of C1 Concept 1. Also R1 §4: don't force ECS on singletons (RNG, frame budget, score) — plain structs in resources, unit-testable without a `World`.
- **Rust idiom:** `impl Plugin for PhysicsPlugin` per subsystem; test pattern from Bevy's own `how_to_test_systems.rs`: build minimal `App`, insert only needed resources/messages, drive with `app.update()` (never `app.run()` under `MinimalPlugins` — its runner loops forever); simulate input via `ButtonInput::press()`. ⚠️ *Conflict C-4 below: Bevy version pin determines whether the events API is unified or split into buffered Messages vs observer Events.*
- **Enforcement:** every plugin ships at least one headless integration test (the GTest-per-public-function Article 7 analogue); CI runs the full test suite windowless.

## 15. CI is the constitution's teeth: fmt → clippy(-D warnings) → tests (incl. determinism replay + zero-alloc) → cargo-deny; perf via criterion baselines, never wall-clock asserts

- **Guideline:** Every merge passes, in fail-fast order: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings` (with the lint tables from Guidelines 6–8 in `Cargo.toml` so local == CI), the full test suite including the deterministic-replay hash test and the zero-allocation frame test, and `cargo deny check`; performance regressions are tracked with criterion baselines (or `iai-callgrind` instruction counts), not `assert!(elapsed < 16ms)` in `cargo test`.
- **Derives from:** R5 §2–§4; R4 §5 (lint-tier table: keep `correctness` deny, promote `suspicious`/`complexity`/`perf`/`style` to deny in CI, cherry-pick `pedantic`/`restriction` — never enable those groups wholesale); C1 Concept 4 (budgets are only enforceable if measurable); constitution Article 7 (every public function tested).
- **Rust idiom:** replay test = seed → step N fixed ticks → hash state via `to_bits()` → rerun → assert identical (mirrors the C++ trace-digest test; validated in the 07-07 port). Frame-budget monitoring stays an in-game debug resource (rolling window, `frame_budget.h` analogue), not a unit-test assertion — shared CI runners make wall-clock asserts flaky. `#[expect(lint, reason = "...")]` for justified lint exceptions (warns when stale). Cache cargo/target in CI: Bevy cold builds dominate cost.
- **Enforcement:** this guideline *is* the enforcement layer; branch protection requires the pipeline green.

---

## Conflicts flagged for HITL review

These are contradictions or unresolved decisions *between research notes*. They are not
resolved in this document; the constitution generation step (§4.3.1) must not proceed until
a reviewer picks a side.

- **C-1 — RNG algorithm: `StdRng` vs pinned algorithm crate.** R3 §1 (with live rand-docs
  citations) recommends **against** `StdRng` for anything persisted or compared across runs
  (non-portable, exempt from reproducibility policy) and prescribes `rand_chacha`/
  `rand_xoshiro` pinned. R5 §2's replay-test pattern — and the validated 07-07 port — use
  `StdRng::seed_from_u64`. Both work for *same-binary* determinism; they diverge for replay
  files, lockstep, and cross-version golden traces. **Decision needed:** which reproducibility
  bar (see C-3) — that answer picks the crate. Guideline 10 currently follows R3.
- **C-2 — `smallvec`'s standing.** R2 §3 explicitly warns `smallvec` "defeats no-per-frame-
  allocation guarantees" (silent heap spill) and demotes it to a locality optimization; R5 §5
  lists it alongside `with_capacity`/`arrayvec` as a preferred prevention idiom. **Decision
  needed:** allow `smallvec` in sim crates (trusting the alloc test gate to catch spills) or
  ban it from budget-bearing paths. Guideline 3 currently follows R2's stricter reading.
- **C-3 — Determinism bar unspecified.** R3 §4 offers three tiers (same-platform / cross-
  platform bitwise / fully portable lockstep) and explicitly says "state which bar the spec
  targets" — no other note answers. The choice changes Guideline 10 (RNG crate), Guideline 11
  (whether `libm` replaces std transcendentals), and Guideline 15 (whether golden hashes run
  per-target in CI). **Decision needed** before constitution generation.
- **C-4 — Bevy version pin and the events API split.** R1 was researched against Bevy 0.19
  docs (events split into buffered Messages vs observer `Event`s); R5 and the validated port
  used Bevy 0.16 (unified events API), and the training doc pins an older version still.
  Idiom names in Guidelines 12/14 (and any constitution article naming event APIs) depend on
  the pin. **Decision needed:** pin the workshop Bevy version; re-verify API names against it.
- **C-5 — Partial success vs `Result` as default fallible surface.** R4 §2 recommends
  "prefer `Result<(), E>` unless profiling or ergonomics argue otherwise"; C1 Concept 6
  defines capacity exhaustion as a *normal partial-success outcome* carrying an achieved
  count (`try_spawn` spawns k-of-n) — which a bare `Result<(), E>` cannot express. R4 itself
  concedes the `#[must_use]` status-struct escape hatch. **Decision needed:** a stated rule
  for when an API is all-or-nothing `Result` vs count-and-status partial success, so the
  constitution doesn't encode a blanket answer. Guideline 5 currently permits both with the
  partial-success form as the capacity-path default.

---

## HITL Resolutions (2026-07-10)

These are binding decisions for the constitution-generation prompt (§4.3.1). Each closes
one conflict flagged above; the constitution should cite the resolution, not re-litigate it.

- **C-1 (RNG crate) → resolved: `rand::StdRng::seed_from_u64`, not a pinned algorithm crate.**
  This training's cross-language bar is *behavioral* determinism (same seed ⇒ same stream,
  reproducible within one build), not cross-*version* wire compatibility of a serialized
  replay format — no shipped product depends on this repo's traces surviving a `rand`
  upgrade. Revisit if a real save-game/replay-file format is ever introduced.
- **C-2 (smallvec) → resolved: excluded from the budget-guideline list; permitted only as a
  non-sim, non-budget locality optimization (e.g. transient editor tooling), never in
  `FixedUpdate` systems.** R2's objection (silent heap spill defeats the whole point of a
  declared budget) wins over R5's more casual inclusion.
- **C-3 (determinism bar) → resolved: same-platform, same-build bitwise determinism
  only.** Matches this workshop's actual validated bar (run-twice hash equality at a fixed
  seed, §4.7's finding from the 07-07 test run) — not cross-platform bitwise, not lockstep
  netcode. Cross-platform bitwise (banning libm transcendentals/FMA) is out of scope unless
  a future spec explicitly requires netcode-grade replay.
- **C-4 (Bevy version) → resolved: pin to the latest stable Bevy at implementation time
  (0.16.x confirmed working in this session's render spike and the 07-07 port), not the
  training doc's stale 0.13 example.** The constitution should name a version *policy*
  ("latest stable, re-verified per session") rather than hard-coding a number that will rot.
- **C-5 (partial success vs `Result`) → resolved: stated rule — capacity/budget-bearing
  APIs (spawn/emit/allocate-from-pool) return a count-and-status value (partial success is
  normal, per C1 Concept 6); all other fallible operations return plain `Result<T, E>`.**
  This mirrors the C++ side's own split (`try_spawn` returns a count; `allocate` returns
  null) and gives the constitution a crisp per-API-class rule instead of a blanket policy.
