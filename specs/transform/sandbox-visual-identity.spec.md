# Sandbox Visual Identity — Language-Agnostic Specification

> **Scope note:** This spec covers the sandbox renderer's **exact visual identity**
> (`apps/sandbox/app.cpp`: background, particle/node/edge styling, orbital arena
> rendering, effects, and the HUD palette) so a port can look **identical**, not just
> equivalent. It supersedes the "exact pixel layout … descoped" reduction in
> [sandbox-hud.spec.md](sandbox-hud.spec.md) §6 for everything visual — that spec still
> owns HUD *content* and the three-tier guarantee; this one owns colors, sizes, and
> animation formulas (Phase A5 — full-fidelity closure). Colors are 8-bit RGBA
> `(r, g, b, a)`; sizes are pixels at the reference layout; `t` is wall-clock seconds.
> Drawing-API specifics (immediate mode vs. retained) remain out of scope — the
> *output* is normative, not the API.

## 1. Canvas & Coordinate System

| Property        | Value                                                        |
| --------------- | ------------------------------------------------------------ |
| Window          | 1280 × 720, resizable; title `ea-sandbox - engine_demo showcase` |
| UI scale        | `width / 1280` — every HUD font size and layout offset multiplies by this |
| Clear color     | (8, 10, 16, 255)                                             |
| World→screen    | World x ∈ ~[−3.5, 3.5], y ∈ ~[−2.5, 2.5] mapped to the viewport; radius-to-pixels uses the horizontal scale (a world radius r spans `world_to_screen(r,0) − world_to_screen(0,0)` px) |
| Text shadow     | Every HUD string draws twice: offset (+1, +1) in (0, 0, 0, 180), then the tint on top |

## 2. Background Layers (drawn in order)

1. **Deep-space gradient:** full-window vertical gradient (8, 12, 28, 255) → (2, 2, 6, 255).
2. **Breathing grid:** vertical lines at world x = −3…3 (spanning y ±2.5) and
   horizontal lines at world y = −2…2 (spanning x ±3.5); width 1, color
   (60, 80, 140, α) with `α = 40 + 30·pulse`, `pulse = 0.5 + 0.5·sin(0.8·t)`.
3. **Vignette:** 80-px gradient bands on all four edges — horizontal bands fade
   (0,0,0,120) → transparent (left/right), vertical bands (0,0,0,100) → transparent
   (top/bottom).
4. **Dust motes:** 60 cosmetic screen-space motes, deterministically seeded per index
   i with integer hashing (`h₀ = i·2654435761`, then LCG steps
   `h ← h·1664525 + 1013904223` between draws): x, y uniform [0,1),
   `speed = 0.005 + (h%100)/10000`, `phase = (h%628)/100`, `size = 1.0 + (h%20)/10` px,
   `α = 20 + h%40`. Motion: upward drift `y − speed·t·0.1` wrapped to [0,1); sway
   `x + 0.01·sin(0.5·t + phase)`. Color (180, 200, 255, α).
5. **World floor reference:** line from world (−3, −2) to (3, −2), width 2,
   color (60, 70, 100, 220).

## 3. Speed Color Ramp (the signature palette)

`color_from_speed(v)`: map v ∈ [0, 8] m/s → `t = clamp(v/8, 0, 1)`, smoothstep
`t₂ = t²(3 − 2t)`, then a 3-segment gradient (u = local segment fraction), α = 255:

| Segment       | r            | g            | b            | Visual        |
| ------------- | ------------ | ------------ | ------------ | ------------- |
| t₂ < 0.33     | 0.1u         | 0.3 + 0.5u   | 0.8 + 0.2u   | deep blue → cyan |
| t₂ < 0.66     | 0.1 + 0.8u   | 0.8 − 0.5u   | 1.0 − 0.2u   | cyan → hot pink/magenta |
| else (u/0.34) | 0.9 + 0.1u   | 0.3 + 0.7u   | 0.8 + 0.2u   | magenta → white |

Channels scale ×255 to 8-bit.

## 4. Free Particles & Trails

- **Trails** (toggleable, default on): for each particle, 9 segments over its 10-point
  ring buffer, oldest → newest. Segment k: width `2.5·(1 − p) + 0.5` where
  `p = (k+1)/10`; color = speed ramp of the segment speed (point distance × 60);
  α = `p·180` (older segments thinner and fainter).
- **Particle body** (4 bloom layers, radius `R = radius·60·pulse` px,
  `pulse = 1 + 0.3·sin(4t + 0.7·i)` per index i; core color from the speed ramp):
  1. Outer bloom: 4R, core color at α 20
  2. Mid glow: 2.2R, α 50
  3. Core: R, full color
  4. Hot center (only when speed > 3): 0.4R, white at
     `α = clamp((speed − 3)/5, 0, 1)·200`

## 5. Constraints (edges) & Nodes

- **Edge:** glow pass width 4 in (100, 160, 255, 40), then core width 2 in
  (200, 220, 255, 230).
- **Anchor node:** 9-px halo (255, 200, 60, 40) + centered 10×10 golden square
  (255, 210, 120, 255).
- **Dynamic node:** three concentric circles — 10 px (100, 200, 255, 30),
  6 px (140, 220, 255, 80), 4 px (200, 245, 255, 230).
- **Held node:** two pulsing rings at radius 14·p and 16·p px,
  `p = 1 + 0.2·sin(6t)`, colors (255, 80, 220, 255) and (255, 80, 220, 140).

## 6. VFX Particles (render-only layer)

Radius `r = max(size·60, 1.5)` px; `α = clamp(life_fraction, 0, 1)·220`
(life_fraction = remaining/total lifetime → fade-out):

1. Glow: 2r in (255, 210, 110, α/3)
2. Core: r in (255, 225, 140, α)

## 7. Orbital Arena Scene Rendering

Replaces the free-particle/edge/node draws entirely when active.

- **Player colors** (also used by the HUD match panel):
  P0 cyan (80, 190, 255), P1 orange (255, 150, 60), P2 green (120, 230, 120),
  P3 purple (210, 120, 255) — all α 255.
- **Bounds:** rectangle outline at world ±half_extent, color (90, 110, 160, 200).
- **Field particles:** each drawn as a 4-px halo (particle color at α/4) + 2-px core.
  Color: neutral gray-blue (170, 180, 210, 200) when unclaimed; otherwise the color of
  the **nearest active well whose influence radius contains it** (squared-distance
  compare), at α 230. Tinting only applies once wells are placed (match playing or
  game over).
- **Wells** (only when placed; per well, player color `pc`):
  1. Influence ring: outline at influence radius, (pc, 45)
  2. Capture glow: filled circle at 2.5 × capture-radius-px, (pc, 40)
  3. Capture ring: outline at capture radius, pc
  4. Core: 6-px filled pc; 3-px white center (255, 255, 255, 230)
  5. Label `P<i>` at offset (+10, −10) px, size 14, pc (with text shadow)

## 8. Full-Screen Effects

- **Shockwave** (perf-bomb trigger; 1.5 s): rings expand from screen center to
  `0.6·width·τ` (τ = progress 0→1). Three rings at radius `R·(1 − 0.08·ring)`, color
  (180, 120, 255, α/(ring+1)) with `α = (1 − τ)·200`. For τ < 0.2 a central flash:
  filled circle 0.3·R in (255, 200, 255, (1 − τ/0.2)·60).
- **Screen shake** (perf-bomb trigger; 0.5 s, intensity 12 px): all world/HUD draws
  translate by `decay·12·(sin 47t, cos 53t)`, `decay = 1 − τ`.
- **Gravity-well indicator** (while `G` held; at the mouse cursor,
  `p = 0.7 + 0.3·sin(5t)`): filled 40p px (160, 80, 255, 30), filled 20p px
  (200, 120, 255, 50), ring 60p px (180, 100, 255, 80).

## 9. HUD Palette & Metrics (exact values for [sandbox-hud.spec.md](sandbox-hud.spec.md)'s layout)

Font sizes are pre-scale (multiply by ui_scale); left column x = 16, lines at
y = 12, 40, 62, 80, 98 (+ orbital panel at 120, then 142 + 20 per player).

| Element                    | Size | Color                                              |
| -------------------------- | ---- | --------------------------------------------------- |
| Title line                 | 22   | warm cream (255, 240, 200, 255)                     |
| seed/substeps/speed line   | 16   | near-white (245, 245, 245, 255)                     |
| Drift + frame lines        | 14   | neutral (200, 210, 220) / warn (255, 220, 90) / critical (255, 120, 90) — thresholds per hud spec §4 |
| trace_digest line          | 14   | neutral; baseline-mismatch highlight (255, 140, 220) |
| Orbital match line         | 16   | (255, 240, 200); score lines 16 in player colors; `WINNER: P<i>` 20 in winner's color |
| Counts panel (top-right)   | 14   | bg (0, 0, 0, 140); text (220, 230, 240)             |
| Control hints (bottom-left)| 12   | (170, 180, 200, 220), two lines at height −42 / −24 |
| Histogram panel            | 240×88 px, bottom-right inset 20 | bg (0, 0, 0, 140); border (80, 100, 140, 180); label 14 (180, 200, 240) |
| Histogram bar              | 28 px wide | green (120, 220, 120, 230) ≤ 16.667 ms; amber (220, 200, 90, 230); red (220, 90, 90, 230) > 33.333 ms; scale caps at 50 ms |
| Reference lines            | —    | 60 Hz (120, 220, 120, 200) labeled `60Hz` 10 px (160, 230, 160, 220); 30 Hz (220, 90, 90, 200) labeled `30Hz` 10 px (230, 160, 160, 220) |
| Avg readout                | 14   | `%.2f ms  avg` in (220, 220, 240, 255)              |

## 10. Guarantees

| Guarantee            | Description                                                       |
| --------------------- | -------------------------------------------------------------------- |
| Deterministic styling | Every animated value is a pure function of wall-clock time, index hashing, or sim state — no unseeded randomness |
| Render-only           | Nothing in this spec reads back into simulation state or the digest  |
| Resolution-independent| All layout via ui_scale; world geometry via the world→screen mapping |
| Layer order           | Background → dust → floor → scene → shockwave → G-indicator → HUD → histogram, exactly as listed |

## 11. Constraints & Exclusions

- No allocation in the draw path (fixed-size text buffers).
- **Out of scope:** platform presentation workarounds (DWM/Metal/compositor handling),
  font rasterization details, and the JSONL telemetry layer — none are part of the
  game's visual identity.
