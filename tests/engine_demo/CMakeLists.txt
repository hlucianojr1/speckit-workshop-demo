function(engine_demo_add_test test_name)
    add_executable(${test_name} ${ARGN})
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
engine_demo_add_test(test_particle_pool     test_particle_pool.cpp)
engine_demo_add_test(test_force_applicator  test_force_applicator.cpp)
engine_demo_add_test(test_emitter           test_emitter.cpp)
engine_demo_add_test(test_vfx_determinism   test_vfx_determinism.cpp)

# Sandbox renderer coordinate-transform tests.
# viewport.h is pure math (no raylib dependency) — include via project root.
engine_demo_add_test(test_sandbox_render test_sandbox_render.cpp)
target_include_directories(test_sandbox_render PRIVATE
    ${CMAKE_SOURCE_DIR}
)

# Sandbox scene VFX tests (Feature 002). scene.cpp is raylib-free by design, so it's
# compiled directly into this test target — the same pattern as test_sandbox_render above.
engine_demo_add_test(test_scene_vfx test_scene_vfx.cpp ${CMAKE_SOURCE_DIR}/apps/sandbox/scene.cpp)
target_include_directories(test_scene_vfx PRIVATE
    ${CMAKE_SOURCE_DIR}
)
