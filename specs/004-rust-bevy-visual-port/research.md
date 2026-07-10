# Phase 0 Research: Rust/Bevy Visual Port

All items below were either resolved from prior validated session knowledge (the render spike
at `c:\code\bevy-render-spike`, and the 2026-07-07 Rust port session recorded in repo memory) or
decided directly from `specs/transform/*.spec.md` + `rust-constitution.md`. The spec (`spec.md`)
had zero outstanding `NEEDS CLARIFICATION` markers, so this phase focuses on closing
**implementation-technique** unknowns the spec deliberately left to planning.

## R1 — Rendering on a no-GPU VM

- **Decision**: Use Bevy's `DefaultPlugins` unmodified (full render + winit) for both run
  modes. Do not force `MinimalPlugins` or a headless render-to-texture path.
- **Rationale**: Verified working this session in a throwaway spike (`bevy-render-spike`):
  `DefaultPlugins` renders successfully through wgpu's automatic DX12 WARP software-adapter
  fallback on this VM (no hardware GPU), and
  `bevy::render::view::screenshot::{Screenshot, save_to_disk}` successfully wrote a PNG. No
  special adapter-selection code was required — wgpu picks WARP automatically when no hardware
  adapter is present on Windows/DX12.
- **Alternatives considered**: `MinimalPlugins` + manual render-to-texture (rejected — much
  higher implementation cost, and the spike already proves the simpler path works); forcing a
  software GL adapter via Mesa DLLs (rejected — that recipe was needed for the C++/raylib
  sandbox, not for wgpu/DX12, and is unnecessary here).
- **Risk carried forward**: WARP fallback is a *VM-specific* property of this environment. On a
  machine with a real GPU, `DefaultPlugins` will use hardware rendering instead — behaviorally
  equivalent for this feature, but re-verify if the crate is ever run in a different CI
  environment with neither a GPU nor a DX12 WARP-capable driver stack (e.g. some Linux CI
  runners lack any adapter at all). No code branches on this; it is purely an environment
  property of wherever `cargo run`/`cargo test` executes.

## R2 — Screenshot triggering mechanism

- **Decision**: `ScreenshotPlugin` counts elapsed `FixedUpdate` ticks in a resource; once the
  count reaches `SimConfig.warmup_frames`, a one-shot `Update` system spawns
  `Screenshot::primary_window()` with an `.observe(save_to_disk(output_path))` callback (Bevy
  0.16 API, confirmed in the spike), then requests `AppExit` once the observer reports the file
  was written (success) or failed (non-zero exit, per FR-011).
- **Rationale**: Matches FR-010/FR-011 exactly: no file before warm-up elapses, a clear
  non-panicking failure path, and a success-status exit for automation (FR-011, SC-002).
- **Alternatives considered**: Screenshotting on the very first frame the counter is reached
  inside `FixedUpdate` itself (rejected — screenshot capture must happen after the frame has
  actually been rendered, which happens in `Update`/render extraction, not inside the fixed
  sim step).

## R3 — CLI argument parsing

- **Decision**: `clap` 4, derive API, two run modes (`Windowed` default, `Screenshot { warmup:
  u32, out: PathBuf }`), plus a global `--seed <u64>` defaulting to 42.
- **Rationale**: `clap`'s derive macros give `#[must_use]`-friendly `Result`-based parsing
  (Article II) and produce clear `--help`/error output for free, satisfying FR-011's "clear,
  non-crashing failure" bar for bad CLI input as well as bad output paths.
- **Alternatives considered**: Hand-rolled `std::env::args()` parsing (rejected — more code, no
  free validation/help text, higher risk of an accidental `unwrap`/`expect` violating Article I).

## R4 — Zero-allocation verification (Article IV / FR-014)

- **Decision**: A `#[global_allocator]` wrapper (`CountingAllocator`, in `src/alloc.rs`) around
  `std::alloc::System` with two `AtomicUsize` counters (bytes allocated, bytes deallocated,
  incremented/read with `Ordering::SeqCst`). The `zero_alloc.rs` test builds a headless `App`
  with `MinimalPlugins` + `RngPlugin` + `PhysicsPlugin` + `FrameBudgetPlugin` (no rendering),
  runs a warm-up batch of `app.update()` calls, snapshots both counters, runs N more updates,
  and asserts the counters are unchanged.
- **Rationale**: Directly mirrors Article IV's enforcement clause and the C++ reference's
  `allocator::bytes_used()` before/after pattern (see repo memory: 001/005-particle-vfx-subsystem
  sessions used the identical technique). A global allocator is process-wide, so this test must
  run in its own test binary (`tests/zero_alloc.rs`) to avoid other tests' allocations polluting
  the counters — `cargo test` already isolates each `tests/*.rs` file into its own binary/process.
- **Alternatives considered**: `dhat`/heap-profiling crates (rejected — adds a dependency and
  output-parsing step for a check simple atomic counters answer directly); `assert_no_alloc`
  crate (rejected — its scoped guard aborts the whole process on violation rather than producing
  a normal test failure, which is harder to read in CI output).

## R5 — Deterministic snapshot/comparison for FR-012

- **Decision**: `SimulationSnapshot { seed: u64, tick: u32, bodies: Vec<(f64, f64, f64, f64)> }`
  (position x/y + velocity x/y per body, sorted by a stable body index, not by `Entity` — see
  data-model.md) built by a headless-`App` helper function shared between `determinism.rs` and
  future tooling. Two runs are compared via `PartialEq`/`assert_eq!` on the whole struct
  (`f64` bit-for-bit equality is expected and sufficient here since both runs execute the exact
  same deterministic arithmetic — no cross-platform claim is made, per FR-015).
  `TimeUpdateStrategy::ManualDuration` drives exactly N fixed ticks per run (confirmed working
  API from the 2026-07-07 session).
- **Rationale**: Directly satisfies US3/FR-012/FR-015 without needing any hashing scheme; a
  plain struct equality check is simpler and more debuggable (a failing assertion shows exactly
  which body's field diverged) than the C++ reference's opaque digest string.
- **Alternatives considered**: Porting the C++ trace-digest/hash approach 1:1 (rejected — flagged
  in repo memory from the 2026-07-07 session as producing a value that cannot be compared
  cross-language anyway, and the language-agnostic specs never require a specific digest
  algorithm — only reproducibility).

## R6 — Constraint iteration order (Article VI / physics-constraint.spec.md §4.1)

- **Decision**: Collect `(Entity, Entity, rest_length)` constraint tuples once per solve pass via
  a query, sort by the canonical key `(min(a, b), max(a, b))` using `Entity`'s built-in `Ord`
  impl (confirmed available in Bevy 0.16), then iterate in that fixed order for every solver
  iteration. Two-body position updates use `Query::get_many_mut([a, b])` (confirmed working
  Bevy 0.16 API) to get simultaneous mutable access to both endpoints.
- **Rationale**: This is a direct, literal transformation of physics-constraint.spec.md §4.1's
  "constraints kept sorted by canonical key" requirement — `Entity: Ord` means no custom ID type
  is needed (a case where the framework already delivers the spec's guarantee, per
  rust-guidelines' "ownership already delivers this concept, stop" principle applied to
  ordering instead of ownership).
- **Alternatives considered**: A custom `u64` body-id type mirroring the C++ spec's abstract
  `id` field exactly (rejected — adds a redundant parallel indexing scheme when `Entity` already
  provides a total order and a generation-safe handle in one type).

## R7 — Visual rendering technique for bodies/links/boundary

- **Decision**: `Gizmos::circle_2d` for the arena boundary and every body (anchor drawn in a
  visually distinct color/size per FR-004/SC-001), `Gizmos::line_2d` for every active constraint
  link. A single `Camera2d` is spawned once at startup.
- **Rationale**: Gizmos render every frame as part of the normal render graph (they appear in
  `Screenshot` captures — confirmed by nothing in the Bevy 0.16 docs excluding gizmos from the
  main pass), require no mesh/material asset setup, and keep the visuals system trivially
  allocation-free after startup (no per-frame entity spawning for lines).
- **Alternatives considered**: `Mesh2d` + `ColorMaterial` circle sprites for bodies (rejected for
  this feature — more asset-setup code for a cosmetic difference; Gizmos are legible enough for
  SC-001's "identify within 5 seconds" bar and match the constraint links' natural
  line-primitive representation, keeping the whole visuals module in one consistent style).

## Open Questions / Risks flagged for follow-up (non-blocking)

1. **Criterion perf-gate (Article VII)**: The constitution expects `criterion` baselines as a CI
   perf-regression gate. No functional requirement in this spec mandates a criterion benchmark
   (the frame-budget *resource* is required; a criterion *baseline comparison job* is a broader
   CI concern). Deferred to a future feature/CI-wiring task; not a gate for this plan.
2. **CI workflow file**: FR-001 only requires the crate be excluded from the CMake build, not
   that a GitHub Actions job be added. `quickstart.md` documents the exact local commands
   (`cargo fmt`, `clippy`, `test`, `deny check`) so a CI job can be added later without further
   design work.
3. **WARP-adapter portability**: flagged under R1 above — re-verify render path if this crate is
   ever executed on a CI runner other than this workshop VM.
4. **Body count**: spec Assumptions leave exact orbiting-body count (3–8) to planning; this plan
   fixes it at a `SimConfig` constant of 5 (visually legible, matches the "several" scenario
   language) with all 5 linked to the anchor directly (a simple hub topology) — sufficient for
   FR-004/SC-001 without over-specifying a topology the spec never mandates. This is an
   implementation default, not a new user-facing requirement, and may be changed freely during
   `/speckit.tasks`/implementation without a spec update.
