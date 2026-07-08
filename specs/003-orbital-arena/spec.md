# Feature Specification: Orbital Arena — Competitive Gravity-Well Game

**Feature Branch**: `006-orbital-arena`  
**Created**: 2026-07-08  
**Status**: Draft  
**Input**: User description: "Orbital Arena — a competitive 2D physics game built on the engine_demo foundation. 2-4 players each control a gravity well. Free-floating particles are attracted to wells. Players score by capturing particles into their well. Power-ups spawn periodically. First to 100 points wins."

**Constitution**: Bound by the Orbital Arena constitution (Articles 1–11, [specs/orbital-arena/constitution.md](../orbital-arena/constitution.md)). This spec covers only the **delta** over existing engine_demo subsystems (see Dependencies).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Control a Gravity Well and Capture Particles (Priority: P1)

A player steers their gravity well around the arena and modulates its pull strength. Nearby free-floating particles curve toward the well; when a particle falls into the well's capture zone it is absorbed and the player's score increases.

**Why this priority**: This is the core game loop — attraction, pursuit, and capture. Without it there is no game; with only this, a single player can already play a meaningful sandbox round.

**Independent Test**: Start a match with one active well and a field of particles; move the well near a particle cluster at full strength and verify particles accelerate toward the well and are removed (and counted) when they enter the capture zone.

**Acceptance Scenarios**:

1. **Given** a particle within the well's influence radius, **When** the player holds full strength, **Then** the particle visibly accelerates toward the well each tick.
2. **Given** a particle crossing into the well's capture radius, **When** the tick completes, **Then** the particle is removed from play and the owning player's score increases by that particle's point value.
3. **Given** a player sets strength to minimum, **When** particles pass nearby, **Then** their trajectories are unaffected (zero pull at minimum strength).
4. **Given** a well at the arena boundary, **When** the player steers outward, **Then** the well stops at the boundary and never leaves the arena.
5. **Given** two wells at *exactly* equal distance from one particle inside both capture radii on the same tick, **When** capture is resolved, **Then** the outcome is deterministic and does not depend on player index (Article 9): the particle is captured by neither and remains in play until the distances differ.

---

### User Story 2 - Win a Match (Scoring & Match Flow) (Priority: P2)

Players gather in a lobby, ready up, watch a countdown, play until someone reaches 100 points, and see a game-over screen declaring the winner with final scores.

**Why this priority**: Turns the P1 sandbox loop into a competitive game with a beginning, middle, and end. Depends on P1 capture events as its input.

**Independent Test**: Drive a scripted match (deterministic inputs) through lobby → countdown → playing → game_over and verify the first player to reach 100 points is declared winner and scores freeze immediately.

**Acceptance Scenarios**:

1. **Given** a lobby with 2–4 joined players, **When** all players are ready, **Then** the match transitions to a 3-second countdown.
2. **Given** the countdown is running, **When** it expires, **Then** the match enters `playing`, all wells activate with identical starting conditions (Article 9), and scores start at 0.
3. **Given** a player at 99 points, **When** they capture a 1-point particle, **Then** the match transitions to `game_over` on that same tick, that player is the winner, and no further captures score.
4. **Given** two players who would both reach ≥100 on the same tick, **When** the tick resolves, **Then** the match enters sudden-death overtime and the next single capture decides the winner (no player-index tie-break, per Article 9).
5. **Given** the match is in `game_over`, **When** results are acknowledged, **Then** the match returns to `lobby` with scores reset.
6. **Given** the match is in `lobby` or `countdown`, **When** a player provides movement input, **Then** wells do not move and no scoring occurs (inputs outside `playing` have no game effect).

---

### User Story 3 - Power-Ups Shake Up the Match (Priority: P3)

Periodically a power-up pickup appears at a position all players can see and contest. The first well to reach it consumes it and gains a timed effect.

**Why this priority**: Adds contest points and comeback potential, but the game is fully playable without it.

**Independent Test**: Run a deterministic match with a fixed seed; verify power-ups spawn on schedule at seed-determined positions, are consumed by first-arrival, apply their effect for the stated duration, and expire cleanly.

**Acceptance Scenarios**:

1. **Given** a match in `playing`, **When** the spawn interval (10 s) elapses and fewer than 3 pickups are on the field, **Then** one power-up spawns at a position determined by the shared match seed (Article 9 — verifiable by all players).
2. **Given** a spawned pickup, **When** a well's capture zone first touches it, **Then** that player gains the effect and the pickup disappears; if two wells touch it on the same tick at exactly equal distance, neither consumes it (same rule as US1 scenario 5).
3. **Given** a player with an active *Strength Surge* (double pull strength, 5 s), **When** 5 s elapse, **Then** pull strength reverts to the player-controlled value.
4. **Given** a player with an active *Double Points* (10 s), **When** they capture particles, **Then** each capture scores twice its base value; captures after expiry score base value.
5. **Given** a *Particle Flood* is consumed, **When** the effect triggers, **Then** a burst of additional neutral particles spawns at seed-determined positions available to all players equally.
6. **Given** any active effect, **When** the match transitions to `game_over`, **Then** all effects end immediately.

---

### User Story 4 - Fair, Replayable Matches (Priority: P4)

A spectator (or a dispute review) replays a finished match from its seed and recorded input log and observes an identical match — same captures, same scores, same winner.

**Why this priority**: Mandated by Articles 10–11; it is the verification backbone for every other story, but delivers direct user value (spectating/replay) only after the game itself works.

**Independent Test**: Record the input log of a 1000-frame scripted match; replay it from the same seed and verify the per-frame state hashes (every 60 frames) and final scoreboard match exactly.

**Acceptance Scenarios**:

1. **Given** a completed match's seed and input log, **When** the match is replayed, **Then** every 60-frame state hash and the final result are identical to the original run.
2. **Given** the same scripted inputs assigned to different player slots (inputs swapped between players), **When** two matches run, **Then** the outcomes are mirror-equivalent — no advantage attaches to any player index (Article 9).

---

### Edge Cases

- **Simultaneous capture claims**: one particle inside two capture zones on the same tick → nearest well wins; exact distance tie → no capture that tick (deterministic, index-free).
- **Reaching 100 mid-effect**: Double Points pushing a player from 99 past 100 in one capture still counts fully; match ends that tick.
- **Empty arena**: particle replenishment keeps the field populated; if captures outpace spawning, the match continues (no deadlock) and new particles arrive on the deterministic spawn schedule.
- **Player departure mid-match**: a departed player's well deactivates (zero strength, no capture); the match continues for remaining players; if fewer than 2 remain, the match transitions to `game_over` with the last remaining player as winner.
- **Pickup cap**: no more than 3 uncollected pickups on the field; spawn timer keeps running but spawns are skipped until below the cap.
- **Countdown interruption**: a player un-readying or leaving during countdown returns the match to `lobby`.
- **Input outside playing state**: silently ignored for gameplay, but still recorded in the input log for faithful replay.

## Requirements *(mandatory)*

### Functional Requirements

**Gravity Well**

- **FR-001**: Each of 2–4 players MUST own exactly one gravity well with identical capabilities and starting conditions (position symmetry, equal base strength).
- **FR-002**: Players MUST be able to steer their well continuously within arena bounds; the well's speed is capped and identical for all players.
- **FR-003**: Players MUST be able to vary their well's pull strength continuously between zero and a fixed maximum; attraction applied to each particle scales with strength and diminishes with distance, reaching zero beyond a finite influence radius.
- **FR-004**: A well MUST capture any particle whose position enters its capture radius, subject to the deterministic contention rule (FR-007).

**Scoring**

- **FR-005**: Each captured standard particle MUST award 1 point to the capturing player; scores are per-player, start at 0, and only change during the `playing` state.
- **FR-006**: The match MUST end the moment exactly one player's score reaches or exceeds 100; that player is the winner.
- **FR-007**: When two or more wells could capture the same particle (or pickup) on the same tick, the nearest well MUST win; on an exact distance tie, no capture occurs that tick. Resolution MUST be independent of player index (Article 9) and fully deterministic (Article 10).
- **FR-008**: If two or more players would reach ≥100 on the same tick, the match MUST enter sudden-death: the next tick on which exactly one contender captures decides the winner.

**Power-Ups**

- **FR-009**: During `playing`, a power-up pickup MUST spawn every 10 seconds at a position derived from the shared match seed, unless 3 uncollected pickups already exist (spawn skipped, timer continues).
- **FR-010**: The pickup type MUST be drawn from exactly three kinds, each seed-determined: *Strength Surge* (pull strength doubled for 5 s), *Double Points* (captures score 2× for 10 s), *Particle Flood* (immediate seed-determined burst of neutral particles).
- **FR-011**: A pickup MUST be consumed by the first well whose capture zone reaches it (contention per FR-007); timed effects MUST expire exactly at their stated duration and all effects MUST end at `game_over`.

**Match State Machine**

- **FR-012**: The match MUST progress only through the states `lobby → countdown → playing → game_over → lobby`, with these transitions: all joined players ready → countdown; 3-second countdown elapsed → playing; win condition (FR-006/FR-008) or fewer than 2 active players → game_over; results acknowledged → lobby (scores reset).
- **FR-013**: A player leaving or un-readying during `countdown` MUST return the match to `lobby`; a player leaving during `playing` MUST deactivate their well while the match continues (unless fewer than 2 players remain).
- **FR-014**: Game-affecting input MUST be honored only in the `playing` state; in all other states it has no gameplay effect.

**Input**

- **FR-015**: Each player's input per tick MUST consist of a 2-axis steering direction and a strength value, mapped to exactly one well; the mapping is fixed at lobby time.
- **FR-016**: All player inputs MUST be sampled once per simulation tick and recorded in order into a per-match input log sufficient to replay the match (Article 10).

**Determinism & Replay (cross-cutting)**

- **FR-017**: Given the same match seed and input log, a replay MUST reproduce identical game state — verified by state hashes taken every 60 frames and by the final scoreboard (Article 10).
- **FR-018**: The complete match state at any tick MUST be expressible as a flat, self-contained snapshot with no references into live memory, so it can be stored, transmitted, and restored (Article 11).
- **FR-019**: All in-match randomness (power-up timing positions/types, particle spawn positions) MUST derive from the single shared match seed disclosed to all players (Article 9).

### Key Entities

- **Gravity Well**: A player's avatar. Attributes: owning player, position, velocity, current strength (0..max), influence radius, capture radius, active/inactive.
- **Particle**: A neutral capturable object. Attributes: position, velocity, point value (standard = 1). Removed on capture.
- **Power-Up Pickup**: A field object. Attributes: kind (Strength Surge / Double Points / Particle Flood), spawn position, spawn tick.
- **Active Effect**: A timed modifier on a player. Attributes: kind, owning player, remaining duration.
- **Match**: The authoritative session. Attributes: state (lobby/countdown/playing/game_over), shared seed, tick counter, per-player scores, winner, roster of 2–4 players with ready flags.
- **Input Record**: One player's input for one tick (steering axes + strength), appended to the match input log.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A full 2-player match can be played from lobby to declared winner without manual intervention beyond player inputs; first player to 100 points always wins.
- **SC-002**: Replaying any finished match from its seed and input log reproduces the identical winner, final scores, and all periodic state checkpoints, 100% of the time across 1000-frame test matches.
- **SC-003**: Swapping the same input scripts between player slots produces mirror-equivalent outcomes — measured score difference of zero attributable to player index.
- **SC-004**: A capture is reflected in the capturing player's score on the same simulation tick it occurs (players perceive scoring as instantaneous).
- **SC-005**: Matches support 2, 3, and 4 players with the full particle field while maintaining the engine's real-time frame budget (no perceptible slowdown at 4 players).
- **SC-006**: Power-ups spawn on schedule in 100% of test matches, and every timed effect ends within one tick of its stated duration.

## Assumptions

- **Arena**: a single fixed-size 2D bounded arena; wells clamp at boundaries; particles that would exit are kept in play by the boundary (no despawn-by-exit). Arena layout gives all starting positions rotational symmetry (fairness).
- **Point values**: all standard particles are worth 1 point; only the *Double Points* effect changes scoring.
- **Particle replenishment**: the arena maintains a target particle population; replacements spawn on a deterministic, seed-derived schedule so captures don't exhaust the field.
- **Countdown length** is 3 seconds; power-up cadence 10 s; effect durations 5 s (Strength Surge) and 10 s (Double Points); pickup cap 3. These are tunable constants, not new mechanics.
- **Local multiplayer**: all 2–4 players play on one machine (distinct input mappings); networked play is out of scope, but Articles 10–11 keep the design rollback/netcode-ready.
- **Rendering/visualization** of the arena is out of scope for this feature (handled by the sandbox app as a separate task); this spec covers game rules and state only.
- **Tie that never resolves** (two contenders in sudden-death indefinitely) is accepted as an unbounded overtime; no match timer exists in v1.

### Dependencies (existing engine_demo subsystems — reuse, not re-specify)

- `engine_demo::allocator` — all containers/pools (Article 4).
- `engine_demo::ecs::world` — entity storage for wells, particles, pickups.
- `engine_demo::physics` — integration/constraint machinery for particle motion.
- `engine_demo::sim::game_loop` + `sim::rng` — fixed-tick simulation and seeded randomness (Articles 5, 10).
- `engine_demo::frame_budget` — real-time budget enforcement (Article 6).
- `engine_demo::vfx` (feature 001: `particle_pool`, `emitter`, `force_applicator`) — particle storage, spawning, and attraction-force application; gravity wells are expected to drive `force_applicator`-style attractors rather than introducing a new force system.
