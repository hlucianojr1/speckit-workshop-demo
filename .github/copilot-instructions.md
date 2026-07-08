# Project: engine_demo (C++20 Game Engine — Spec-Kit Workshop)

## Build Commands

- Configure: `cmake --preset default-debug` (requires `VCPKG_ROOT`)
- Build: `cmake --build --preset default-debug`
- Test: `ctest --preset default-debug --output-on-failure`
- Skip sandbox/raylib: add `-DENGINE_DEMO_BUILD_SANDBOX=OFF` to configure

## Architecture

- Language: C++20 with `-fno-exceptions -fno-rtti`
- Containers: EASTL (never `std::` containers outside labeled interop boundaries)
- Dependencies: vcpkg manifest (`vcpkg.json`) — EASTL, GoogleTest, fmt, nlohmann-json, raylib
- Library: `include/engine_demo/` + `src/engine_demo/` (allocator, ecs, physics, sim, frame_budget, vfx)
- Demo app: `apps/sandbox/` (raylib, optional)
- Tests: GoogleTest, wired via CTest in `tests/engine_demo/`

## Spec-Kit Constitution

Ground truth: `specs/constitution.md` (machine copy: `.specify/memory/constitution.md`).
Every code change must satisfy all 8 articles.

## Key Constraints

- No heap allocation in inner loops (Article 6)
- Every public function must have a GTest (Article 7)
- All containers take an explicit allocator (Article 4)
- Deterministic sim paths: seeded RNG, `double` accumulators (Article 5)

<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan:
specs/003-orbital-arena/plan.md
<!-- SPECKIT END -->
