// Deterministic RNG used by simulation paths.
//
// Constitutional article 5 (determinism): seed must be carried at full uint64_t width.
//
// FIX BUG-005: the 64-bit seed is XOR-folded into the engine seed (see rng.cpp).

#pragma once

// Interop boundary (constitution Article 3): std::mt19937 is used here because EASTL
// does not ship a Mersenne Twister engine.
#include <cstdint>
#include <random>

namespace engine_demo::sim {

class [[nodiscard]] rng {
   public:
    explicit rng(std::uint64_t seed) noexcept;

    rng(const rng&) = default;
    rng& operator=(const rng&) = default;
    rng(rng&&) noexcept = default;
    rng& operator=(rng&&) noexcept = default;

    [[nodiscard]] std::uint32_t next_u32() noexcept;
    [[nodiscard]] double next_double_unit() noexcept;

   private:
    std::mt19937 m_engine;
};

}  // namespace engine_demo::sim
