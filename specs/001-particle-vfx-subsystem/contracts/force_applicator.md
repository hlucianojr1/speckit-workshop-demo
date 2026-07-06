# Contract: `engine_demo::vfx` force applicators

Header: `include/engine_demo/vfx/force_applicator.h`
Source: `src/engine_demo/vfx/force_applicator.cpp`
Test: `tests/engine_demo/test_force_applicator.cpp`

## Declarations (contract surface)

```cpp
namespace engine_demo::vfx {

struct gravity_force {
    float acceleration[3]{0.0f, -9.81f, 0.0f};
};

struct wind_force {
    float acceleration[3]{};
};

struct turbulence_force {
    float strength{0.0f};
    float frequency{1.0f};
};

using force_applicator = eastl::variant<gravity_force, wind_force, turbulence_force>;

// Returns this applicator's acceleration contribution (m/s^2) for one particle on one
// tick. `rng` is consumed only by `turbulence_force`, in the fixed order documented in
// research.md §4 — callers MUST invoke this once per live particle per tick, in stable
// dense-array index order, to preserve determinism (Article 5).
void apply_force(const force_applicator& force,
                  engine_demo::sim::rng& rng,
                  float out_acceleration[3]) noexcept;

}  // namespace engine_demo::vfx
```

## Behavior / pre- and post-conditions

- **`gravity_force` / `wind_force`**: `apply_force` writes `acceleration` verbatim into
  `out_acceleration`; `rng` is never consumed.
- **`turbulence_force`**: `apply_force` draws exactly three values from `rng` via
  `next_double_unit()` (one per axis), maps each from `[0, 1)` to `[-1, 1)`, scales by
  `strength`, and writes the result into `out_acceleration`. `frequency` is reserved for a
  future time-varying noise function; for v1 it is stored but does not change the per-call
  draw count (kept as a documented, tested constant pass-through to avoid a breaking
  signature change later).
- **Composition**: Callers (see `contracts/emitter.md`) sum every attached applicator's
  `out_acceleration` before integrating it into a particle's velocity for that tick
  (`velocity += sum(acceleration) * dt`), per the additive-composition decision in
  research.md §3.
- **No allocation**: `apply_force` never allocates; `force_applicator` is stored by value in
  a pre-reserved `eastl::vector` on the owning `emitter`.

## Test obligations (Article 7)

- Happy path: `gravity_force{ {0, -9.81f, 0} }` produces `out_acceleration == {0, -9.81f, 0}`
  and does not advance `rng` (verified by comparing `rng.next_u32()` before/after against an
  independently-seeded reference `rng`).
- Happy path: `wind_force` behaves identically to `gravity_force` for pass-through semantics
  with a different configured vector.
- Happy path: `turbulence_force` output is always within `[-strength, strength]` per axis.
- Edge case: `turbulence_force{ strength: 0.0f }` always yields `{0, 0, 0}` regardless of
  `rng` state, but still consumes exactly 3 draws from `rng` (consumption count must stay
  constant so downstream determinism does not depend on `strength`'s value).
- Determinism: two identically-seeded `rng` instances fed into `apply_force` for the same
  `turbulence_force` config produce identical `out_acceleration` (`EXPECT_EQ`).
