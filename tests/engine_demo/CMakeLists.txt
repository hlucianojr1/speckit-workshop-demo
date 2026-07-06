function(engine_demo_add_test test_name)
    add_executable(${test_name} ${ARGN})
    # NOTE: do NOT link GTest::gmock here. No test in this repo uses gmock, and with
    # vcpkg's dynamic (x64-windows) GTest build, linking gmock.dll alongside
    # gtest_main.dll splits GTest's global test registry across DLL boundaries: every
    # test binary then registers ZERO tests but still exits 0, so ctest reports a false
    # "100% tests passed". If a future test genuinely needs gmock, link it on that one
    # target only.
    target_link_libraries(${test_name} PRIVATE
        engine_demo
        GTest::gtest
        GTest::gtest_main
    )
    engine_demo_set_target_options(${test_name})
    add_test(NAME ${test_name} COMMAND ${test_name})
endfunction()

engine_demo_add_test(test_allocator      test_allocator.cpp)
engine_demo_add_test(test_frame_budget   test_frame_budget.cpp)
engine_demo_add_test(test_game_loop      test_game_loop.cpp)
engine_demo_add_test(test_rng            test_rng.cpp)
engine_demo_add_test(test_world          test_world.cpp)
engine_demo_add_test(test_constraint     test_constraint.cpp)
engine_demo_add_test(test_particle_pool  test_particle_pool.cpp)
engine_demo_add_test(test_force_applicator test_force_applicator.cpp)
engine_demo_add_test(test_emitter         test_emitter.cpp)
engine_demo_add_test(test_vfx_determinism test_vfx_determinism.cpp)

# Sandbox renderer coordinate-transform tests.
# viewport.h is pure math (no raylib dependency) — include via project root.
engine_demo_add_test(test_sandbox_render test_sandbox_render.cpp)
target_include_directories(test_sandbox_render PRIVATE
    ${CMAKE_SOURCE_DIR}
)
