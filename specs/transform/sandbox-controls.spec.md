# Sandbox Controls & Application Loop — Language-Agnostic Specification

> **Scope note:** This spec covers the interactive application shell
> (`apps/sandbox/app.cpp`'s input handling and frame loop, `apps/sandbox/main.cpp`'s
> usage text): the complete keyboard/mouse map, the per-frame loop contract, and the
> demo features (perf bomb, gravity well, crash key). Prior phases specced only the
> LMB-drag adaptation ([interactive-control](orbital-arena-interactive-control.spec.md));
> Phase A5 (full-fidelity closure) specs the whole surface. Visual feedback for these
> controls (shake, shockwave, G-indicator, held-node ring) is in
> [sandbox-visual-identity.spec.md](sandbox-visual-identity.spec.md); simulation
> effects (bursts, grabs) are in [sandbox-scenes.spec.md](sandbox-scenes.spec.md) §7.

## 1. Purpose

Define every player-facing control and the frame-loop timing rules so a port *plays*
identically — same keys, same responses, same pacing behavior.

## 2. Keyboard Map

| Key         | Action                                                                     |
| ----------- | --------------------------------------------------------------------------- |
| `1`–`5`     | Switch scene: rope / pendulum tower / cloth / particle storm / orbital arena (keeps seed, resets clocks — [scenes spec](sandbox-scenes.spec.md) §3.1) |
| `Space`     | Pause / resume                                                              |
| `S`         | Single-step one fixed step — **only while paused** (ignored otherwise)      |
| `R`         | Reseed with the **same** seed. Records the pre-reseed digest as a baseline; the HUD digest line highlights if the trajectory ever diverges from a replay of that baseline — a live determinism check |
| `H`         | Toggle HUD                                                                  |
| `T`         | Toggle particle trails                                                      |
| `+` / `=` and `−` (incl. keypad) | Simulation speed × / ÷ 1.25, clamped to [0.125, 4.0]  |
| `P`         | **Perf bomb** (demo): spawns 5000 particles as 10 bursts of 500 at x = −2 + 4·(b/9), y alternating ±0.5 — deliberately blowing the frame budget for the live perf-regression demo — and fires the shockwave + screen shake effects |
| `G` (held)  | **Cursor gravity well:** each frame, every free particle within world radius 2.5 of the cursor receives an inverse-square pull — `f = min(8 / (d² + 0.1), 20)` applied along the unit direction, scaled by 1/60 — where d includes a +0.01 offset to avoid singularity. Affects sim state (digest changes) — an intentionally interactive force |
| `F1`        | Deliberate crash (null write) — crash-dump demo feature; mechanism out of scope |
| `Esc` / close | Quit                                                                      |

## 3. Mouse Map

Cursor position converts screen→world through the inverse viewport mapping each frame.

| Input             | Action                                                                |
| ----------------- | ---------------------------------------------------------------------- |
| LMB press         | Grab the nearest non-anchor body within world radius **0.4** ([scenes](sandbox-scenes.spec.md) §7) |
| LMB held          | Drag the held body (pinned to cursor each substep)                     |
| LMB release       | Release without imparting cursor velocity                              |
| RMB press         | Spawn **12** sim-affecting free particles **plus** a 24-particle render-only VFX burst at the cursor |

## 4. Frame Loop Contract

Each frame, in order:

1. **Poll input exactly once per frame**, before reading any input state (edge-triggered
   press events are consumed by polling; polling more than once per frame silently
   swallows events — reference-implementation lesson).
2. Handle keyboard/mouse (§2–3).
3. **Timing:** dt = self-measured wall-clock gap between frame starts (do not trust the
   graphics library's frame time), multiplied by the speed factor, then **clamped to
   0.1 s** (≈ 6 substeps) so window drags / focus loss cannot queue hundreds of
   substeps and freeze the app.
4. **Step:** `scene.step(dt)` when running; one fixed step when single-stepping;
   nothing when paused.
5. **Render** per [visual-identity](sandbox-visual-identity.spec.md) layer order; HUD
   only when enabled.
6. **Frame limiter:** sleep the remainder of a 1/60 s budget, measured against the
   frame's actual work time (a real sleep, not a busy-wait — yields the thread to the
   OS event loop).
7. FPS shown in the HUD = frames counted per rolling 1-second window (self-measured).

## 5. Screenshot Mode (`--screenshot PATH [--warmup N]`)

Interactive mode variant for captures: every frame advances the sim by exactly **one
fixed step × speed** (deterministic, wall-clock-independent), and after N warmup frames
(default **120**) the frame is written to PATH and the app exits. Used for
reference-vs-port visual comparison.

## 6. Guarantees

| Guarantee              | Description                                                    |
| ----------------------- | ---------------------------------------------------------------- |
| Bounded catch-up        | 0.1 s dt clamp + max-8-substep loop clamp ⇒ no spiral of death   |
| Pause purity            | A paused frame performs zero simulation work                     |
| Reseed determinism      | R with an unchanged seed reproduces the identical trajectory     |
| Sim/render input split  | RMB adds both a sim burst and a render burst; G affects sim; trails/HUD toggles never do |
| Event delivery          | Single poll per frame ⇒ no lost edge-triggered presses           |

## 7. Constraints & Exclusions

- No exceptions/panics in the loop; all failure paths degrade gracefully.
- **Out of scope:** telemetry flags (`--debug`, `--telemetry-*`, `--watchdog`,
  `--no-crash-handlers`), crash handlers, hang watchdog, OS timer-resolution and
  presentation workarounds — operational tooling, not game behavior.
