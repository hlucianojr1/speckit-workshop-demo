//! Deterministic sinusoidal autopilot (sandbox-scenes.spec.md §6): steering inputs are
//! a pure function of the arena's own tick index — no rng, no wall clock — so the
//! entire match trajectory is reproducible from (seed, tick count) alone.

use bevy::math::Vec2;

use super::input_log::{InputFrame, TickInputs};
use super::types::{MAX_PLAYERS, TICK_SECONDS};

/// Builds one tick's autopilot inputs for `player_count` players (reference
/// `make_orbital_inputs`). Frequencies are *detuned* per player (not merely
/// phase-shifted) — pure phase shifts make player 1's input the exact negation of
/// player 0's, which locks the arena's bit-exact mirror-fairness (rotational-symmetry
/// Article) into a permanent sudden-death tie (spec §6 Fairness note).
#[must_use]
pub fn make_orbital_inputs(tick: u64, player_count: u8) -> TickInputs {
    let mut inputs = TickInputs::default();
    if player_count == 0 {
        return inputs;
    }
    #[allow(clippy::cast_precision_loss)] // tick counts fit comfortably in f64's mantissa
    let t = tick as f64 * TICK_SECONDS;
    for p in 0..player_count.min(MAX_PLAYERS as u8) {
        let phase = std::f64::consts::TAU * f64::from(p) / f64::from(player_count);
        let fx = 0.90 + 0.07 * f64::from(p);
        let fy = 0.53 + 0.05 * f64::from(p);
        let steer = Vec2::new(
            (t * fx + phase).sin() as f32,
            (t * fy + phase + 1.3).sin() as f32,
        );
        if let Some(frame) = inputs.players.get_mut(p as usize) {
            *frame = InputFrame {
                steer,
                strength: 1.0,
            };
        }
    }
    inputs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_a_pure_function_of_tick_and_player_count() {
        let a = make_orbital_inputs(42, 2);
        let b = make_orbital_inputs(42, 2);
        assert_eq!(a.players[0].steer, b.players[0].steer);
        assert_eq!(a.players[1].steer, b.players[1].steer);
    }

    #[test]
    fn players_are_detuned_not_merely_phase_shifted() {
        let inputs = make_orbital_inputs(100, 2);
        // Pure phase shift would make player 1's steer the exact negation of player 0's;
        // detuned frequencies must not satisfy that for a generic tick.
        assert_ne!(inputs.players[1].steer, -inputs.players[0].steer);
    }

    #[test]
    fn full_strength_every_tick() {
        let inputs = make_orbital_inputs(7, 2);
        assert_eq!(inputs.players[0].strength, 1.0);
        assert_eq!(inputs.players[1].strength, 1.0);
    }

    #[test]
    fn zero_players_produces_no_input() {
        let inputs = make_orbital_inputs(7, 0);
        assert_eq!(inputs.players[0], InputFrame::default());
    }
}
