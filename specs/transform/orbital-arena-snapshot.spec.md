# Orbital Arena — Match Snapshot & State Hash: Language-Agnostic Specification

> **Scope note:** This spec covers the flat POD match snapshot and the field-wise
> FNV-1a 64 state hash (`include/orbital_arena/snapshot.h`). It was **explicitly
> descoped by Phase A2**; Phase A5 (full-fidelity closure) removes that descope. The
> snapshot realizes game Article 11 (spectator-safe state); the hash realizes the
> drift-detection half of Article 10 (lockstep replay). Sampling cadence and history
> are owned by the [orchestration spec](orbital-arena-orchestration.spec.md).

## 1. Purpose

Represent the entire deterministic match state as one flat, pointer-free record — so a
struct copy IS the serialization — and reduce it to a single 64-bit hash for
cross-client drift detection.

## 2. Snapshot Data Model (field order below IS the hash order)

| #  | Field               | Type / shape                              | Notes                                    |
| -- | ------------------- | ----------------------------------------- | ----------------------------------------- |
| 1  | state               | match state enum                          | lobby / countdown / playing / game_over  |
| 2  | tick                | u64                                       | fixed-step tick counter                  |
| 3  | seed                | u64                                       | the shared match seed                    |
| 4  | player_count        | u8                                        |                                          |
| 5  | scores              | u32 × 4 (max players)                     |                                          |
| 6  | winner              | i8                                        | −1 = none                                |
| 7  | sudden_death        | bool                                      |                                          |
| 8  | wells               | 4 × { position 2×f32, velocity 2×f32, strength f32, active bool } |          |
| 9  | effects             | 4 × { kind enum, remaining_ticks u32 }    | remaining_ticks 0 = inactive             |
| 10 | pickups             | 3 × { kind enum, position 2×f32, spawn_tick u64, alive bool } |                       |
| 11 | powerup_timer_ticks | u32                                       | remaining spawn-timer ticks              |
| 12 | live_particle_count | u32                                       |                                          |
| 13 | particles           | 512 × { position 2×f32, velocity 2×f32 }  | only the first live_particle_count are state |

- Capacity invariant: snapshot particle capacity (512) ≥ target field size (500), so
  every live particle always fits.
- Particle records are the **2D projection** of the field particle (position +
  velocity only). Field particles never expire naturally (lifetime is effectively
  infinite; capture is the only removal path), so {position, velocity} fully
  characterizes one.
- **No pointers, no handles, no containers** anywhere in the record.

## 3. state_hash(snapshot) → u64

FNV-1a 64 (offset basis `0xcbf29ce484222325`, prime `0x100000001b3`) applied
**field-wise over every field's value in declared order — never over raw struct
bytes** (struct padding is indeterminate and would break cross-compiler equality).
Particle records beyond `live_particle_count` are slack, not state, and are excluded.
Pure function; allocation-free.

## 4. Operations (arena-level, see orchestration spec)

- **capture_snapshot()** — copies every field out of live state. Const, allocation-free.
- **restore(snapshot)** — installs every snapshot field verbatim (match phase, tick,
  scores, wells, effects, pickups, timer, particles).

**Documented limitation (normative):** rng engine internals are NOT flat-POD
representable and are NOT part of the snapshot. Restore is therefore a *state*
restore, not a *replay resume*: full replay is defined as (seed, input log) from tick
0 — see [input-log spec](orbital-arena-input-log.spec.md).

## 5. Guarantees

| Guarantee              | Description                                                       |
| ----------------------- | ------------------------------------------------------------------ |
| Spectator-safe          | Flat record; a memcpy/struct copy is a complete serialization      |
| Drift detection         | Two clients in lockstep produce identical hash sequences; the first differing sample localizes the desync tick to within one sampling interval |
| Padding-independence    | Field-wise hashing gives identical hashes across compilers/ABIs     |
| Allocation-free         | Capture, restore, and hash never allocate                          |

## 6. Constraints (constitutional)

- No exceptions/panics.
- Hash sampled by the arena every **60 ticks** into a pre-reserved history
  (orchestration §5); sampling stops silently when the history is full.
