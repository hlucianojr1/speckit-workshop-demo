# Phase 0 Research: Sandbox VFX Visualization

Six decisions that shape [data-model.md](data-model.md) and [contracts/](contracts/). All
unknowns from the spec are resolved below — no `NEEDS CLARIFICATION` markers remain.

## 1. Repositioning an emitter per burst: `set_shape`, not a new emitter

**Decision**: Add one small library method, `engine_demo::vfx::emitter::set_shape(emitter_shape) noexcept`,
that overwrites `m_cfg.shape` only. Construct exactly one `emitter` in `scene`'s constructor
and reuse it for every right-click burst and every collision spark, calling `set_shape(...)`
immediately before each `try_emit(...)`.

**Rationale**: `emitter`'s constructor reserves `m_forces` (capacity 4) and resizes
`m_scratch` to `pool.capacity()` once (Article 6: real-time, zero allocation in the hot
path). Bursts/sparks happen at arbitrary, per-event world positions, so *something* about
the emitter must change per event. The tempting shortcut — construct a fresh `emitter` at
each cursor/contact position — move-assigns or move-constructs a new `emitter` each event,
and `emitter`'s move members are `= default`, but a *freshly constructed* emitter still runs
its constructor body (`m_forces.reserve(4)`, `m_scratch.resize(pool.capacity())`) once per
click/bounce, i.e. an allocation from the arena on every burst — a direct Article 6
violation hiding behind ordinary-looking code. `set_shape` avoids this: `emitter_shape` is
an `eastl::variant` over three small, non-owning structs (`point_shape`, `cone_shape`,
`sphere_shape`), so assigning a new variant value is a trivial copy, never a heap/arena
allocation.

**Alternatives considered**: (a) One emitter per event, discarded after use — rejected,
reallocates every burst (Article 6). (b) A pool-of-emitters preallocated up front — rejected
as needless complexity; a single emitter's shape can be repointed every call because bursts
are synchronous, single-threaded, and non-overlapping within a frame.

## 2. Shape choice: `sphere_shape` (tiny radius), not `point_shape`

**Decision**: Both the right-click burst and collision sparks use `sphere_shape{center,
radius}` (radius ≈ 0.02–0.03 world units), repositioned via `set_shape` before each
`try_emit`. Neither uses `point_shape`.

**Rationale**: Reading `emitter.cpp`'s `sample_one()` shows `point_shape` always samples
direction as a fixed `(0, 1, 0)` — a point emitter is a directional jet, not a radial burst;
every particle would fly the same way. `sphere_shape` draws a uniformly-distributed
direction (3 `rng` draws) per particle, which is exactly a radial spark/burst look, and its
small radius also gives a touch of spawn-position jitter instead of every particle spawning
from the identical point. This is a direct consequence of the *existing* Feature 001
contract — no changes to `emitter.cpp`'s sampling logic are needed or proposed.

**Alternatives considered**: `cone_shape` — rejected, adds a direction/half-angle the
spec never asked for and looks directional rather than radial; `point_shape` — rejected per
above (degenerate, all-particles-same-direction look).

## 3. VFX random stream isolation (Article 5 + FR-005)

**Decision**: The sandbox's shared `m_vfx_emitter` is constructed once, in `scene`'s
constructor/`rebuild_scene()`, with `emitter_config::seed = m_seed ^ kVfxSeedSalt` — a
constant, compile-time XOR fold of the scene's own seed. `m_rng` (the scene's existing
random stream, used for rope/particle-storm physics) is never read by any VFX code path.

**Rationale**: `emitter` already owns a private `engine_demo::sim::rng` seeded once from
`emitter_config::seed` — Feature 001 built this in for exactly this reason. The trap is
purely at the *call site*: it would be easy to write `cfg.seed = m_rng.next_u32();` when
constructing the emitter, which both consumes a draw from the scene's stream (shifting
every subsequent physics rng draw and changing `state_digest()`) and re-seeds on every
`reseed()` (since `reset_vfx()` runs after `m_rng` is reseeded in `rebuild_scene()`). Folding
`m_seed` directly (not through `m_rng`) keeps VFX reproducible per scene seed without ever
touching the scene's draw sequence.

**Alternatives considered**: A second independently-user-supplied VFX seed — rejected as
unnecessary API surface; XOR-folding the existing scene seed is simpler and still fully
deterministic per seed.

## 4. Resetting VFX state on reseed/switch_scene (edge case)

**Decision**: Add a private `scene::reset_vfx() noexcept` that destroys and placement-new
reconstructs both `m_vfx_pool` and `m_vfx_emitter` in place, called from `rebuild_scene()`
immediately after the existing `reset_solver()` call.

**Rationale**: `engine_demo::vfx::particle_pool` has no `clear()`/`reset()` method (by
design — Feature 001 never needed one). `scene::reset_solver()` already establishes the
exact idiom this codebase uses for "rebuild a member that owns arena-backed containers in
place": `m_solver.~constraint_solver(); new (&m_solver) engine_demo::physics::constraint_solver{m_alloc};`.
Mirroring it for VFX is consistent with existing style and requires no new library API.
Reconstruction happens only on an explicit user action (press `R` to reseed, or switch scene
number keys) — not a per-frame hot path — so the one-time arena reallocation it performs is
the same accepted tradeoff `reset_solver()` already makes, not a new Article 6 concern.

**Alternatives considered**: Force-expire via `age_and_retire(1e9)` — works without any new
code, but leaves `m_vfx_emitter`'s shape/rng-position state stale between scenes for no
benefit over the placement-new approach already established in this file; rejected in favor
of consistency with `reset_solver()`.

## 5. Alpha fade without a new "total lifetime" field

**Decision**: `m_vfx_emitter`'s `emitter_config` uses a single fixed
`lifetime_min_seconds == lifetime_max_seconds == kVfxLifetimeSeconds` (no randomized
lifetime span). The renderer computes fade as
`life_fraction = clamp(remaining_lifetime_seconds / kVfxLifetimeSeconds, 0, 1)` and maps
that directly to alpha.

**Rationale**: `engine_demo::vfx::particle` stores only `remaining_lifetime_seconds` (no
"initial lifetime" field — Feature 001 never needed one, and adding one now would be a
library change this feature's scope excludes). Fixing the emitter's lifetime range to a
single constant makes the denominator known and constant at the sandbox layer, so fade math
needs no new library field.

**Alternatives considered**: Add an `initial_lifetime_seconds` field to `particle` —
rejected, unnecessary library surface growth for a render-only cosmetic need; a fixed
lifetime constant is simpler and sufficient.

## 6. Test strategy: a raylib-free scene test target

**Decision**: Add `tests/engine_demo/test_scene_vfx.cpp`, wired into
`tests/engine_demo/CMakeLists.txt` the same way `test_sandbox_render` already is: an
`engine_demo_add_test` target that compiles `apps/sandbox/scene.cpp` directly (via
`target_sources`) and includes via `${CMAKE_SOURCE_DIR}`, without linking raylib.

**Rationale**: `apps/sandbox/scene.h`/`scene.cpp` include only `EASTL`, `engine_demo/*`,
and standard headers — confirmed no raylib/telemetry dependency — so `scene` is already
unit-testable in isolation. This precedent already exists in this exact file
(`test_sandbox_render` compiles `apps/sandbox/viewport.h` the same way). This lets
`/speckit.tasks` schedule a genuine red→green test for `spawn_vfx_burst`, spark-on-bounce,
and the digest-parity assertion, without standing up a graphical raylib window in CI.

**Alternatives considered**: Manual-only verification (no automated `scene`-level test) —
rejected, violates Article 7 and FR-011 (every new public function needs a GTest); testing
through the full `ea-sandbox` executable — rejected, unnecessarily couples the test to
raylib/windowing.
