# Quickstart: Particle VFX Subsystem

## Build & test

```powershell
cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure -R "particle_pool|force_applicator|emitter|vfx"
```

## Minimal usage example

```cpp
#include "engine_demo/allocator.h"
#include "engine_demo/vfx/emitter.h"
#include "engine_demo/vfx/particle.h"

#include <array>

using namespace engine_demo;
using namespace engine_demo::vfx;

std::array<std::byte, 1 << 20> storage{};
allocator alloc{storage.data(), storage.size()};

// 1. Pool: fixed capacity, allocated once.
particle_pool pool{alloc, /*capacity=*/500};

// 2. Emitter: cone shape, seeded RNG, variance ranges.
emitter_config cfg{};
cfg.shape = cone_shape{.origin = {0, 0, 0}, .direction = {0, 1, 0}, .half_angle_radians = 0.4f};
cfg.seed = 12345;
cfg.speed_min = 2.0f;
cfg.speed_max = 5.0f;
cfg.lifetime_min_seconds = 0.5;
cfg.lifetime_max_seconds = 1.5;

emitter spark_emitter{alloc, pool, cfg};
[[maybe_unused]] vfx_status add_status = spark_emitter.add_force(gravity_force{});

// 3. Trigger a burst from a gameplay event (e.g. a collision callback).
emit_result result = spark_emitter.try_emit(64);
if (result.status == vfx_status::pool_exhausted) {
    // Fewer than 64 particles were spawned; pool is at/near capacity.
}

// 4. Advance with the fixed-step game loop, independently of the physics solver.
constexpr double kFixedStep = 1.0 / 60.0;
spark_emitter.tick(kFixedStep);

// 5. Hand live particle state to a rendering step (out of scope for this subsystem).
for (const particle& p : pool.live_particles()) {
    // render(p.position, p.color, p.size);
}
```

## Integration points

- **ECS World**: An `emitter`/`particle_pool` pair may be attached as data referenced by an
  ECS entity (e.g., a component holding an `emitter*` or an index into a world-owned table),
  but the VFX subsystem itself has no dependency on `engine_demo::ecs::world` — it is plain
  data + functions, matching `constraint_solver`'s independence from `world`.
- **Game loop**: Call `emitter::tick(dt_seconds)` once per fixed step produced by
  `engine_demo::sim::game_loop::advance`, using the same `dt_seconds` as the physics step —
  but never invoking `constraint_solver::solve` (User Story 1 / SC-003).
- **Frame budget**: Time the call site the same way `apps/sandbox/scene.cpp` already times
  the physics step: wrap `emitter::tick(...)` with a stopwatch and call
  `frame_budget::record_sample(elapsed_ms)` on the app's existing `frame_budget` instance
  (research.md §6). The VFX subsystem does not take a `frame_budget` dependency itself.
