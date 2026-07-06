# Contract: `engine_demo::vfx::particle_pool`

Header: `include/engine_demo/vfx/particle.h`
Source: `src/engine_demo/vfx/particle.cpp`
Test: `tests/engine_demo/test_particle_pool.cpp`

## Declarations (contract surface)

```cpp
namespace engine_demo::vfx {

enum class vfx_status : std::uint8_t {
    ok = 0,
    pool_exhausted,
    invalid_argument,
};

struct particle {
    double remaining_lifetime_seconds{0.0};
    float position[3]{};
    float velocity[3]{};
    float color[4]{};
    float size{1.0f};
};

struct spawn_params {
    float position[3]{};
    float velocity[3]{};
    double lifetime_seconds{1.0};
    float color[4]{1.0f, 1.0f, 1.0f, 1.0f};
    float size{1.0f};
};

struct emit_result {
    std::uint32_t spawned{0};
    vfx_status status{vfx_status::ok};
};

class [[nodiscard]] particle_pool {
   public:
    particle_pool(allocator& alloc, std::size_t capacity) noexcept;

    particle_pool(const particle_pool&) = delete;
    particle_pool& operator=(const particle_pool&) = delete;
    particle_pool(particle_pool&&) noexcept = default;
    particle_pool& operator=(particle_pool&&) noexcept = default;

    // Spawns as many of `params[0..count)` as free capacity allows. Never allocates.
    [[nodiscard]] emit_result try_spawn(eastl::span<const spawn_params> params) noexcept;

    // Ages every live particle by `dt_seconds` and retires (swap-removes) any whose
    // remaining lifetime reaches <= 0. Never allocates. Does not apply forces —
    // callers (emitter::tick) integrate velocity into position and apply forces
    // before/via this call as documented in contracts/emitter.md.
    void age_and_retire(double dt_seconds) noexcept;

    [[nodiscard]] eastl::span<particle> live_particles() noexcept;
    [[nodiscard]] eastl::span<const particle> live_particles() const noexcept;

    [[nodiscard]] std::size_t capacity() const noexcept;
    [[nodiscard]] std::size_t live_count() const noexcept;
    [[nodiscard]] std::size_t free_count() const noexcept;
};

}  // namespace engine_demo::vfx
```

## Behavior / pre- and post-conditions

- **Construction**: `capacity == 0` is accepted (an always-exhausted, always-empty pool);
  no allocation occurs beyond the one-time reserve inside the constructor.
- **`try_spawn`**: Spawns `min(params.size(), free_count())` particles, in the order given by
  `params`, writing each into the next free dense-array slot. Returns
  `{spawned, vfx_status::ok}` when `spawned == params.size()`, otherwise
  `{spawned, vfx_status::pool_exhausted}`. Never allocates; never partially-writes a single
  particle's fields (whole `particle` written atomically from one `spawn_params` entry).
  Existing live particles are never modified by `try_spawn`.
- **`age_and_retire`**: For every live particle, `remaining_lifetime_seconds -= dt_seconds`.
  Any particle whose `remaining_lifetime_seconds <= 0.0` after decrement is retired via
  swap-remove with the last live particle (§ research.md 1), without changing the relative
  scan order of the particles that remain live. `dt_seconds == 0.0` is a no-op (Edge Case:
  paused frame) — no particle state changes and no particle is retired.
- **`live_particles`**: Returns a span over exactly `[0, live_count())` of the internal dense
  array; the span is invalidated by the next `try_spawn` or `age_and_retire` call.
- **Determinism**: For a fixed sequence of `try_spawn`/`age_and_retire` calls with identical
  arguments, the resulting `live_particles()` contents are byte-identical across runs.

## Test obligations (Article 7)

- Happy path: spawn N ≤ capacity, verify `live_count() == N` and each particle's fields match
  the corresponding `spawn_params`.
- Edge case: spawn requesting more than free capacity returns
  `vfx_status::pool_exhausted` with `spawned == free_count()` (including the `free_count() == 0`
  case), and no existing particle is modified.
- Edge case: `age_and_retire(0.0)` changes nothing.
- Happy path: a particle whose lifetime exactly reaches `0.0` on a given call is retired
  during that same call.
- Determinism: two pools built and driven through an identical call sequence produce
  identical `live_particles()` snapshots (`EXPECT_EQ`, not `EXPECT_NEAR`).
