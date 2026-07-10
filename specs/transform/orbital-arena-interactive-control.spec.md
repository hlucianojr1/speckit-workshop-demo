# Orbital Arena — Interactive Well Control: Specification (New Capability)

> **This spec documents a NEW capability, not a reverse-spec of existing C++ behavior.**
> The reference C++ Orbital Arena scene (`apps/sandbox/scene.cpp`'s `make_orbital_inputs`)
> drives BOTH players with a scripted deterministic sinusoidal autopilot — there is no
> mouse-driven well control in the reference implementation. What the reference C++
> sandbox DOES have is a generic, scene-agnostic "LMB drag node" mechanic
> (`app.cpp`: `grab_nearest_rope_node` / `drag_held_node`) that lets the player grab and
> drag the nearest PHYSICS NODE (rope/pendulum/cloth body) to the cursor. This spec adapts
> that generic interaction pattern — "press LMB near an object, drag it to the cursor,
> release to let go" — to a gravity well instead of a rope node, as a new, explicitly
> invented capability for this training's Rust port. It is written as a spec (not just
> code) so its origin and scope are as auditable as every reverse-specced subsystem.

## 1. Purpose

Let a player click-and-drag one gravity well directly, making the default Rust scene
interactively explorable instead of purely a scripted autopilot demo.

## 2. Behavioral Contract

- While the left mouse button is held AND the cursor (converted to world space) is within
  a grab radius of the designated interactive well (reference: well index 0), that well's
  position is set to the cursor's world position every frame, clamped to the arena's
  square bounds (same clamp rule as `orbital-arena-gravity-well.spec.md` §3.3).
- The well's velocity while dragged is derived from consecutive positions (for visual/
  physics consistency with anything that reads well velocity), not left stale.
- All other wells continue under their existing driving logic (autopilot or otherwise)
  unaffected by the drag.
- On release, the dragged well resumes its normal driving logic (autopilot) from the
  current tick — an abrupt position discontinuity on release is ACCEPTABLE for this
  minimal interactive addition (unlike the reference "LMB drag node," which has no
  "autopilot to resume" concept for rope nodes).
- Grabbing is idempotent per press: pressing LMB when the cursor is not near the
  interactive well is a no-op (does not grab a different well or object).

## 3. Guarantees

| Guarantee               | Description                                                             |
| ------------------------ | -------------------------------------------------------------------------- |
| Bounded                 | The dragged well's position never leaves the arena bounds (§2)             |
| Non-destructive          | Dragging never removes/adds entities, never allocates                     |
| Deterministic elsewhere  | Every OTHER well and all particles are unaffected by drag input — the
  drag is purely additive to well 0's own position update                    |
| No panics               | Missing cursor / missing camera / no active window are handled gracefully (early-return), never a panic |

## 4. Constraints (constitutional)

- No exceptions/panics (Article I): cursor-position queries that can fail (cursor outside
  window, no primary window) must be handled with early-return, not `unwrap()`.
- No allocation in the drag system's steady state (Article IV).
- This is explicitly a **New** capability (per the Inherited/Adapted/New tagging
  convention used throughout `specs/transform/`), not Inherited or Adapted from any single
  C++ header — it is a synthesis of two existing patterns (`app.cpp`'s generic drag
  mechanic + `gravity_well`'s existing position/bounds model) applied somewhere the
  reference implementation does not apply it.
