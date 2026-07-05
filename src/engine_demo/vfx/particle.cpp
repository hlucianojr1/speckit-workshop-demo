// Particle pool — dense-array storage with swap-remove recycling.
//
// FIX: none yet (initial implementation).

#include "engine_demo/vfx/particle.h"

namespace engine_demo::vfx {

particle_pool::particle_pool(allocator& alloc, std::size_t capacity) noexcept
    : m_alloc{alloc}, m_capacity{capacity}, m_particles{m_alloc} {
    // Single allocation for the lifetime of the pool (Article 6): all subsequent
    // try_spawn / age_and_retire calls only assign into already-allocated slots.
    m_particles.resize(capacity);
}

emit_result particle_pool::try_spawn(eastl::span<const spawn_params> params) noexcept {
    std::uint32_t spawned = 0;
    for (const spawn_params& src : params) {
        if (m_live_count >= m_capacity) {
            break;
        }
        particle& dst = m_particles[m_live_count];
        dst.remaining_lifetime_seconds = src.lifetime_seconds;
        dst.position[0] = src.position[0];
        dst.position[1] = src.position[1];
        dst.position[2] = src.position[2];
        dst.velocity[0] = src.velocity[0];
        dst.velocity[1] = src.velocity[1];
        dst.velocity[2] = src.velocity[2];
        dst.color[0] = src.color[0];
        dst.color[1] = src.color[1];
        dst.color[2] = src.color[2];
        dst.color[3] = src.color[3];
        dst.size = src.size;
        ++m_live_count;
        ++spawned;
    }
    const vfx_status status =
        (static_cast<std::size_t>(spawned) == params.size()) ? vfx_status::ok : vfx_status::pool_exhausted;
    return {spawned, status};
}

void particle_pool::age_and_retire(double dt_seconds) noexcept {
    if (dt_seconds == 0.0) {
        return;
    }
    std::size_t i = 0;
    while (i < m_live_count) {
        particle& p = m_particles[i];
        p.remaining_lifetime_seconds -= dt_seconds;
        if (p.remaining_lifetime_seconds <= 0.0) {
            // Swap-remove: move the last live particle into slot i (Article 6 — O(1),
            // cache-friendly, deterministic given a deterministic scan order).
            --m_live_count;
            if (i != m_live_count) {
                m_particles[i] = m_particles[m_live_count];
            }
            continue;  // re-examine slot i, which now holds the swapped-in particle.
        }
        ++i;
    }
}

eastl::span<particle> particle_pool::live_particles() noexcept {
    return eastl::span<particle>(m_particles.data(), m_live_count);
}

eastl::span<const particle> particle_pool::live_particles() const noexcept {
    return eastl::span<const particle>(m_particles.data(), m_live_count);
}

}  // namespace engine_demo::vfx
