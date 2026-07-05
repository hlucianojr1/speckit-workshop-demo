// Constraint solver — single-pass position-projection over a key-sorted body table.
//
// FIX BUG-004: bodies live in an eastl::vector_map (deterministic, key-sorted) and
// constraints are kept sorted by their canonical (min(a,b), max(a,b)) key at insertion,
// so projection order — and therefore convergence — is identical across runs and across
// construction orders (Article 5).

#pragma once

#include <EASTL/vector.h>
#include <EASTL/vector_map.h>

#include "engine_demo/allocator.h"

#include <cstdint>

namespace engine_demo::physics {

struct body {
    std::uint64_t id{0};
    double position_x{0.0};
    double position_y{0.0};
    double position_z{0.0};
    double inverse_mass{1.0};
};

struct distance_constraint {
    std::uint64_t a{0};
    std::uint64_t b{0};
    double rest_length{1.0};
};

class [[nodiscard]] constraint_solver {
   public:
    explicit constraint_solver(allocator& alloc) noexcept;

    void add_body(body b) noexcept;
    void add_constraint(distance_constraint c) noexcept;

    // Resolve all constraints for one frame. Returns the number of iterations spent.
    [[nodiscard]] std::uint32_t solve(std::uint32_t max_iterations) noexcept;

    // Read-only lookup. Returns nullptr if no body with `id` exists. The pointer is
    // valid until the next add_body / solve call.
    [[nodiscard]] const body* try_get_body(std::uint64_t id) const noexcept;

   private:
    using body_map = eastl::vector_map<std::uint64_t,
                                       body,
                                       eastl::less<std::uint64_t>,
                                       eastl_allocator_ref>;
    using constraint_vec = eastl::vector<distance_constraint, eastl_allocator_ref>;

    eastl_allocator_ref m_alloc;
    body_map m_bodies;
    constraint_vec m_constraints;
};

}  // namespace engine_demo::physics
