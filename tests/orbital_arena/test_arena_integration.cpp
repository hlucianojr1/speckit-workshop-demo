// GoogleTest for orbital_arena::arena (T001 smoke test).
//
// Replaced by the real integration tests in T016; for now this proves the
// orbital_arena library + test target are wired into CTest and actually run.

#include "orbital_arena/arena.h"

#include <gtest/gtest.h>

namespace {

TEST(arena_integration, scaffolding_smoke_status_enum_is_usable) {
    EXPECT_EQ(orbital_arena::arena_status::ok, orbital_arena::arena_status::ok);
    EXPECT_NE(orbital_arena::arena_status::ok, orbital_arena::arena_status::wrong_state);
    EXPECT_EQ(orbital_arena::kMaxPlayers, 4);
    EXPECT_EQ(orbital_arena::kWinScore, 100u);
}

}  // namespace
