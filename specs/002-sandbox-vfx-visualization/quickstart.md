# Quickstart: Sandbox VFX Visualization

Manual verification steps once Feature 002's tasks are implemented. Assumes the Windows/VS
2022/vcpkg build setup already recorded in this workspace (`cmake --preset default-debug`).

## 1. Build and run the full gate

```powershell
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

All existing tests stay green; new tests (`test_emitter`'s `set_shape` cases,
`test_scene_vfx`) pass.

## 2. Golden digest A/B check (SC-001 — the decisive check)

```powershell
./build/apps/sandbox/ea-sandbox --headless --seed 42 --frames 600 --out feature-trace.csv
```

Compare the printed `trace_digest` against the pre-feature baseline recorded in
`baseline-trace.csv` / this session's earlier baseline run. **They must be identical.** If
they differ, VFX state leaked into `state_digest()` or `m_vfx_emitter` drew from `m_rng`
instead of its own salted seed — see research.md §3–§5 for the traps this guards against.

## 3. Visual smoke test

```powershell
./build/apps/sandbox/ea-sandbox.exe
```

- Right-click anywhere: the existing 12-particle physics burst still appears, **and** a
  fading radial VFX burst appears at the same point, layered on top.
- Let a free particle bounce off a wall in the `rope`, `pendulum_tower`, or `cloth` scene
  (keys `1`/`2`/`3`): a small spark burst appears at the contact point.
- Switch to `particle_storm` (key `4`): particles wrap around the edges — no spark bursts
  there, but right-click bursts still work.
- The top-right HUD panel shows a live VFX particle count that rises on burst/spark and
  falls back toward 0 as particles expire.

(On a VM without hardware OpenGL, per repo memory: copy the Mesa llvmpipe DLLs next to the
exe first, or use `--screenshot <relative-path>` for a single-frame capture.)

## 4. Frame budget check (SC-003)

While running interactively, press `P` (perf-bomb) to spawn a large number of physics
particles, and right-click repeatedly to keep the VFX pool near capacity. Watch the HUD's
`frame_avg=...ms` line — it should stay close to the existing baseline; the VFX tick/emit
cost is bounded by `kVfxPoolCapacity` (2048) and adds only a fixed, small per-frame cost on
top of the existing physics particle budget.
