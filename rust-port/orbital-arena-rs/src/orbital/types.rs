//! Orbital Arena shared constants and status enum
//! (orbital-arena-orchestration.spec.md §2, mirroring `include/orbital_arena/types.h`).

/// Roster bounds (orchestration spec §2.1).
pub const MAX_PLAYERS: usize = 4;
pub const MIN_PLAYERS: u8 = 2;

/// Win rule (orbital-arena-scoring.spec.md §2).
pub const WIN_SCORE: u32 = 100;

/// Fixed-step timing — all gameplay durations are integer tick counts (Article V).
pub const COUNTDOWN_TICKS: u32 = 180;
pub const POWERUP_SPAWN_INTERVAL_TICKS: u32 = 600;
pub const STRENGTH_SURGE_TICKS: u32 = 300;
pub const DOUBLE_POINTS_TICKS: u32 = 600;
pub const STATE_HASH_INTERVAL_TICKS: u64 = 60;
pub const REPLENISH_INTERVAL_TICKS: u64 = 60;

/// Power-up field cap (orbital-arena-powerups.spec.md §2.2).
pub const MAX_FIELD_PICKUPS: usize = 3;

/// Particle field sizing (orchestration spec §2.1/§2.2).
pub const TARGET_PARTICLE_COUNT: usize = 500;
pub const MAX_SNAPSHOT_PARTICLES: usize = 512;

/// Game-default arena half-extent (orchestration spec §2.1) — the sandbox embedding
/// overrides this to 2.0 (sandbox-scenes.spec.md §4.5).
pub const ARENA_HALF_EXTENT: f32 = 5.0;

/// Fixed simulation step, `f64` per Article V (orchestration spec §2.2).
pub const TICK_SECONDS: f64 = 1.0 / 60.0;

/// Input-log capacity (orbital-arena-input-log.spec.md §2.3).
pub const MAX_RECORDED_TICKS: usize = 4096;

/// Salted rng sub-seeds (orchestration spec §2.3) — keep flood/replenishment draw
/// counts from ever perturbing the game-rule stream's fixed draw order.
pub const PARTICLE_SEED_SALT: u64 = 0x9E37_79B9_7F4A_7C15;
pub const FIELD_SEED_SALT: u64 = 0xC2B2_AE3D_27D4_EB4F;

/// Initial speed cap for field particles (orchestration spec §4.3).
pub const FIELD_PARTICLE_SPEED_MAX: f32 = 0.1;

/// Effectively-infinite particle lifetime — capture (zeroing) is the only removal path
/// (orchestration spec §2.2/§4.3).
pub const FIELD_PARTICLE_LIFETIME_SECONDS: f64 = 1.0e9;

/// Neutral burst size requested by a consumed Particle Flood (powerup spec §2.1).
pub const FLOOD_BURST_COUNT: u32 = 100;

/// Status return for fallible orbital-arena operations (Article I: no panics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaStatus {
    Ok,
    InvalidArgument,
    WrongState,
    LogFull,
}
