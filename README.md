# Spec-Kit Workshop Demo — engine_demo

A self-contained workshop repository for **GitHub Copilot Spec-Kit** training, built around
`engine_demo` — a synthetic C++20 game-engine workspace. It is intentionally small and is
**not** production code.

## Start here

The full training material lives at
[docs/speckit-workshop-training.md](docs/speckit-workshop-training.md):

- **Part 0** — Live demo: Spec-Kit drives a Particle VFX Subsystem from problem statement
  to compiled, tested C++ code
- **Part 1** — Foundations: the five-stage flow, HITL gates, context windows, best practices
- **Part 2** — Hands-on practice: Audio Event Bus and Scene Transition features
- **Part 3** — Designing a new game ("Orbital Arena") on the existing foundation
- **Part 4** — Cross-language transformation (C++ → Rust/Bevy)

## What's here

```text
docs/                  # the workshop training document
include/engine_demo/   # public headers
src/engine_demo/       # implementations
tests/engine_demo/     # GoogleTest binaries
apps/sandbox/          # playable 2D physics sandbox (raylib)
cmake/                 # CompilerOptions.cmake, Dependencies.cmake
specs/                 # spec-kit constitution + per-feature specs
.specify/              # spec-kit scaffolding (templates, scripts, workflows)
.github/               # Copilot customization (spec-kit prompts + agents)
third_party/           # vendored or submoduled deps (EASTL pinned)
```

## Build (verify before the demo)

Prerequisites: CMake 3.28+, Ninja, vcpkg (manifest mode), a recent C++20 compiler
(MSVC 19.38+ / GCC 13+ / Clang 17+).

```bash
cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

The build is `-fno-exceptions -fno-rtti` everywhere. EASTL is the only allowed container
library in committed code.

## Run the playable sandbox

The `apps/sandbox/ea-sandbox` binary is the visual entry point — a 2D verlet-rope-plus-
particles scene driven by `engine_demo::physics::constraint_solver`,
`engine_demo::sim::game_loop`, `engine_demo::sim::rng`, and `engine_demo::frame_budget`.

```bash
# Interactive (opens a window):
./build/apps/sandbox/ea-sandbox

# Headless determinism trace:
./build/apps/sandbox/ea-sandbox --headless --seed 42 --frames 600 --out trace.csv
```

To opt out of building the sandbox (e.g., on a runner without raylib system deps):

```bash
cmake --preset default-debug -DENGINE_DEMO_BUILD_SANDBOX=OFF
```

See [apps/sandbox/README.md](apps/sandbox/README.md) for details.

## Constitution

The non-negotiables for any code in this workspace are codified in
[`specs/constitution.md`](specs/constitution.md). Spec-Kit's `/implement` honors them, and
every workshop exercise reviews generated code against the 8 articles.
