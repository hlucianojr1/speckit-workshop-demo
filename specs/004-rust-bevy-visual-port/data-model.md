# Data Model: Rust/Bevy Visual Port

Derived from `specs/transform/{ecs-world,physics-constraint,game-loop,rng,frame-budget}.spec.md`
and `spec.md`'s Key Entities section. Types are named as they will appear in the Rust source;
field types already reflect Article VI's `f64`-for-physics rule.

## Components (attached to Bevy `Entity`)

### `Body`

| Field | Type | Notes |
|---|---|---|
| `position` | `DVec2` (f64) | World-space position; render systems cast to `f32`/`Vec2` at draw time only |
| `velocity` | `DVec2` (f64) | |
| `inverse_mass` | `f64` | `0.0` ⇒ immovable (maps directly to physics-constraint.spec.md §2.1) |

### `Anchor`

Zero-sized marker component on the single central body. Its `Body.inverse_mass` is `0.0`
(enforced at spawn time), so the constraint solver never moves it — this satisfies
FR-004 Scenario 1 and physics-constraint.spec.md's "infinite-mass respect" guarantee without a
separate boolean flag.

### `ConstraintLink`

| Field | Type | Notes |
|---|---|---|
| `a`, `b` | `Entity` | The two connected bodies; order does not matter (matches spec §2.2) |
| `rest_length` | `f64` | Target separation |

Stored as components on their own link entities (not inline in `Body`) so the solver system can
query all links independent of body count, and so `Query::get_many_mut([a, b])` can borrow both
endpoint bodies mutably per link without aliasing the link's own entity.

## Resources (singletons)

### `SimConfig`

| Field | Type | Notes |
|---|---|---|
| `seed` | `u64` | Default 42 (FR-007); full 64 bits passed to `StdRng::seed_from_u64` |
| `run_mode` | `RunMode` | `Windowed` or `Screenshot { warmup_frames: u32, output_path: PathBuf }` |

`RunMode` is a plain closed `enum` (Article VIII — no dynamic dispatch needed for two variants).

### `DeterministicRng`

Wraps a single `rand::rngs::StdRng`, constructed once at startup from `SimConfig.seed`
(rng.spec.md §3 `construct`). Used only during body-placement jitter at startup — this feature
has no other runtime RNG consumer, so "next_u32"/"next_double_unit" map directly to `StdRng`'s
own `next_u32`/`gen::<f64>()` calls; no wrapper methods are added beyond the `Resource` newtype.

### `FrameBudget`

| Field | Type | Notes |
|---|---|---|
| `samples` | `[f64; 64]` (fixed array) | Window size 64 per frame-budget.spec.md §2 default; reserved inline, never heap-allocated |
| `cursor` | `usize` | Next write index, mod 64 |
| `count` | `usize` | Samples recorded so far, capped at 64 |

`record_sample`/`rolling_average` implemented exactly per frame-budget.spec.md §3–4 (warm-up
counted once, zero-samples average is `0.0`, 64-bit accumulation).

### `ScreenshotState`

| Field | Type | Notes |
|---|---|---|
| `fixed_ticks_elapsed` | `u32` | Incremented once per `FixedUpdate` |
| `triggered` | `bool` | Set once the one-shot screenshot system has fired, to prevent re-triggering |

Only inserted when `SimConfig.run_mode` is `Screenshot`.

## Fixed-step loop

No custom accumulator type is written (game-loop.spec.md's `advance`/`accumulator_seconds`
operations map directly onto Bevy's built-in `Time::<Fixed>` + `FixedUpdate` schedule, which
already implements the identical accumulator algorithm — `fixed_step_seconds` = `1.0/60.0` via
`Time::<Fixed>::from_hz(60.0)`, `max_substeps` = Bevy's own `MaxDeltaTime`-driven substep cap).
This is a second "framework already delivers this guarantee" case (alongside R6's `Entity: Ord`
and the ECS-world generational-safety point in plan.md's Summary) — documented here rather than
re-implemented, consistent with the rust-guidelines principle of not re-deriving what ownership
(or, here, the host framework) already provides.

## Zero-allocation design (Article IV / FR-014)

- Body/link entities are all spawned once during `Startup` (fixed count, no runtime spawn/despawn
  — Article V is N/A for this feature, see plan.md Constitution Check).
- `FrameBudget.samples` is a fixed-size array (`[f64; 64]`), not a `Vec` — no `with_capacity`
  needed since there is no growth to reserve against.
- The constraint solver's per-solve sorted-order buffer (R6) is sized once via
  `Vec::with_capacity(link_count)` at `Startup` and `clear()`+repopulated every `FixedUpdate`
  call (capacity never exceeded since link count is fixed after startup).
- `CountingAllocator` (research.md R4) is the mechanism that proves this empirically in
  `tests/zero_alloc.rs`.

## Simulation State Snapshot (test-only type, `spec.md` Key Entities)

### `SimulationSnapshot`

| Field | Type | Notes |
|---|---|---|
| `seed` | `u64` | Seed the run was constructed with |
| `tick` | `u32` | Fixed-tick count at capture time |
| `bodies` | `Vec<(f64, f64, f64, f64)>` | `(position.x, position.y, velocity.x, velocity.y)` per body, ordered by a stable spawn-order index (see research.md R5) — NOT by `Entity`, since `Entity` indices are an implementation detail of spawn order that is already stable run-to-run for this feature, but sorting explicitly by a game-assigned index keeps the snapshot comparison robust to future incidental spawn-order changes |

Built only inside `tests/determinism.rs` via a small helper in `src/lib.rs` (`fn
capture_snapshot(app: &App) -> SimulationSnapshot`), not a runtime resource used by the app
itself.
