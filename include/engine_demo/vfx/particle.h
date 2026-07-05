// Particle storage — fixed-capacity pool of purely-visual particle state.
//
// Constitutional articles satisfied:
//   - 1 (no exceptions: vfx_status enum)
//   - 2 (no RTTI)
//   - 3, 4 (EASTL, allocator-aware: eastl::vector<particle, eastl_allocator_ref>)
//   - 5 (determinism: remaining_lifetime_seconds is a double time accumulator)
//   - 6 (real-time: dense array reserved once at construction, zero allocation afterward)

#pragma once

#include "engine_demo/allocator.h"

#include <EASTL/span.h>
#include <EASTL/vector.h>

#include <cstddef>
#include <cstdint>

namespace engine_demo::vfx {

// Status return for fallible VFX operations (Article 1: no exceptions).
enum class vfx_status : std::uint8_t {
    ok = 0,
    pool_exhausted,
    invalid_argument,
};

// Visual-only particle state. Never read or written by engine_demo::physics::constraint_solver.
struct particle {
    // Time accumulator (Article 5): double. Particle is retired once this reaches <= 0.0.
    double remaining_lifetime_seconds{0.0};
    // Render-boundary values (Article 5 permits float here).
    float position[3]{};
    float velocity[3]{};
    float color[4]{1.0f, 1.0f, 1.0f, 1.0f};
    float size{1.0f};
};

// Initial state for one particle, as produced by an emitter and consumed by particle_pool.
struct spawn_params {
    float position[3]{};
    float velocity[3]{};
    double lifetime_seconds{1.0};
    float color[4]{1.0f, 1.0f, 1.0f, 1.0f};
    float size{1.0f};
};

// Outcome of a try_spawn call.
struct emit_result {
    std::uint32_t spawned{0};
    vfx_status status{vfx_status::ok};
};

// Fixed-capacity, dense-array particle store. Capacity is fixed once at construction;
// try_spawn / age_and_retire never allocate afterward (Article 6).
class [[nodiscard]] particle_pool {
   public:
    particle_pool(allocator& alloc, std::size_t capacity) noexcept;

    particle_pool(const particle_pool&) = delete;
    particle_pool& operator=(const particle_pool&) = delete;
    particle_pool(particle_pool&&) noexcept = default;
    particle_pool& operator=(particle_pool&&) noexcept = default;

    // Spawns as many of params[0..count) as free capacity allows (0 <= spawned <=
    // params.size()). Never allocates. Never modifies existing live particles.
    [[nodiscard]] emit_result try_spawn(eastl::span<const spawn_params> params) noexcept;

    // Ages every live particle by dt_seconds and retires (swap-removes) any whose
    // remaining lifetime reaches <= 0. dt_seconds == 0.0 is a no-op. Never allocates.
    void age_and_retire(double dt_seconds) noexcept;

    [[nodiscard]] eastl::span<particle> live_particles() noexcept;
    [[nodiscard]] eastl::span<const particle> live_particles() const noexcept;

    [[nodiscard]] std::size_t capacity() const noexcept { return m_capacity; }
    [[nodiscard]] std::size_t live_count() const noexcept { return m_live_count; }
    [[nodiscard]] std::size_t free_count() const noexcept { return m_capacity - m_live_count; }

   private:
    eastl_allocator_ref m_alloc;
    std::size_t m_capacity;
    std::size_t m_live_count{0};
    // Dense storage sized to m_capacity once at construction (Article 6). Indices
    // [0, m_live_count) are live; the remainder are unused slack.
    eastl::vector<particle, eastl_allocator_ref> m_particles;
};

}  // namespace engine_demo::vfx
