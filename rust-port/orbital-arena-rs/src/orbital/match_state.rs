//! Match lifecycle state machine: lobby → countdown → playing → game_over → lobby
//! (orbital-arena-match.spec.md).

use super::types::{ArenaStatus, COUNTDOWN_TICKS, MAX_PLAYERS, MIN_PLAYERS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MatchState {
    Lobby = 0,
    Countdown = 1,
    Playing = 2,
    GameOver = 3,
}

#[derive(Debug, Clone, Copy, Default)]
struct PlayerSlot {
    joined: bool,
    ready: bool,
    departed: bool,
}

/// Reported by `tick()` so the caller can run playing-entry side effects exactly once
/// on the transition tick (orbital-arena-orchestration.spec.md §4.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transition {
    pub from: MatchState,
    pub to: MatchState,
}

#[derive(Debug)]
pub struct Match {
    state: MatchState,
    slots: [PlayerSlot; MAX_PLAYERS],
    countdown_ticks: u32,
}

impl Default for Match {
    fn default() -> Self {
        Self {
            state: MatchState::Lobby,
            slots: [PlayerSlot::default(); MAX_PLAYERS],
            countdown_ticks: 0,
        }
    }
}

impl Match {
    #[must_use]
    pub fn state(&self) -> MatchState {
        self.state
    }

    #[must_use]
    pub fn gameplay_input_enabled(&self) -> bool {
        self.state == MatchState::Playing
    }

    #[must_use]
    pub fn countdown_remaining_ticks(&self) -> u32 {
        self.countdown_ticks
    }

    /// Valid only in lobby (spec §3).
    pub fn join(&mut self, player: u8) -> ArenaStatus {
        if self.state != MatchState::Lobby {
            return ArenaStatus::WrongState;
        }
        let Some(slot) = self.slots.get_mut(player as usize) else {
            return ArenaStatus::InvalidArgument;
        };
        if slot.joined {
            return ArenaStatus::InvalidArgument;
        }
        *slot = PlayerSlot {
            joined: true,
            ready: false,
            departed: false,
        };
        ArenaStatus::Ok
    }

    /// Valid in lobby and countdown; un-readying during countdown interrupts it back
    /// to lobby (spec §3).
    pub fn set_ready(&mut self, player: u8, ready: bool) -> ArenaStatus {
        if self.state != MatchState::Lobby && self.state != MatchState::Countdown {
            return ArenaStatus::WrongState;
        }
        let Some(slot) = self.slots.get_mut(player as usize) else {
            return ArenaStatus::InvalidArgument;
        };
        if !slot.joined {
            return ArenaStatus::InvalidArgument;
        }
        slot.ready = ready;
        if self.state == MatchState::Countdown && !ready {
            self.state = MatchState::Lobby;
            self.countdown_ticks = 0;
        }
        ArenaStatus::Ok
    }

    /// Drives lobby→countdown→playing transitions (spec §2); called once per arena
    /// tick before gameplay systems.
    pub fn tick(&mut self) -> Option<Transition> {
        match self.state {
            MatchState::Lobby => {
                if self.joined_count() >= MIN_PLAYERS && self.all_joined_ready() {
                    self.state = MatchState::Countdown;
                    self.countdown_ticks = COUNTDOWN_TICKS;
                    return Some(Transition {
                        from: MatchState::Lobby,
                        to: MatchState::Countdown,
                    });
                }
                None
            }
            MatchState::Countdown => {
                self.countdown_ticks -= 1;
                if self.countdown_ticks == 0 {
                    self.state = MatchState::Playing;
                    return Some(Transition {
                        from: MatchState::Countdown,
                        to: MatchState::Playing,
                    });
                }
                None
            }
            MatchState::Playing | MatchState::GameOver => None,
        }
    }

    /// Departing during lobby/countdown removes the roster slot; departing during
    /// playing/game_over flags the slot departed without changing roster size (spec §3).
    pub fn leave(&mut self, player: u8) {
        let Some(slot) = self.slots.get_mut(player as usize) else {
            return;
        };
        if !slot.joined {
            return;
        }
        match self.state {
            MatchState::Lobby => {
                *slot = PlayerSlot::default();
            }
            MatchState::Countdown => {
                *slot = PlayerSlot::default();
                self.state = MatchState::Lobby;
                self.countdown_ticks = 0;
            }
            MatchState::Playing | MatchState::GameOver => {
                slot.departed = true;
            }
        }
    }

    pub fn enter_game_over(&mut self) {
        if self.state == MatchState::Playing {
            self.state = MatchState::GameOver;
        }
    }

    /// game_over → lobby; ready flags cleared, departed players removed (spec §3).
    pub fn acknowledge_results(&mut self) -> ArenaStatus {
        if self.state != MatchState::GameOver {
            return ArenaStatus::WrongState;
        }
        self.state = MatchState::Lobby;
        for slot in &mut self.slots {
            if slot.departed {
                *slot = PlayerSlot::default();
            }
            slot.ready = false;
        }
        ArenaStatus::Ok
    }

    #[must_use]
    pub fn joined_count(&self) -> u8 {
        u8::try_from(self.slots.iter().filter(|s| s.joined).count()).unwrap_or(u8::MAX)
    }

    #[must_use]
    pub fn active_count(&self) -> u8 {
        u8::try_from(
            self.slots
                .iter()
                .filter(|s| s.joined && !s.departed)
                .count(),
        )
        .unwrap_or(u8::MAX)
    }

    fn all_joined_ready(&self) -> bool {
        self.slots.iter().all(|s| !s.joined || s.ready)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ready_match(player_count: u8) -> Match {
        let mut m = Match::default();
        for p in 0..player_count {
            assert_eq!(m.join(p), ArenaStatus::Ok);
            assert_eq!(m.set_ready(p, true), ArenaStatus::Ok);
        }
        m
    }

    #[test]
    fn lobby_transitions_to_countdown_once_min_players_ready() {
        let mut m = ready_match(2);
        let transition = m.tick();
        assert_eq!(
            transition,
            Some(Transition {
                from: MatchState::Lobby,
                to: MatchState::Countdown
            })
        );
        assert_eq!(m.state(), MatchState::Countdown);
    }

    #[test]
    fn countdown_transitions_to_playing_after_exactly_180_ticks() {
        let mut m = ready_match(2);
        m.tick(); // lobby -> countdown
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
    fn un_readying_during_countdown_interrupts_back_to_lobby() {
        let mut m = ready_match(2);
        m.tick(); // lobby -> countdown
        assert_eq!(m.set_ready(0, false), ArenaStatus::Ok);
        assert_eq!(m.state(), MatchState::Lobby);
    }

    #[test]
    fn acknowledge_results_returns_to_lobby_and_clears_ready() {
        let mut m = ready_match(2);
        m.tick();
        for _ in 0..COUNTDOWN_TICKS {
            m.tick();
        }
        m.enter_game_over();
        assert_eq!(m.acknowledge_results(), ArenaStatus::Ok);
        assert_eq!(m.state(), MatchState::Lobby);
    }

    #[test]
    fn acknowledge_results_fails_outside_game_over() {
        let mut m = Match::default();
        assert_eq!(m.acknowledge_results(), ArenaStatus::WrongState);
    }

    #[test]
    fn join_fails_when_already_joined() {
        let mut m = Match::default();
        assert_eq!(m.join(0), ArenaStatus::Ok);
        assert_eq!(m.join(0), ArenaStatus::InvalidArgument);
    }
}
