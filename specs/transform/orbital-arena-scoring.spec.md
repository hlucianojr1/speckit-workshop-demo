# Orbital Arena — Scoring and Capture Contention: Language-Agnostic Specification

## 1. Purpose

Resolve which player (if any) captures a contested object each tick, award points, and
evaluate the win condition. This is what turns a physics simulation (gravity wells pulling
particles) into a competitive GAME.

## 2. Data Model

| Field          | Type                    | Notes                                             |
| -------------- | ----------------------- | -------------------------------------------------- |
| scores         | array[uint32] per player | One counter per player slot                      |
| winner         | optional player index   | Set once, then frozen (no further score changes)   |
| sudden_death   | bool                    | True once ≥2 players are simultaneously above the win threshold |

Reference win threshold: 100 points.

## 3. Operations

### 3.1 Resolve Capture (single position)

Given the set of wells and one contested position: among the wells whose capture radius
(§ gravity-well.spec.md 3.2) contains the position, find the one at the **strictly**
smallest distance. Iterate wells in a fixed index order but compare distance VALUES only
— the result must never depend on well iteration order, only on the distances themselves.

- No well's capture radius contains the position ⇒ no winner, no tie.
- Exactly one nearest well ⇒ that well's player wins.
- Two or more wells exactly tied at the strict minimum distance ⇒ no winner, tie = true
  (contested captures are voided, not arbitrarily broken by index — this is a fairness
  guarantee, not an edge-case afterthought).

### 3.2 Award Capture

Add a base point value to a player's score (doubled if a "double points" effect is active
for that player — see [orbital-arena-powerups.spec.md](orbital-arena-powerups.spec.md)).
No-op if a winner has already
been latched (score freeze after game over).

### 3.3 Resolve Captures (whole field, one tick)

Apply §3.1 to every live contested object in one pass (fixed, stable iteration order —
never a hash-map or otherwise unordered iteration). Each captured object: award its
winning well's player 1 point (§3.2), then remove the object from play. Produce a
bitmask/set of "which players captured at least one object this tick" for §3.4.

### 3.4 Evaluate Win

Called once per tick, after all captures resolve:

- Exactly one player is at/above the win threshold ⇒ latch that player as winner, return it.
- Two or more players are at/above the threshold simultaneously ⇒ enter sudden-death (no
  winner yet).
- While in sudden-death: if exactly one of the threshold-or-above players captured an
  object THIS tick, that player wins (first to score again after the tie breaks it).
- Once a winner is latched, re-evaluation always returns the same winner (frozen).

### 3.5 Reset

Clears scores, winner, and sudden-death — used when a match restarts.

## 4. Guarantees

| Guarantee                  | Description                                                             |
| ---------------------------- | ------------------------------------------------------------------------ |
| Index-independent fairness   | Contention resolution never uses player/well index as a tiebreaker — only distance |
| Score freeze                 | No score changes after a winner is latched                             |
| Deterministic evaluation      | Same sequence of captures ⇒ same winner, same tick, every run           |
| No allocation                | All of §3 is pure computation over fixed-size inputs, no heap traffic    |

## 5. Constraints (constitutional) and Scope Reduction

- No exceptions/panics; no RNG in this module.
- Deterministic, stable iteration order (Article: determinism).
- **Documented scope reduction for the Rust port (superseded for full fidelity):** the
  minimal Phase A2 port omitted power-ups entirely (every capture worth exactly 1
  point). **Phase A5 removes that reduction:** the timed effects that modify
  `award_capture`'s point value are now specced in
  [orbital-arena-powerups.spec.md](orbital-arena-powerups.spec.md), and a
  full-fidelity recreation implements them. The base rules in §3 are unchanged.
