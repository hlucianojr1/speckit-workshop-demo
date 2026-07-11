# Orbital Arena — Match Orchestration (Aggregate Root): Language-Agnostic Specification

> **Scope note:** This spec covers the arena aggregate root
> (`include/orbital_arena/arena.h`, `src/orbital_arena/arena.cpp`) — the deterministic
> fixed-step tick pipeline that composes the subsystems specced individually in
> [gravity-well](orbital-arena-gravity-well.spec.md),
> [scoring](orbital-arena-scoring.spec.md), [match](orbital-arena-match.spec.md),
> [powerups](orbital-arena-powerups.spec.md),
> [input-log](orbital-arena-input-log.spec.md), and
> [snapshot](orbital-arena-snapshot.spec.md) — plus the pieces that exist only at the
> composition level: rng stream topology, rotationally-symmetric well placement and
> particle replenishment, flood bursts, and the hash history. This closes the last
> game-layer gap left by Phase A2 (Phase A5 — full-fidelity closure).

## 1. Purpose

Run one complete competitive match — lobby to game over — as a single deterministic
fixed-step system: (seed, input sequence) fully determines every subsequent state.

## 2. Configuration & Constants

### 2.1 Arena configuration (validated at creation; invalid → creation fails, no crash)

| Field             | Default | Valid range                    | Notes                                     |
| ----------------- | ------- | ------------------------------ | ------------------------------------------ |
| seed              | 1       | any u64                        | Shared match seed, visible to all players  |
| player_count      | 2       | 2 – 4                          |                                            |
| half_extent       | **5.0** | > 0                            | **Configuration, not constant:** the game default is 5.0; the reference sandbox embeds the arena with **2.0** to fit its view ([sandbox-scenes](sandbox-scenes.spec.md) §4.5). Visual-parity ports should use the sandbox value |
| particle_capacity | 500     | 1 – 512 (snapshot capacity)    | Target field size                          |

### 2.2 Gameplay constants (all durations are integer ticks at 60 ticks/s)

| Constant                  | Value              | Meaning                                  |
| ------------------------- | ------------------ | ----------------------------------------- |
| Tick                      | 1/60 s (double)    | Fixed step                                |
| Win score                 | 100                |                                           |
| Countdown                 | 180 ticks (3 s)    |                                           |
| Power-up spawn interval   | 600 ticks (10 s)   |                                           |
| Strength surge duration   | 300 ticks (5 s)    |                                           |
| Double points duration    | 600 ticks (10 s)   |                                           |
| State-hash interval       | 60 ticks (1 s)     |                                           |
| Replenish interval        | 60 ticks (1 s)     |                                           |
| Max field pickups         | 3                  |                                           |
| Flood burst size          | 100 particles      |                                           |
| Field particle max speed  | 0.1 units/s        | Initial drift; radial infall dominates    |
| Field particle lifetime   | effectively infinite (10⁹ s) | Capture is the only removal path |
| Input log capacity        | 4096 ticks         |                                           |
| Hash history capacity     | 4096/60 + 1 = 69 samples | Reserved once; sampling stops when full |

### 2.3 RNG stream topology (three isolated streams — normative)

| Stream        | Seed                          | Consumers (fixed draw order)                        |
| ------------- | ----------------------------- | ---------------------------------------------------- |
| game-rule     | `seed`                        | Power-up spawns only: x, y, kind per spawn           |
| flood emitter | `seed XOR 0x9E3779B97F4A7C15` | Particle Flood burst sampling only                   |
| field         | `seed XOR 0xC2B2AE3D27D4EB4F` | Replenishment: x, y, heading, speed per group        |

Salted sub-seeds guarantee that flood/replenishment draw *counts* can never perturb the
game-rule stream's fixed draw order.

## 3. Lifecycle Operations

- **create(allocator, config)** — validates config; allocates **all** pools, the input
  log, and the hash history up front; returns empty on invalid config or allocator
  exhaustion. After creation, ticking never allocates.
- **join / set_ready / leave / acknowledge_results** — lobby passthroughs to the
  [match state machine](orbital-arena-match.spec.md). Player indices are restricted to
  [0, player_count) so the identity player→well mapping stays dense.

## 4. The Tick Pipeline (one fixed step — this order IS the determinism backbone)

1. **Ingest:** clamp every player's raw input frame
   ([input-log spec](orbital-arena-input-log.spec.md) §3.1) and append the clamped
   frame to the log — in every match state. The tick's return status is the log's
   append status (`log_full` after 4096 ticks; gameplay still advances).
2. **Match state machine tick.** If this tick transitions into *playing*, run the
   playing-entry side effects (§4.1) exactly once and **skip step 3 this tick**.
3. **Playing systems** (only while playing):
   - **3a.** Inputs → wells: steer/strength drive each well's kinematic step
     ([gravity-well spec](orbital-arena-gravity-well.spec.md) §3.3), clamped to
     [−half_extent, +half_extent].
   - **3b/3c.** One fixed-order pass over live particles: sum every active well's
     radial acceleration (well index order; each well's strength multiplied by the
     owner's strength-surge multiplier), integrate velocity then position
     (Euler, dt = 1/60), then **reflect** at the arena bounds per axis
     (`pos > h → pos = 2h − pos`, velocity negated; symmetric at −h).
   - **3d.** Capture contention + same-tick scoring
     ([scoring spec](orbital-arena-scoring.spec.md)), with per-player double-points
     flags from the power-up system. Captured particles get their lifetime zeroed.
   - **3e.** Pickup consumption ([powerups spec](orbital-arena-powerups.spec.md)
     §3.2); merged flood requests spawn via the flood emitter (§4.2).
   - **3f.** Power-up spawn schedule (§powerups 3.1) — the only game-rule rng consumer.
   - **3g.** Effect timers decrement.
   - **3h.** Win / sudden-death evaluation; on a win: enter game over and end all
     effects immediately.
4. **Retirement:** age-and-retire the particle pool by one tick — captured particles
   (zeroed lifetime) physically leave here.
5. **Replenishment** (§4.3) on schedule.
6. **Hash sampling:** every 60 ticks (tick % 60 == 0), append
   `state_hash(capture_snapshot())` to the history (until full).
7. **Advance** the tick counter. The whole tick is self-measured into a frame-budget
   window (reference: ~0.08 ms at 500 particles, debug build, vs. the 16.67 ms budget).

### 4.1 Playing entry (runs once per match start)

Reset scores; clear all power-up state; place wells **rotationally symmetric**: the
base position is (half_extent · 0.5, 0) and well i receives the i-th of n symmetric
copies (§4.4). Wells activate with strength 0.

### 4.2 Flood bursts

Flood spawns go through the salted flood emitter (sphere of radius half_extent centered
at origin, speed 0 – 0.1, infinite lifetime); partial fill on pool exhaustion is
acceptable. Sampled 3D positions/velocities are projected onto the 2D game plane.

### 4.3 Field replenishment (deterministic, symmetric)

Every 60 ticks, while the deficit (capacity − live count) is at least one *group*
(group = player_count): draw **one** base position and velocity from the field stream —
fixed order x, y, heading, speed (position uniform over the arena square, heading
uniform angle, speed uniform 0 – 0.1) — then spawn `player_count` rotationally
symmetric copies of it (§4.4). Remainders smaller than a group are left unspawned until
next time.

### 4.4 Rotational symmetry (bit-exactness rule — normative)

The k-th of n symmetric copies of a 2D vector uses **exact sign/index forms, no
trigonometry**, for n = 2 (point reflection: negate both components) and n = 4 (exact
quarter turns by component swap + negation); n = 3 uses fixed cos/sin(120°) constants.
This is what makes game Article 9 *bit-exact*: for 2 players the entire world evolves
as an exact point reflection under mirrored inputs. (Consequence, verified in the
field: exactly-mirrored autopilot inputs produce a permanent tie — demo scripts must
detune per-player inputs. See [sandbox-scenes](sandbox-scenes.spec.md) §6.)

## 5. Snapshot / Hash / Replay Surface

- **capture_snapshot() / restore(snapshot)** — per the
  [snapshot spec](orbital-arena-snapshot.spec.md); restore installs every field
  verbatim. RNG internals are not snapshot state.
- **hash_history()** — the per-60-tick hash sequence; two clients in lockstep must
  produce identical sequences (Article 10).
- **Replay:** a fresh arena (same seed) fed the recorded log reproduces the identical
  hash history.

## 6. Queries (render/HUD boundary)

Match state, per-player score, winner (−1 = none), current tick, wells span, pickups
span, particle pool (read-only), and the tick-cost frame budget. Reading never mutates.

## 7. Guarantees

| Guarantee              | Description                                                     |
| ----------------------- | ------------------------------------------------------------------ |
| Lockstep determinism    | (seed, input sequence) → identical state and hash history          |
| Bit-exact fairness      | Symmetric placement/replenishment via exact arithmetic (§4.4)      |
| Zero steady-state allocation | Everything reserved at creation; tick never allocates          |
| Graceful saturation     | log_full / pool exhaustion / hash-history-full all degrade, never fail |
| Real-time               | Tick cost self-measured; reference ~0.08 ms ≪ 16.67 ms budget      |

## 8. Constraints (constitutional)

- No exceptions/panics; creation returns empty, tick returns a status (Article 1).
- All containers/pools take the injected allocator (Article 4).
- All gameplay durations are integer tick counts (Article 5).
- Implementation hazard (reference field note): the particle pool is placement-new'd at
  an allocator-stable address because the emitter holds an internal pointer to it that
  must survive moves; an embedding scene must destroy the arena **before** freeing the
  arena allocator's backing memory.
