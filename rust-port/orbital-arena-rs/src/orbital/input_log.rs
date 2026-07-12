//! Per-tick player input frames and the append-only replay log
//! (orbital-arena-input-log.spec.md). Player→well mapping is identity by roster slot:
//! player i's frame always drives well i.

use bevy::math::Vec2;

use super::types::{ArenaStatus, MAX_PLAYERS};

/// One player's steering input for one fixed tick (spec §2.1).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct InputFrame {
    pub steer: Vec2,
    pub strength: f32,
}

/// All players' inputs for one tick (spec §2.2, identity slot mapping).
#[derive(Debug, Clone, Copy, Default)]
pub struct TickInputs {
    pub players: [InputFrame; MAX_PLAYERS],
}

fn clamp_component(value: f32, lo: f32, hi: f32) -> f32 {
    if value.is_nan() {
        0.0 // deterministic NaN policy (spec §3.1) — never re-clamps differently on replay
    } else {
        value.clamp(lo, hi)
    }
}

/// Clamps a raw frame: steer axes to `[-1, 1]`, strength to `[0, 1]`. NaN maps to 0
/// (spec §3.1).
#[must_use]
pub fn clamp_input(raw: InputFrame) -> InputFrame {
    InputFrame {
        steer: Vec2::new(
            clamp_component(raw.steer.x, -1.0, 1.0),
            clamp_component(raw.steer.y, -1.0, 1.0),
        ),
        strength: clamp_component(raw.strength, 0.0, 1.0),
    }
}

/// Append-only per-match input log (spec §2.3/§3.2). Records inputs for every tick in
/// every match state; gameplay rules honor them only in `playing`.
#[derive(Debug)]
pub struct InputLog {
    max_ticks: usize,
    frames: Vec<TickInputs>,
}

impl InputLog {
    #[must_use]
    pub fn new(max_ticks: usize) -> Self {
        Self {
            max_ticks,
            frames: Vec::with_capacity(max_ticks),
        }
    }

    /// Appends one tick's inputs. Returns `LogFull` (frame dropped, no reallocation)
    /// once `max_ticks` entries are stored (Article VI).
    pub fn append(&mut self, frame: TickInputs) -> ArenaStatus {
        if self.frames.len() >= self.max_ticks {
            return ArenaStatus::LogFull;
        }
        self.frames.push(frame);
        ArenaStatus::Ok
    }

    #[must_use]
    pub fn size(&self) -> usize {
        self.frames.len()
    }

    /// Precondition: `tick < size()`; returns `None` otherwise rather than panicking
    /// (Article I).
    #[must_use]
    pub fn at(&self, tick: usize) -> Option<&TickInputs> {
        self.frames.get(tick)
    }

    /// Resets for a new match; retains the reserved capacity (no allocation).
    pub fn clear(&mut self) {
        self.frames.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_input_clamps_steer_and_strength() {
        let raw = InputFrame {
            steer: Vec2::new(5.0, -5.0),
            strength: 2.0,
        };
        let clamped = clamp_input(raw);
        assert_eq!(clamped.steer, Vec2::new(1.0, -1.0));
        assert_eq!(clamped.strength, 1.0);
    }

    #[test]
    fn clamp_input_maps_nan_to_zero() {
        let raw = InputFrame {
            steer: Vec2::new(f32::NAN, 0.5),
            strength: f32::NAN,
        };
        let clamped = clamp_input(raw);
        assert_eq!(clamped.steer.x, 0.0);
        assert_eq!(clamped.strength, 0.0);
    }

    #[test]
    fn append_reports_log_full_without_dropping_gameplay() {
        let mut log = InputLog::new(2);
        assert_eq!(log.append(TickInputs::default()), ArenaStatus::Ok);
        assert_eq!(log.append(TickInputs::default()), ArenaStatus::Ok);
        assert_eq!(log.append(TickInputs::default()), ArenaStatus::LogFull);
        assert_eq!(log.size(), 2);
    }

    #[test]
    fn clear_retains_capacity() {
        let mut log = InputLog::new(4);
        log.append(TickInputs::default());
        log.clear();
        assert_eq!(log.size(), 0);
        assert_eq!(log.append(TickInputs::default()), ArenaStatus::Ok);
    }

    #[test]
    fn at_returns_none_out_of_range() {
        let log = InputLog::new(4);
        assert!(log.at(0).is_none());
    }
}
