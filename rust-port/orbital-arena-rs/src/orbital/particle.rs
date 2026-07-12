//! Fixed-capacity field-particle pool with dense-array swap-remove recycling — the
//! exact algorithm `engine_demo::vfx::particle_pool` uses in the C++ reference, since
//! live-particle *order* feeds the cross-language state digest (sandbox-scenes.spec.md
//! §8), not just the values.

use bevy::math::Vec2;

use super::types::FIELD_PARTICLE_LIFETIME_SECONDS;

/// One field particle attracted by every active well (orbital-arena-gravity-well.spec.md
/// — position/velocity only; capture removes it by zeroing `remaining_lifetime_seconds`,
/// orbital-arena-scoring.spec.md §3.3).
#[derive(Debug, Clone, Copy)]
pub struct FieldParticle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub remaining_lifetime_seconds: f64,
}

impl Default for FieldParticle {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            remaining_lifetime_seconds: FIELD_PARTICLE_LIFETIME_SECONDS,
        }
    }
}

/// Dense-array particle pool: `slots[0..live_count)` are live, the rest are pre-reserved
/// scratch capacity (Article VI: one allocation at construction, never again).
#[derive(Debug)]
pub struct ParticlePool {
    slots: Vec<FieldParticle>,
    live_count: usize,
}

impl ParticlePool {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: vec![FieldParticle::default(); capacity],
            live_count: 0,
        }
    }

    #[must_use]
    pub fn live_count(&self) -> usize {
        self.live_count
    }

    #[must_use]
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }

    #[must_use]
    pub fn live(&self) -> &[FieldParticle] {
        self.slots.get(..self.live_count).unwrap_or(&[])
    }

    pub fn live_mut(&mut self) -> &mut [FieldParticle] {
        let end = self.live_count;
        self.slots.get_mut(..end).unwrap_or(&mut [])
    }

    /// Appends one particle if the pool has free capacity; no-op (returns `false`)
    /// on exhaustion — never panics (Article I).
    pub fn try_spawn_one(&mut self, particle: FieldParticle) -> bool {
        if self.live_count >= self.slots.len() {
            return false;
        }
        if let Some(slot) = self.slots.get_mut(self.live_count) {
            *slot = particle;
        }
        self.live_count += 1;
        true
    }

    /// Ages every live particle by `dt_seconds`; swap-removes any whose remaining
    /// lifetime reaches <= 0 — a forward scan that, on removal, moves the last live
    /// slot into the freed index and re-examines that index without advancing (matches
    /// the C++ reference's `age_and_retire` exactly, since order feeds the digest).
    pub fn age_and_retire(&mut self, dt_seconds: f64) {
        if dt_seconds == 0.0 {
            return;
        }
        let mut i = 0usize;
        while i < self.live_count {
            let Some(p) = self.slots.get_mut(i) else {
                break;
            };
            p.remaining_lifetime_seconds -= dt_seconds;
            if p.remaining_lifetime_seconds <= 0.0 {
                self.live_count -= 1;
                if i != self.live_count {
                    self.slots.swap(i, self.live_count);
                }
                continue; // re-examine slot i, which now holds the swapped-in particle
            }
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn particle_with_lifetime(seconds: f64) -> FieldParticle {
        FieldParticle {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            remaining_lifetime_seconds: seconds,
        }
    }

    #[test]
    fn try_spawn_one_fails_at_capacity() {
        let mut pool = ParticlePool::new(1);
        assert!(pool.try_spawn_one(FieldParticle::default()));
        assert!(!pool.try_spawn_one(FieldParticle::default()));
        assert_eq!(pool.live_count(), 1);
    }

    #[test]
    fn age_and_retire_removes_expired_particles() {
        let mut pool = ParticlePool::new(4);
        pool.try_spawn_one(particle_with_lifetime(1.0));
        pool.try_spawn_one(particle_with_lifetime(0.001));
        pool.try_spawn_one(particle_with_lifetime(1.0));
        pool.age_and_retire(0.01);
        assert_eq!(pool.live_count(), 2);
    }

    #[test]
    fn age_and_retire_is_noop_for_zero_dt() {
        let mut pool = ParticlePool::new(2);
        pool.try_spawn_one(particle_with_lifetime(0.0));
        pool.age_and_retire(0.0);
        assert_eq!(pool.live_count(), 1);
    }

    #[test]
    fn swap_remove_preserves_remaining_live_particles() {
        let mut pool = ParticlePool::new(3);
        pool.try_spawn_one(FieldParticle {
            position: Vec2::new(1.0, 0.0),
            velocity: Vec2::ZERO,
            remaining_lifetime_seconds: 0.001,
        });
        pool.try_spawn_one(FieldParticle {
            position: Vec2::new(2.0, 0.0),
            velocity: Vec2::ZERO,
            remaining_lifetime_seconds: 1.0,
        });
        pool.age_and_retire(0.01);
        assert_eq!(pool.live_count(), 1);
        assert_eq!(
            pool.live().first().map(|p| p.position),
            Some(Vec2::new(2.0, 0.0))
        );
    }
}
