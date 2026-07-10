//! Capture contention and scoring (orbital-arena-scoring.spec.md).

use bevy::math::DVec2;
use bevy::prelude::*;

use super::well::{is_within_capture, GravityWell};

/// Win threshold (spec §2 reference value).
pub const WIN_SCORE: u32 = 100;
/// Maximum player slots (spec §2 reference: 2..=4; this port uses `game::WELL_COUNT`).
pub const MAX_PLAYERS: usize = 4;

/// Outcome of one contention resolution (spec §3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureResult {
    pub winner_index: Option<u8>,
    pub tie: bool,
}

/// Per-player scores plus match-outcome latches (spec §2).
#[derive(Resource, Debug, Clone)]
pub struct ScoreTable {
    pub scores: [u32; MAX_PLAYERS],
    pub winner: Option<u8>,
    pub sudden_death: bool,
    /// Bitmask: bit i set iff player i captured >= 1 particle this tick (spec §3.3/§3.4).
    pub scored_this_tick: u32,
}

impl Default for ScoreTable {
    fn default() -> Self {
        Self {
            scores: [0; MAX_PLAYERS],
            winner: None,
            sudden_death: false,
            scored_this_tick: 0,
        }
    }
}

/// FR-007 contention (spec §3.1): among wells whose capture radius contains `pos`,
/// returns the STRICTLY nearest by squared distance; an exact tie voids the capture
/// (`winner_index: None, tie: true`) rather than breaking it by index.
#[must_use]
pub fn resolve_capture(wells: &[GravityWell], pos: DVec2) -> CaptureResult {
    let mut best: Option<(u8, f64)> = None;
    let mut tie = false;

    for well in wells {
        if !is_within_capture(well, pos) {
            continue;
        }
        let distance_sq = (pos - well.position).length_squared();
        match best {
            None => {
                best = Some((well.player_index, distance_sq));
            }
            Some((_, best_sq)) if distance_sq < best_sq => {
                best = Some((well.player_index, distance_sq));
                tie = false;
            }
            Some((_, best_sq)) if distance_sq == best_sq => {
                tie = true;
            }
            _ => {}
        }
    }

    if tie {
        CaptureResult {
            winner_index: None,
            tie: true,
        }
    } else {
        CaptureResult {
            winner_index: best.map(|(player, _)| player),
            tie: false,
        }
    }
}

/// Awards `base_value` (doubled if `double_points`) to `player`'s score. No-op once a
/// winner has been latched (spec §3.2 score freeze).
pub fn award_capture(table: &mut ScoreTable, player: u8, base_value: u32, double_points: bool) {
    if table.winner.is_some() {
        return;
    }
    let Some(slot) = table.scores.get_mut(player as usize) else {
        return;
    };
    *slot += base_value * if double_points { 2 } else { 1 };
}

/// Evaluated once per tick after captures resolve (spec §3.4). `scored_this_tick` is the
/// bitmask produced by resolving every particle's capture this tick.
pub fn evaluate_win(table: &mut ScoreTable, player_count: u8, scored_this_tick: u32) -> Option<u8> {
    if let Some(winner) = table.winner {
        return Some(winner);
    }

    let contenders: Vec<u8> = (0..player_count)
        .filter(|&i| {
            table
                .scores
                .get(i as usize)
                .is_some_and(|&s| s >= WIN_SCORE)
        })
        .collect();

    if table.sudden_death {
        let scoring_contenders: Vec<u8> = contenders
            .iter()
            .copied()
            .filter(|&i| scored_this_tick & (1 << i) != 0)
            .collect();
        if scoring_contenders.len() == 1 {
            table.winner = scoring_contenders.first().copied();
        }
        return table.winner;
    }

    match contenders.len() {
        1 => {
            table.winner = contenders.first().copied();
        }
        n if n >= 2 => {
            table.sudden_death = true;
        }
        _ => {}
    }
    table.winner
}

/// Clears scores, winner, and sudden-death (spec §3.5) — used when a match restarts.
pub fn reset_scores(table: &mut ScoreTable) {
    *table = ScoreTable::default();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn well_at(player_index: u8, position: DVec2) -> GravityWell {
        GravityWell {
            player_index,
            position,
            velocity: DVec2::ZERO,
            strength: 1.0,
            influence_radius: 100.0,
            capture_radius: 10.0,
            active: true,
        }
    }

    #[test]
    fn no_capture_when_outside_every_well() {
        let wells = [well_at(0, DVec2::new(50.0, 0.0))];
        let result = resolve_capture(&wells, DVec2::ZERO);
        assert_eq!(
            result,
            CaptureResult {
                winner_index: None,
                tie: false
            }
        );
    }

    #[test]
    fn nearest_well_wins() {
        let wells = [
            well_at(0, DVec2::new(5.0, 0.0)),
            well_at(1, DVec2::new(-8.0, 0.0)),
        ];
        let result = resolve_capture(&wells, DVec2::ZERO);
        assert_eq!(result.winner_index, Some(0));
        assert!(!result.tie);
    }

    #[test]
    fn exact_tie_voids_capture_rather_than_picking_lowest_index() {
        let wells = [
            well_at(0, DVec2::new(5.0, 0.0)),
            well_at(1, DVec2::new(-5.0, 0.0)),
        ];
        let result = resolve_capture(&wells, DVec2::ZERO);
        assert_eq!(
            result,
            CaptureResult {
                winner_index: None,
                tie: true
            }
        );
    }

    #[test]
    fn award_capture_accumulates_and_doubles() {
        let mut table = ScoreTable::default();
        award_capture(&mut table, 0, 1, false);
        award_capture(&mut table, 0, 1, true);
        assert_eq!(table.scores[0], 3);
    }

    #[test]
    fn award_capture_is_frozen_after_winner_latched() {
        let mut table = ScoreTable {
            winner: Some(0),
            ..ScoreTable::default()
        };
        award_capture(&mut table, 0, 1, false);
        assert_eq!(table.scores[0], 0);
    }

    #[test]
    fn evaluate_win_latches_sole_contender() {
        let mut table = ScoreTable::default();
        table.scores[0] = WIN_SCORE;
        let winner = evaluate_win(&mut table, 2, 0);
        assert_eq!(winner, Some(0));
        assert_eq!(table.winner, Some(0));
    }

    #[test]
    fn evaluate_win_enters_sudden_death_on_simultaneous_threshold() {
        let mut table = ScoreTable::default();
        table.scores[0] = WIN_SCORE;
        table.scores[1] = WIN_SCORE;
        let winner = evaluate_win(&mut table, 2, 0);
        assert_eq!(winner, None);
        assert!(table.sudden_death);
    }

    #[test]
    fn sudden_death_breaks_on_next_sole_scorer() {
        let mut table = ScoreTable::default();
        table.scores[0] = WIN_SCORE;
        table.scores[1] = WIN_SCORE;
        evaluate_win(&mut table, 2, 0); // enters sudden death
        let winner = evaluate_win(&mut table, 2, 0b01); // player 0 scores again
        assert_eq!(winner, Some(0));
    }

    #[test]
    fn winner_is_frozen_across_repeated_evaluation() {
        let mut table = ScoreTable::default();
        table.scores[0] = WIN_SCORE;
        evaluate_win(&mut table, 2, 0);
        table.scores[1] = WIN_SCORE + 50; // should not matter anymore
        let winner = evaluate_win(&mut table, 2, 0);
        assert_eq!(winner, Some(0));
    }

    #[test]
    fn reset_scores_clears_everything() {
        let mut table = ScoreTable::default();
        table.scores[0] = 10;
        table.winner = Some(0);
        table.sudden_death = true;
        reset_scores(&mut table);
        assert_eq!(table.scores, [0; MAX_PLAYERS]);
        assert_eq!(table.winner, None);
        assert!(!table.sudden_death);
    }
}
