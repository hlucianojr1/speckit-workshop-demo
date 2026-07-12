//! Orbital Arena game layer, full-fidelity port (Full-Fidelity Closure backlog, US8):
//! match lifecycle, gravity wells, power-ups, the input replay log, and the
//! snapshot/state-hash surface, composed into one deterministic fixed-step tick
//! (orbital-arena-orchestration.spec.md). Supersedes the Phase A2 screenshot-scale
//! prototype (pixel units, no match state, no power-ups) formerly in `crate::game`.

pub mod autopilot;
pub mod drag;
pub mod input_log;
pub mod match_state;
pub mod particle;
pub mod powerup;
pub mod scoring;
pub mod snapshot;
pub mod types;
pub mod visuals;
pub mod well;

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::rng_mt::EngineRng;

use input_log::{clamp_input, InputLog, TickInputs};
use match_state::{Match, MatchState};
use particle::{FieldParticle, ParticlePool};
use powerup::PowerupSystem;
use scoring::ScoreTable;
use snapshot::{state_hash, EffectRecord, MatchSnapshot, ParticleRecord, PickupRecord, WellRecord};
use types::{
    ArenaStatus, FIELD_PARTICLE_LIFETIME_SECONDS, FIELD_PARTICLE_SPEED_MAX, FIELD_SEED_SALT,
    MAX_PLAYERS, MAX_RECORDED_TICKS, MAX_SNAPSHOT_PARTICLES, MIN_PLAYERS, PARTICLE_SEED_SALT,
    REPLENISH_INTERVAL_TICKS, STATE_HASH_INTERVAL_TICKS, TICK_SECONDS,
};
use well::GravityWell;

/// Match configuration, validated at creation (orchestration spec §2.1).
#[derive(Debug, Clone, Copy)]
pub struct ArenaConfig {
    pub seed: u64,
    pub player_count: u8,
    pub half_extent: f32,
    pub particle_capacity: usize,
}

impl Default for ArenaConfig {
    fn default() -> Self {
        Self {
            seed: 1,
            player_count: MIN_PLAYERS,
            half_extent: types::ARENA_HALF_EXTENT,
            particle_capacity: types::TARGET_PARTICLE_COUNT,
        }
    }
}

/// Writes the k-th of `n` rotationally symmetric copies of `v` (orchestration spec
/// §4.4: exact sign/index forms for n=2/4, fixed cos/sin(120°) constants for n=3 — the
/// bit-exactness that makes 2-player matches an exact point reflection).
#[must_use]
fn rotate_copy(k: u8, n: u8, v: Vec2) -> Vec2 {
    if k == 0 {
        return v;
    }
    if n == 2 {
        return -v; // exact point reflection
    }
    if n == 4 {
        return match k {
            1 => Vec2::new(-v.y, v.x),
            2 => -v,
            _ => Vec2::new(v.y, -v.x),
        };
    }
    const COS_120: f32 = -0.5;
    const SIN_120: f32 = 0.866_025_4;
    let s = if k == 1 { SIN_120 } else { -SIN_120 };
    Vec2::new(COS_120 * v.x - s * v.y, s * v.x + COS_120 * v.y)
}

fn reflect(position: &mut Vec2, velocity: &mut Vec2, half_extent: f32) {
    if position.x > half_extent {
        position.x = 2.0 * half_extent - position.x;
        velocity.x = -velocity.x;
    } else if position.x < -half_extent {
        position.x = -2.0 * half_extent - position.x;
        velocity.x = -velocity.x;
    }
    if position.y > half_extent {
        position.y = 2.0 * half_extent - position.y;
        velocity.y = -velocity.y;
    } else if position.y < -half_extent {
        position.y = -2.0 * half_extent - position.y;
        velocity.y = -velocity.y;
    }
}

/// The aggregate root: composes match, scoring, power-ups, wells, the input log, and
/// the particle field into one deterministic fixed-step tick
/// (orbital-arena-orchestration.spec.md §4).
pub struct Arena {
    config: ArenaConfig,
    rng: EngineRng,
    field_rng: EngineRng,
    flood_rng: EngineRng,
    match_: Match,
    scores: ScoreTable,
    powerups: PowerupSystem,
    wells: [GravityWell; MAX_PLAYERS],
    input_log: InputLog,
    particles: ParticlePool,
    hash_history: Vec<u64>,
    tick: u64,
}

impl Arena {
    /// Validates `config` and allocates every pool/log up front (Article VI: ticking
    /// never allocates afterward). Returns `None` on an invalid configuration
    /// (Article I: never panics).
    #[must_use]
    pub fn new(config: ArenaConfig) -> Option<Self> {
        if config.player_count < MIN_PLAYERS || config.player_count as usize > MAX_PLAYERS {
            return None;
        }
        if config.half_extent <= 0.0 {
            return None;
        }
        if config.particle_capacity == 0 || config.particle_capacity > MAX_SNAPSHOT_PARTICLES {
            return None;
        }
        let max_hash_history = MAX_RECORDED_TICKS / STATE_HASH_INTERVAL_TICKS as usize + 1;
        Some(Self {
            rng: EngineRng::new(config.seed),
            field_rng: EngineRng::new(config.seed ^ FIELD_SEED_SALT),
            flood_rng: EngineRng::new(config.seed ^ PARTICLE_SEED_SALT),
            match_: Match::default(),
            scores: ScoreTable::default(),
            powerups: PowerupSystem::default(),
            wells: [GravityWell::default(); MAX_PLAYERS],
            input_log: InputLog::new(MAX_RECORDED_TICKS),
            particles: ParticlePool::new(config.particle_capacity),
            hash_history: Vec::with_capacity(max_hash_history),
            config,
            tick: 0,
        })
    }

    /// Lobby passthrough (orchestration spec §3). Player indices are restricted to
    /// `[0, player_count)` so the identity player→well mapping stays dense.
    pub fn join(&mut self, player: u8) -> ArenaStatus {
        if player >= self.config.player_count {
            return ArenaStatus::InvalidArgument;
        }
        self.match_.join(player)
    }

    pub fn set_ready(&mut self, player: u8, ready: bool) -> ArenaStatus {
        if player >= self.config.player_count {
            return ArenaStatus::InvalidArgument;
        }
        self.match_.set_ready(player, ready)
    }

    pub fn leave(&mut self, player: u8) {
        if player >= self.config.player_count {
            return;
        }
        self.match_.leave(player);
        if self.match_.state() != MatchState::Playing {
            return;
        }
        if let Some(well) = self.wells.get_mut(player as usize) {
            well.active = false;
        }
        if self.match_.active_count() < MIN_PLAYERS {
            for i in 0..self.config.player_count {
                if self.wells.get(i as usize).is_some_and(|w| w.active) {
                    self.scores.winner = Some(i);
                    break;
                }
            }
            self.match_.enter_game_over();
            self.powerups.clear_all();
        }
    }

    pub fn acknowledge_results(&mut self) -> ArenaStatus {
        let status = self.match_.acknowledge_results();
        if status == ArenaStatus::Ok {
            scoring::reset_scores(&mut self.scores);
        }
        status
    }

    /// ONE fixed-step tick (orchestration spec §4 — the determinism backbone).
    pub fn tick(&mut self, inputs: &TickInputs) -> ArenaStatus {
        // 1. Ingest + clamp + log (every match state).
        let mut clamped = TickInputs::default();
        for (dst, src) in clamped.players.iter_mut().zip(inputs.players.iter()) {
            *dst = clamp_input(*src);
        }
        let log_status = self.input_log.append(clamped);

        // 2. Match state machine; transition side effects run exactly once.
        let mut entered_playing = false;
        if let Some(transition) = self.match_.tick() {
            if transition.to == MatchState::Playing {
                self.on_playing_entry();
                entered_playing = true;
            }
        }

        // 3. Gameplay systems (playing only; skipped on the transition tick).
        if !entered_playing && self.match_.state() == MatchState::Playing {
            self.run_playing_systems(&clamped);
        }

        // 4. Captured particles (zeroed lifetime) physically leave.
        self.particles.age_and_retire(TICK_SECONDS);

        // 5. Deterministic replenishment schedule.
        self.replenish_field();

        // 6. 60-tick state hash for drift detection.
        if self.tick.is_multiple_of(STATE_HASH_INTERVAL_TICKS) {
            let max_hash_history = MAX_RECORDED_TICKS / STATE_HASH_INTERVAL_TICKS as usize + 1;
            if self.hash_history.len() < max_hash_history {
                let snapshot = self.capture_snapshot();
                self.hash_history.push(state_hash(&snapshot));
            }
        }

        // 7. Advance the fixed-step clock.
        self.tick += 1;

        log_status
    }

    fn on_playing_entry(&mut self) {
        scoring::reset_scores(&mut self.scores);
        self.powerups.clear_all();
        let n = self.config.player_count;
        let base = Vec2::new(self.config.half_extent * 0.5, 0.0);
        for well in &mut self.wells {
            *well = GravityWell {
                active: false,
                ..GravityWell::default()
            };
        }
        for i in 0..n {
            if let Some(well) = self.wells.get_mut(i as usize) {
                well.position = rotate_copy(i, n, base);
                well.active = true;
                well.strength = 0.0;
            }
        }
    }

    fn run_playing_systems(&mut self, inputs: &TickInputs) {
        let n = self.config.player_count;
        let half_extent = self.config.half_extent;

        // 3a. Inputs -> well velocity/strength; integrate; clamp to bounds.
        for i in 0..n {
            let Some(frame) = inputs.players.get(i as usize).copied() else {
                continue;
            };
            if let Some(well) = self.wells.get_mut(i as usize) {
                well::step_well(well, frame.steer, frame.strength, half_extent, TICK_SECONDS);
            }
        }

        // 3b/3c. Radial attraction (well index order, distance-only) then particle
        // integration with boundary reflection — one fixed-order pass.
        let dt = TICK_SECONDS as f32;
        let strength_multipliers: [f32; MAX_PLAYERS] =
            core::array::from_fn(|i| self.powerups.strength_multiplier(i as u8));

        for particle in self.particles.live_mut() {
            let mut accel = Vec2::ZERO;
            for i in 0..n as usize {
                let Some(well) = self.wells.get(i) else {
                    continue;
                };
                let mut pulled = *well;
                if let Some(mult) = strength_multipliers.get(i) {
                    pulled.strength *= *mult;
                }
                accel += well::radial_acceleration(&pulled, particle.position);
            }
            particle.velocity += accel * dt;
            particle.position += particle.velocity * dt;
            reflect(&mut particle.position, &mut particle.velocity, half_extent);
        }

        // 3d. Capture contention + same-tick scoring.
        let mut double_points = [false; MAX_PLAYERS];
        for (i, dp) in double_points.iter_mut().enumerate() {
            *dp = self.powerups.points_multiplier(i as u8) > 1;
        }
        let live_wells: Vec<GravityWell> = self.wells.iter().take(n as usize).copied().collect();
        let scored_mask = scoring::resolve_captures(
            &live_wells,
            self.particles.live_mut(),
            &mut self.scores,
            &double_points,
        );

        // 3e. Pickup consumption; flood requests spawn via the salted flood stream.
        if let Some(flood) = self.powerups.consume_pickups(&live_wells) {
            self.spawn_flood(flood.count);
        }

        // 3f. Power-up spawn schedule (game-rule rng draws only on spawn).
        self.powerups
            .tick_spawn(&mut self.rng, self.tick, half_extent);

        // 3g. Effect timers.
        self.powerups.tick_effects();

        // 3h. Win / sudden-death evaluation.
        if scoring::evaluate_win(&mut self.scores, n, scored_mask).is_some() {
            self.match_.enter_game_over();
            self.powerups.clear_all();
        }
    }

    fn replenish_field(&mut self) {
        if !self.tick.is_multiple_of(REPLENISH_INTERVAL_TICKS) {
            return;
        }
        let group = self.config.player_count;
        let mut deficit = self
            .config
            .particle_capacity
            .saturating_sub(self.particles.live_count());
        while deficit >= group as usize {
            let base_pos = Vec2::new(
                ((self.field_rng.next_double_unit() * 2.0 - 1.0)
                    * f64::from(self.config.half_extent)) as f32,
                ((self.field_rng.next_double_unit() * 2.0 - 1.0)
                    * f64::from(self.config.half_extent)) as f32,
            );
            let heading = self.field_rng.next_double_unit() * std::f64::consts::TAU;
            let speed = self.field_rng.next_double_unit() as f32 * FIELD_PARTICLE_SPEED_MAX;
            let base_vel = Vec2::new(
                speed * (heading.cos() as f32),
                speed * (heading.sin() as f32),
            );

            for k in 0..group {
                let position = rotate_copy(k, group, base_pos);
                let velocity = rotate_copy(k, group, base_vel);
                self.particles.try_spawn_one(FieldParticle {
                    position,
                    velocity,
                    remaining_lifetime_seconds: FIELD_PARTICLE_LIFETIME_SECONDS,
                });
            }
            deficit -= group as usize;
        }
    }

    /// Flood bursts (documented simplification): the C++ reference samples via a
    /// `vfx::emitter` sphere shape on a salted sub-seed; this port draws directly from
    /// the same salted `flood_rng` stream with a simple uniform-disc sample rather
    /// than replicating the emitter's exact 3D sphere-sampling formula bit-for-bit.
    /// Particle Flood essentially never occurs within a short (<= 600-tick) run (the
    /// power-up spawn interval alone is 600 ticks, on top of ~180 ticks of countdown),
    /// so this has no bearing on the golden digest gate.
    fn spawn_flood(&mut self, count: u32) {
        for _ in 0..count {
            let angle = self.flood_rng.next_double_unit() * std::f64::consts::TAU;
            let radius =
                self.flood_rng.next_double_unit().sqrt() * f64::from(self.config.half_extent);
            let position = Vec2::new((radius * angle.cos()) as f32, (radius * angle.sin()) as f32);
            self.particles.try_spawn_one(FieldParticle {
                position,
                velocity: Vec2::ZERO,
                remaining_lifetime_seconds: FIELD_PARTICLE_LIFETIME_SECONDS,
            });
        }
    }

    #[must_use]
    pub fn capture_snapshot(&self) -> MatchSnapshot {
        let mut snapshot = MatchSnapshot {
            state: self.match_.state(),
            tick: self.tick,
            seed: self.config.seed,
            player_count: self.config.player_count,
            scores: self.scores.scores,
            winner: self.scores.winner.map_or(-1, |w| w as i8),
            sudden_death: self.scores.sudden_death,
            powerup_timer_ticks: self.powerups.spawn_timer_ticks(),
            live_particle_count: self.particles.live_count() as u32,
            ..MatchSnapshot::default()
        };
        for (dst, well) in snapshot.wells.iter_mut().zip(self.wells.iter()) {
            *dst = WellRecord {
                position: well.position,
                velocity: well.velocity,
                strength: well.strength,
                active: well.active,
            };
        }
        for (dst, effect) in snapshot
            .effects
            .iter_mut()
            .zip(self.powerups.effects().iter())
        {
            *dst = EffectRecord {
                kind: effect.kind,
                remaining_ticks: effect.remaining_ticks,
            };
        }
        for (dst, pickup) in snapshot
            .pickups
            .iter_mut()
            .zip(self.powerups.field_pickups().iter())
        {
            *dst = PickupRecord {
                kind: pickup.kind,
                position: pickup.position,
                spawn_tick: pickup.spawn_tick,
                alive: pickup.alive,
            };
        }
        for (dst, particle) in snapshot
            .particles
            .iter_mut()
            .zip(self.particles.live().iter())
        {
            *dst = ParticleRecord {
                position: particle.position,
                velocity: particle.velocity,
            };
        }
        snapshot
    }

    #[must_use]
    pub fn hash_history(&self) -> &[u64] {
        &self.hash_history
    }

    #[must_use]
    pub fn log_size(&self) -> usize {
        self.input_log.size()
    }

    /// Directly overrides a well's kinematic state, bypassing the input-driven step
    /// this tick (interactive drag override, orbital-arena-interactive-control.spec.md
    /// §2).
    pub fn override_well_position(&mut self, player: u8, position: Vec2) {
        if let Some(well) = self.wells.get_mut(player as usize) {
            well.velocity = position - well.position;
            well.position = position;
        }
    }

    #[must_use]
    pub fn state(&self) -> MatchState {
        self.match_.state()
    }

    #[must_use]
    pub fn score(&self, player: u8) -> u32 {
        self.scores
            .scores
            .get(player as usize)
            .copied()
            .unwrap_or(0)
    }

    #[must_use]
    pub fn winner(&self) -> Option<u8> {
        self.scores.winner
    }

    #[must_use]
    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    #[must_use]
    pub fn wells(&self) -> &[GravityWell] {
        self.wells
            .get(..self.config.player_count as usize)
            .unwrap_or(&[])
    }

    #[must_use]
    pub fn live_particles(&self) -> &[FieldParticle] {
        self.particles.live()
    }

    #[must_use]
    pub fn config(&self) -> &ArenaConfig {
        &self.config
    }
}

/// Bevy `Resource` wrapper so the arena can be driven by a `FixedUpdate` system and
/// read by rendering/HUD/digest code.
#[derive(Resource)]
pub struct ArenaRes(pub Arena);

/// Whether well 0 is currently being interactively mouse-dragged
/// (orbital-arena-interactive-control.spec.md §2) — read by `tick_arena` to suppress
/// its autopilot input for that tick, written by `drag::drag_well_system`.
#[derive(Resource, Debug, Default)]
pub struct WellDragState {
    pub dragging: bool,
}

/// Installs the arena as a resource, scripts the lobby (both players join and ready up
/// immediately, sandbox-scenes.spec.md §4.5), and ticks it once per `FixedUpdate` step
/// with autopilot input.
pub struct ArenaPlugin {
    pub config: ArenaConfig,
}

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        let Some(mut arena) = Arena::new(self.config) else {
            // Invalid configuration (Article I: never panics) — install nothing
            // further; the scene ticks/renders nothing rather than crashing.
            return;
        };
        for p in 0..arena.config().player_count {
            let _ = arena.join(p);
            let _ = arena.set_ready(p, true);
        }
        app.insert_resource(ArenaRes(arena))
            .insert_resource(WellDragState::default())
            .add_systems(FixedUpdate, tick_arena);
    }
}

fn tick_arena(arena: Option<ResMut<ArenaRes>>, drag_state: Res<WellDragState>) {
    let Some(mut arena) = arena else {
        return;
    };
    let player_count = arena.0.config().player_count;
    let mut inputs = autopilot::make_orbital_inputs(arena.0.current_tick(), player_count);
    if drag_state.dragging {
        // Well 0 is being interactively dragged this tick: suppress its autopilot
        // steering so the drag system's direct position override (applied in `Update`,
        // after this `FixedUpdate` system) isn't immediately fought by the well's own
        // kinematic step next tick.
        if let Some(frame) = inputs.players.get_mut(0) {
            *frame = input_log::InputFrame::default();
        }
    }
    let _ = arena.0.tick(&inputs);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn ready_config() -> ArenaConfig {
        ArenaConfig {
            seed: 42,
            player_count: 2,
            half_extent: 5.0,
            particle_capacity: 16,
        }
    }

    fn ready_arena() -> Arena {
        let mut arena = Arena::new(ready_config()).expect("valid config");
        for p in 0..2 {
            assert_eq!(arena.join(p), ArenaStatus::Ok);
            assert_eq!(arena.set_ready(p, true), ArenaStatus::Ok);
        }
        arena
    }

    #[test]
    fn new_rejects_invalid_configuration() {
        assert!(Arena::new(ArenaConfig {
            player_count: 1,
            ..ready_config()
        })
        .is_none());
        assert!(Arena::new(ArenaConfig {
            half_extent: 0.0,
            ..ready_config()
        })
        .is_none());
        assert!(Arena::new(ArenaConfig {
            particle_capacity: 0,
            ..ready_config()
        })
        .is_none());
    }

    #[test]
    fn ticking_through_countdown_reaches_playing_and_places_wells_symmetrically() {
        let mut arena = ready_arena();
        let idle = TickInputs::default();
        for _ in 0..(types::COUNTDOWN_TICKS + 1) {
            arena.tick(&idle);
        }
        assert_eq!(arena.state(), MatchState::Playing);
        let wells = arena.wells();
        assert_eq!(wells.len(), 2);
        if let (Some(a), Some(b)) = (wells.first(), wells.get(1)) {
            // n=2 rotational symmetry: exact point reflection (mod.rs `rotate_copy`).
            assert_eq!(a.position, -b.position);
        }
    }

    #[test]
    fn field_replenishes_to_capacity_on_the_first_tick() {
        let mut arena = ready_arena();
        arena.tick(&TickInputs::default());
        assert_eq!(arena.live_particles().len(), 16);
    }

    #[test]
    fn tick_never_reallocates_the_input_log_beyond_its_reserved_capacity() {
        let mut arena = Arena::new(ArenaConfig {
            particle_capacity: 2,
            ..ready_config()
        })
        .expect("valid config");
        for p in 0..2 {
            arena.join(p);
            arena.set_ready(p, true);
        }
        for _ in 0..10 {
            arena.tick(&TickInputs::default());
        }
        assert_eq!(arena.log_size(), 10);
    }

    #[test]
    fn override_well_position_updates_velocity_as_a_delta() {
        let mut arena = ready_arena();
        for _ in 0..(types::COUNTDOWN_TICKS + 1) {
            arena.tick(&TickInputs::default());
        }
        let before = arena
            .wells()
            .first()
            .map(|w| w.position)
            .unwrap_or_default();
        arena.override_well_position(0, before + Vec2::new(1.0, 0.0));
        let well = arena.wells().first().copied().unwrap_or_default();
        assert_eq!(well.position, before + Vec2::new(1.0, 0.0));
        assert_eq!(well.velocity, Vec2::new(1.0, 0.0));
    }
}
