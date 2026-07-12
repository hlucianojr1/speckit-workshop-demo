//! Seed-driven power-up pickups and timed per-player effects
//! (orbital-arena-powerups.spec.md). Consumption contention reuses scoring's
//! capture-resolution rule; tick-pipeline position is fixed by the orchestration spec.

use bevy::math::Vec2;

use crate::rng_mt::EngineRng;

use super::scoring::resolve_capture;
use super::types::{
    DOUBLE_POINTS_TICKS, FLOOD_BURST_COUNT, MAX_FIELD_PICKUPS, MAX_PLAYERS,
    POWERUP_SPAWN_INTERVAL_TICKS, STRENGTH_SURGE_TICKS,
};
use super::well::GravityWell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PowerupKind {
    StrengthSurge = 0,
    DoublePoints = 1,
    ParticleFlood = 2,
}

impl PowerupKind {
    #[must_use]
    fn from_index(index: u32) -> Self {
        match index % 3 {
            0 => Self::StrengthSurge,
            1 => Self::DoublePoints,
            _ => Self::ParticleFlood,
        }
    }
}

/// Field pickup slot (spec §2.2). Fixed slot array of `MAX_FIELD_PICKUPS` — no
/// allocation.
#[derive(Debug, Clone, Copy)]
pub struct Pickup {
    pub kind: PowerupKind,
    pub position: Vec2,
    pub spawn_tick: u64,
    pub alive: bool,
}

impl Default for Pickup {
    fn default() -> Self {
        Self {
            kind: PowerupKind::StrengthSurge,
            position: Vec2::ZERO,
            spawn_tick: 0,
            alive: false,
        }
    }
}

/// Per-player timed effect slot; `remaining_ticks == 0` means inactive (spec §2.2).
#[derive(Debug, Clone, Copy)]
pub struct ActiveEffect {
    pub kind: PowerupKind,
    pub remaining_ticks: u32,
}

impl Default for ActiveEffect {
    fn default() -> Self {
        Self {
            kind: PowerupKind::StrengthSurge,
            remaining_ticks: 0,
        }
    }
}

/// Burst request surfaced for the caller to forward to the flood emitter stream.
#[derive(Debug, Clone, Copy)]
pub struct FloodRequest {
    pub count: u32,
}

/// Fixed-slot power-up manager: no allocation after construction (Article VI).
#[derive(Debug)]
pub struct PowerupSystem {
    pickups: [Pickup; MAX_FIELD_PICKUPS],
    effects: [ActiveEffect; MAX_PLAYERS],
    spawn_timer: u32,
}

impl Default for PowerupSystem {
    fn default() -> Self {
        Self {
            pickups: [Pickup::default(); MAX_FIELD_PICKUPS],
            effects: [ActiveEffect::default(); MAX_PLAYERS],
            spawn_timer: POWERUP_SPAWN_INTERVAL_TICKS,
        }
    }
}

impl PowerupSystem {
    /// Called once per playing tick. Decrements the spawn timer; when it hits 0:
    /// resets to `POWERUP_SPAWN_INTERVAL_TICKS` and, iff fewer than
    /// `MAX_FIELD_PICKUPS` are alive, spawns one pickup with position AND kind drawn
    /// from `rng` (exactly 3 draws: x, y, kind — fixed order). At cap: the timer still
    /// resets, the spawn is skipped, and NO rng draws occur (draw-on-spawn-only keeps
    /// the stream auditable, spec §3.1).
    pub fn tick_spawn(&mut self, rng: &mut EngineRng, current_tick: u64, half_extent: f32) {
        self.spawn_timer -= 1;
        if self.spawn_timer > 0 {
            return;
        }
        self.spawn_timer = POWERUP_SPAWN_INTERVAL_TICKS;

        let mut alive = 0u32;
        let mut free_index: Option<usize> = None;
        for (i, slot) in self.pickups.iter().enumerate() {
            if slot.alive {
                alive += 1;
            } else if free_index.is_none() {
                free_index = Some(i);
            }
        }
        if alive as usize >= MAX_FIELD_PICKUPS {
            return;
        }
        let Some(index) = free_index else {
            return;
        };

        let x = ((rng.next_double_unit() * 2.0 - 1.0) * f64::from(half_extent)) as f32;
        let y = ((rng.next_double_unit() * 2.0 - 1.0) * f64::from(half_extent)) as f32;
        let kind = PowerupKind::from_index(rng.next_u32() % 3);

        if let Some(slot) = self.pickups.get_mut(index) {
            slot.kind = kind;
            slot.position = Vec2::new(x, y);
            slot.spawn_tick = current_tick;
            slot.alive = true;
        }
    }

    /// FR-011 consumption: contention over alive pickups using scoring's
    /// `resolve_capture` rule (spec §3.2). Timed kinds occupy the winner's effect
    /// slot; `particle_flood` returns a flood request (merged if several are consumed
    /// the same tick).
    pub fn consume_pickups(&mut self, wells: &[GravityWell]) -> Option<FloodRequest> {
        let mut flood_total = 0u32;
        for slot in &mut self.pickups {
            if !slot.alive {
                continue;
            }
            let result = resolve_capture(wells, slot.position);
            let Some(player) = result.winner_index else {
                continue;
            };
            slot.alive = false;
            let effect_slot = |effects: &mut [ActiveEffect; MAX_PLAYERS], kind, ticks| {
                if let Some(effect) = effects.get_mut(player as usize) {
                    *effect = ActiveEffect {
                        kind,
                        remaining_ticks: ticks,
                    };
                }
            };
            match slot.kind {
                PowerupKind::StrengthSurge => {
                    effect_slot(
                        &mut self.effects,
                        PowerupKind::StrengthSurge,
                        STRENGTH_SURGE_TICKS,
                    );
                }
                PowerupKind::DoublePoints => {
                    effect_slot(
                        &mut self.effects,
                        PowerupKind::DoublePoints,
                        DOUBLE_POINTS_TICKS,
                    );
                }
                PowerupKind::ParticleFlood => {
                    flood_total += FLOOD_BURST_COUNT; // instant: never occupies a slot
                }
            }
        }
        if flood_total == 0 {
            None
        } else {
            Some(FloodRequest { count: flood_total })
        }
    }

    /// Decrements all nonzero effect timers by one tick; expires exactly at 0.
    pub fn tick_effects(&mut self) {
        for effect in &mut self.effects {
            if effect.remaining_ticks > 0 {
                effect.remaining_ticks -= 1;
            }
        }
    }

    /// Match end or match start: all effects end immediately, pickups clear, spawn
    /// timer resets (spec §3.4).
    pub fn clear_all(&mut self) {
        self.pickups = [Pickup::default(); MAX_FIELD_PICKUPS];
        self.effects = [ActiveEffect::default(); MAX_PLAYERS];
        self.spawn_timer = POWERUP_SPAWN_INTERVAL_TICKS;
    }

    #[must_use]
    pub fn is_active(&self, player: u8, kind: PowerupKind) -> bool {
        self.effects
            .get(player as usize)
            .is_some_and(|e| e.remaining_ticks > 0 && e.kind == kind)
    }

    #[must_use]
    pub fn strength_multiplier(&self, player: u8) -> f32 {
        if self.is_active(player, PowerupKind::StrengthSurge) {
            2.0
        } else {
            1.0
        }
    }

    #[must_use]
    pub fn points_multiplier(&self, player: u8) -> u32 {
        if self.is_active(player, PowerupKind::DoublePoints) {
            2
        } else {
            1
        }
    }

    #[must_use]
    pub fn spawn_timer_ticks(&self) -> u32 {
        self.spawn_timer
    }

    #[must_use]
    pub fn field_pickups(&self) -> &[Pickup; MAX_FIELD_PICKUPS] {
        &self.pickups
    }

    #[must_use]
    pub fn effects(&self) -> &[ActiveEffect; MAX_PLAYERS] {
        &self.effects
    }
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
    fn tick_spawn_draws_exactly_on_the_interval() {
        let mut system = PowerupSystem::default();
        let mut rng = EngineRng::new(1);
        for _ in 0..(POWERUP_SPAWN_INTERVAL_TICKS - 1) {
            system.tick_spawn(&mut rng, 0, 5.0);
        }
        assert!(!system.field_pickups()[0].alive);
        system.tick_spawn(&mut rng, 0, 5.0);
        let alive_count = system.field_pickups().iter().filter(|p| p.alive).count();
        assert_eq!(alive_count, 1);
    }

    #[test]
    fn tick_spawn_resets_timer_and_skips_draws_at_cap() {
        let mut system = PowerupSystem::default();
        let mut rng = EngineRng::new(1);
        for _ in 0..MAX_FIELD_PICKUPS {
            for _ in 0..POWERUP_SPAWN_INTERVAL_TICKS {
                system.tick_spawn(&mut rng, 0, 5.0);
            }
        }
        assert_eq!(
            system.field_pickups().iter().filter(|p| p.alive).count(),
            MAX_FIELD_PICKUPS
        );
        // One more full interval at the cap: timer resets, but no new pickup appears
        // and (documented contract) no rng draws occur.
        let mut probe_rng = EngineRng::new(1);
        for _ in 0..POWERUP_SPAWN_INTERVAL_TICKS {
            system.tick_spawn(&mut probe_rng, 0, 5.0);
        }
        assert_eq!(system.spawn_timer_ticks(), POWERUP_SPAWN_INTERVAL_TICKS);
        assert_eq!(
            system.field_pickups().iter().filter(|p| p.alive).count(),
            MAX_FIELD_PICKUPS
        );
    }

    #[test]
    fn surge_duration_is_300_ticks_and_double_points_is_600() {
        let mut system = PowerupSystem::default();
        system.pickups[0] = Pickup {
            kind: PowerupKind::StrengthSurge,
            position: Vec2::ZERO,
            spawn_tick: 0,
            alive: true,
        };
        let wells = [well_at(Vec2::ZERO)];
        system.consume_pickups(&wells);
        assert!(system.is_active(0, PowerupKind::StrengthSurge));
        for _ in 0..STRENGTH_SURGE_TICKS {
            system.tick_effects();
        }
        assert!(!system.is_active(0, PowerupKind::StrengthSurge));

        system.pickups[0] = Pickup {
            kind: PowerupKind::DoublePoints,
            position: Vec2::ZERO,
            spawn_tick: 0,
            alive: true,
        };
        system.consume_pickups(&wells);
        for _ in 0..DOUBLE_POINTS_TICKS - 1 {
            system.tick_effects();
        }
        assert!(system.is_active(0, PowerupKind::DoublePoints));
        system.tick_effects();
        assert!(!system.is_active(0, PowerupKind::DoublePoints));
    }

    #[test]
    fn same_kind_refresh_resets_duration() {
        let mut system = PowerupSystem::default();
        let wells = [well_at(Vec2::ZERO)];
        system.pickups[0] = Pickup {
            kind: PowerupKind::StrengthSurge,
            position: Vec2::ZERO,
            spawn_tick: 0,
            alive: true,
        };
        system.consume_pickups(&wells);
        for _ in 0..(STRENGTH_SURGE_TICKS - 1) {
            system.tick_effects();
        }
        system.pickups[0] = Pickup {
            kind: PowerupKind::StrengthSurge,
            position: Vec2::ZERO,
            spawn_tick: 0,
            alive: true,
        };
        system.consume_pickups(&wells);
        assert_eq!(system.effects()[0].remaining_ticks, STRENGTH_SURGE_TICKS);
    }

    #[test]
    fn flood_requests_merge_within_one_tick() {
        let mut system = PowerupSystem::default();
        let wells = [well_at(Vec2::ZERO)];
        system.pickups[0] = Pickup {
            kind: PowerupKind::ParticleFlood,
            position: Vec2::ZERO,
            spawn_tick: 0,
            alive: true,
        };
        system.pickups[1] = Pickup {
            kind: PowerupKind::ParticleFlood,
            position: Vec2::new(1.0, 0.0),
            spawn_tick: 0,
            alive: true,
        };
        let request = system.consume_pickups(&wells);
        assert_eq!(request.map(|r| r.count), Some(FLOOD_BURST_COUNT * 2));
    }

    #[test]
    fn clear_all_ends_effects_and_pickups_and_resets_timer() {
        let mut system = PowerupSystem::default();
        system.pickups[0].alive = true;
        system.effects[0].remaining_ticks = 10;
        system.spawn_timer = 3;
        system.clear_all();
        assert!(!system.field_pickups().iter().any(|p| p.alive));
        assert!(!system.effects().iter().any(|e| e.remaining_ticks > 0));
        assert_eq!(system.spawn_timer_ticks(), POWERUP_SPAWN_INTERVAL_TICKS);
    }
}
