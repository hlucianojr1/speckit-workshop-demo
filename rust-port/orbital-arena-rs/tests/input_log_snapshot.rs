//! T064: input clamping/logging and the snapshot state hash
//! (orbital-arena-{input-log,snapshot}.spec.md), driven through the public `orbital`
//! API.

use bevy::math::Vec2;

use orbital_arena_rs::orbital::input_log::{clamp_input, InputFrame, InputLog, TickInputs};
use orbital_arena_rs::orbital::snapshot::{state_hash, MatchSnapshot};
use orbital_arena_rs::orbital::types::{ArenaStatus, MAX_RECORDED_TICKS};

#[test]
fn clamp_input_clamps_post_clamp_values_into_range() {
    let raw = InputFrame {
        steer: Vec2::new(3.0, -3.0),
        strength: 5.0,
    };
    let clamped = clamp_input(raw);
    assert_eq!(clamped.steer, Vec2::new(1.0, -1.0));
    assert_eq!(clamped.strength, 1.0);
}

#[test]
fn clamp_input_maps_nan_to_zero_deterministically() {
    let raw = InputFrame {
        steer: Vec2::new(f32::NAN, f32::NAN),
        strength: f32::NAN,
    };
    let clamped = clamp_input(raw);
    assert_eq!(clamped, InputFrame::default());
}

#[test]
fn log_full_after_capacity_ticks_but_never_reallocates_beyond_it() {
    let mut log = InputLog::new(4096);
    assert_eq!(MAX_RECORDED_TICKS, 4096);
    for _ in 0..4096 {
        assert_eq!(log.append(TickInputs::default()), ArenaStatus::Ok);
    }
    assert_eq!(log.append(TickInputs::default()), ArenaStatus::LogFull);
    assert_eq!(log.size(), 4096);
    // Still full on a second attempt — gameplay would keep advancing upstream;
    // only recording stops.
    assert_eq!(log.append(TickInputs::default()), ArenaStatus::LogFull);
    assert_eq!(log.size(), 4096);
}

#[test]
fn field_wise_hash_over_declared_order_is_deterministic() {
    let a = MatchSnapshot::default();
    let b = MatchSnapshot::default();
    assert_eq!(state_hash(&a), state_hash(&b));

    let c = MatchSnapshot {
        tick: 42,
        ..MatchSnapshot::default()
    };
    assert_ne!(state_hash(&a), state_hash(&c));
}

#[test]
fn hash_is_sampled_every_60_ticks_by_the_arena() {
    use orbital_arena_rs::orbital::{Arena, ArenaConfig};

    let config = ArenaConfig {
        seed: 42,
        player_count: 2,
        half_extent: 5.0,
        particle_capacity: 16,
    };
    let Some(mut arena) = Arena::new(config) else {
        unreachable!("valid config must construct an arena");
    };
    arena.join(0);
    arena.join(1);
    arena.set_ready(0, true);
    arena.set_ready(1, true);

    let inputs = TickInputs::default();
    for _ in 0..120 {
        arena.tick(&inputs);
    }
    // Ticks 0, 60, 120 fall on a 60-tick boundary within the first 121 ticks (0..=120
    // inclusive of both endpoints since tick 120 samples on its own iteration's start
    // condition) — at least 2 samples must have been recorded by now.
    assert!(arena.hash_history().len() >= 2);
}
