//! Deterministic state digest (sandbox-scenes.spec.md §8) — T043 of the Full-Fidelity
//! Closure backlog.
//!
//! 64-bit FNV-1a over each `f64` value's IEEE-754 bit pattern, mixed byte-by-byte
//! least-significant byte first, in the spec's exact order: every body position (spawn
//! order, x then y), then every free particle (spawn order, x then y), then — for the
//! orbital arena scene only — the match block (tick, state, scores, wells, live
//! particles; lands with T066/US8).

use bevy::prelude::*;

use crate::body::{Body, SpawnIndex};
use crate::free_particles::FreeParticle;
use crate::orbital::ArenaRes;

/// FNV-1a 64-bit offset basis (sandbox-scenes.spec.md §8).
pub const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a 64-bit prime.
pub const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Incremental FNV-1a 64 hasher matching the reference `fnv1a_mix` exactly.
#[derive(Debug, Clone, Copy)]
pub struct Fnv1a64(u64);

impl Fnv1a64 {
    #[must_use]
    pub fn new() -> Self {
        Self(FNV_OFFSET_BASIS)
    }

    /// Mixes one byte: `h ^= b; h *= prime` (wrapping).
    pub fn mix_byte(&mut self, byte: u8) {
        self.0 ^= u64::from(byte);
        self.0 = self.0.wrapping_mul(FNV_PRIME);
    }

    /// Mixes a double's IEEE-754 bit pattern byte-by-byte, least-significant byte
    /// first — the reference implementation's per-double loop.
    pub fn mix_f64(&mut self, value: f64) {
        self.mix_u64(value.to_bits());
    }

    /// Mixes a 64-bit value byte-by-byte, least-significant byte first (used by the
    /// orbital-arena snapshot hash — orbital-arena-snapshot.spec.md §3 — for fields
    /// that are not `f64`s but still want the same byte-mixing discipline).
    pub fn mix_u64(&mut self, value: u64) {
        let mut bits = value;
        for _ in 0..8 {
            self.mix_byte((bits & 0xFF) as u8);
            bits >>= 8;
        }
    }

    /// Mixes a 32-bit value (zero-extended).
    pub fn mix_u32(&mut self, value: u32) {
        self.mix_u64(u64::from(value));
    }

    /// Mixes an 8-bit value.
    pub fn mix_u8(&mut self, value: u8) {
        self.mix_byte(value);
    }

    /// Mixes a bool as 0/1.
    pub fn mix_bool(&mut self, value: bool) {
        self.mix_byte(u8::from(value));
    }

    #[must_use]
    pub fn finish(self) -> u64 {
        self.0
    }
}

impl Default for Fnv1a64 {
    fn default() -> Self {
        Self::new()
    }
}

/// Digest of the current simulation state per sandbox-scenes.spec.md §8:
///
/// 1. Every body position in spawn order: x then y.
/// 2. Every free particle in spawn order: x then y.
/// 3. Orbital arena scene only (T066/US8): tick count, match state (as its integer
///    id), each player's score (player order), each well's x and y (well order), each
///    live arena particle's x and y (live order) — every value converted to `f64`
///    before mixing, matching the reference `scene::state_digest`'s orbital block.
///
/// Render-only state (VFX particles, trails, HUD) is excluded by construction.
pub fn state_digest(world: &mut World) -> u64 {
    let mut hash = Fnv1a64::new();

    let mut bodies: Vec<(u32, f64, f64)> = world
        .query::<(&SpawnIndex, &Body)>()
        .iter(world)
        .map(|(index, body)| (index.0, body.position.x, body.position.y))
        .collect();
    bodies.sort_unstable_by_key(|&(index, _, _)| index);
    for (_, x, y) in &bodies {
        hash.mix_f64(*x);
        hash.mix_f64(*y);
    }

    let mut particles: Vec<(u32, f64, f64)> = world
        .query::<(&SpawnIndex, &FreeParticle)>()
        .iter(world)
        .map(|(index, particle)| (index.0, particle.position.x, particle.position.y))
        .collect();
    particles.sort_unstable_by_key(|&(index, _, _)| index);
    for (_, x, y) in &particles {
        hash.mix_f64(*x);
        hash.mix_f64(*y);
    }

    if let Some(arena) = world.get_resource::<ArenaRes>() {
        hash.mix_f64(arena.0.current_tick() as f64);
        hash.mix_f64(f64::from(arena.0.state() as u8));
        for i in 0..u8::try_from(arena.0.wells().len()).unwrap_or(0) {
            hash.mix_f64(f64::from(arena.0.score(i)));
        }
        for well in arena.0.wells() {
            hash.mix_f64(f64::from(well.position.x));
            hash.mix_f64(f64::from(well.position.y));
        }
        for particle in arena.0.live_particles() {
            hash.mix_f64(f64::from(particle.position.x));
            hash.mix_f64(f64::from(particle.position.y));
        }
    }

    hash.finish()
}
