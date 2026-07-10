# Orbital Arena — Gravity Well: Language-Agnostic Specification

> **Scope note:** This spec covers the player-controlled gravity well used by the Orbital
> Arena GAME (`include/orbital_arena/gravity_well.h`), which is a DIFFERENT subsystem from
> the generic `physics::constraint_solver` covered in
> [physics-constraint.spec.md](physics-constraint.spec.md). The constraint solver is a
> rigid-link/rope solver; gravity wells are radial-attraction fields with a capture radius.
> They do not share code or formulas.

## 1. Purpose

A gravity well is a player-controlled avatar: a moving point that radially attracts nearby
objects (inverse-square falloff, clamped at close range) and "captures" objects that enter
a smaller inner radius.

## 2. Data Model

| Field            | Type         | Notes                                                |
| ---------------- | ------------ | ----------------------------------------------------- |
| position         | 2 × float    | World-space location                                  |
| velocity         | 2 × float    | Current velocity (kinematic, not integrated by others) |
| strength          | float [0,1]  | Player input, normalized                              |
| influence_radius | float        | Identical for all wells; beyond this, zero attraction |
| capture_radius   | float        | Identical for all wells; strictly inside = captured   |
| active           | bool         | False when the player has departed                    |

Constants (reference values): max well speed 2.0 units/s; max pull acceleration 40.0
units/s² at zero clamped distance and strength 1; minimum squared distance 0.01 (prevents
division blow-up at the well's own position).

## 3. Operations

### 3.1 Radial Acceleration

Given a well and a target position, compute the acceleration (units/s²) the well exerts on
that position:

- Zero if the well is inactive, strength is zero, or the target is farther than
  `influence_radius`.
- Otherwise: magnitude = `strength × max_pull_accel × (min_distance_sq / max(distance_sq, min_distance_sq))`
  — inverse-square falloff with a clamped minimum distance (prevents a singularity at the
  well's exact position).
- Direction: from the target toward the well's position.

### 3.2 Is Within Capture

True if and only if the target position is **strictly** inside `capture_radius` (compared
via squared distance — no square root needed) AND the well is active. A position exactly
on the boundary is NOT captured.

### 3.3 Step (Kinematic Update)

Advances a well by one fixed tick given a steering input:

1. Normalize the steer vector; clamp its magnitude to 1. Velocity = `steer_direction ×
   min(|steer|, 1) × max_well_speed`.
2. Position += velocity × dt.
3. Position is clamped to the arena's square bounds `[-half_extent, +half_extent]` on both
   axes.
4. Strength is set from the (separately clamped to [0,1]) strength input.
5. An inactive well does not move (position/velocity unchanged; strength may still update —
   implementation's choice, since inactive wells exert no acceleration regardless per §3.1).

## 4. Guarantees

| Guarantee               | Description                                                          |
| ------------------------ | --------------------------------------------------------------------- |
| Fairness                 | The acceleration/capture/step formulas take no player-index parameter — identical rules for every well |
| No singularity           | The clamped minimum distance prevents division by zero / infinite acceleration at the well's own position |
| Bounded movement         | A well's position never leaves the arena's square bounds              |
| Determinism              | Pure functions of (well state, inputs) — no RNG, no global state, no allocation |

## 5. Constraints (constitutional)

- No exceptions/panics; all functions are total (defined for every input).
- No allocation, no RNG draws in any of §3's operations.
- All arithmetic may use single-precision float (the reference C++ implementation does;
  this is a rendering/gameplay-feel subsystem, not a `double`-accumulator sim path per the
  engine's Article 5 — unlike the engine-foundation subsystems in Phase A, per-frame
  positions here are NOT required to be `double`).
