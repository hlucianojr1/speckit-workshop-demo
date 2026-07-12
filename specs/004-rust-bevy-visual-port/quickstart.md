# Quickstart: `rust-port/orbital-arena-rs`

Standalone Cargo project — not built by CMake, not referenced by the top-level `CMakeLists.txt`.
All commands below are run from `rust-port/orbital-arena-rs/`.

## Build

```powershell
cd rust-port/orbital-arena-rs
cargo build
```

First cold build pulls in Bevy's default (render+winit) features; expect a multi-minute first
compile (subsequent builds are incremental). Prior validated timing on this VM with
default-features on was not yet captured this session (the 2026-07-07 session's ~1m45s figure
was for `default-features = false`, headless-only — expect longer with rendering enabled).

## Run (windowed — human observation)

```powershell
cargo run                        # default scene: the Orbital Arena game (--scene arena)
cargo run -- --scene constraint  # the rope / VFX / free-particle scene (US6/US7)
```

`--scene arena` (the default) opens the Orbital Arena match (US8): two gravity wells, an
attracted particle field, and a match HUD (state/tick/scores/winner). `--scene constraint`
(alias `rope`) opens the original US1 scene this quickstart first described: one anchor
body, orbiting bodies, constraint links, and the arena boundary — now also carrying US6/US7's
exact rope geometry and full visual identity. On this no-hardware-GPU VM, wgpu automatically
falls back to the DX12 WARP software adapter (verified working) — no extra flags needed.

`pendulum`, `cloth`, and `storm` are accepted by `--scene` but exit with a clear "not
implemented yet" error (US9 remains open — see training doc §4.2e).

## Run (screenshot-capture mode — automated evidence)

```powershell
cargo run -- screenshot --warmup-frames 300 --out ../../docs/screenshots/part4-rust-arena.png
cargo run -- --scene constraint screenshot --warmup-frames 300 --out ../../docs/screenshots/part4-rust-rope.png
```

Runs 300 fixed-timestep ticks (5 seconds of sim time at 60 Hz) of whichever `--scene` is
selected (default `arena`), then writes the PNG and exits 0 (US2, SC-002).

## Run (headless digest mode — cross-language golden check, US5)

```powershell
cargo run -- headless --scene constraint --seed 42 --frames 600 --out trace.csv
```

Simulates `--frames` fixed steps with no window and no GPU (`MinimalPlugins`), writes a
per-frame CSV trace, and prints `trace_digest=<16 hex> frames=<N> scene=<name>` — the
cross-language acceptance instrument behind `sandbox-scenes.spec.md` §8. `--scene
constraint` (the rope scene) reproduces the C++ reference's golden digest
`9dc3bd72a4f7f31a` bit-for-bit; `--scene arena` (the Orbital Arena game) runs but does
**not** currently reproduce the reference's `919d2feba5bdbeac` bit-for-bit — see the
training doc's §4.2e for the root-caused explanation (a documented, spec-sanctioned
tolerance-fallback finding, not an unexplained bug). `pendulum`/`cloth`/`storm` exit with a
clear argument error (US9 not yet implemented).

## Test

```powershell
cargo test
```

109 tests total (71 library unit tests + 38 integration tests across the files below), all
headless (`MinimalPlugins` or no Bevy `App` at all) — no window, no GPU required, safe to run
in any CI environment:

- `tests/determinism.rs` — same seed twice ⇒ identical snapshot; different seed ⇒ differs (FR-012).
- `tests/constraint_solver.rs` — separation converges toward rest length; anchor never moves (FR-013).
- `tests/zero_alloc.rs` — zero allocation delta across N post-warm-up `FixedUpdate` frames (FR-014).
- `tests/digest.rs` / `tests/headless_csv.rs` — FNV-1a digest correctness and the headless CSV's exact format (US5).
- `tests/rope_scene.rs` — exact rope geometry, constraint count, and free-particle rng draw order (US6).
- `tests/match_machine.rs` / `tests/powerups.rs` / `tests/input_log_snapshot.rs` — the Orbital Arena match state machine, power-up spawn/effect/consumption rules, and input-log/snapshot-hash contracts (US8).

If cold-build parallelism causes `rustc` to be killed by an out-of-memory error on a
memory-constrained VM, retry with `cargo test -j 2` (or lower) to reduce parallel codegen
units.

## Lint / format / dependency gates (Article X)

```powershell
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo deny check
```

`cargo deny` config (`deny.toml`) enforces Article VI's ban on `OsRng`/non-`StdRng` RNG engines
and Article II's ban on `anyhow` in sim-path dependencies.

## Verifying the CMake-exclusion requirement (FR-001)

```powershell
cd C:\code\speckit-workshop-demo
Select-String -Path CMakeLists.txt,cmake\*.cmake -Pattern "rust-port" -Quiet
# expect: no output / $false — rust-port/ is never referenced
```

## Known limitations (as of US8 / Phase 11)

- **US9 (pendulum tower, cloth, particle storm scenes) and US10 (full control map /
  app-loop parity) are not implemented.** `--scene pendulum|cloth|storm` is accepted by the
  CLI and exits 3 with a clear message rather than crashing; only the LMB well-drag control
  exists (no pause/step/speed controls, perf-bomb demo, shockwave, screen shake, or cursor
  gravity well).
- **The `arena` scene's headless digest is not bit-exact** against the C++ reference
  (`919d2feba5bdbeac`) — see the training doc's §4.2e for the root-caused explanation. The
  `constraint` (rope) scene's digest IS bit-exact.
- See `specs/004-rust-bevy-visual-port/tasks.md` Phases 12–14 for the tracked follow-up work.
