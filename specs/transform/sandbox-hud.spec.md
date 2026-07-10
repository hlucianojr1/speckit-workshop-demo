# Sandbox HUD / Telemetry Display — Language-Agnostic Specification

> **Scope note:** This spec covers `apps/sandbox/app.cpp`'s on-screen HUD — the telemetry
> overlay shared by ALL sandbox scenes (rope, pendulum tower, cloth, particle storm, orbital
> arena), plus the orbital-arena-specific match panel it draws when an arena is active. This
> was NOT covered by Phase A (engine subsystems) or Phase A2 (gravity well/scoring/match
> simulation logic) — those specs covered simulation state; this spec covers how that state
> is DISPLAYED. Reverse-specced from `apps/sandbox/app.cpp`'s `draw_hud`/`draw_histogram`.

## 1. Purpose

Give the player/observer a live, legible readout of simulation identity (seed, tick/step
count), timing health (wall vs. sim time drift, frame budget), determinism evidence (a
state digest), and — for the orbital arena scene specifically — match state and per-player
scores, without obscuring the simulation view.

## 2. Layout (screen-space, scales with window width)

| Region                | Content                                                                 |
| ---------------------- | ------------------------------------------------------------------------ |
| Top-left, line 1      | Title: `"<APP NAME> showcase - <scene name>"`                          |
| Top-left, line 2      | `seed=<u64>  substeps=<u64>  speed=<float>x  [PAUSED]` (paused flag only shown when paused) |
| Top-left, line 3      | `wall=<sec>s  sim=<sec>s  drift=<+/-ms>` — drift = (wall − sim) × 1000  |
| Top-left, line 4      | `frame_avg=<ms>  fps=<int>` — rolling average frame time                |
| Top-left, line 5      | `trace_digest=<16-hex-digit>` — a deterministic state hash for A/B comparison |
| Top-left, orbital panel (only when a match is active) | `match=<state>  tick=<u64>`, then one line per player: `P<i> score: <u32>`, then `WINNER: P<i>` once a winner is latched |
| Top-right             | Boxed counts panel: `bodies=<n>  edges=<n>  particles=<n>  vfx=<n>`     |
| Bottom-left, line 1   | Scene/simulation control hints                                          |
| Bottom-left, line 2   | Mouse/action control hints                                              |
| Bottom-right          | Frame-budget histogram widget (§3)                                     |

## 3. Frame-Budget Histogram Widget

A small boxed panel, bottom-right:

- A single vertical bar whose height is proportional to the current rolling-average frame
  time (0 to a fixed cap, e.g. 50 ms), colored by threshold (see §4).
- Two horizontal reference lines: one at the 60Hz target (≈16.67 ms), one at the 30Hz
  threshold (≈33.33 ms), each labeled.
- A text readout: `"<ms> ms  avg"`.

## 4. Color-Coding Rules (a display MUST implement these thresholds, exact values are
   implementation-configurable but the THREE-TIER pattern is the guarantee)

| Metric          | Normal color | Warning threshold  | Critical threshold  |
| ---------------- | ------------ | -------------------- | --------------------- |
| Drift (\|ms\|)  | neutral      | > 50 ms → amber      | > 200 ms → red        |
| Frame average    | neutral      | > 16.67 ms → amber   | > 33.33 ms → red      |
| Trace digest     | neutral      | differs from a recorded baseline → highlight (any distinct accent color) | — |

This is a general pattern: **every live telemetry number that has a "budget" gets a
three-tier color (ok / warning / critical)**, not just the frame-time histogram.

## 5. Guarantees

| Guarantee                  | Description                                                              |
| ---------------------------- | --------------------------------------------------------------------------- |
| Read-only                   | The HUD never mutates simulation state — it is a pure function of it        |
| No allocation in draw path  | Text buffers are fixed-size/stack; drawing performs no per-frame heap traffic |
| Legible at any window size  | All layout values scale with a single `ui_scale` factor derived from window width |
| Scene-agnostic core         | Lines 1–5 and the histogram render identically regardless of which scene is active; the orbital match panel is the only scene-conditional addition |

## 6. Constraints (constitutional) and Scope Reduction

- No exceptions/panics; formatting failures are not possible with fixed-format numeric
  printf-style templates.
- Real-time (Article 6): the HUD itself must stay within the same frame budget it reports.
- **Documented scope reduction for the Rust port:** telemetry/event logging
  (`telemetry::get().emit_event(...)` calls throughout the reference `app.cpp`) is NOT
  covered by this spec — it is a structured-logging concern orthogonal to the visual HUD.
  Exact pixel-perfect layout (fonts, shadow offsets, gradient backgrounds, vignette,
  animated grid) is also descoped — the Rust port should reproduce the CONTENT and
  three-tier color guarantee (§4), not raylib's specific immediate-mode drawing calls.
