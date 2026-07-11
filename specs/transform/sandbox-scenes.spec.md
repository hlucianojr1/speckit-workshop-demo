# Sandbox Scene Catalog & Simulation Loop — Language-Agnostic Specification

> **Scope note:** This spec covers `apps/sandbox/scene.h`/`scene.cpp`'s **scene layer** —
> the five runtime-selectable scenes, their exact construction geometry and rng draw
> order, the per-substep simulation loop (verlet integration + per-scene force/boundary
> rules), motion trails, interactive spawn operations, and the deterministic state
> digest with its recorded golden values. It composes the engine-foundation subsystems
> already specced in Phase A ([physics-constraint](physics-constraint.spec.md),
> [game-loop](game-loop.spec.md), [rng](rng.spec.md), [frame-budget](frame-budget.spec.md),
> [allocator](allocator.spec.md)) and the render-only
> [VFX system](vfx-particle-system.spec.md). The free-particle *data model* and
> rule-isolation guarantee are in [sandbox-free-particles.spec.md](sandbox-free-particles.spec.md);
> this spec supplies the exact per-scene rules and constants that spec descopes.
> **Full-fidelity spec (Phase A5):** unlike earlier phases, nothing here is descoped —
> every constant required to reproduce the golden digests byte-for-byte is normative.

## 1. Purpose

Define the five showcase scenes so that an independent implementation, fed the same
seed, produces the **identical** deterministic state digest per frame — the strongest
possible cross-implementation equivalence check.

## 2. Scene Kinds

| Id | Name             | Selector key / CLI name          | Contents                                        |
| -- | ---------------- | -------------------------------- | ----------------------------------------------- |
| 0  | rope             | `1` / `rope` (default)           | 24-node verlet chain + 32 free particles        |
| 1  | pendulum tower   | `2` / `pendulum`, `pendulum_tower` | 4 chains of varying length + 24 free particles |
| 2  | cloth            | `3` / `cloth`                    | 12×8 constraint grid, no free particles         |
| 3  | particle storm   | `4` / `storm`, `particle_storm`  | 500 free particles, twin gravity wells          |
| 4  | orbital arena    | `5` / `orbital`, `orbital_arena` | Embedded Orbital Arena match (autopilot-driven) |

## 3. Shared Scene State and Constants

| Constant                | Value       | Meaning                                            |
| ----------------------- | ----------- | -------------------------------------------------- |
| Arena (allocator) size  | 8 MiB       | Linear bump arena; scene switches deliberately leak old allocations into it |
| Fixed step              | 1/60 s      | Simulation substep (double accumulator)            |
| Max substeps per frame  | 8           | Game-loop clamp (see [game-loop spec](game-loop.spec.md)) |
| Gravity                 | 9.81 (−y)   | Applied to verlet bodies at full strength; to free particles at ×0.25 in bounce scenes |
| Solver iterations       | 8           | Constraint-projection passes per substep           |
| Trail length            | 10          | Ring-buffer positions per free particle (render-only) |
| World view              | x ∈ [−3.5, 3.5] approx.; floor reference at y = −2 | Renderer viewport basis |

The scene owns one seeded rng stream (`m_rng`, seeded with the scene seed). **Every rng
draw below is normative in both formula and order** — a single extra or reordered draw
changes the golden digest. The VFX emitter uses a separate stream seeded with
`seed XOR 0x564658` and draws nothing from `m_rng` (see §9).

### 3.1 Scene lifecycle operations

- **construct(seed):** seeds `m_rng`, builds the rope scene (default kind).
- **reseed(seed):** re-seeds `m_rng`, zeroes sim/wall clocks and substep counter,
  releases any held node, rebuilds the current kind. Reseeding with the *same* seed must
  reproduce the identical state trajectory (interactive `R` key relies on this).
- **switch_scene(kind):** same reset as reseed but keeps the current seed and changes
  kind. Both operations rebuild the constraint solver and the VFX pool/emitter from
  scratch and tear down any embedded arena.
- Body ids are assigned 1, 2, 3, … in creation order. Anchor bodies have inverse mass 0;
  dynamic bodies have inverse mass 1.

## 4. Scene Construction (exact geometry + rng draw order)

### 4.1 Rope (kind 0)

Preserved byte-for-byte from the original demo — do not change body order, rng draw
order, or initial-position math.

- 24 bodies in a chain. Node *i* at
  `x = sin(0.61) · 0.30 · i`, `y = 3.0 − cos(0.61) · 0.30 · i` (anchor at (0, 3), fixed
  tilt angle 0.61 rad, rest length 0.30). Node 0 is the only anchor.
- 23 distance constraints linking consecutive nodes, rest length 0.30.
- Then 32 free particles, each consuming **exactly 5 rng draws in this order**
  (`u` = uniform [0,1) draw):
  `x = u·6 − 3`, `y = u·2 − 1`, `vx = u·2 − 1`, `vy = u·2 − 1`, `radius = 0.04 + u·0.04`.

### 4.2 Pendulum Tower (kind 1)

- 4 chains with lengths {16, 12, 18, 10}, anchored at x = {−2.4, −0.8, 0.8, 2.4},
  y = 1.8, rest length 0.22.
- Per chain c (one rng draw *before* placing its bodies):
  `tilt = (u·0.9 − 0.45) + (c even ? +0.4 : −0.4)`; node *i* offset per step is
  `(sin(tilt)·0.22, −cos(tilt)·0.22)`. First node of each chain is an anchor; chain
  nodes are linked by rest-0.22 constraints.
- Then 24 free particles, 5 draws each in order:
  `x = u·6 − 3`, `y = u·1.5 − 1.5`, `vx = u·1.6 − 0.8`, `vy = u·1.0`,
  `radius = 0.04 + u·0.03`.

### 4.3 Cloth (kind 2)

- 12 columns × 8 rows spanning x ∈ [−2.5, 2.5], y from 1.5 (top) down to −1.0.
  `dx = 5.0/11`, `dy = 2.5/7`. Row-major placement (row 0 first, columns left→right);
  the entire top row is anchored.
- **One rng draw per body**, consumed even for anchors:
  `y jitter = (u − 0.5) · 0.02` added to the body's y.
- Constraints, all with rest length × slack 1.04, added in this exact order:
  1. Horizontal: for each row, columns (c, c+1) — rest `dx`.
  2. Vertical: for each row pair (r, r+1), each column — rest `dy`.
  3. One shear diagonal per cell, direction alternating by parity of (r+c):
     even → (r,c)-(r+1,c+1), odd → (r,c+1)-(r+1,c) — rest `sqrt(dx² + dy²)`.
  Full shear (both diagonals) is deliberately omitted: it over-constrains an
  8-iteration solver and visually freezes the cloth; the slack lets gravity drape it.
- No free particles.

### 4.4 Particle Storm (kind 3)

- No bodies/constraints. 500 free particles, **6 draws each** in order:
  `x = u·6 − 3`, `y = u·4 − 2`, `speed = 0.5 + u·1.5`, `angle = u·2π`,
  then `vx = cos(angle)·speed`, `vy = sin(angle)·speed`, `radius = 0.025 + u·0.03`.

### 4.5 Orbital Arena (kind 4)

- No bodies, constraints, or scene free particles — the embedded match
  ([orchestration spec](orbital-arena-orchestration.spec.md)) owns the whole world.
- Arena configuration: `seed = scene seed`, 2 players, `half_extent = 2.0`
  (a **sandbox override** of the game default 5.0, chosen to fit the view),
  particle capacity 500.
- Scripted lobby: both players join and set ready immediately, so the match counts
  down and starts on its own. If arena creation fails (allocator exhaustion) the scene
  renders empty without crashing.
- The arena draws only from its own salted rng streams; `m_rng` is never consumed, so
  every other scene's draw order (and digest) is untouched by this scene's existence.

## 5. Simulation Substep (executed per fixed step, in this order)

1. **Verlet integration** of every dynamic body:
   `next = cur + (cur − prev)` plus `−9.81·dt²` on y; `prev ← cur`. Anchors don't move.
2. **Held-node pin:** if a node is grabbed (§7), overwrite its position *and* verlet
   prev with the cursor target (drag wins over physics; no pseudo-velocity on release).
3. **Constraint solve:** 8 projection iterations.
4. **Free-particle / world rule** — exactly one of three, per scene kind
   (rule isolation per [sandbox-free-particles.spec.md](sandbox-free-particles.spec.md)):
   - **Orbital arena:** delegate the entire world step to the embedded arena, feeding it
     autopilot inputs (§6).
   - **Particle storm:** twin fixed gravity wells at (−1.2, 0.4) and (1.2, −0.3);
     per particle, per well: `r² = dx² + dy² + 0.15` (softening), accel magnitude
     `1.4 / r²` along the normalized direction to the well, `v += dir·accel·dt`. Then
     velocity drag ×0.999 per axis, integrate position, and **wrap** at
     x = ±3.5 (span 7) and y = ±2.5 (span 5).
   - **All other scenes (bounce rule):** `vy −= 9.81·dt·0.25` (quarter gravity),
     integrate position, then clamp-and-reflect at x = ±3.0 and y ∈ [−2.0, 2.0] with
     restitution ×0.85 on the reflected axis, emitting a **VFX spark burst (6
     particles)** at every contact point (§9).
5. **VFX tick:** age/retire live VFX particles by dt (render-only; after all spark
   emission for the step).
6. `sim_time += dt`.

**Per frame** (after all substeps): trails roll once — each particle's current position
overwrites the oldest slot of its 10-entry ring buffer (only if ≥ 1 substep ran) — and
the frame's wall-clock time (ms) is recorded into the frame-budget window.

## 6. Orbital Autopilot (deterministic scripted inputs)

Inputs are a pure function of the arena's own tick index — no rng, no wall clock:

```text
t = tick · (1/60)
for player p of n:
  phase   = 2π·p/n
  fx      = 0.90 + 0.07·p        # detuned per player — see Fairness note
  fy      = 0.53 + 0.05·p
  steer.x = sin(t·fx + phase)
  steer.y = sin(t·fy + phase + 1.3)
  strength = 1.0
```

**Fairness note (normative):** frequencies are *detuned* per player, not merely
phase-shifted. Pure phase shifts make player 1's input the exact negation of player
0's, and the arena's bit-exact mirror-fairness (game Article 9) then locks the match
in a permanent sudden-death tie.

## 7. Interactive Operations

| Operation                    | Contract                                                                 |
| ---------------------------- | ------------------------------------------------------------------------ |
| grab_nearest_node(wx, wy, r) | Selects the nearest **non-anchor** body within radius r (squared-distance compare); returns whether one was grabbed |
| drag_held_node(wx, wy)       | Updates the pin target (applied in substep step 2)                       |
| release_held_node()          | Re-syncs verlet prev to current position so cursor motion imparts no velocity, then clears the hold |
| spawn_particle_burst(wx, wy, n) | Appends n free particles at (wx, wy); per particle, **4 draws in order**: `angle = u·2π`, `speed = 1.5 + u·2.5` → velocity, `radius = 0.04 + u·0.03`. Trail slots are appended in lockstep, initialized to the spawn point. These particles enter the digest — bursts are sim-affecting |
| spawn_vfx_burst(wx, wy)      | Repositions the shared VFX emitter to (wx, wy) and emits 24 particles — render-only, never affects the digest (§9) |

## 8. State Digest (determinism proof)

64-bit FNV-1a (offset basis `0xcbf29ce484222325`, prime `0x100000001b3`), mixing each
`double` value's IEEE-754 bit pattern **byte-by-byte, least-significant byte first**,
in this exact order:

1. Every body position, in body-list order: x then y.
2. Every free particle, in list order: x then y.
3. Orbital arena scene only: tick count, match state (as its integer id), each player's
   score (player order), each well's x and y (well order), each live arena particle's
   x and y (live order) — every value converted to `double` before mixing.

Trails, VFX particles, held-node state, and HUD state are **excluded** — render-only.

### 8.1 Golden digests (seed 42, 600 fixed-step frames — normative acceptance values)

Recorded from the reference implementation (two independent runs, byte-identical;
2026-07-11):

| Scene           | `trace_digest`     |
| --------------- | ------------------ |
| rope            | `9dc3bd72a4f7f31a` |
| pendulum tower  | `dee045cb412df634` |
| cloth           | `3cbd246289e0cf63` |
| particle storm  | `fd2df9d9c889a7fc` |
| orbital arena   | `919d2feba5bdbeac` |

> Historical note: an older comment in the reference source cites rope digest
> `33a6319d856d4869`; that value pre-dates the addition of the rope scene's 32 free
> particles and is obsolete. The table above is ground truth for the current game.

An implementation matches this spec **iff** all five digests match. (A bit-identical
digest additionally requires the same floating-point evaluation of the formulas above
and the engine rng from [rng.spec.md](rng.spec.md); implementations on platforms where
that is unattainable should fall back to trajectory-tolerance comparison of the
headless CSV columns — see [sandbox-headless.spec.md](sandbox-headless.spec.md).)

## 9. VFX Wiring (render-only layer)

One shared pool + emitter per scene (contract: [vfx-particle-system.spec.md](vfx-particle-system.spec.md)):

| Parameter            | Value                                    |
| -------------------- | ---------------------------------------- |
| Pool capacity        | 2048                                     |
| Emitter seed         | `scene seed XOR 0x564658`                |
| Shape                | sphere, radius 0.03, repositioned per event via set_shape (never reconstructed — reconstruction would leak arena memory per burst) |
| Speed range          | 1.5 – 4.0                                |
| Lifetime             | 0.6 s (min == max)                       |
| Spawn color          | RGBA (1.0, 0.85, 0.4, 1.0) normalized    |
| Size range           | 0.02 – 0.05                              |
| Right-click burst    | 24 particles                             |
| Collision spark      | 6 particles                              |

Pool exhaustion is graceful (emit returns a status; no crash, no partial corruption).

## 10. Guarantees

| Guarantee                | Description                                                         |
| ------------------------- | -------------------------------------------------------------------- |
| Determinism               | (seed, kind, frame count) fully determines the digest — golden table §8.1 |
| RNG stream isolation      | VFX and the embedded arena never draw from the scene rng             |
| Rule isolation            | Exactly one free-particle rule per scene kind; never blended         |
| Reseed reproducibility    | reseed(same seed) reproduces the identical trajectory                |
| Render/sim separation     | Trails + VFX are excluded from the digest by construction            |
| No steady-state allocation | All spawning after construction uses pre-sized pools/vectors; scene rebuilds are the only (accepted, bounded) arena consumers |
| Graceful exhaustion       | Arena-creation or VFX-pool exhaustion degrades (empty render / skipped spawn), never crashes |

## 11. Constraints (constitutional)

- No exceptions/panics; fallible operations return status values (Article 1).
- Deterministic sim paths: double accumulators/positions, seeded rng only (Article 5).
- Real-time: no allocation in the substep hot path (Article 6).
- Destruction-order hazard (reference field note): any subsystem placement-new'd inside
  the scene's arena must be torn down **before** the arena's backing buffer is freed.
