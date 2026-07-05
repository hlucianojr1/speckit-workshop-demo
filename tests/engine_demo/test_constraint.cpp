// GoogleTest for engine_demo::physics::constraint_solver.

#include "engine_demo/physics/constraint.h"

#include <array>
#include <gtest/gtest.h>

namespace {

TEST(constraint_solver, two_body_distance_constraint_converges) {
    std::array<std::byte, 4096> buffer{};
    engine_demo::allocator alloc{buffer.data(), buffer.size()};
    engine_demo::physics::constraint_solver s{alloc};

    s.add_body({1u, 0.0, 0.0, 0.0, 1.0});
    s.add_body({2u, 2.0, 0.0, 0.0, 1.0});
    s.add_constraint({1u, 2u, 1.0});

    const std::uint32_t iters = s.solve(8);
    EXPECT_EQ(iters, 8u);
}

// REGRESSION TEST for BUG-004 (non-determinism). Guards deterministic projection order:
// two solvers fed identical data in opposite insertion orders must converge bit-exactly.
TEST(constraint_solver, solve_is_deterministic_across_construction_orders) {
    std::array<std::byte, 8192> buffer_fwd{};
    std::array<std::byte, 8192> buffer_rev{};
    engine_demo::allocator alloc_fwd{buffer_fwd.data(), buffer_fwd.size()};
    engine_demo::allocator alloc_rev{buffer_rev.data(), buffer_rev.size()};
    engine_demo::physics::constraint_solver fwd{alloc_fwd};
    engine_demo::physics::constraint_solver rev{alloc_rev};

    const engine_demo::physics::body bodies[] = {
        {1u, 0.0, 0.0, 0.0, 1.0},
        {2u, 2.0, 0.0, 0.0, 1.0},
        {3u, 2.0, 2.0, 0.0, 1.0},
        {4u, 0.0, 2.0, 0.0, 1.0},
    };
    const engine_demo::physics::distance_constraint constraints[] = {
        {1u, 2u, 1.0},
        {2u, 3u, 1.0},
        {3u, 4u, 1.0},
        {4u, 1u, 1.0},
    };

    for (const auto& b : bodies)
        fwd.add_body(b);
    for (const auto& c : constraints)
        fwd.add_constraint(c);

    for (int i = 3; i >= 0; --i)
        rev.add_body(bodies[i]);
    for (int i = 3; i >= 0; --i)
        rev.add_constraint(constraints[i]);

    (void)fwd.solve(8);
    (void)rev.solve(8);

    for (const auto& b : bodies) {
        const auto* pf = fwd.try_get_body(b.id);
        const auto* pr = rev.try_get_body(b.id);
        ASSERT_NE(pf, nullptr);
        ASSERT_NE(pr, nullptr);
        EXPECT_EQ(pf->position_x, pr->position_x) << "body " << b.id;
        EXPECT_EQ(pf->position_y, pr->position_y) << "body " << b.id;
        EXPECT_EQ(pf->position_z, pr->position_z) << "body " << b.id;
    }
}

}  // namespace
