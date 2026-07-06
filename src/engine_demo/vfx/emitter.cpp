// engine_demo::vfx::emitter implementation.
// See specs/004-particle-vfx-subsystem/contracts/emitter.md and
// specs/004-particle-vfx-subsystem/research.md §4 (fixed rng draw order).

#include "engine_demo/vfx/emitter.h"

#include <cmath>

namespace engine_demo::vfx {

namespace {

constexpr float kPi = 3.14159265358979323846f;

void normalize3(float v[3]) noexcept {
    float const len = std::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    if (len > 1e-8f) {
        v[0] /= len;
        v[1] /= len;
        v[2] /= len;
    }
}

void cross3(const float a[3], const float b[3], float out[3]) noexcept {
    out[0] = a[1] * b[2] - a[2] * b[1];
    out[1] = a[2] * b[0] - a[0] * b[2];
    out[2] = a[0] * b[1] - a[1] * b[0];
}

// Builds an orthonormal (right, fwd) basis perpendicular to a (assumed unit-length) `up`.
void make_basis(const float up[3], float right[3], float fwd[3]) noexcept {
    float reference[3] = {0.0f, 1.0f, 0.0f};
    if (std::fabs(up[1]) > 0.999f) {
        reference[0] = 1.0f;
        reference[1] = 0.0f;
        reference[2] = 0.0f;
    }
    cross3(reference, up, right);
    normalize3(right);
    cross3(up, right, fwd);
}

// Samples a candidate's initial position and unit direction per research.md §4's fixed
// draw order. Consumes m_rng in a shape-dependent draw count (0 for point, 2 for cone,
// 3 for sphere) before the shared speed/lifetime/size draws.
void sample_shape(const emitter_shape& shape,
                   engine_demo::sim::rng& rng,
                   float out_position[3],
                   float out_direction[3]) noexcept {
    if (const point_shape* p = eastl::get_if<point_shape>(&shape)) {
        out_position[0] = p->position[0];
        out_position[1] = p->position[1];
        out_position[2] = p->position[2];
        // point_shape has no direction parameter; default to a fixed "up" emission
        // direction (matches cone_shape's default direction), consuming no rng draws.
        out_direction[0] = 0.0f;
        out_direction[1] = 1.0f;
        out_direction[2] = 0.0f;
        return;
    }

    if (const cone_shape* c = eastl::get_if<cone_shape>(&shape)) {
        out_position[0] = c->origin[0];
        out_position[1] = c->origin[1];
        out_position[2] = c->origin[2];

        float up[3] = {c->direction[0], c->direction[1], c->direction[2]};
        normalize3(up);
        float right[3];
        float fwd[3];
        make_basis(up, right, fwd);

        double const azimuth_t = rng.next_double_unit();
        double const polar_t = rng.next_double_unit();
        float const azimuth = static_cast<float>(azimuth_t) * 2.0f * kPi;
        float const polar = static_cast<float>(polar_t) * c->half_angle_radians;

        float const cos_polar = std::cos(polar);
        float const sin_polar = std::sin(polar);
        float const cos_az = std::cos(azimuth);
        float const sin_az = std::sin(azimuth);

        for (int k = 0; k < 3; ++k) {
            out_direction[k] =
                up[k] * cos_polar + (right[k] * cos_az + fwd[k] * sin_az) * sin_polar;
        }
        normalize3(out_direction);
        return;
    }

    const sphere_shape* s = eastl::get_if<sphere_shape>(&shape);
    // Uniform-in-volume sampling via spherical coordinates (3 draws): cube-root radius
    // fraction keeps the *volume* density uniform, and every sample lies within radius
    // by construction (guarantees the contract's "position lies within radius" bound).
    double const r_frac = rng.next_double_unit();
    double const theta_frac = rng.next_double_unit();
    double const phi_frac = rng.next_double_unit();

    float const r = s->radius * static_cast<float>(std::cbrt(r_frac));
    float const theta = static_cast<float>(theta_frac) * 2.0f * kPi;
    float const phi = static_cast<float>(std::acos(1.0 - 2.0 * phi_frac));

    float const sin_phi = std::sin(phi);
    float const dir[3] = {sin_phi * std::cos(theta), std::cos(phi), sin_phi * std::sin(theta)};

    out_position[0] = s->center[0] + r * dir[0];
    out_position[1] = s->center[1] + r * dir[1];
    out_position[2] = s->center[2] + r * dir[2];
    out_direction[0] = dir[0];
    out_direction[1] = dir[1];
    out_direction[2] = dir[2];
}

}  // namespace

emitter::emitter(allocator& alloc, particle_pool& pool, emitter_config cfg) noexcept
    : m_cfg{cfg},
      m_rng{cfg.seed},
      m_pool{&pool},
      m_forces{eastl_allocator_ref{alloc}},
      m_scratch{eastl_allocator_ref{alloc}} {
    m_forces.reserve(kMaxForces);
    m_scratch.reserve(m_pool->capacity());
}

vfx_status emitter::add_force(force_applicator force) noexcept {
    if (m_forces.size() >= kMaxForces) {
        return vfx_status::invalid_argument;
    }
    m_forces.push_back(force);
    return vfx_status::ok;
}

emit_result emitter::try_emit(std::uint32_t count) noexcept {
    if (count == 0) {
        return emit_result{0, vfx_status::ok};
    }

    // research.md §7: a request that cannot possibly fit the whole pool is rejected
    // immediately, without consuming m_rng.
    if (static_cast<std::size_t>(count) > m_pool->capacity()) {
        return emit_result{0, vfx_status::pool_exhausted};
    }

    m_scratch.clear();
    for (std::uint32_t i = 0; i < count; ++i) {
        float position[3];
        float direction[3];
        sample_shape(m_cfg.shape, m_rng, position, direction);

        double const speed_t = m_rng.next_double_unit();
        float const speed =
            m_cfg.speed_min + static_cast<float>(speed_t) * (m_cfg.speed_max - m_cfg.speed_min);

        double const lifetime_t = m_rng.next_double_unit();
        double const lifetime = m_cfg.lifetime_min_seconds +
                                 lifetime_t * (m_cfg.lifetime_max_seconds - m_cfg.lifetime_min_seconds);

        double const size_t = m_rng.next_double_unit();
        float const size =
            m_cfg.size_min + static_cast<float>(size_t) * (m_cfg.size_max - m_cfg.size_min);

        // FR-013: a non-positive sampled lifetime is silently skipped, not a failure.
        if (lifetime <= 0.0) {
            continue;
        }

        spawn_params sp{};
        sp.position[0] = position[0];
        sp.position[1] = position[1];
        sp.position[2] = position[2];
        sp.velocity[0] = direction[0] * speed;
        sp.velocity[1] = direction[1] * speed;
        sp.velocity[2] = direction[2] * speed;
        sp.lifetime_seconds = lifetime;
        sp.color[0] = m_cfg.color[0];
        sp.color[1] = m_cfg.color[1];
        sp.color[2] = m_cfg.color[2];
        sp.color[3] = m_cfg.color[3];
        sp.size = size;
        m_scratch.push_back(sp);
    }

    return m_pool->try_spawn(
        eastl::span<const spawn_params>{m_scratch.data(), m_scratch.size()});
}

void emitter::tick(double dt_seconds) noexcept {
    if (dt_seconds == 0.0) {
        return;
    }

    float const dt = static_cast<float>(dt_seconds);
    eastl::span<particle> live = m_pool->live_particles();
    for (particle& p : live) {
        float accel[3] = {0.0f, 0.0f, 0.0f};
        for (const force_applicator& force : m_forces) {
            float contribution[3];
            apply_force(force, m_rng, contribution);
            accel[0] += contribution[0];
            accel[1] += contribution[1];
            accel[2] += contribution[2];
        }
        p.velocity[0] += accel[0] * dt;
        p.velocity[1] += accel[1] * dt;
        p.velocity[2] += accel[2] * dt;
        p.position[0] += p.velocity[0] * dt;
        p.position[1] += p.velocity[1] * dt;
        p.position[2] += p.velocity[2] * dt;
    }

    m_pool->age_and_retire(dt_seconds);
}

}  // namespace engine_demo::vfx
