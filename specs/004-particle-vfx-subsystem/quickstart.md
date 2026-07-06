# Quickstart: Particle VFX Subsystem

This walks through constructing an emitter, attaching forces, emitting a burst, and ticking
it — the minimum sequence an integrator (e.g. `apps/sandbox`) needs.

```cpp
#include <engine_demo/allocator.h>
#include <engine_demo/vfx/particle.h>
#include <engine_demo/vfx/force_applicator.h>
#include <engine_demo/vfx/emitter.h>

using namespace engine_demo;
using namespace engine_demo::vfx;

// 1. Backing storage — one arena-backed allocator, as with every other subsystem.
alignas(std::max_align_t) std::byte arena[1 << 16];
allocator alloc{arena, sizeof(arena)};

// 2. Fixed-capacity pool, sized once.
particle_pool pool{alloc, /*capacity=*/500};

// 3. Emitter configuration: a cone burst of sparks with a deterministic seed.
emitter_config cfg{};
cfg.shape = cone_shape{.origin = {0.0f, 0.0f, 0.0f},
                       .direction = {0.0f, 1.0f, 0.0f},
                       .half_angle_radians = 0.35f};
cfg.seed = 42;
cfg.speed_min = 2.0f;
cfg.speed_max = 5.0f;
cfg.lifetime_min_seconds = 0.4;
cfg.lifetime_max_seconds = 0.9;
cfg.color[0] = 1.0f;  // opaque orange-ish spark
cfg.size_min = 0.05f;
cfg.size_max = 0.12f;

emitter spark_emitter{alloc, pool, cfg};
(void)spark_emitter.add_force(gravity_force{});
(void)spark_emitter.add_force(turbulence_force{.strength = 0.3f, .frequency = 4.0f});

// 4. Trigger a burst (e.g. on a physics collision event). Independent of the physics
//    solver — no engine_demo::physics or engine_demo::ecs type is touched.
emit_result result = spark_emitter.try_emit(20);
if (result.status == vfx_status::pool_exhausted) {
    // Handle gracefully — no exception is thrown (Article 1).
}

// 5. Tick once per fixed simulation step (same cadence as engine_demo::sim::game_loop),
//    independently of when/whether the physics solver ticks this frame.
constexpr double kFixedStep = 1.0 / 60.0;
spark_emitter.tick(kFixedStep);

// 6. Render-only consumption (rendering itself is out of scope for this feature).
for (const particle& p : spark_emitter.pool().live_particles()) {
    // draw_particle(p.position, p.color, p.size);
}
```

## Frame budget integration (research.md §6)

```cpp
frame_budget vfx_budget{alloc};

// In the app's per-frame update, alongside the existing physics-step timing pattern:
auto const t0 = /* high_resolution_clock::now() */;
spark_emitter.tick(kFixedStep);
auto const t1 = /* high_resolution_clock::now() */;
vfx_budget.record_sample(/* milliseconds between t0 and t1 */);
```

## Determinism check (User Story 3 / SC-003)

```cpp
allocator alloc_a{arena_a, sizeof(arena_a)};
allocator alloc_b{arena_b, sizeof(arena_b)};
particle_pool pool_a{alloc_a, 500};
particle_pool pool_b{alloc_b, 500};
emitter emitter_a{alloc_a, pool_a, cfg};  // same cfg, same seed
emitter emitter_b{alloc_b, pool_b, cfg};

for (int frame = 0; frame < 1000; ++frame) {
    if (frame == 0) {
        emitter_a.try_emit(20);
        emitter_b.try_emit(20);
    }
    emitter_a.tick(kFixedStep);
    emitter_b.tick(kFixedStep);
}

// pool_a.live_particles() and pool_b.live_particles() must be byte-identical.
```
