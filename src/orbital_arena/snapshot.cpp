// Orbital Arena snapshot hashing — see include/orbital_arena/snapshot.h for the contract.

#include "orbital_arena/snapshot.h"

#include <cstring>

namespace orbital_arena {

namespace {

constexpr std::uint64_t kFnvOffsetBasis = 14695981039346656037ull;
constexpr std::uint64_t kFnvPrime = 1099511628211ull;

// Folds `n` bytes into the running FNV-1a 64 hash.
void fold_bytes(std::uint64_t& h, const unsigned char* bytes, std::size_t n) noexcept {
    for (std::size_t i = 0; i < n; ++i) {
        h ^= static_cast<std::uint64_t>(bytes[i]);
        h *= kFnvPrime;
    }
}

void fold_u8(std::uint64_t& h, std::uint8_t v) noexcept {
    fold_bytes(h, reinterpret_cast<const unsigned char*>(&v), sizeof v);
}

void fold_u32(std::uint64_t& h, std::uint32_t v) noexcept {
    fold_bytes(h, reinterpret_cast<const unsigned char*>(&v), sizeof v);
}

void fold_u64(std::uint64_t& h, std::uint64_t v) noexcept {
    fold_bytes(h, reinterpret_cast<const unsigned char*>(&v), sizeof v);
}

// Floats are folded by bit pattern (exact, no rounding — Article 5 bit-exactness).
void fold_f32(std::uint64_t& h, float v) noexcept {
    std::uint32_t bits = 0;
    std::memcpy(&bits, &v, sizeof bits);
    fold_u32(h, bits);
}

void fold_bool(std::uint64_t& h, bool v) noexcept {
    fold_u8(h, v ? 1u : 0u);
}

}  // namespace

std::uint64_t state_hash(const match_snapshot& s) noexcept {
    std::uint64_t h = kFnvOffsetBasis;

    // Declared field order (snapshot.h) — field-wise, never raw struct bytes (D5).
    fold_u8(h, static_cast<std::uint8_t>(s.state));
    fold_u64(h, s.tick);
    fold_u64(h, s.seed);
    fold_u8(h, s.player_count);
    for (std::uint32_t score : s.scores) {
        fold_u32(h, score);
    }
    fold_u8(h, static_cast<std::uint8_t>(s.winner));
    fold_bool(h, s.sudden_death);
    for (const well_record& w : s.wells) {
        fold_f32(h, w.position[0]);
        fold_f32(h, w.position[1]);
        fold_f32(h, w.velocity[0]);
        fold_f32(h, w.velocity[1]);
        fold_f32(h, w.strength);
        fold_bool(h, w.active);
    }
    for (const effect_record& e : s.effects) {
        fold_u8(h, static_cast<std::uint8_t>(e.kind));
        fold_u32(h, e.remaining_ticks);
    }
    for (const pickup_record& p : s.pickups) {
        fold_u8(h, static_cast<std::uint8_t>(p.kind));
        fold_f32(h, p.position[0]);
        fold_f32(h, p.position[1]);
        fold_u64(h, p.spawn_tick);
        fold_bool(h, p.alive);
    }
    fold_u32(h, s.powerup_timer_ticks);
    fold_u32(h, s.live_particle_count);

    // Only live records are state; slack slots are excluded (see header contract).
    const std::uint32_t live = s.live_particle_count < kMaxSnapshotParticles
                                   ? s.live_particle_count
                                   : static_cast<std::uint32_t>(kMaxSnapshotParticles);
    for (std::uint32_t i = 0; i < live; ++i) {
        fold_f32(h, s.particles[i].position[0]);
        fold_f32(h, s.particles[i].position[1]);
        fold_f32(h, s.particles[i].velocity[0]);
        fold_f32(h, s.particles[i].velocity[1]);
    }
    return h;
}

}  // namespace orbital_arena
