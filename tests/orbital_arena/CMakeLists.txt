# orbital_arena tests — GoogleTest targets wired into CTest.

function(orbital_arena_add_test test_name)
    add_executable(${test_name} ${ARGN})
    # NOTE: do NOT link GTest::gmock here. With vcpkg's dynamic (x64-windows) GTest
    # build, linking gmock.dll alongside gtest_main.dll splits GTest's global test
    # registry across DLL boundaries: the binary registers ZERO tests but exits 0,
    # so ctest reports a false "100% tests passed". Same rule as engine_demo_add_test.
    target_link_libraries(${test_name} PRIVATE
        orbital_arena
        GTest::gtest
        GTest::gtest_main
    )
    engine_demo_set_target_options(${test_name})
    add_test(NAME ${test_name} COMMAND ${test_name})
endfunction()

orbital_arena_add_test(test_arena_integration test_arena_integration.cpp)
orbital_arena_add_test(test_input             test_input.cpp)
orbital_arena_add_test(test_gravity_well      test_gravity_well.cpp)
orbital_arena_add_test(test_scoring           test_scoring.cpp)
