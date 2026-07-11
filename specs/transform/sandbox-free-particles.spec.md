# Sandbox Free-Particle Physics — Language-Agnostic Specification

> **Scope note:** Reverse-specced from `apps/sandbox/scene.cpp`'s `m_particles` /
> `substep()` — a lightweight, scene-local ballistic particle simulation that is
> DISTINCT from both `physics::constraint_solver` (rope/rigid-link bodies) and
> `vfx::particle_pool` (render-only sparks/dust). This is the subsystem behind the
> "particles=N" HUD counter and the visible bouncing/orbiting dots in every base sandbox
> scene screenshot. It was never reverse-specced in any earlier phase.

## 1. Purpose

A small set of free-floating point particles with simple ballistic motion (position +
velocity, no constraints), driven by scene-specific force/boundary rules, that gives the
base sandbox scenes visual life beyond the rigid rope/cloth/pendulum bodies. Distinct
from VFX particles: these ARE part of the simulated (though not gameplay-critical) world
and DO contribute to the scene's deterministic state (unlike VFX particles, which are
strictly render-only).

## 2. Data Model

| Field    | Type          | Notes                    |
| -------- | ------------- | -------------------------- |
| position | 2 × float64   | World-space location       |
| velocity | 2 × float64   | Current velocity           |
| radius   | float64       | Visual size, set at spawn  |

A fixed initial population exists per scene (reference: 32); more can be added later via
a burst-spawn operation (position, count) → each new particle gets a random outward
velocity and a random radius within a small range, drawn from the scene's own seeded RNG.

## 3. Per-Scene Force and Boundary Rules (this is the interesting part — behavior is
   NOT universal, it varies by which "stage" is active)

| Scene variant                  | Force applied                              | Boundary behavior                                                     |
| -------------------------------- | -------------------------------------------- | -------------------------------------------------------------------------- |
| Default ("rope"-like stage)     | Light downward gravity (weaker than the rope's own gravity, for visual variety) | Bounce off a fixed rectangular boundary with velocity damping (~0.85×); **each bounce triggers a VFX collision spark** (see vfx-particle-system.spec.md §5) at the contact point |
| Twin-well variant                | Two fixed-position inverse-square attraction wells + soft velocity damping (~0.999×/tick, to prevent runaway orbits) | Wraps around a generous boundary instead of bouncing (particles feel "infinite") — no bounce, no sparks |
| Delegated variant                 | N/A — this scene kind hands its ENTIRE free-particle simulation to a different subsystem (its own game/embedded system) | N/A |

## 4. Operations

| Operation           | Signature (abstract)             | Behavior                                              |
| -------------------- | ----------------------------------- | -------------------------------------------------------- |
| burst_spawn          | `(position, count) → void`         | Appends `count` new particles at `position` with randomized outward velocity/radius (drawn from the scene's RNG stream — NOT the VFX emitter's stream, a separate consumer) |
| step (per force-rule table in §3) | `(dt_seconds) → void` | Integrates every particle by one fixed tick per its scene variant's rule |

## 5. Guarantees

| Guarantee                  | Description                                                              |
| ---------------------------- | ---------------------------------------------------------------------------- |
| Deterministic per variant    | Same scene variant + same RNG stream + same sequence of bursts ⇒ identical trajectories |
| Bounded growth               | Bursts add particles; nothing removes them automatically in this subsystem (they persist for the scene's lifetime) — a caller-side cap on burst frequency/count is the only bound |
| Scene-state contributing     | Unlike VFX particles, free-particle position/velocity ARE part of the scene's deterministic state — a state hash/digest over the scene must include them |
| Rule isolation                | Each scene variant's rule is self-contained; switching variants swaps the rule wholesale, never blends two variants' forces |

## 6. Constraints (constitutional) and Scope Reduction

- No exceptions/panics; no unbounded per-frame allocation once the fixed initial
  population exists (a `try_spawn`-shaped bound is a reasonable target for a port even
  though the reference C++ implementation here uses an always-growing `eastl::vector`
  for burst-added particles — a port MAY choose a capacity-reserved fixed-size collection
  instead, which is a strictly stronger guarantee than the reference).
- **Documented scope reduction for the Rust port:** only the "default (rope-like)"
  variant's force/boundary rule (§3 row 1) is required for parity with the reference
  screenshots that motivated this spec. The twin-well and delegated variants are
  out of scope for this port (the delegated variant's role is already served by the
  Phase A2/A3 Orbital Arena game's own particle field, which follows a related but not
  identical rule set).
