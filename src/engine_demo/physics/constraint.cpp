// Constraint solver.
//
// FIX BUG-004: body state lives in a key-sorted eastl::vector_map and constraints are
// kept sorted by their canonical (min(a,b), max(a,b)) key at insertion. Projection order
// is therefore deterministic across runs and across construction orders (Article 5).

#include "engine_demo/physics/constraint.h"

#include <cmath>

namespace engine_demo::physics {

namespace {

struct canonical_key {
    std::uint64_t lo;
    std::uint64_t hi;
};

[[nodiscard]] canonical_key key_of(const distance_constraint& c) noexcept {
    return c.a < c.b ? canonical_key{c.a, c.b} : canonical_key{c.b, c.a};
}

[[nodiscard]] bool key_less(const canonical_key& x, const canonical_key& y) noexcept {
    return x.lo != y.lo ? x.lo < y.lo : x.hi < y.hi;
}

}  // namespace

constraint_solver::constraint_solver(allocator& alloc) noexcept
    : m_alloc{alloc}, m_bodies{m_alloc}, m_constraints{m_alloc} {}

void constraint_solver::add_body(body b) noexcept {
    m_bodies[b.id] = b;
}

void constraint_solver::add_constraint(distance_constraint c) noexcept {
    // Deterministic ordering (Article 5): keep the vector sorted by canonical key so
    // projection order never depends on insertion order.
    auto it = m_constraints.begin();
    const canonical_key k = key_of(c);
    while (it != m_constraints.end() && !key_less(k, key_of(*it))) {
        ++it;
    }
    m_constraints.insert(it, c);
}

const body* constraint_solver::try_get_body(std::uint64_t id) const noexcept {
    auto it = m_bodies.find(id);
    if (it == m_bodies.end()) {
        return nullptr;
    }
    return &it->second;
}

std::uint32_t constraint_solver::solve(std::uint32_t max_iterations) noexcept {
    for (std::uint32_t iter = 0; iter < max_iterations; ++iter) {
        for (auto& c : m_constraints) {
            // Deterministic: vector_map lookup + canonical constraint order (Article 5).
            auto it_a = m_bodies.find(c.a);
            auto it_b = m_bodies.find(c.b);
            if (it_a == m_bodies.end() || it_b == m_bodies.end())
                continue;

            body& a = it_a->second;
            body& b_ref = it_b->second;

            const double dx = b_ref.position_x - a.position_x;
            const double dy = b_ref.position_y - a.position_y;
            const double dz = b_ref.position_z - a.position_z;
            const double length = std::sqrt(dx * dx + dy * dy + dz * dz);
            if (length < 1.0e-12)
                continue;

            const double diff = (length - c.rest_length) / length;
            const double w_sum = a.inverse_mass + b_ref.inverse_mass;
            if (w_sum < 1.0e-12)
                continue;

            const double k_a = a.inverse_mass / w_sum;
            const double k_b = b_ref.inverse_mass / w_sum;
            a.position_x += k_a * dx * diff;
            a.position_y += k_a * dy * diff;
            a.position_z += k_a * dz * diff;
            b_ref.position_x -= k_b * dx * diff;
            b_ref.position_y -= k_b * dy * diff;
            b_ref.position_z -= k_b * dz * diff;
        }
    }
    return max_iterations;
}

}  // namespace engine_demo::physics
