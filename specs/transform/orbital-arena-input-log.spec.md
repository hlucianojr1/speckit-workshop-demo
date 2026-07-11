# Orbital Arena — Input Frames & Replay Log: Language-Agnostic Specification

> **Scope note:** This spec covers the per-tick player input model and the append-only
> replay log (`include/orbital_arena/input.h`). It was **explicitly descoped by Phase
> A2**; Phase A5 (full-fidelity closure) removes that descope. Replay is the game's
> lockstep-determinism proof (game Article 10): a match is fully defined by
> (seed, input log).

## 1. Purpose

Capture every player's steering input for every fixed tick in a bounded, append-only
log, storing exactly the values the simulation consumed — so replaying the log against
the same seed reproduces the match bit-for-bit.

## 2. Data Model

### 2.1 Input frame (one player, one tick)

| Field    | Type      | Range (post-clamp)     |
| -------- | --------- | ----------------------- |
| steer    | 2 × float | each axis in [−1, 1]    |
| strength | float     | [0, 1]                  |

### 2.2 Tick inputs

One input frame per roster slot (fixed array of max players = 4). **Player→well
mapping is identity by roster slot, fixed at lobby time** — player i's frame always
drives well i; no remapping exists.

### 2.3 Log capacity

The log is reserved once at construction for a maximum of **4096 ticks**
(`kMaxRecordedTicks` — comfortably above 1000-frame test matches) and never
reallocates.

## 3. Operations

### 3.1 clamp_input(raw) → frame

Clamps steer axes to [−1, 1] and strength to [0, 1]. **NaN maps to 0** — a
deterministic policy, so replaying logged values can never re-clamp differently.

### 3.2 append(tick_inputs) → status

Appends one tick's (already-clamped) inputs. Once the capacity is reached, returns a
`log_full` status and drops the frame — **gameplay still advances**; only recording
stops. Never allocates.

### 3.3 at(tick), size(), clear()

Random access by tick index (precondition: index < size). `clear()` resets for a new
match while retaining the reserved capacity.

## 4. Behavioral Contract

- **Log-what-you-simulate:** the log stores **post-clamp** values — the exact numbers
  the simulation consumed, not the raw device input.
- **Record in every state:** inputs are logged for every tick in every match state
  (lobby, countdown, playing, game over); gameplay rules honor them only while
  playing.
- **Replay definition:** feeding `log.at(0..size)` back into a fresh arena created
  with the same seed reproduces the identical state-hash history (see
  [snapshot spec](orbital-arena-snapshot.spec.md) and
  [orchestration spec](orbital-arena-orchestration.spec.md) §6).

## 5. Guarantees

| Guarantee          | Description                                                     |
| ------------------- | ---------------------------------------------------------------- |
| Determinism         | Post-clamp storage + NaN→0 policy make replay bit-exact          |
| Bounded memory      | One up-front reservation; append never allocates (Article 6)     |
| Graceful saturation | log_full is a status, not an error state; simulation continues   |
| Identity mapping    | Slot i ↔ well i, immutable for the match                        |

## 6. Constraints (constitutional)

- No exceptions/panics; saturation returns a status value.
- Storage comes from the injected allocator at construction (Article 4).
- Reference note: on long interactive runs (> 4096 ticks) `log_full` is expected and
  the embedding scene intentionally ignores it.
