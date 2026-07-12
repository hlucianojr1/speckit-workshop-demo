//! Deterministic capture contention and per-player scoring
//! (orbital-arena-scoring.spec.md). Well index == player index (identity roster
//! mapping).

use bevy::math::Vec2;

use super::particle::FieldParticle;
use super::types::{MAX_PLAYERS, WIN_SCORE};
use super::well::{is_within_capture, GravityWell};

/// Outcome of one contention resolution (spec §3.1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CaptureResult {
    pub winner_index: Option<u8>,
    pub tie: bool,
}

/// Per-player scores plus match-outcome latches (spec §2).
#[derive(Debug, Clone, Copy)]
pub struct ScoreTable {
    pub scores: [u32; MAX_PLAYERS],
    pub winner: Option<u8>,
    pub sudden_death: bool,
}

impl Default for ScoreTable {
    fn default() -> Self {
        Self {
            scores: [0; MAX_PLAYERS],
            winner: None,
            sudden_death: false,
        }
    }
}

/// FR-007 contention (spec §3.1): among wells whose capture radius contains `pos`,
/// returns the STRICTLY nearest by squared distance; an exact tie voids the capture
/// rather than breaking it by index. Iterates wells in fixed index order but compares
/// distance VALUES only.
#[must_use]
pub fn resolve_capture(wells: &[GravityWell], pos: Vec2) -> CaptureResult {
    let mut best_dist_sq = 0.0f32;
    let mut winner: Option<u8> = None;
    let mut tie = false;

    for (i, well) in wells.iter().enumerate() {
        if !is_within_capture(well, pos) {
            continue;
        }
        let dist_sq = (well.position - pos).length_squared();
        if winner.is_none() || dist_sq < best_dist_sq {
            best_dist_sq = dist_sq;
            winner = u8::try_from(i).ok();
            tie = false;
        } else if dist_sq == best_dist_sq {
            tie = true;
        }
    }

    if tie {
        CaptureResult {
            winner_index: None,
            tie: true,
        }
    } else {
        CaptureResult {
            winner_index: winner,
            tie: false,
        }
    }
}

/// Awards `base_value` (doubled if `double_points`) to `player`'s score. No-op once a
/// winner has been latched (score freeze, spec §3.2).
pub fn award_capture(table: &mut ScoreTable, player: u8, base_value: u32, double_points: bool) {
    if table.winner.is_some() {
        return;
    }
    let Some(slot) = table.scores.get_mut(player as usize) else {
        return;
    };
    *slot += if double_points {
        base_value * 2
    } else {
        base_value
    };
}

/// Applies §3.1 contention to every live particle in one fixed-order pass (spec §3.3):
/// the winning well's player is awarded 1 point (doubled per `double_points`) and the
/// particle's lifetime is zeroed so the caller's `age_and_retire` removes it this tick.
/// Returns a bitmask: bit i set iff player i captured >= 1 particle this tick.
pub fn resolve_captures(
    wells: &[GravityWell],
    particles: &mut [FieldParticle],
    table: &mut ScoreTable,
    double_points: &[bool; MAX_PLAYERS],
) -> u32 {
    let mut scored_mask = 0u32;
    for particle in particles.iter_mut() {
        let result = resolve_capture(wells, particle.position);
        let Some(player) = result.winner_index else {
            continue;
        };
        let doubled = double_points.get(player as usize).copied().unwrap_or(false);
        award_capture(table, player, 1, doubled);
        particle.remaining_lifetime_seconds = 0.0;
        scored_mask |= 1u32 << player;
    }
    scored_mask
}

/// Evaluated once per tick after all captures resolve (spec §3.4). Once a winner is
/// latched, re-evaluation always returns it unchanged (score freeze).
#[must_use]
pub fn evaluate_win(table: &mut ScoreTable, player_count: u8, scored_this_tick: u32) -> Option<u8> {
    if let Some(winner) = table.winner {
        return Some(winner);
    }
    let count = player_count.min(MAX_PLAYERS as u8);

    if table.sudden_death {
        let mut sole_scorer: Option<u8> = None;
        let mut scoring_contenders = 0u8;
        for i in 0..count {
            let contender = table
                .scores
                .get(i as usize)
                .is_some_and(|&s| s >= WIN_SCORE);
            let scored = scored_this_tick & (1 << i) != 0;
            if contender && scored {
                scoring_contenders += 1;
                sole_scorer = Some(i);
            }
        }
        if scoring_contenders == 1 {
            table.winner = sole_scorer;
        }
        return table.winner;
    }

    let mut crosser: Option<u8> = None;
    let mut crossers = 0u8;
    for i in 0..count {
        if table
            .scores
            .get(i as usize)
            .is_some_and(|&s| s >= WIN_SCORE)
        {
            crossers += 1;
            crosser = Some(i);
        }
    }
    if crossers == 1 {
        table.winner = crosser;
    } else if crossers >= 2 {
        table.sudden_death = true;
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

    fn well_at(position: Vec2) -> GravityWell {
        GravityWell {
            position,
            velocity: Vec2::ZERO,
            strength: 1.0,
            influence_radius: 100.0,
            capture_radius: 10.0,
            active: true,
        }
    }

    #[test]
    fn no_capture_when_outside_every_well() {
        let wells = [well_at(Vec2::new(50.0, 0.0))];
        let result = resolve_capture(&wells, Vec2::ZERO);
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
        let wells = [well_at(Vec2::new(5.0, 0.0)), well_at(Vec2::new(-8.0, 0.0))];
        let result = resolve_capture(&wells, Vec2::ZERO);
        assert_eq!(result.winner_index, Some(0));
        assert!(!result.tie);
    }

    #[test]
    fn exact_tie_voids_capture_rather_than_picking_lowest_index() {
        let wells = [well_at(Vec2::new(5.0, 0.0)), well_at(Vec2::new(-5.0, 0.0))];
        let result = resolve_capture(&wells, Vec2::ZERO);
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
        assert_eq!(table.scores.first().copied(), Some(3));
    }

    #[test]
    fn award_capture_is_frozen_after_winner_latched() {
        let mut table = ScoreTable {
            winner: Some(0),
            ..ScoreTable::default()
        };
        award_capture(&mut table, 0, 1, false);
        assert_eq!(table.scores.first().copied(), Some(0));
    }

    #[test]
    fn evaluate_win_latches_sole_contender() {
        let mut table = ScoreTable {
            scores: [WIN_SCORE, 0, 0, 0],
            ..ScoreTable::default()
        };
        let winner = evaluate_win(&mut table, 2, 0);
        assert_eq!(winner, Some(0));
        assert_eq!(table.winner, Some(0));
    }

    #[test]
    fn evaluate_win_enters_sudden_death_on_simultaneous_threshold() {
        let mut table = ScoreTable {
            scores: [WIN_SCORE, WIN_SCORE, 0, 0],
            ..ScoreTable::default()
        };
        let winner = evaluate_win(&mut table, 2, 0);
        assert_eq!(winner, None);
        assert!(table.sudden_death);
    }

    #[test]
    fn sudden_death_breaks_on_next_sole_scorer() {
        let mut table = ScoreTable {
            scores: [WIN_SCORE, WIN_SCORE, 0, 0],
            ..ScoreTable::default()
        };
        let _ = evaluate_win(&mut table, 2, 0); // enters sudden death
        let winner = evaluate_win(&mut table, 2, 0b01);
        assert_eq!(winner, Some(0));
    }

    #[test]
    fn winner_is_frozen_across_repeated_evaluation() {
        let mut table = ScoreTable {
            scores: [WIN_SCORE, 0, 0, 0],
            ..ScoreTable::default()
        };
        let _ = evaluate_win(&mut table, 2, 0);
        table.scores[1] = WIN_SCORE + 50; // should not matter anymore
        let winner = evaluate_win(&mut table, 2, 0);
        assert_eq!(winner, Some(0));
    }

    #[test]
    fn reset_scores_clears_everything() {
        let mut table = ScoreTable {
            scores: [10, 0, 0, 0],
            winner: Some(0),
            sudden_death: true,
        };
        reset_scores(&mut table);
        assert_eq!(table.scores, [0; MAX_PLAYERS]);
        assert_eq!(table.winner, None);
        assert!(!table.sudden_death);
    }
}
