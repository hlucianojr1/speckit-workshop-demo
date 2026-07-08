# orbital_arena — game-rules static library (plan.md "CMake Wiring").
# Links engine_demo (transitively: EASTL); zero edits under engine_demo itself.

add_library(orbital_arena STATIC
    arena.cpp
    gravity_well.cpp
    input.cpp
)

target_include_directories(orbital_arena PUBLIC
    ${CMAKE_SOURCE_DIR}/include
)

target_link_libraries(orbital_arena PUBLIC
    engine_demo
)

engine_demo_set_target_options(orbital_arena)
