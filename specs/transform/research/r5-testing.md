# R5 — Testing & CI Practices for Rust Games (Bevy)

Research note for the C++ → Rust/Bevy transformation. No code committed from this note.

## 1. Headless Bevy App tests (system-level integration)

- Bevy's `App` runs fully headless: build with `MinimalPlugins` (TaskPoolPlugin, FrameCountPlugin,
  TimePlugin, ScheduleRunnerPlugin — no window, no renderer) or even a bare `App::new()` plus only
  the resources/messages a system needs. [Bevy docs: `MinimalPlugins`](https://docs.rs/bevy/latest/bevy/struct.MinimalPlugins.html)
- The canonical pattern (from Bevy's own [`tests/how_to_test_systems.rs`](https://github.com/bevyengine/bevy/blob/main/tests/how_to_test_systems.rs)):
  1. `let mut app = App::new();` insert resources, register events/messages, `add_systems(...)`.
  2. Spawn test entities via `app.world_mut().spawn(...)`.
  3. Drive frames explicitly with `app.update()` — one call = one full schedule pass.
  4. Assert against `app.world().get::<Component>(entity)`, resources, and event queues.
- For fixed-timestep logic, insert `Time<Fixed>` and advance time manually
  (`TimeUpdateStrategy::ManualDuration`) so tests step exact 60 Hz ticks instead of wall-clock time
  [from training knowledge — verified working on Bevy 0.16 in the 2026-07-07 dry run].
- Input can be simulated by inserting `ButtonInput<KeyCode>` and calling `.press()` / `.clear()`
  between updates (same Bevy test file). This replaces the C++ headless-harness approach
  (`ea-sandbox --headless`) with in-process integration tests — cheaper and CI-friendly.
- Trade-off: `MinimalPlugins`' `ScheduleRunnerPlugin` loops forever by default; in tests never call
  `app.run()` — call `app.update()` N times, or use `ScheduleRunnerPlugin::run_once`.

## 2. Deterministic replay tests

- Pattern: seed a `Resource`-wrapped RNG (`StdRng::seed_from_u64(SEED)`), step the app a fixed
  number of `FixedUpdate` ticks, hash the observable state (positions/velocities quantized or
  bit-exact via `to_bits()`), then repeat the whole run and assert identical hashes
  [from training knowledge — mirrors this repo's C++ trace-digest test; validated in Rust on 07-07:
  identical `state_hash` across two binary invocations].
- Sources of nondeterminism to control: query iteration order (sort by `Entity` or a stable key
  before order-sensitive math), parallel system ordering (use explicit `.chain()`/`.before()`),
  `HashMap` iteration (randomized hasher — use `BTreeMap` or sorted drains on sim paths),
  and `f32` fused-multiply-add differences across targets [from training knowledge].
- Replay-file variant: record input events per tick, replay them into a fresh app, compare final
  hash. Bevy ships `CiTestingPlugin` (feature `bevy_ci_testing`) for scripted frame-driven CI runs
  ([Bevy docs, MinimalPlugins page](https://docs.rs/bevy/latest/bevy/struct.MinimalPlugins.html)).

## 3. Perf assertions in tests vs criterion benchmarks

- **Criterion** is the standard: statistics-driven micro-benchmarking that stores results run-to-run
  and auto-detects regressions ([Criterion.rs book](https://bheisler.github.io/criterion.rs/book/criterion_rs.html)).
  Put solver/step benches in `benches/`, run `cargo bench` locally or on a dedicated perf job.
- **Do not** put hard wall-clock assertions (`assert!(elapsed < 16ms)`) in `cargo test`: shared CI
  runners have noisy neighbors → flaky tests [from training knowledge]. Acceptable compromises:
  - Assert *algorithmic* budgets instead (iteration counts, entity caps, zero allocations — §5).
  - Keep a frame-budget `Resource` (rolling-window average, like this repo's `frame_budget.h`) and
    assert it *in-game* behind a debug flag, not in unit tests.
  - If CI perf gating is required, use criterion baselines (`--save-baseline` / compare) or
    `iai-callgrind` (instruction counts — deterministic, CI-safe) [from training knowledge].

## 4. Lint/format/supply-chain gates in CI

- `cargo fmt --all -- --check` — formatting gate, zero-config [from training knowledge; cargo docs].
- `cargo clippy --all-targets --all-features -- -D warnings` — deny all warnings; optionally enable
  `clippy::pedantic` selectively and `#![deny(clippy::unwrap_used, clippy::expect_used)]` on game
  logic crates (maps to this repo's no-unwrap gate) [from training knowledge].
- `cargo deny check` — lints the dependency graph: licenses, security advisories (RustSec), banned
  crates, duplicate versions. Official GitHub Action: `EmbarkStudios/cargo-deny-action`
  ([cargo-deny book](https://embarkstudios.github.io/cargo-deny/)). Embark uses it for game projects.
- Suggested job order (fail fast, cheap → expensive): fmt → clippy → test → deny → (nightly) bench.
- Cache `~/.cargo` + `target/` (e.g. `Swatinem/rust-cache`); Bevy cold builds are the dominant CI
  cost [from training knowledge — 5–15 min cold compile observed with default features].

## 5. Detecting per-frame allocations in tests

Maps to constitution Article 6 / Article 8 ("no allocation in inner loops"). Options:

- **Counting `GlobalAlloc` wrapper** (recommended for this project): a `#[global_allocator]` that
  wraps `System` and bumps an `AtomicUsize` per alloc. Test = snapshot counter → run N warm frames
  via `app.update()` → assert counter unchanged. Warm-up frames are essential: first ticks
  legitimately grow `Vec`s/archetype storage [from training knowledge].
- **`assert_no_alloc` crate**: panics (or aborts) if any allocation occurs inside a closure —
  sharper failure location, but panic-on-alloc interacts badly with `cargo test`'s unwinding;
  use `warn_debug` mode in tests [from training knowledge].
- **`dhat-rs`**: heap profiling with programmatic assertions (`dhat::assertions` on block counts);
  heavier, better for diagnosing *where* the allocation came from [from training knowledge].
- Caveat: these instruments the test binary globally, so run alloc tests single-threaded
  (`--test-threads=1`) or key counters by thread to avoid cross-test noise [from training knowledge].
- Complementary static approach: prefer `with_capacity`/`arrayvec`/`smallvec` per R2, and review
  `clippy::large_stack_arrays` / allocation-prone APIs in hot systems.

## Sources

- Live: Bevy `MinimalPlugins` rustdoc (docs.rs, v0.19); Bevy `tests/how_to_test_systems.rs` (github.com/bevyengine/bevy, main); Criterion.rs book (bheisler.github.io); cargo-deny book (embarkstudios.github.io).
- [from training knowledge]: `TimeUpdateStrategy::ManualDuration` test stepping; nondeterminism sources; wall-clock-assert flakiness & iai-callgrind; clippy lint tiers; alloc-detection crates (`assert_no_alloc`, `dhat-rs`) and warm-up caveat; CI caching. Marked inline above.
