//! T062: match state machine transitions (orbital-arena-match.spec.md), driven through
//! the public `orbital_arena_rs::orbital` API.

use orbital_arena_rs::orbital::match_state::{Match, MatchState, Transition};
use orbital_arena_rs::orbital::types::{ArenaStatus, COUNTDOWN_TICKS};

fn ready_match(player_count: u8) -> Match {
    let mut m = Match::default();
    for p in 0..player_count {
        assert_eq!(m.join(p), ArenaStatus::Ok);
        assert_eq!(m.set_ready(p, true), ArenaStatus::Ok);
    }
    m
}

#[test]
fn lobby_to_countdown_to_playing_over_180_ticks() {
    let mut m = ready_match(2);
    assert_eq!(m.state(), MatchState::Lobby);

    let transition = m.tick();
    assert_eq!(
        transition,
        Some(Transition {
            from: MatchState::Lobby,
            to: MatchState::Countdown
        })
    );
    assert_eq!(m.state(), MatchState::Countdown);
    assert_eq!(m.countdown_remaining_ticks(), COUNTDOWN_TICKS);

    let mut last_transition = None;
    for _ in 0..COUNTDOWN_TICKS {
        last_transition = m.tick();
    }
    assert_eq!(m.state(), MatchState::Playing);
    assert_eq!(
        last_transition,
        Some(Transition {
            from: MatchState::Countdown,
            to: MatchState::Playing
        })
    );
}

#[test]
fn un_readying_during_countdown_interrupts_it_back_to_lobby() {
    let mut m = ready_match(2);
    m.tick(); // lobby -> countdown
    assert_eq!(m.state(), MatchState::Countdown);

    assert_eq!(m.set_ready(0, false), ArenaStatus::Ok);
    assert_eq!(m.state(), MatchState::Lobby);
    assert_eq!(m.countdown_remaining_ticks(), 0);
}

#[test]
fn acknowledge_returns_to_lobby_only_from_game_over() {
    let mut m = ready_match(2);
    assert_eq!(m.acknowledge_results(), ArenaStatus::WrongState);

    m.tick();
    for _ in 0..COUNTDOWN_TICKS {
        m.tick();
    }
    assert_eq!(m.state(), MatchState::Playing);
    m.enter_game_over();
    assert_eq!(m.state(), MatchState::GameOver);

    assert_eq!(m.acknowledge_results(), ArenaStatus::Ok);
    assert_eq!(m.state(), MatchState::Lobby);
}

#[test]
fn gameplay_input_is_enabled_only_while_playing() {
    let mut m = ready_match(2);
    assert!(!m.gameplay_input_enabled());
    m.tick();
    assert!(!m.gameplay_input_enabled());
    for _ in 0..COUNTDOWN_TICKS {
        m.tick();
    }
    assert!(m.gameplay_input_enabled());
}

#[test]
fn leave_during_playing_flags_departed_without_shrinking_roster() {
    let mut m = ready_match(2);
    m.tick();
    for _ in 0..COUNTDOWN_TICKS {
        m.tick();
    }
    assert_eq!(m.active_count(), 2);
    m.leave(0);
    assert_eq!(m.joined_count(), 2); // roster size unchanged
    assert_eq!(m.active_count(), 1); // but no longer active
}

#[test]
fn lobby_requires_at_least_the_minimum_player_count() {
    let mut m = Match::default();
    assert_eq!(m.join(0), ArenaStatus::Ok);
    assert_eq!(m.set_ready(0, true), ArenaStatus::Ok);
    // Only 1 of the minimum 2 players ready: no transition yet.
    assert_eq!(m.tick(), None);
    assert_eq!(m.state(), MatchState::Lobby);
}
