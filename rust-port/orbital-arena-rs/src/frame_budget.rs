//! `FrameBudgetPlugin`: fixed-size rolling window of frame durations
//! (frame-budget.spec.md §2-4, data-model.md §Resources `FrameBudget`).

use bevy::prelude::*;

/// Rolling-window size (frame-budget.spec.md §2 default).
pub const WINDOW_SIZE: usize = 64;

/// Fixed-size rolling window of recent frame durations, in milliseconds. Never
/// heap-allocated — `samples` is an inline array, reserved once at `Default::default()`.
#[derive(Resource, Debug, Clone, Copy)]
pub struct FrameBudget {
    pub samples: [f64; WINDOW_SIZE],
    pub cursor: usize,
    pub count: usize,
}

impl Default for FrameBudget {
    fn default() -> Self {
        Self {
            samples: [0.0; WINDOW_SIZE],
            cursor: 0,
            count: 0,
        }
    }
}

impl FrameBudget {
    /// O(1), allocation-free (frame-budget.spec.md §5 "No record allocation").
    pub fn record_sample(&mut self, milliseconds: f64) {
        if let Some(slot) = self.samples.get_mut(self.cursor) {
            *slot = milliseconds;
        }
        self.cursor = (self.cursor + 1) % WINDOW_SIZE;
        if self.count < WINDOW_SIZE {
            self.count += 1;
        }
    }

    /// `min(total recorded, window size)` (frame-budget.spec.md §3 `sample_count`).
    pub fn sample_count(&self) -> usize {
        self.count
    }

    /// Mean of the samples recorded so far; before the window is full this is the mean of
    /// exactly `count` samples (each counted exactly once — frame-budget.spec.md §4's
    /// "warm-up counted once" rule); once full, the mean is over all `WINDOW_SIZE` slots.
    /// Zero samples ⇒ `0.0` (§4).
    pub fn rolling_average(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        let sum: f64 = self.samples.iter().take(self.count).sum();
        sum / self.count as f64
    }
}

pub struct FrameBudgetPlugin;

impl Plugin for FrameBudgetPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FrameBudget::default())
            .add_systems(Update, record_frame_time);
    }
}

fn record_frame_time(time: Res<Time>, mut budget: ResMut<FrameBudget>) {
    budget.record_sample(time.delta_secs_f64() * 1000.0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolling_average_before_window_full_counts_each_sample_once() {
        let mut budget = FrameBudget::default();
        budget.record_sample(10.0);
        budget.record_sample(20.0);
        budget.record_sample(30.0);
        assert_eq!(budget.sample_count(), 3);
        assert!((budget.rolling_average() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn rolling_average_after_window_full_uses_last_window_size_samples() {
        let mut budget = FrameBudget::default();
        for i in 0..(WINDOW_SIZE + 10) {
            budget.record_sample(i as f64);
        }
        assert_eq!(budget.sample_count(), WINDOW_SIZE);
        // The oldest 10 samples (0..10) have been overwritten; the remaining window
        // holds 10..(WINDOW_SIZE + 10).
        let expected_sum: f64 = (10..(WINDOW_SIZE + 10)).map(|v| v as f64).sum();
        let expected_average = expected_sum / WINDOW_SIZE as f64;
        assert!((budget.rolling_average() - expected_average).abs() < 1e-9);
    }

    #[test]
    fn zero_samples_average_is_zero() {
        let budget = FrameBudget::default();
        assert_eq!(budget.rolling_average(), 0.0);
    }
}
