// Emitter — shape sampling, force composition, and per-tick integration.
//
// FIX: none yet (initial implementation).

#include "engine_demo/vfx/emitter.h"

#include <cmath>

namespace engine_demo::vfx {

namespace {

constexpr double kPi = 3.14159265358979323846;

[[nodiscard]] double lerp(double lo, double hi, double t) noexcept {
    return lo + (hi - lo) * t;
}

void normalize3(const float in[3], float out[3]) noexcept {
    const double length = std::sqrt(static_cast<double>(in[0]) * in[0] + static_cast<double>(in[1]) * in[1] +
                                     static_cast<double>(in[2]) * in[2]);
    if (length < 1.0e-12) {
        out[0] = 0.0f;
        out[1] = 1.0f;
        out[2] = 0.0f;
        return;
    }
    out[0] = static_cast<float>(in[0] / length);
    out[1] = static_cast<float>(in[1] / length);
    out[2] = static_cast<float>(in[2] / length);
}

void cross3(const float a[3], const float b[3], float out[3]) noexcept {
    out[0] = a[1] * b[2] - a[2] * b[1];
    out[1] = a[2] * b[0] - a[0] * b[2];
    out[2] = a[0] * b[1] - a[1] * b[0];
}

// Builds an orthonormal (tangent, bitangent) basis perpendicular to a unit `axis`.
void orthonormal_basis(const float axis[3], float tangent[3], float bitangent[3]) noexcept {
    const float world_up[3] = {0.0f, 1.0f, 0.0f};
    const float fallback_up[3] = {1.0f, 0.0f, 0.0f};
    const float dot_up = axis[0] * world_up[0] + axis[1] * world_up[1] + axis[2] * world_up[2];
    const float* up = (std::fabs(dot_up) > 0.999f) ? fallback_up : world_up;
    float raw_tangent[3];
    cross3(up, axis, raw_tangent);
    normalize3(raw_tangent, tangent);
    cross3(axis, tangent, bitangent);
}

}  // namespace

emitter::emitter(allocator& alloc, particle_pool& pool, emitter_config cfg) noexcept
    : m_cfg{cfg},
      m_rng{cfg.seed},
      m_pool{&pool},
      m_forces{eastl_allocator_ref{alloc}},
      m_scratch{eastl_allocator_ref{alloc}} {
    m_forces.reserve(kMaxForces);
    // Sized once at construction (Article 6): try_emit never grows this buffer.
    m_scratch.resize(pool.capacity());
}

vfx_status emitter::add_force(force_applicator force) noexcept {
    if (m_forces.size() >= kMaxForces) {
        return vfx_status::invalid_argument;
    }
    m_forces.push_back(force);
    return vfx_status::ok;
}

void emitter::set_shape(emitter_shape shape) noexcept {
    m_cfg.shape = shape;
}

spawn_params emitter::sample_one() noexcept {
    spawn_params p{};
    float direction[3] = {0.0f, 1.0f, 0.0f};

    if (const auto* point = eastl::get_if<point_shape>(&m_cfg.shape)) {
        p.position[0] = point->position[0];
        p.position[1] = point->position[1];
        p.position[2] = point->position[2];
        // No rng draw: a point emitter's position is fixed. Direction defaults to +Y.
    } else if (const auto* cone = eastl::get_if<cone_shape>(&m_cfg.shape)) {
        p.position[0] = cone->origin[0];
        p.position[1] = cone->origin[1];
        p.position[2] = cone->origin[2];

        // 2 draws: azimuth around the cone axis, polar offset within half_angle_radians.
        const double u1 = m_rng.next_double_unit();
        const double u2 = m_rng.next_double_unit();
        const double cos_half = std::cos(static_cast<double>(cone->half_angle_radians));
        const double cos_theta = 1.0 - u1 * (1.0 - cos_half);
        const double sin_theta = std::sqrt(std::max(0.0, 1.0 - cos_theta * cos_theta));
        const double phi = 2.0 * kPi * u2;
        const double local_x = sin_theta * std::cos(phi);
        const double local_y = sin_theta * std::sin(phi);
        const double local_z = cos_theta;

        float axis[3];
        normalize3(cone->direction, axis);
        float tangent[3];
        float bitangent[3];
        orthonormal_basis(axis, tangent, bitangent);
        for (int i = 0; i < 3; ++i) {
            direction[i] = static_cast<float>(local_x * tangent[i] + local_y * bitangent[i] + local_z * axis[i]);
        }
    } else if (const auto* sphere = eastl::get_if<sphere_shape>(&m_cfg.shape)) {
        // 3 draws: a uniformly-distributed direction (2 draws) and a volumetric radius
        // fraction (1 draw) together give a uniform point within the sphere's volume.
        const double u1 = m_rng.next_double_unit();
        const double u2 = m_rng.next_double_unit();
        const double u3 = m_rng.next_double_unit();
        const double z = 1.0 - 2.0 * u1;
        const double r_xy = std::sqrt(std::max(0.0, 1.0 - z * z));
        const double phi = 2.0 * kPi * u2;
        direction[0] = static_cast<float>(r_xy * std::cos(phi));
        direction[1] = static_cast<float>(r_xy * std::sin(phi));
        direction[2] = static_cast<float>(z);
        const double radius_fraction = std::cbrt(u3);
        const double dist = static_cast<double>(sphere->radius) * radius_fraction;
        p.position[0] = sphere->center[0] + static_cast<float>(direction[0] * dist);
        p.position[1] = sphere->center[1] + static_cast<float>(direction[1] * dist);
        p.position[2] = sphere->center[2] + static_cast<float>(direction[2] * dist);
    }

    const double speed = lerp(m_cfg.speed_min, m_cfg.speed_max, m_rng.next_double_unit());
    const double lifetime = lerp(m_cfg.lifetime_min_seconds, m_cfg.lifetime_max_seconds, m_rng.next_double_unit());
    const double size = lerp(m_cfg.size_min, m_cfg.size_max, m_rng.next_double_unit());

    p.velocity[0] = static_cast<float>(direction[0] * speed);
    p.velocity[1] = static_cast<float>(direction[1] * speed);
    p.velocity[2] = static_cast<float>(direction[2] * speed);
    p.lifetime_seconds = lifetime;
    p.color[0] = m_cfg.color[0];
    p.color[1] = m_cfg.color[1];
    p.color[2] = m_cfg.color[2];
    p.color[3] = m_cfg.color[3];
    p.size = static_cast<float>(size);
    return p;
}

emit_result emitter::try_emit(std::uint32_t count) noexcept {
    if (count == 0) {
        return {0, vfx_status::ok};
    }
    const std::size_t requested = (static_cast<std::size_t>(count) < m_scratch.size())
                                       ? static_cast<std::size_t>(count)
                                       : m_scratch.size();
    for (std::size_t i = 0; i < requested; ++i) {
        m_scratch[i] = sample_one();
    }
    emit_result result = m_pool->try_spawn(eastl::span<const spawn_params>(m_scratch.data(), requested));
    // The pool only sees `requested` (<= count) entries; if we had to truncate the
    // request to fit the scratch buffer, the caller's original `count` was not fully
    // satisfiable, so the overall status must reflect that even if the pool itself
    // reports `ok` for the smaller, truncated request.
    if (result.spawned < count) {
        result.status = vfx_status::pool_exhausted;
    }
    return result;
}

void emitter::tick(double dt_seconds) noexcept {
    if (dt_seconds == 0.0) {
        return;
    }
    const float dt = static_cast<float>(dt_seconds);
    for (particle& p : m_pool->live_particles()) {
        float acceleration[3] = {0.0f, 0.0f, 0.0f};
        for (const force_applicator& force : m_forces) {
            float contribution[3];
            apply_force(force, m_rng, contribution);
            acceleration[0] += contribution[0];
            acceleration[1] += contribution[1];
            acceleration[2] += contribution[2];
        }
        p.velocity[0] += acceleration[0] * dt;
        p.velocity[1] += acceleration[1] * dt;
        p.velocity[2] += acceleration[2] * dt;
        p.position[0] += p.velocity[0] * dt;
        p.position[1] += p.velocity[1] * dt;
        p.position[2] += p.velocity[2] * dt;
    }
    m_pool->age_and_retire(dt_seconds);
}

}  // namespace engine_demo::vfx
