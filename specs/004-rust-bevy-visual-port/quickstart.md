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
cargo run
```

Opens a window: one anchor body, orbiting bodies, constraint links, and the arena boundary,
animating continuously (US1). On this no-hardware-GPU VM, wgpu automatically falls back to the
DX12 WARP software adapter (verified working this session) — no extra flags needed.

## Run (screenshot-capture mode — automated evidence)

```powershell
cargo run -- screenshot --warmup-frames 300 --out ../../docs/screenshots/part4-rust-arena.png
```

Runs 300 fixed-timestep ticks (5 seconds of sim time at 60 Hz), then writes the PNG and exits 0
(US2, SC-002).

## Test

```powershell
cargo test
```

Runs, at minimum:

- `tests/determinism.rs` — same seed twice ⇒ identical snapshot; different seed ⇒ differs (FR-012).
- `tests/constraint_solver.rs` — separation converges toward rest length; anchor never moves (FR-013).
- `tests/zero_alloc.rs` — zero allocation delta across N post-warm-up `FixedUpdate` frames (FR-014).

All tests are headless (`MinimalPlugins`) — no window, no GPU required, safe to run in any CI
environment.

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
