# Plugin Contracts

Per Article IX, each subsystem is one `Plugin`, addable to a bare/`MinimalPlugins` `App` without
pulling in rendering, and each ships at least one headless integration test. This is the
per-plugin interface every `/speckit.tasks` task must honor.

## `RngPlugin`

- **Adds**: `DeterministicRng` resource (`Startup`, before any system that consumes randomness).
- **Reads `SimConfig.seed`.**
- **Headless-testable**: yes — no rendering dependency.
- **Test**: constructing two `App`s with the same seed yields identical first-N `next_u32`/
  `gen::<f64>()` draws (supports research.md R5's snapshot equality indirectly).

## `PhysicsPlugin`

- **Adds**: `Body`, `Anchor`, `ConstraintLink` components; a `Startup` system spawning bodies +
  links; a `FixedUpdate` system running the sorted-order solver (data-model.md, research.md R6).
- **Reads**: `DeterministicRng` (for initial jitter), `SimConfig`.
- **Headless-testable**: yes.
- **Tests**: `tests/constraint_solver.rs` (FR-013 — separation converges, anchor never moves);
  `tests/determinism.rs` uses this plugin's spawned state as the thing being snapshotted.

## `FrameBudgetPlugin`

- **Adds**: `FrameBudget` resource; an `Update` system recording each frame's duration.
- **Headless-testable**: yes (no window needed to measure `Time::delta()`).
- **Test**: recording N synthetic samples produces the expected rolling average (mirrors
  frame-budget.spec.md §4 exactly, including the warm-up-counted-once rule).

## `VisualsPlugin`

- **Adds**: `Startup` system spawning `Camera2d`; an `Update` system drawing bodies/links/
  boundary via `Gizmos` (research.md R7).
- **Requires rendering** — this plugin is only added in the actual binary (`main.rs`), never in
  headless tests. Its own correctness (does a circle appear where a body is) is verified visually
  via the screenshot deliverable, not via an automated pixel test — out of scope for this
  feature's automated test suite (spec's Independent Tests for US1 are manual/visual by design).

## `ScreenshotPlugin`

- **Adds**: `ScreenshotState` resource (only when `SimConfig.run_mode` is `Screenshot`); a
  `FixedUpdate` system incrementing `fixed_ticks_elapsed`; an `Update` system that, once the
  warm-up count is reached and `!triggered`, spawns the `Screenshot` + `save_to_disk` observer
  and sets `triggered = true`.
- **Requires rendering** (screenshots need a rendered frame) — not added to headless test `App`s.
- **Manual/integration verification**: exercised by actually running
  `cargo run -- screenshot --warmup-frames N --out path.png` (documented in quickstart.md), since
  it depends on the real render pipeline, not something a headless `MinimalPlugins` test can
  observe.

## Composition

`main.rs` builds: `DefaultPlugins` + `RngPlugin` + `PhysicsPlugin` + `FrameBudgetPlugin` +
`VisualsPlugin` + (`ScreenshotPlugin` iff `RunMode::Screenshot`).

Headless tests build: `MinimalPlugins` + `RngPlugin` + `PhysicsPlugin` (+ `FrameBudgetPlugin`
where relevant) — never `VisualsPlugin`/`ScreenshotPlugin`/`DefaultPlugins`.
