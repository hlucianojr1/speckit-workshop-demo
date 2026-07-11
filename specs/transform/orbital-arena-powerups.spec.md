# Orbital Arena — Power-Up System: Language-Agnostic Specification

> **Scope note:** This spec covers the seed-driven power-up pickups and timed per-player
> effects of the Orbital Arena game (`include/orbital_arena/powerup.h`). It was
> **explicitly descoped by Phase A2** ([scoring](orbital-arena-scoring.spec.md) §
> constraints); Phase A5 (full-fidelity closure) removes that descope. Consumption
> contention reuses scoring's capture-resolution rule; the tick-pipeline position of
> every operation here is fixed by the
> [orchestration spec](orbital-arena-orchestration.spec.md).

## 1. Purpose

Periodically spawn neutral pickups on the field from the shared match seed; when a
player's well reaches one, grant a timed (or instant) effect that modifies gameplay —
all deterministically, fairly, and without allocation.

## 2. Data Model

### 2.1 Power-up kinds

| Kind            | Effect                                                      | Duration            |
| --------------- | ------------------------------------------------------------ | ------------------- |
| strength_surge  | Multiplies the winner's well pull strength                   | 300 ticks (5 s)     |
| double_points   | Winner's captures score double points                        | 600 ticks (10 s)    |
| particle_flood  | Instant: requests a neutral burst of **100** field particles | none (never occupies a slot) |

### 2.2 Storage (fixed slots — zero allocation after construction)

- **Field pickups:** exactly **3** slots (`kMaxFieldPickups`). Each holds: kind,
  2D position, spawn tick, alive flag.
- **Active effects:** exactly one slot per player (max 4 players). Each holds: kind +
  remaining ticks; `remaining_ticks == 0` means inactive.
- **Spawn timer:** a countdown initialized (and always reset) to **600 ticks (10 s)**.

**Effect slot rule:** one active-effect slot per player; the latest consumed *timed*
pickup occupies it (replacing any different-kind effect), and consuming the *same* kind
again refreshes the duration to full.

## 3. Operations (all called once per playing tick, in pipeline order — see orchestration §4)

### 3.1 tick_spawn(rng, current_tick, half_extent)

Decrements the spawn timer. When it reaches 0:

- Reset the timer to 600 ticks.
- Iff fewer than 3 pickups are alive: spawn one, drawing from the **game-rule rng**
  exactly **3 draws in fixed order** — x, y (both uniform over
  [−half_extent, +half_extent]), then kind (uniform over the 3 kinds).
- At the cap: the timer still resets, the spawn is skipped, and **no rng draws occur**
  (draw-on-spawn-only keeps the shared stream auditable — fairness Article 9: the
  randomization source is the shared seed, visible to all players).

### 3.2 consume_pickups(wells) → optional flood request

For each alive pickup, resolve contention using **scoring's capture rule** verbatim
(strict nearest well whose capture zone reaches the pickup; an exact distance tie means
nobody consumes it — see [scoring spec](orbital-arena-scoring.spec.md)). On a win:

- Timed kinds occupy/refresh the winner's effect slot (rule §2.2).
- particle_flood contributes 100 to a merged flood request returned to the caller
  (multiple floods the same tick merge into one request). Flood spawn *positions* are
  drawn by the arena's salted emitter stream, never the game-rule stream.

### 3.3 tick_effects()

Decrements every nonzero effect timer by one tick; an effect expires exactly when its
timer hits 0.

### 3.4 clear_all()

Match end (game_over) or match start: all effects end immediately, all pickups are
cleared, and the spawn timer resets to 600 for the next match.

### 3.5 Queries

- `is_active(player, kind)` — slot check.
- `strength_multiplier(player)` — > 1 iff strength_surge active (reference value: the
  multiplier applied to the well's pull strength during attraction).
- `points_multiplier(player)` — 2 iff double_points active, else 1.
- Snapshot support: expose the remaining spawn-timer ticks, and restore pickups /
  effects / timer verbatim from a snapshot ([snapshot spec](orbital-arena-snapshot.spec.md)).

## 4. Guarantees

| Guarantee            | Description                                                            |
| --------------------- | ------------------------------------------------------------------------ |
| Determinism           | All randomness from the injected shared-seed rng, fixed draw order (x, y, kind), draws only on actual spawns |
| Fairness              | No player-index parameter in spawn or consumption; contention uses the index-independent capture rule; shared seed is verifiable by all players |
| Zero allocation       | Fixed slot arrays only; nothing allocates after construction            |
| Integer-tick timing   | All durations are integer tick counts (never float accumulation)        |

## 5. Constraints (constitutional)

- No exceptions/panics; consumption returns an optional value, never throws.
- All timers/durations in ticks at 60 ticks/second (game Article 5).
- The rng passed to consumption is part of the contract surface but unused in v1
  (documented: flood positions come from the emitter sub-seed).
