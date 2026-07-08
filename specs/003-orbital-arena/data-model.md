# Data Model: Orbital Arena

**Feature**: 006-orbital-arena | **Date**: 2026-07-08
**Sources**: [spec.md](spec.md) Key Entities + FRs; [research.md](research.md) D1–D8.

All types live in `namespace orbital_arena`. All containers are EASTL with an explicit
`engine_demo::allocator&` (Articles 3–4). All fallible operations return `arena_status`
(Article 1). No virtual functions, no RTTI (Article 2).

## Constants (`arena.h`, all `constexpr`)

| Constant | Value | Source |
|---|---|---|
| `kMaxPlayers` | 4 | FR-001 |
| `kMinPlayers` | 2 | FR-001 |
| `kWinScore` | 100 | FR-006 |
| `kTicksPerSecond` | 60 | existing fixed step 1/60 |
| `kCountdownTicks` | 180 (3 s) | FR-012 |
| `kPowerupSpawnIntervalTicks` | 600 (10 s) | FR-009 |
| `kMaxFieldPickups` | 3 | FR-009 |
| `kStrengthSurgeTicks` | 300 (5 s) | FR-010 |
| `kDoublePointsTicks` | 600 (10 s) | FR-010 |
| `kStateHashIntervalTicks` | 60 | Article 10 |
| `kTargetParticleCount` | 500 | assumption (particle_storm parity) |
| `kMaxSnapshotParticles` | 512 | D8 (≥ target population) |
| `kArenaHalfExtent` | tunable, e.g. 5.0f | assumption (fixed bounded arena) |

## Enums

```text
arena_status : uint8_t   { ok, invalid_argument, wrong_state, log_full }
match_state  : uint8_t   { lobby, countdown, playing, game_over }
powerup_kind : uint8_t   { strength_surge, double_points, particle_flood }
```

## Entities

### `gravity_well` (gravity_well.h)

Player avatar. Value type, POD.

| Field | Type | Rules |
|---|---|---|
| `position` | `float[2]` | clamped to arena bounds every tick (US1-4) |
| `velocity` | `float[2]` | magnitude ≤ `kMaxWellSpeed`, identical cap for all (FR-002) |
| `strength` | `float` | 0..1 normalized player input × `kMaxStrength` (FR-003) |
| `influence_radius` | `float` | identical for all wells (FR-001); pull = 0 beyond it |
| `capture_radius` | `float` | identical for all wells |
| `active` | `bool` | false when player departs (FR-013) |

Free functions: `radial_acceleration(well, particle_pos) -> float[2]` (inverse-square,
clamped min distance, zero beyond influence radius, scales with `strength`);
`is_within_capture(well, pos) -> bool` (squared-distance compare, no sqrt).

### Particle — **reused**: `engine_demo::vfx::particle`

No new type. Point value is implicitly 1 (all standard particles, spec assumption);
capture zeroes `remaining_lifetime_seconds` (D2). Replenishment/flood bursts go through
a `vfx::emitter` owned by the arena (D6 salted seed).

### `pickup` (powerup.h)

| Field | Type | Rules |
|---|---|---|
| `kind` | `powerup_kind` | seed-drawn on spawn (FR-010) |
| `position` | `float[2]` | seed-drawn within arena bounds (FR-009) |
| `spawn_tick` | `uint64_t` | bookkeeping / snapshot |
| `alive` | `bool` | max 3 alive; slot array of `kMaxFieldPickups` (no allocation) |

### `active_effect` (powerup.h)

One slot per player (an effect of each timed kind can be active; v1: latest of same kind
refreshes duration — simplest deterministic rule, documented in contract).

| Field | Type | Rules |
|---|---|---|
| `kind` | `powerup_kind` | `particle_flood` never occupies a slot (instant) |
| `remaining_ticks` | `uint32_t` | decremented in `playing` only; 0 = inactive; all cleared at `game_over` (FR-011) |

### `player_slot` (match.h)

| Field | Type | Rules |
|---|---|---|
| `joined` | `bool` | roster fixed at lobby (FR-015) |
| `ready` | `bool` | all joined ready → countdown (FR-012) |
| `departed` | `bool` | during playing: well deactivates (FR-013) |

### `score_table` (scoring.h)

`uint32_t scores[kMaxPlayers]`, all 0 outside `playing` history; mutated only via
`award_capture(player, base_value, double_points_active)` during `playing` (FR-005).
`winner` : `int8_t` (−1 = none). `sudden_death` : `bool` (FR-008).

### `input_frame` / input log (input.h)

| Field | Type | Rules |
|---|---|---|
| `steer` | `float[2]` | clamped to unit box on ingest (FR-015) |
| `strength` | `float` | clamped 0..1 on ingest |

`tick_inputs` = `input_frame[kMaxPlayers]`. Log: `eastl::vector<tick_inputs>` reserved
once for `kMaxRecordedTicks` (Article 6); append returns `log_full` when exhausted.
All inputs logged regardless of match state; rules honor them only in `playing`
(FR-014, FR-016).

### `match_snapshot` (snapshot.h) — Article 11

Flat POD, no pointers/handles/containers (FR-018):

```text
match_state  state
uint64_t     tick
uint64_t     seed
uint8_t      player_count
uint32_t     scores[4]
int8_t       winner
bool         sudden_death
well_record  wells[4]           # pos, vel, strength, active
effect_record effects[4]        # kind, remaining_ticks
pickup_record pickups[3]        # kind, pos, spawn_tick, alive
uint32_t     powerup_timer_ticks
uint32_t     live_particle_count
particle_record particles[512]  # pos[2], vel[2]
```

`state_hash(snapshot) -> uint64_t`: field-wise FNV-1a 64 in declared order (D5).

### `arena` (arena.h) — aggregate root

Owns: `engine_demo::allocator&` (injected), `ecs::world`, `vfx::particle_pool`,
`vfx::emitter`, `sim::rng` (game-rule stream), `engine_demo::frame_budget`,
`match` (state machine), `score_table`, pickup/effect arrays, input log.

## State Transitions (`match.h`)

```text
lobby      --all joined (≥2) ready-->                    countdown (timer := 180)
countdown  --player unready/leave-->                     lobby
countdown  --timer == 0-->                               playing (scores := 0, wells reset symmetric)
playing    --exactly one player ≥ 100 on tick-->         game_over (winner := that player)
playing    --≥2 players reach ≥100 same tick-->          playing [sudden_death := true]
playing[sudden_death] --exactly one contender captures--> game_over
playing    --active players < 2-->                       game_over (winner := last remaining)
game_over  --acknowledge-->                              lobby (scores reset, effects cleared)
```

Invariants:

- Scores mutate only in `playing` (FR-005); effects tick down only in `playing`.
- All effects end instantly on entry to `game_over` (FR-011).
- Well movement/strength input honored only in `playing` (FR-014).
- Every tick in every state appends to the input log (edge case: faithful replay).

## Tick Pipeline (`arena::tick`, fixed order — determinism backbone)

1. Ingest + clamp + log `tick_inputs` (any state).
2. Match state machine update (ready/countdown/acknowledge transitions).
3. If `playing`:
   a. Apply inputs → well velocity/strength; integrate wells; clamp to bounds.
   b. Apply radial attraction from each active well to every live particle
      (well index order 0..N, particle index order — fixed draw-free loop).
   c. Integrate particle velocities → positions; reflect at arena bounds.
   d. Resolve particle captures (D3 contention); zero lifetimes; award scores
      (×2 under Double Points).
   e. Resolve pickup consumption (same contention rule); apply effects
      (flood → emitter burst with seed-derived positions).
   f. Power-up spawn schedule (timer, cap 3, rng draws only on spawn).
   g. Decrement effect timers; expire.
   h. Win/sudden-death evaluation (FR-006/008); possibly → `game_over`.
4. `particle_pool.age_and_retire(dt)` (captures physically removed here — D2).
5. Replenishment emitter tick (deterministic schedule).
6. Every 60 ticks: capture snapshot → `state_hash` → append to hash history.
7. `tick += 1`.
