// engine_demo::vfx::particle_pool implementation.
// See specs/004-particle-vfx-subsystem/research.md §1, §7 and
// specs/004-particle-vfx-subsystem/contracts/particle_pool.md.

#include "engine_demo/vfx/particle.h"

namespace engine_demo::vfx {

particle_pool::particle_pool(allocator& alloc, std::size_t capacity) noexcept
    : m_capacity{capacity}, m_particles{eastl_allocator_ref{alloc}} {
    m_particles.reserve(capacity);
    m_particles.resize(capacity);
}

emit_result particle_pool::try_spawn(eastl::span<const spawn_params> params) noexcept {
    // All-or-nothing (research.md §7 / FR-006): reject the whole batch if it does not fit.
    if (params.size() > free_count()) {
        return emit_result{0, vfx_status::pool_exhausted};
    }

    for (std::size_t i = 0; i < params.size(); ++i) {
        spawn_params const& src = params[i];
        particle& dst = m_particles[m_live_count + i];
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
    }
    m_live_count += params.size();

    return emit_result{static_cast<std::uint32_t>(params.size()), vfx_status::ok};
}

void particle_pool::age_and_retire(double dt_seconds) noexcept {
    if (dt_seconds == 0.0) {
        return;
    }

    for (std::size_t i = 0; i < m_live_count;) {
        particle& p = m_particles[i];
        p.remaining_lifetime_seconds -= dt_seconds;
        if (p.remaining_lifetime_seconds <= 0.0) {
            // Swap-remove with the last live particle; do not advance the scan index so
            // the swapped-in element is itself examined next.
            --m_live_count;
            if (i != m_live_count) {
                m_particles[i] = m_particles[m_live_count];
            }
        } else {
            ++i;
        }
    }
}

eastl::span<particle> particle_pool::live_particles() noexcept {
    return eastl::span<particle>{m_particles.data(), m_live_count};
}

eastl::span<const particle> particle_pool::live_particles() const noexcept {
    return eastl::span<const particle>{m_particles.data(), m_live_count};
}

}  // namespace engine_demo::vfx
