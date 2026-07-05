# Contract: `engine_demo::vfx::emitter::set_shape` (library addition)

Header: `include/engine_demo/vfx/emitter.h`
Source: `src/engine_demo/vfx/emitter.cpp`
Test: `tests/engine_demo/test_emitter.cpp` (new case added to the existing file)

This is the only change to the `engine_demo::vfx` library made by Feature 002. Everything
else in `particle.h`, `force_applicator.h`, and `emitter.h` is used exactly as Feature 001
left it.

## Declaration (contract surface)

```cpp
namespace engine_demo::vfx {

class [[nodiscard]] emitter {
   public:
    // ... existing declarations unchanged ...

    // Replaces this emitter's shape configuration in place. Does not touch m_rng, m_pool,
    // m_forces, or m_scratch, and performs no allocation (emitter_shape is a variant of
    // small non-owning structs — assignment is a trivial copy). Safe to call between any
    // two try_emit()/tick() calls, including every frame.
    void set_shape(emitter_shape shape) noexcept;
};

}  // namespace engine_demo::vfx
```

## Behavior / pre- and post-conditions

- **Pre-condition**: none (always callable; `emitter_shape` default-constructs to
  `point_shape{}` if never set).
- **Post-condition**: `m_cfg.shape == shape` (by value); all other emitter state
  (`m_rng`'s internal position, `m_pool` pointer, `m_forces` contents, `m_scratch` capacity)
  is unchanged.
- **Allocation**: none, under any input — required so callers may invoke `set_shape` once
  per burst/spark event without violating Article 6.
- **Determinism**: does not consume `m_rng`; calling `set_shape` any number of times between
  `try_emit` calls does not change the rng draw sequence consumed by subsequent
  `try_emit`/`tick` calls.

## Test obligations (Article 7)

Added to `tests/engine_demo/test_emitter.cpp`:

1. **Happy path**: construct an emitter with `point_shape`, call `set_shape(sphere_shape{...})`,
   then `try_emit(1)`; assert the spawned particle's position falls within the new sphere
   (not at the old point position).
2. **Edge case — no reallocation**: record `engine_demo::allocator::bytes_used()` before and
   after several `set_shape` calls (interleaved with `try_emit`/`tick`); assert `bytes_used()`
   is unchanged by `set_shape` itself (mirrors the existing zero-allocation test pattern used
   for `try_emit`/`tick` in this file).
3. **Edge case — rng sequence unaffected**: construct two identically-seeded emitters; call
   `set_shape` a different number of times on each between identical `try_emit` calls;
   assert both produce identical spawned-particle sequences (determinism unaffected by
   `set_shape` call count).
