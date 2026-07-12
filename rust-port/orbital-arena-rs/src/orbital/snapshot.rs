//! Flat POD match snapshot + field-wise FNV-1a 64 state hash
//! (orbital-arena-snapshot.spec.md). Realizes spectator-safe state (a struct copy IS
//! the serialization) and the drift-detection half of lockstep replay: two arenas in
//! lockstep must produce identical hash sequences.
//!
//! NOTE: this is the arena's OWN internal `state_hash`/`hash_history` (sampled every
//! `STATE_HASH_INTERVAL_TICKS`, orbital-arena-orchestration.spec.md §4/§5) — a
//! DIFFERENT hash from `crate::digest::state_digest`'s cross-language scene digest
//! (sandbox-scenes.spec.md §8), which only needs tick/state/scores/wells/live-particle
//! positions. `state_hash` here need not be cross-language bit-exact; it only has to be
//! internally consistent (same Rust state ⇒ same hash, changed state ⇒ changed hash).

use bevy::math::Vec2;

use crate::digest::Fnv1a64;

use super::match_state::MatchState;
use super::powerup::PowerupKind;
use super::types::{MAX_FIELD_PICKUPS, MAX_PLAYERS, MAX_SNAPSHOT_PARTICLES};

/// Per-player well state (spec §2, field #8).
#[derive(Debug, Clone, Copy, Default)]
pub struct WellRecord {
    pub position: Vec2,
    pub velocity: Vec2,
    pub strength: f32,
    pub active: bool,
}

/// Per-player timed-effect slot; `remaining_ticks == 0` means inactive (spec §2, #9).
#[derive(Debug, Clone, Copy)]
pub struct EffectRecord {
    pub kind: PowerupKind,
    pub remaining_ticks: u32,
}

impl Default for EffectRecord {
    fn default() -> Self {
        Self {
            kind: PowerupKind::StrengthSurge,
            remaining_ticks: 0,
        }
    }
}

/// Field pickup slot mirror (spec §2, #10).
#[derive(Debug, Clone, Copy)]
pub struct PickupRecord {
    pub kind: PowerupKind,
    pub position: Vec2,
    pub spawn_tick: u64,
    pub alive: bool,
}

impl Default for PickupRecord {
    fn default() -> Self {
        Self {
            kind: PowerupKind::StrengthSurge,
            position: Vec2::ZERO,
            spawn_tick: 0,
            alive: false,
        }
    }
}

/// Live particle state relevant to game rules — 2D projection of the field particle
/// (spec §2, #13).
#[derive(Debug, Clone, Copy, Default)]
pub struct ParticleRecord {
    pub position: Vec2,
    pub velocity: Vec2,
}

/// Flat POD snapshot (spec §2): no pointers, no handles, no containers — field order
/// below IS the hash order.
#[derive(Debug, Clone)]
pub struct MatchSnapshot {
    pub state: MatchState,
    pub tick: u64,
    pub seed: u64,
    pub player_count: u8,
    pub scores: [u32; MAX_PLAYERS],
    pub winner: i8,
    pub sudden_death: bool,
    pub wells: [WellRecord; MAX_PLAYERS],
    pub effects: [EffectRecord; MAX_PLAYERS],
    pub pickups: [PickupRecord; MAX_FIELD_PICKUPS],
    pub powerup_timer_ticks: u32,
    pub live_particle_count: u32,
    pub particles: [ParticleRecord; MAX_SNAPSHOT_PARTICLES],
}

impl Default for MatchSnapshot {
    fn default() -> Self {
        Self {
            state: MatchState::Lobby,
            tick: 0,
            seed: 0,
            player_count: 0,
            scores: [0; MAX_PLAYERS],
            winner: -1,
            sudden_death: false,
            wells: [WellRecord::default(); MAX_PLAYERS],
            effects: [EffectRecord::default(); MAX_PLAYERS],
            pickups: [PickupRecord::default(); MAX_FIELD_PICKUPS],
            powerup_timer_ticks: 0,
            live_particle_count: 0,
            particles: [ParticleRecord::default(); MAX_SNAPSHOT_PARTICLES],
        }
    }
}

/// Field-wise FNV-1a 64 over every field's value in declared order — never over raw
/// struct bytes (spec §3). Particle records beyond `live_particle_count` are slack,
/// not state, and are excluded. Pure, allocation-free.
#[must_use]
pub fn state_hash(s: &MatchSnapshot) -> u64 {
    let mut h = Fnv1a64::new();
    h.mix_u8(s.state as u8);
    h.mix_u64(s.tick);
    h.mix_u64(s.seed);
    h.mix_u8(s.player_count);
    for score in s.scores {
        h.mix_u32(score);
    }
    h.mix_u8(s.winner as u8);
    h.mix_bool(s.sudden_death);
    for well in s.wells {
        h.mix_u32(well.position.x.to_bits());
        h.mix_u32(well.position.y.to_bits());
        h.mix_u32(well.velocity.x.to_bits());
        h.mix_u32(well.velocity.y.to_bits());
        h.mix_u32(well.strength.to_bits());
        h.mix_bool(well.active);
    }
    for effect in s.effects {
        h.mix_u8(effect.kind as u8);
        h.mix_u32(effect.remaining_ticks);
    }
    for pickup in s.pickups {
        h.mix_u8(pickup.kind as u8);
        h.mix_u32(pickup.position.x.to_bits());
        h.mix_u32(pickup.position.y.to_bits());
        h.mix_u64(pickup.spawn_tick);
        h.mix_bool(pickup.alive);
    }
    h.mix_u32(s.powerup_timer_ticks);
    h.mix_u32(s.live_particle_count);
    for particle in s.particles.iter().take(s.live_particle_count as usize) {
        h.mix_u32(particle.position.x.to_bits());
        h.mix_u32(particle.position.y.to_bits());
        h.mix_u32(particle.velocity.x.to_bits());
        h.mix_u32(particle.velocity.y.to_bits());
    }
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_snapshots_hash_identically() {
        let a = MatchSnapshot::default();
        let b = MatchSnapshot::default();
        assert_eq!(state_hash(&a), state_hash(&b));
    }

    #[test]
    fn changing_tick_changes_the_hash() {
        let a = MatchSnapshot::default();
        let b = MatchSnapshot {
            tick: 1,
            ..MatchSnapshot::default()
        };
        assert_ne!(state_hash(&a), state_hash(&b));
    }

    #[test]
    fn changing_a_score_changes_the_hash() {
        let a = MatchSnapshot::default();
        let b = MatchSnapshot {
            scores: [1, 0, 0, 0],
            ..MatchSnapshot::default()
        };
        assert_ne!(state_hash(&a), state_hash(&b));
    }

    #[test]
    fn particles_beyond_live_count_are_excluded_from_the_hash() {
        let mut a = MatchSnapshot {
            live_particle_count: 1,
            ..MatchSnapshot::default()
        };
        if let Some(first) = a.particles.get_mut(0) {
            first.position = Vec2::new(1.0, 2.0);
        }
        let mut b = a.clone();
        // Mutate slack beyond live_particle_count — must not affect the hash.
        if let Some(second) = b.particles.get_mut(1) {
            second.position = Vec2::new(99.0, 99.0);
        }
        assert_eq!(state_hash(&a), state_hash(&b));
    }
}
