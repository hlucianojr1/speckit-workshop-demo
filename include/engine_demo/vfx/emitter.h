// Particle emitter — samples particles from a shape into a particle_pool and ticks
// composed force applicators.
//
// Constitutional articles satisfied:
//   - 1 (no exceptions: vfx_status/emit_result for fallible operations)
//   - 2 (no RTTI: eastl::variant tagged union for emitter_shape)
//   - 3, 4 (EASTL-first, allocator-aware containers)
//   - 5 (determinism: single seeded rng, fixed draw order)
//   - 6 (real-time: force list + scratch spawn buffer reserved once at construction)
//
// See specs/004-particle-vfx-subsystem/contracts/emitter.md for the full contract.

#pragma once

#include <EASTL/variant.h>
#include <EASTL/vector.h>

#include "engine_demo/allocator.h"
#include "engine_demo/sim/rng.h"
#include "engine_demo/vfx/force_applicator.h"
#include "engine_demo/vfx/particle.h"

#include <cstdint>

namespace engine_demo::vfx {

struct point_shape {
    float position[3]{};
};

struct cone_shape {
    float origin[3]{};
    float direction[3]{0.0f, 1.0f, 0.0f};
    float half_angle_radians{0.5f};
};

struct sphere_shape {
    float center[3]{};
    float radius{1.0f};
};

using emitter_shape = eastl::variant<point_shape, cone_shape, sphere_shape>;

struct emitter_config {
    emitter_shape shape{point_shape{}};
    std::uint64_t seed{1};
    float speed_min{0.0f};
    float speed_max{1.0f};
    double lifetime_min_seconds{1.0};
    double lifetime_max_seconds{1.0};
    float color[4]{1.0f, 1.0f, 1.0f, 1.0f};
    float size_min{1.0f};
    float size_max{1.0f};
};

class [[nodiscard]] emitter {
   public:
    emitter(allocator& alloc, particle_pool& pool, emitter_config cfg) noexcept;

    emitter(const emitter&) = delete;
    emitter& operator=(const emitter&) = delete;
    emitter(emitter&&) noexcept = default;
    emitter& operator=(emitter&&) noexcept = default;

    // Attaches a force applicator (up to a small fixed capacity reserved at
    // construction). Returns invalid_argument if the capacity is exhausted.
    [[nodiscard]] vfx_status add_force(force_applicator force) noexcept;

    // Samples up to `count` candidates from cfg.shape and the variance ranges, discards
    // any candidate with a sampled lifetime <= 0.0 (FR-013), then forwards the remainder
    // to the referenced particle_pool in one all-or-nothing try_spawn call
    // (research.md §7). See contracts/particle_pool.md for exhaustion semantics.
    [[nodiscard]] emit_result try_emit(std::uint32_t count) noexcept;

    // Applies the sum of all attached force applicators to every live particle in the
    // referenced pool, integrates velocity into position, then ages/retires expired
    // particles via the pool. Callable independently of the physics constraint solver
    // (FR-005).
    void tick(double dt_seconds) noexcept;

    [[nodiscard]] const particle_pool& pool() const noexcept { return *m_pool; }

   private:
    static constexpr std::size_t kMaxForces = 4;

    using force_vec = eastl::vector<force_applicator, eastl_allocator_ref>;
    using spawn_scratch_vec = eastl::vector<spawn_params, eastl_allocator_ref>;

    emitter_config m_cfg;
    engine_demo::sim::rng m_rng;
    particle_pool* m_pool;
    force_vec m_forces;
    spawn_scratch_vec m_scratch;
};

}  // namespace engine_demo::vfx
