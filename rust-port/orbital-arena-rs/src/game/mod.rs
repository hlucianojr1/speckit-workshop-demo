//! Orbital Arena game layer (Phase A2 extension): gravity wells, an attracted particle
//! field, and capture/scoring — the mechanics that define the Part 3 game's visual
//! identity (specs/transform/orbital-arena-{gravity-well,scoring,match}.spec.md).
//!
//! Scope reduction (documented in orbital-arena-match.spec.md §5 and
//! orbital-arena-scoring.spec.md §5): no lobby/countdown/game-over state machine, no
//! power-ups, no input replay log, no snapshot/state-hash. The scene runs directly in an
//! always-"playing" state so wells + capture + scoring can be demonstrated on their own.

pub mod drag;
pub mod scoring;
pub mod visuals;
pub mod well;

use bevy::math::DVec2;
use bevy::prelude::*;
use rand::Rng;

use crate::alloc::snapshot as alloc_snapshot;
use crate::rng::{insert_rng, DeterministicRng};
use scoring::ScoreTable;
use well::GravityWell;

/// Number of gravity wells / players (orbital-arena-match.spec.md §2 reference: 2..=4).
pub const WELL_COUNT: usize = 2;
/// Fixed particle-field population (capacity-reserved once at `Startup`, Article IV).
pub const PARTICLE_COUNT: usize = 160;
/// Square arena half-extent (orbital-arena-gravity-well.spec.md §3.3).
pub const HALF_EXTENT: f64 = 220.0;
/// Fixed capture radius shared by every well (orbital-arena-gravity-well.spec.md §2).
pub const CAPTURE_RADIUS: f64 = 22.0;
/// Fixed influence radius shared by every well.
pub const INFLUENCE_RADIUS: f64 = 260.0;
/// Deterministic well-patrol angular speed (radians/tick) — a pure function of the tick
/// counter substitutes for interactive player input (no wall clock, Article VI).
const WELL_ANGULAR_SPEED: f64 = 0.01;
const WELL_ORBIT_RADIUS: f64 = 120.0;

/// One field particle attracted by every active well (data model per
/// orbital-arena-gravity-well.spec.md — position/velocity only; capture removes it).
#[derive(Component, Debug, Clone, Copy)]
pub struct FieldParticle {
    pub position: DVec2,
    pub velocity: DVec2,
}

/// Deterministic tick counter driving well patrol motion (substitutes for player input).
#[derive(Resource, Debug, Default)]
pub struct GameTick(pub u64);

/// Whether the interactive well (player 0) is currently being mouse-dragged
/// (specs/transform/orbital-arena-interactive-control.spec.md §2) — read by `drive_wells`
/// to skip its own autopilot update while a drag is in progress, written by
/// `drag::drag_well_system` (only present when `GameVisualsPlugin`/`WellDragPlugin` are
/// added, i.e. never in headless tests).
#[derive(Resource, Debug, Default)]
pub struct WellDragState {
    pub dragging: bool,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameTick::default())
            .insert_resource(ScoreTable::default())
            .insert_resource(WellDragState::default())
            .add_systems(Startup, spawn_wells_and_field.after(insert_rng))
            .add_systems(
                FixedUpdate,
                (
                    drive_wells,
                    integrate_particles,
                    resolve_captures,
                    evaluate_win,
                )
                    .chain(),
            );
    }
}

fn spawn_wells_and_field(mut commands: Commands, mut rng: ResMut<DeterministicRng>) {
    for i in 0..WELL_COUNT {
        let base_angle = (i as f64 / WELL_COUNT as f64) * std::f64::consts::TAU;
        commands.spawn(GravityWell {
            player_index: i as u8,
            position: DVec2::new(base_angle.cos(), base_angle.sin()) * WELL_ORBIT_RADIUS,
            velocity: DVec2::ZERO,
            strength: 1.0,
            influence_radius: INFLUENCE_RADIUS,
            capture_radius: CAPTURE_RADIUS,
            active: true,
        });
    }

    for _ in 0..PARTICLE_COUNT {
        let x = rng.0.random_range(-HALF_EXTENT..HALF_EXTENT);
        let y = rng.0.random_range(-HALF_EXTENT..HALF_EXTENT);
        commands.spawn(FieldParticle {
            position: DVec2::new(x, y),
            velocity: DVec2::ZERO,
        });
    }
}

/// Deterministic patrol pattern (pure function of the tick counter — no RNG, no wall
/// clock) standing in for real player steering input (orbital-arena-gravity-well.spec.md
/// §3.3's `step` — simplified here since there is no interactive input to record).
/// Skips the interactive well (player 0) while it is being mouse-dragged
/// (orbital-arena-interactive-control.spec.md §2).
fn drive_wells(
    mut tick: ResMut<GameTick>,
    drag_state: Res<WellDragState>,
    mut wells: Query<&mut GravityWell>,
) {
    tick.0 += 1;
    for mut well in &mut wells {
        if drag_state.dragging && well.player_index == 0 {
            continue;
        }
        let phase = well.player_index as f64 * std::f64::consts::PI;
        let angle = tick.0 as f64 * WELL_ANGULAR_SPEED + phase;
        let target = DVec2::new(angle.cos(), angle.sin()) * WELL_ORBIT_RADIUS;
        well.velocity = target - well.position;
        well.position = target;
        well.position.x = well.position.x.clamp(-HALF_EXTENT, HALF_EXTENT);
        well.position.y = well.position.y.clamp(-HALF_EXTENT, HALF_EXTENT);
    }
}

/// Applies every active well's radial acceleration to every particle, integrates
/// semi-implicit Euler, and reflects off the square arena boundary (Article VI: `f64`).
///
/// `well_scratch` is a `Local` reused every call (`clear()` + `extend()`, never
/// reallocated after its first frame) so this system performs zero per-frame heap
/// allocation (Article IV) despite needing a snapshot of well state to iterate
/// per-particle.
fn integrate_particles(
    time: Res<Time<Fixed>>,
    wells: Query<&GravityWell>,
    mut particles: Query<&mut FieldParticle>,
    mut well_scratch: Local<Vec<GravityWell>>,
) {
    let dt = time.delta_secs_f64();
    well_scratch.clear();
    well_scratch.extend(wells.iter().copied());

    for mut particle in &mut particles {
        let mut accel = DVec2::ZERO;
        for well in well_scratch.iter() {
            accel += well::radial_acceleration(well, particle.position);
        }
        particle.velocity += accel * dt;
        let new_position = particle.position + particle.velocity * dt;
        particle.position = new_position;

        if particle.position.x.abs() > HALF_EXTENT {
            particle.position.x = particle.position.x.clamp(-HALF_EXTENT, HALF_EXTENT);
            particle.velocity.x = -particle.velocity.x;
        }
        if particle.position.y.abs() > HALF_EXTENT {
            particle.position.y = particle.position.y.clamp(-HALF_EXTENT, HALF_EXTENT);
            particle.velocity.y = -particle.velocity.y;
        }
    }
}

/// Resolves capture contention for every particle (orbital-arena-scoring.spec.md §3.1,
/// §3.3), awards points, respawns captured particles at a fresh random position so the
/// field population — and therefore allocation behavior — stays constant. `well_scratch`
/// is reused the same way as in `integrate_particles` (Article IV: no unbounded growth).
fn resolve_captures(
    wells: Query<&GravityWell>,
    mut particles: Query<&mut FieldParticle>,
    mut scores: ResMut<ScoreTable>,
    mut rng: ResMut<DeterministicRng>,
    mut well_scratch: Local<Vec<GravityWell>>,
) {
    well_scratch.clear();
    well_scratch.extend(wells.iter().copied());
    let mut scored_mask: u32 = 0;

    for mut particle in &mut particles {
        let result = scoring::resolve_capture(&well_scratch, particle.position);
        if let Some(player) = result.winner_index {
            scoring::award_capture(&mut scores, player, 1, false);
            scored_mask |= 1 << player;
            // Respawn in place (no allocation): same entity, fresh random state.
            let x = rng.0.random_range(-HALF_EXTENT..HALF_EXTENT);
            let y = rng.0.random_range(-HALF_EXTENT..HALF_EXTENT);
            particle.position = DVec2::new(x, y);
            particle.velocity = DVec2::ZERO;
        }
    }

    scores.scored_this_tick = scored_mask;
}

fn evaluate_win(mut scores: ResMut<ScoreTable>) {
    let mask = scores.scored_this_tick;
    scoring::evaluate_win(&mut scores, WELL_COUNT as u8, mask);
}

/// Diagnostic helper used by tests: current process-wide allocation counters.
#[must_use]
pub fn alloc_counters() -> (usize, usize) {
    alloc_snapshot()
}
