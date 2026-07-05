// Emitter — samples particles from a configured shape and ticks them against attached
// force applicators.
//
// Constitutional articles satisfied:
//   - 1, 2 (no exceptions: vfx_status; no RTTI: eastl::variant shape tagged union)
//   - 3, 4 (EASTL, allocator-aware)
//   - 5 (determinism: single seeded engine_demo::sim::rng, fixed per-particle draw order)
//   - 6 (real-time: scratch spawn buffer reserved once at construction)

#pragma once

#include "engine_demo/allocator.h"
#include "engine_demo/sim/rng.h"
#include "engine_demo/vfx/force_applicator.h"
#include "engine_demo/vfx/particle.h"

#include <EASTL/variant.h>
#include <EASTL/vector.h>

#include <cstddef>
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

// Tagged union (Article 2) over the supported emitter shapes.
using emitter_shape = eastl::variant<point_shape, cone_shape, sphere_shape>;

struct emitter_config {
    emitter_shape shape{point_shape{}};
    // Full-width seed (Article 5); explicitly required, no std::random_device.
    std::uint64_t seed{1};
    float speed_min{0.0f};
    float speed_max{1.0f};
    double lifetime_min_seconds{1.0};
    double lifetime_max_seconds{1.0};
    float color[4]{1.0f, 1.0f, 1.0f, 1.0f};
    float size_min{1.0f};
    float size_max{1.0f};
};

// Configured particle source: owns a seeded rng and zero or more force applicators, and
// draws new particles from a referenced (not owned) particle_pool.
class [[nodiscard]] emitter {
   public:
    emitter(allocator& alloc, particle_pool& pool, emitter_config cfg) noexcept;

    emitter(const emitter&) = delete;
    emitter& operator=(const emitter&) = delete;
    emitter(emitter&&) noexcept = default;
    emitter& operator=(emitter&&) noexcept = default;

    // Attaches a force applicator, up to a small fixed capacity reserved at construction.
    // Returns invalid_argument (and leaves the existing list unchanged) once full.
    [[nodiscard]] vfx_status add_force(force_applicator force) noexcept;

    // Replaces this emitter's shape configuration in place. Does not touch m_rng, m_pool,
    // m_forces, or m_scratch, and performs no allocation (emitter_shape is a variant of
    // small non-owning structs — assignment is a trivial copy). Safe to call between any
    // two try_emit()/tick() calls, including every frame.
    void set_shape(emitter_shape shape) noexcept;

    // Samples `count` particles from this emitter's shape/variance configuration and
    // forwards them to the referenced particle_pool in one batch.
    [[nodiscard]] emit_result try_emit(std::uint32_t count) noexcept;

    // Sums every attached force applicator's contribution for each live particle in the
    // referenced pool, integrates velocity into position, then delegates lifetime
    // aging/retirement to the pool. dt_seconds == 0.0 is a no-op.
    void tick(double dt_seconds) noexcept;

    [[nodiscard]] const particle_pool& pool() const noexcept { return *m_pool; }

   private:
    static constexpr std::size_t kMaxForces = 4;

    [[nodiscard]] spawn_params sample_one() noexcept;

    emitter_config m_cfg;
    engine_demo::sim::rng m_rng;
    particle_pool* m_pool;
    eastl::vector<force_applicator, eastl_allocator_ref> m_forces;
    // Reusable spawn-request buffer sized to the pool's capacity at construction so
    // try_emit never allocates (Article 6), regardless of the burst size requested.
    eastl::vector<spawn_params, eastl_allocator_ref> m_scratch;
};

}  // namespace engine_demo::vfx
