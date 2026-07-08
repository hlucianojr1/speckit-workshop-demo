// Orbital Arena match state machine — see include/orbital_arena/match.h for the contract.

#include "orbital_arena/match.h"

namespace orbital_arena {

arena_status match::join(std::uint8_t player) noexcept {
    if (m_state != match_state::lobby) {
        return arena_status::wrong_state;
    }
    if (player >= kMaxPlayers || m_slots[player].joined) {
        return arena_status::invalid_argument;
    }
    m_slots[player] = player_slot{true, false, false};
    return arena_status::ok;
}

arena_status match::set_ready(std::uint8_t player, bool ready) noexcept {
    if (m_state != match_state::lobby && m_state != match_state::countdown) {
        return arena_status::wrong_state;
    }
    if (player >= kMaxPlayers || !m_slots[player].joined) {
        return arena_status::invalid_argument;
    }
    m_slots[player].ready = ready;
    if (m_state == match_state::countdown && !ready) {
        // Countdown interruption: back to lobby immediately (FR-013, US2 edge).
        m_state = match_state::lobby;
        m_countdown_ticks = 0;
    }
    return arena_status::ok;
}

eastl::optional<match::transition> match::tick() noexcept {
    switch (m_state) {
        case match_state::lobby:
            if (joined_count() >= kMinPlayers && all_joined_ready()) {
                m_state = match_state::countdown;
                m_countdown_ticks = kCountdownTicks;
                return transition{match_state::lobby, match_state::countdown};
            }
            return eastl::nullopt;
        case match_state::countdown:
            // Integer tick counter (research D4): expires after exactly 180 ticks.
            if (--m_countdown_ticks == 0) {
                m_state = match_state::playing;
                return transition{match_state::countdown, match_state::playing};
            }
            return eastl::nullopt;
        case match_state::playing:
        case match_state::game_over:
            return eastl::nullopt;  // win/acknowledge are explicit calls, not tick-driven
    }
    return eastl::nullopt;
}

void match::leave(std::uint8_t player) noexcept {
    if (player >= kMaxPlayers || !m_slots[player].joined) {
        return;
    }
    switch (m_state) {
        case match_state::lobby:
            m_slots[player] = player_slot{};
            break;
        case match_state::countdown:
            // Departure interrupts the countdown: back to lobby (FR-013).
            m_slots[player] = player_slot{};
            m_state = match_state::lobby;
            m_countdown_ticks = 0;
            break;
        case match_state::playing:
        case match_state::game_over:
            // Roster fixed; the slot is flagged and the arena deactivates the well.
            m_slots[player].departed = true;
            break;
    }
}

void match::enter_game_over() noexcept {
    if (m_state == match_state::playing) {
        m_state = match_state::game_over;
    }
}

arena_status match::acknowledge_results() noexcept {
    if (m_state != match_state::game_over) {
        return arena_status::wrong_state;
    }
    m_state = match_state::lobby;
    for (player_slot& slot : m_slots) {
        if (slot.departed) {
            slot = player_slot{};  // departed players left the roster (US2 scenario 5)
        }
        slot.ready = false;
    }
    return arena_status::ok;
}

void match::restore(match_state state, std::uint8_t joined_players,
                    const bool departed[kMaxPlayers]) noexcept {
    const std::uint8_t joined = joined_players < kMaxPlayers ? joined_players : kMaxPlayers;
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        if (i < joined) {
            // Ready flags are not snapshot state: normalize (see header contract).
            m_slots[i] = player_slot{true, state != match_state::lobby, departed[i]};
        } else {
            m_slots[i] = player_slot{};
        }
    }
    m_state = state;
    m_countdown_ticks = state == match_state::countdown ? kCountdownTicks : 0;
}

std::uint8_t match::joined_count() const noexcept {
    std::uint8_t count = 0;
    for (const player_slot& slot : m_slots) {
        if (slot.joined) {
            ++count;
        }
    }
    return count;
}

std::uint8_t match::active_count() const noexcept {
    std::uint8_t count = 0;
    for (const player_slot& slot : m_slots) {
        if (slot.joined && !slot.departed) {
            ++count;
        }
    }
    return count;
}

bool match::all_joined_ready() const noexcept {
    for (const player_slot& slot : m_slots) {
        if (slot.joined && !slot.ready) {
            return false;
        }
    }
    return true;
}

}  // namespace orbital_arena
