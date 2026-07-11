# VFX Particle System — Language-Agnostic Specification

> **Scope note:** Reverse-specced from `include/engine_demo/vfx/{particle,emitter,
> force_applicator}.h` (the original workshop's Feature 001 "Particle VFX Subsystem",
> demonstrated live in Part 0 of the training) plus the sandbox's usage pattern
> (`apps/sandbox/scene.cpp`'s `spawn_vfx_burst`/collision-triggered sparks, Feature 002
> "Sandbox VFX Visualization"). This is a DIFFERENT subsystem from every spec written so
> far in `specs/transform/` — it is purely visual/cosmetic particle FX (sparks, dust),
> never physics, never gameplay-affecting. It was never reverse-specced in Phase A
> (§4.2, which covered the 6 engine-foundation subsystems) or Phase A2/A3 (which covered
> the Orbital Arena game). Its omission was only surfaced by comparing a running
> screenshot of the base sandbox's default scene against the Rust port and noticing the
> spark/dust trails have no equivalent.

## 1. Purpose

Provide a fixed-capacity, render-only particle effects system: short-lived visual
particles (sparks, dust) spawned in bursts or from a continuous emitter, aged and
retired automatically, with composable acceleration effects (gravity, wind, turbulence).

## 2. Data Model

### 2.1 Particle (`particle_pool`'s storage unit)

| Field                      | Type          | Notes                                       |
| --------------------------- | ------------- | -------------------------------------------- |
| remaining_lifetime_seconds | 64-bit float  | Time accumulator; particle retires at <= 0.0 |
| position                   | 3 × float     | Render-boundary precision (32-bit is fine)   |
| velocity                   | 3 × float     | Render-boundary precision                     |
| color                      | 4 × float     | RGBA, default opaque white                    |
| size                       | float         | Default 1.0                                   |

### 2.2 Spawn Parameters

The initial state for one particle, as produced by an emitter and consumed by the pool:
position, velocity, lifetime_seconds, color, size (same shape as §2.1 minus the "already
aged" accumulator).

### 2.3 Emitter Shape (tagged union — three kinds)

| Shape  | Fields                                                          |
| ------ | ----------------------------------------------------------------- |
| Point  | position                                                          |
| Cone   | origin, direction, half_angle_radians                             |
| Sphere | center, radius                                                    |

### 2.4 Force Applicator (tagged union — three kinds)

| Force      | Fields                          | RNG draws                          |
| ----------- | -------------------------------- | ------------------------------------ |
| Gravity     | acceleration (3 × float)        | None                                 |
| Wind        | acceleration (3 × float)        | None                                 |
| Turbulence  | strength, frequency             | Exactly 3 draws per particle per tick (one per axis) |

## 3. Operations

### 3.1 Particle Pool

| Operation      | Signature (abstract)                    | Behavior                                             |
| --------------- | ---------------------------------------- | ------------------------------------------------------ |
| try_spawn       | `(spawn_params[]) → (spawned_count, status)` | Spawns as many of the given params as free capacity allows — a **partial-success** operation (0 ≤ spawned ≤ requested), never all-or-nothing. Never allocates. |
| age_and_retire  | `(dt_seconds) → void`                    | Ages every live particle by dt; retires (removes) any at/below zero remaining lifetime. dt=0 is a no-op. Never allocates. |
| live_particles  | `() → particle[]`                        | Read/write view of the currently live particles         |
| capacity/live_count/free_count | `() → uint`                  | Fixed capacity decided at construction; never resizes    |

### 3.2 Emitter

| Operation  | Signature (abstract)              | Behavior                                                    |
| ----------- | ----------------------------------- | -------------------------------------------------------------- |
| add_force  | `(force_applicator) → status`      | Attaches a force, up to a small fixed capacity (reference: 4); returns an error and leaves the list unchanged once full — never grows unbounded |
| set_shape  | `(emitter_shape) → void`           | Replaces the shape configuration in place; safe between any two ticks; never allocates |
| try_emit   | `(count) → (spawned_count, status)` | Samples `count` particles from the current shape/variance configuration and forwards them to the pool in one batch (uses the emitter's own seeded RNG stream — see §4) |
| tick       | `(dt_seconds) → void`              | Sums every attached force's contribution for each live particle, integrates velocity into position, then ages/retires via the pool. dt=0 is a no-op. |

## 4. Determinism and RNG (critical)

- Each emitter owns **one seeded RNG stream**, entirely separate from any other RNG in
  the simulation (e.g. the main sim/gameplay RNG). VFX activity must never perturb
  another subsystem's draw order.
- Per-particle sampling and per-force application (turbulence) happen in a **fixed,
  stable dense-array index order** — never hash-map or otherwise unordered iteration.
- A collision-triggered spark burst (§5) draws from the SAME emitter stream as any other
  burst from that emitter — there is no separate "spark-only" stream.

## 5. Usage Pattern: Burst and Collision-Triggered Sparks (from Feature 002)

Two common driving patterns sit on top of the emitter API, both purely at the
application/scene layer (not part of the pool/emitter contract itself):

- **Interactive burst**: on a user action (e.g. right-click), emit a burst of N
  particles at the cursor's world position, additive to any other effect at that
  location (never replacing it).
- **Collision-triggered spark**: whenever a physics object bounces off a boundary,
  immediately emit a small burst (reference: ~6 particles) at the contact point. This
  must never be skipped just because the pool is near capacity — pool exhaustion is
  handled gracefully (§6), not by special-casing the caller.

## 6. Guarantees

| Guarantee                | Description                                                                 |
| -------------------------- | ------------------------------------------------------------------------------ |
| Render-only, never gameplay | VFX particles are never read by any physics/scoring/game-rule system         |
| Graceful exhaustion        | A full pool silently spawns fewer particles than requested — never an error/crash |
| No allocation in steady state | Pool and emitter reserve all storage at construction; `try_spawn`/`try_emit`/`tick`/`age_and_retire` never allocate afterward |
| Deterministic              | Same seed + same call sequence ⇒ identical particle trajectories             |
| Composable forces          | Any combination of gravity/wind/turbulence may be attached; order of attachment is the order forces are summed |

## 7. Constraints (constitutional) and Scope Reduction

- No exceptions/panics; all fallible operations return a status/count, never throw.
- **Documented scope reduction for the Rust port:** the minimal port implements gravity
  only (the force actually used by the reference sandbox's collision sparks and ambient
  bursts) — wind and turbulence are specced above for completeness but not required for
  visual parity with the reference screenshots, and are a natural "next task." The port
  also uses a smaller pool capacity than the reference's 2048 (tuned for this training
  VM's software-rendering performance) — capacity is an implementation/tuning choice, not
  a behavioral guarantee.
