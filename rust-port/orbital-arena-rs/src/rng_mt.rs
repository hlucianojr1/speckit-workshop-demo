//! MT19937 engine rng with bit-parity to the C++ reference (`engine_demo::sim::rng`,
//! rng.spec.md) — T044 of the Full-Fidelity Closure backlog.
//!
//! The reference implementation is fully portable: `std::mt19937` (the textbook 32-bit
//! Mersenne Twister, whose output sequence is specified exactly by the C++ standard)
//! plus `next_double_unit() = next_u32() / 2^32`. No `uniform_real_distribution`
//! (which would be implementation-defined) is involved, so an exact Rust replica is
//! possible. Implemented directly (no `rand_mt` dependency — documented deviation from
//! T044's "e.g." suggestion: zero new dependencies and exact control over seeding).
//!
//! This engine exists ONLY for cross-language digest parity with the C++ scenes
//! (sandbox-scenes.spec.md §8). It is not a general-purpose rng; `DeterministicRng`
//! (StdRng) remains the crate's rng for non-parity paths. Article VI is satisfied: the
//! stream is explicitly seeded and deterministic.

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908_b0df;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;

/// Textbook 32-bit Mersenne Twister, matching `std::mt19937`'s specified sequence.
pub struct Mt19937 {
    state: [u32; N],
    index: usize,
}

impl Mt19937 {
    /// Seeds exactly like `std::mt19937{seed}` (Knuth 2002 multiplier initialization).
    #[must_use]
    // Justified local allow: every index below is a loop counter over the fixed-size
    // state array (0..N) — provably in bounds; `.get()` fallbacks would silently
    // corrupt the specified sequence on a logic error instead of failing loudly.
    #[allow(clippy::indexing_slicing)]
    pub fn new(seed: u32) -> Self {
        let mut state = [0u32; N];
        state[0] = seed;
        for i in 1..N {
            let prev = state[i - 1];
            state[i] = 1_812_433_253u32
                .wrapping_mul(prev ^ (prev >> 30))
                .wrapping_add(i as u32);
        }
        Self { state, index: N }
    }

    // Justified local allow: all indices are `i`, `(i + 1) % N`, `(i + M) % N` over the
    // fixed-size state array — provably in bounds by construction.
    #[allow(clippy::indexing_slicing)]
    fn twist(&mut self) {
        for i in 0..N {
            let y = (self.state[i] & UPPER_MASK) | (self.state[(i + 1) % N] & LOWER_MASK);
            let mut next = self.state[(i + M) % N] ^ (y >> 1);
            if y & 1 == 1 {
                next ^= MATRIX_A;
            }
            self.state[i] = next;
        }
        self.index = 0;
    }

    /// Next tempered 32-bit output, identical to `std::mt19937::operator()`.
    #[allow(clippy::indexing_slicing)] // index < N guaranteed by the twist() reset
    pub fn next_u32(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }
        let mut y = self.state[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^ (y >> 18)
    }
}

/// Replica of `engine_demo::sim::rng`: 64-bit seed XOR-folded to 32 bits (the reference's
/// BUG-005 fix), MT19937 underneath, and `next_double_unit = next_u32 / 2^32`.
pub struct EngineRng {
    engine: Mt19937,
}

impl EngineRng {
    /// XOR-folds the high half of the 64-bit seed into the low half (reference
    /// `derive_subseed`), then seeds the engine with the folded 32-bit value.
    #[must_use]
    pub fn new(seed: u64) -> Self {
        let high = (seed >> 32) as u32;
        let low = (seed & 0xFFFF_FFFF) as u32;
        Self {
            engine: Mt19937::new(high ^ low),
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.engine.next_u32()
    }

    /// Uniform double in [0, 1): `next_u32() / 4294967296.0`, exactly as the reference.
    pub fn next_double_unit(&mut self) -> f64 {
        f64::from(self.next_u32()) / 4_294_967_296.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Canonical known-answer values for the reference MT19937 (seed 5489): the first
    /// output is 3499211612 and the 10000th is 4123659995 — published constants of the
    /// original Matsumoto–Nishimura implementation that `std::mt19937` reproduces.
    #[test]
    fn matches_reference_mt19937_sequence_for_default_seed() {
        let mut mt = Mt19937::new(5489);
        assert_eq!(mt.next_u32(), 3_499_211_612);

        let mut mt = Mt19937::new(5489);
        let mut last = 0u32;
        for _ in 0..10_000 {
            last = mt.next_u32();
        }
        assert_eq!(last, 4_123_659_995);
    }

    #[test]
    fn seed_folding_uses_full_64_bit_width() {
        // Two seeds sharing low 32 bits must produce different streams (BUG-005 fix).
        let mut a = EngineRng::new(42);
        let mut b = EngineRng::new(42 | (1u64 << 40));
        assert_ne!(a.next_u32(), b.next_u32());
    }

    #[test]
    fn next_double_unit_is_u32_over_two_pow_32() {
        let mut value_stream = EngineRng::new(42);
        let mut raw_stream = EngineRng::new(42);
        for _ in 0..100 {
            let expected = f64::from(raw_stream.next_u32()) / 4_294_967_296.0;
            let actual = value_stream.next_double_unit();
            assert_eq!(actual, expected);
            assert!((0.0..1.0).contains(&actual));
        }
    }
}
