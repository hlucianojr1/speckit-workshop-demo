---
applyTo: "**/test_*.cpp"
---

## Test File Rules

1. Use GoogleTest (not Catch2 or doctest)
2. Every test constructs its own allocator over a local `std::array<std::byte, N>` buffer —
   never share state between tests
3. Use explicit seeds for any RNG-dependent test
4. Name pattern: `TEST(subsystem_name, behavior_under_test)`
5. Include at least one happy-path and one edge-case test per public function
   (constitution Article 7)
6. Determinism tests assert bit-exact equality (`EXPECT_EQ` on doubles), not `EXPECT_NEAR`,
   when validating replay guarantees (constitution Article 5)
7. Wire new test files into `tests/engine_demo/CMakeLists.txt` via `engine_demo_add_test`
