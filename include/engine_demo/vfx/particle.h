// Particle pool — fixed-capacity, allocation-free-after-construction dense storage.
//
// Constitutional articles satisfied:
//   - 1 (no exceptions: vfx_status enum for all fallible operations)
//   - 3, 4 (EASTL-first, allocator-aware: eastl::vector<particle, eastl_allocator_ref>)
//   - 5 (determinism: double lifetime accumulator; dense-array iteration order)
//   - 6 (real-time: dense array reserved once at construction; try_spawn/age_and_retire
//        never allocate)
//
// See specs/004-particle-vfx-subsystem/contracts/particle_pool.md for the full contract.

#pragma once

#include <EASTL/span.h>
#include <EASTL/vector.h>

#include "engine_demo/allocator.h"

#include <cstddef>
#include <cstdint>

namespace engine_demo::vfx {

// Status returned by fallible vfx operations (Article 1: no exceptions).
enum class vfx_status : std::uint8_t {
    ok = 0,
    pool_exhausted,
    invalid_argument,
};

// Visual-only state for a single particle. Never read or written by
// engine_demo::physics::constraint_solver; never registered with engine_demo::ecs::world.
struct particle {
    // Time accumulator (Article 5): double, decremented every tick. Particle is retired
    // when this reaches <= 0.0. Ordered first to avoid mid-struct padding (8-byte aligned).
    double remaining_lifetime_seconds{0.0};
    float position[3]{};  // render-boundary value (Article 5)
    float velocity[3]{};  // render-boundary value; updated by force applicators each tick
    float color[4]{1.0f, 1.0f, 1.0f, 1.0f};  // RGBA, normalized [0, 1]
    float size{1.0f};                        // uniform world-space scale
};

// Candidate initial state for one particle, forwarded to particle_pool::try_spawn.
struct spawn_params {
    float position[3]{};
    float velocity[3]{};
    double lifetime_seconds{1.0};
    float color[4]{1.0f, 1.0f, 1.0f, 1.0f};
    float size{1.0f};
};

// Result of an emit/spawn operation.
struct emit_result {
    std::uint32_t spawned{0};
    vfx_status status{vfx_status::ok};
};

// Fixed-capacity store of `particle`, backed by a dense array reserved once at
// construction (research.md §1). No allocation occurs after construction.
class [[nodiscard]] particle_pool {
   public:
    particle_pool(allocator& alloc, std::size_t capacity) noexcept;

    particle_pool(const particle_pool&) = delete;
    particle_pool& operator=(const particle_pool&) = delete;
    particle_pool(particle_pool&&) noexcept = default;
    particle_pool& operator=(particle_pool&&) noexcept = default;

    // All-or-nothing (research.md §7 / FR-006): spawns every entry in `params`, or none.
    // Never allocates.
    [[nodiscard]] emit_result try_spawn(eastl::span<const spawn_params> params) noexcept;

    // Ages every live particle by `dt_seconds` and retires (swap-removes) any whose
    // remaining lifetime reaches <= 0. Never allocates. Does not apply forces — callers
    // (emitter::tick) integrate velocity into position and apply forces before this call.
    void age_and_retire(double dt_seconds) noexcept;

    [[nodiscard]] eastl::span<particle> live_particles() noexcept;
    [[nodiscard]] eastl::span<const particle> live_particles() const noexcept;

    [[nodiscard]] std::size_t capacity() const noexcept { return m_capacity; }
    [[nodiscard]] std::size_t live_count() const noexcept { return m_live_count; }
    [[nodiscard]] std::size_t free_count() const noexcept { return m_capacity - m_live_count; }

   private:
    using particle_vec = eastl::vector<particle, eastl_allocator_ref>;

    std::size_t m_capacity{0};
    std::size_t m_live_count{0};
    particle_vec m_particles;
};

}  // namespace engine_demo::vfx
