# Orbital Arena — Match Lifecycle: Language-Agnostic Specification

## 1. Purpose

A state machine governing when gameplay input is honored: players join and ready up in a
lobby, a countdown transitions to active play, and the match ends in a game-over state
that returns to lobby on acknowledgement.

## 2. States

`lobby → countdown → playing → game_over → lobby (loop)`

| State      | Gameplay input honored? | Entry condition                                    |
| ---------- | ------------------------ | --------------------------------------------------- |
| lobby      | No                        | Initial state, or after game-over acknowledgement   |
| countdown  | No                        | All joined players (≥ minimum player count) ready   |
| playing    | Yes                       | Countdown timer reaches zero                        |
| game_over  | No                        | A winner is latched, OR active players drop below the minimum |

Reference: minimum 2 players, maximum 4; countdown lasts a fixed number of ticks (not
wall-clock time — an integer tick count, per the engine's determinism rule).

## 3. Operations (abstract)

| Operation           | Valid states           | Effect                                                        |
| -------------------- | ----------------------- | -------------------------------------------------------------- |
| join                | lobby only              | Adds a player to the roster; error if already full/joined      |
| set_ready           | lobby, countdown        | Un-readying DURING countdown interrupts it back to lobby       |
| tick (state check)  | any (called every tick) | Drives lobby→countdown→playing transitions; described in §2    |
| leave               | any                     | Departing during lobby/countdown removes the roster slot; departing during playing/game_over flags the slot departed without changing roster size |
| acknowledge_results | game_over only          | Clears ready flags and departed players; returns to lobby       |

## 4. Guarantees

| Guarantee                | Description                                                            |
| --------------------------| ------------------------------------------------------------------------ |
| Input gating              | Gameplay-affecting input is honored ONLY in the `playing` state          |
| No orphan transitions     | Every state change is triggered by an explicit, well-defined condition  |
| Deterministic countdown   | The countdown is an integer tick counter, not wall-clock time            |
| Graceful under-population | Dropping below the minimum active-player count ends the match, it does not crash it |

## 5. Constraints (constitutional) and Scope Reduction

- No exceptions/panics; illegal calls for the current state return an error status, they
  never mutate state.
- Pure state machine: no RNG, no floating-point, no containers beyond a small fixed roster.
- **Documented scope reduction for the Rust port:** the minimal Rust port used for this
  training's screenshots does **not** implement the lobby/countdown/game-over state
  machine, the player input log/replay (`include/orbital_arena/input.h`), or the
  snapshot/state-hash serialization (`include/orbital_arena/snapshot.h`). The port runs
  directly in an always-`playing` state so gravity wells and capture/scoring (the visually
  defining mechanics) can be demonstrated without also reproducing the full match
  lifecycle. This is a scope decision for the exercise, not a claim that these subsystems
  are hard to port — they are, in fact, the *easiest* part (plain state machines and POD
  structs) and would be a natural "next task" in a real transformation project.
