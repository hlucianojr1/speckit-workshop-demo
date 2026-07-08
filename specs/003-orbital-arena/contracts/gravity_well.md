# Contract: gravity_well

**Header**: `include/orbital_arena/gravity_well.h` | **Impl**: `src/orbital_arena/gravity_well.cpp`
**Tests**: `tests/orbital_arena/test_gravity_well.cpp`
**Satisfies**: FR-001..FR-004 (partial: capture predicate only), Articles 1, 2, 5, 6, 9.

## Types

```cpp
namespace orbital_arena {

struct gravity_well {
    float position[2]{};
    float velocity[2]{};
    float strength{0.0f};          // normalized 0..1 (player input)
    float influence_radius{2.0f};
    float capture_radius{0.15f};
    bool  active{true};
};

inline constexpr float kMaxWellSpeed    = 2.0f;   // units/s, identical for all (FR-002)
inline constexpr float kMaxPullAccel    = 40.0f;  // accel at min clamped distance, strength=1
inline constexpr float kMinDistanceSq   = 0.01f;  // inverse-square clamp (no div-by-zero)

}  // namespace orbital_arena
```

## Functions

```cpp
// Radial acceleration (units/s^2) exerted by `well` on a point at `pos`.
// - Zero if !well.active, strength == 0, or distance > influence_radius (FR-003).
// - Magnitude: strength * kMaxPullAccel * (kMinDistanceSq / max(dist_sq, kMinDistanceSq)),
//   i.e. inverse-square with clamped minimum distance; direction: toward well.position.
// - Pure function: no allocation, no rng, no global state (Articles 5, 6).
void radial_acceleration(const gravity_well& well,
                         const float pos[2],
                         float out_accel[2]) noexcept;

// True iff `pos` is strictly inside well.capture_radius (squared-distance compare,
// no sqrt) AND well.active. Boundary (exactly equal) is NOT captured.
[[nodiscard]] bool is_within_capture(const gravity_well& well,
                                     const float pos[2]) noexcept;

// Advances the well one fixed step: velocity := clamp(steer * kMaxWellSpeed),
// position += velocity * dt, then position clamped to [-half_extent, +half_extent]^2
// (US1 scenario 4). Inactive wells do not move. dt is the fixed step (seconds).
void step_well(gravity_well& well,
               const float steer[2],
               float strength_input,      // clamped 0..1 before assignment
               float half_extent,
               double dt) noexcept;

}
```

## Behavioral guarantees

- Identical output for identical inputs — no hidden state (Article 5 / 10).
- No player-index parameter exists anywhere in this API (Article 9 structural).
- `strength = 0` ⇒ exactly zero acceleration (US1 scenario 3, exact — not epsilon).

## Test obligations (write first — Article 7)

1. Particle at known distance/strength → expected magnitude and direction (training gate).
2. Zero at `strength == 0`; zero beyond `influence_radius`; clamped at near-zero distance.
3. `is_within_capture`: inside true, outside false, exact-boundary false.
4. `step_well` clamps speed and arena bounds; inactive well never moves.
5. Symmetry: mirrored inputs produce mirrored outputs (Article 9 evidence).
