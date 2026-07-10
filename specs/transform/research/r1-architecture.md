# R1 — Rust/Bevy ECS Architecture: Best Practices

Research note for the C++→Rust transformation pipeline. Sources fetched live 2026-07-10 from bevy.org and docs.rs (Bevy 0.19 docs were current at fetch time). No code in this note per prompt constraints.

## 1. Components vs Resources vs Events — decision rules

| Data shape | Use | Rule of thumb |
|---|---|---|
| Per-entity state (position, velocity, health, team) | **Component** | If two things in the world could each have their own copy, it is a component. |
| Globally unique state (elapsed time, RNG, score table, asset handles, config) | **Resource** | If there is exactly one logical instance for the whole world, it is a resource. [1][3] |
| A thing that *happens* at a moment (collision, powerup collected, match ended) | **Event** (buffered message or observer-triggered) | If it is a fact about a moment rather than ongoing state, it is an event. [4] |

- Prefer many small components over one large struct; e.g. a `Name` split out of `Person` so any entity kind can be named. **Failure mode prevented:** god-structs that force every system to depend on all fields, killing reuse and parallelism (systems conflict on the whole struct even when they touch one field). [1]
- Do not smuggle per-entity data into a resource keyed by entity ID (a hand-rolled map). **Failure mode prevented:** you lose query filtering, change detection, and automatic cleanup on despawn — the classic dangling-ID bug ECS exists to remove.
- Do not model events as boolean "flag" components/resources that a consumer must remember to reset. **Failure mode prevented:** missed or double-processed happenings when frame timing shifts; buffered events are drained/expired by the framework instead. [4]
- Newer Bevy splits "events" into buffered messages (many writers/readers, read next frame or later the same frame) and observer-triggered `Event`s (run observers immediately at trigger time). Use observers for immediate, targeted reactions (often entity-targeted); use buffered messages for fan-out decoupling between systems. **Failure mode prevented:** one-frame lag bugs when an immediate reaction was needed, or hidden synchronous call chains when decoupling was needed. [4]

## 2. System ordering and schedules

- Systems run **in parallel by default whenever their data access allows**; ordering between two systems is *unspecified* unless you declare it. Use explicit chaining/ordering (`.chain()`, `before`/`after`, system sets) only where a real data dependency exists, and leave independent systems unordered so the scheduler can parallelize. [1]
- **Failure mode prevented (by declaring order):** nondeterministic frame-to-frame behavior — e.g. a rename system racing a greet system produces different output per run. The official quick-start demonstrates exactly this hazard and fixes it with chaining. [1]
- **Failure mode prevented (by NOT over-ordering):** a fully serialized schedule that forfeits multi-core throughput.
- **Update vs FixedUpdate:** `Update` runs once per render frame (variable rate). `FixedUpdate` runs at a fixed rate, possibly zero or multiple times per frame, and is the documented home for physics, AI, networking, and game rules. [2]
  - Put simulation/gameplay logic in `FixedUpdate`; put input sampling, camera, animation/visual interpolation, and UI in `Update`.
  - **Failure mode prevented:** frame-rate-dependent physics (fast machines simulate "faster"), tunneling at low FPS, and non-reproducible simulations — fatal for our determinism requirement (constitution Article 5 analog).
  - Corollary: systems reading input inside `FixedUpdate` can miss or double-read per-frame input state; sample in `Update` (or `RunFixedMainLoop` pre-fixed sets) and forward via messages/resources. [2]
- Bevy also provides `FixedPreUpdate`/`FixedPostUpdate` and `PreUpdate`/`PostUpdate` for engine-vs-game phase separation; keep gameplay out of the Pre/Post phases reserved for engine bookkeeping. [2]

## 3. Plugin decomposition

- All engine features — and games themselves — are plugins: units of code that register systems, resources, and events against an `App`. Bevy's own renderer, UI, windowing, etc. ship as plugins in `DefaultPlugins`; `MinimalPlugins` exists for headless/minimal use. [3]
- Practice: one plugin per subsystem (e.g. physics, scoring, vfx, input), owning *all* of that subsystem's registrations, initialized resources, and schedule placement. The plugin's build step is the same builder API as `main`, so logic moves wholesale. [3]
- **Failure modes prevented:**
  - a monolithic `main` where registration order and ownership are invisible — plugins make each subsystem's ECS surface reviewable in one place;
  - untestable features — a plugin can be added to a minimal test `App` (headless, `MinimalPlugins`) without dragging in rendering;
  - feature coupling — headless server builds simply omit render/window plugins rather than `#[cfg]`-riddled code. [3]
- Keep cross-plugin contracts to: public components, events/messages, resources, and named system sets. **Failure mode prevented:** plugins reaching into each other's private systems, recreating tight coupling ECS was meant to remove.

## 4. When NOT to use ECS

ECS is "conceptually a lightweight in-memory database" [5]; it earns its cost when there are *many similar things* processed uniformly. Avoid it (use plain Rust modules/structs, or a resource wrapping ordinary data) when:

- **Singleton subsystems** — score tracker, RNG, frame-budget monitor: one instance, no queries needed → resource holding a plain struct, unit-testable without a `World`.
- **Deep hierarchical or graph data with invariants** (constraint solvers' internal work buffers, spatial acceleration structures) — the algorithm wants owned contiguous data and total control of iteration order, not archetype iteration. Keep it inside a resource; expose results as components.
- **Strictly sequential pipelines** with one instance of each stage — schedule/parallelism machinery adds ceremony with zero win.
- **Small tools/prototypes** where a struct-of-vecs is clearer than entities+queries.

**Failure modes prevented:** (a) "everything is an entity" designs where invariants that must hold across several components can be violated by any system spawning a partial entity; (b) determinism loss when an order-sensitive algorithm is spread across parallel systems with unspecified query iteration order; (c) needless indirection making single-instance logic harder to unit test.

## 5. Carry-over notes for our transformation

- The C++ engine's `ecs::world` generation-safety and O(1) guarantees map onto Bevy's `Entity` (index+generation) natively — spec should demand *behavioral* guarantees, not re-implementation.
- Deterministic sim ⇒ everything stateful in `FixedUpdate`, explicit ordering on every mutation path, seeded RNG as a resource, and no reliance on query iteration order (sort explicitly where order matters).

## Sources

1. Bevy Quick Start — ECS (components, systems, queries, parallel-by-default, `.chain()` ordering): https://bevy.org/learn/quick-start/getting-started/ecs/ — fetched live.
2. Bevy API docs — `FixedUpdate` schedule (fixed-rate gameplay: physics/AI/networking/rules; `Update` for per-frame): https://docs.rs/bevy/latest/bevy/app/struct.FixedUpdate.html (Bevy 0.19) — fetched live.
3. Bevy Quick Start — Plugins & Resources (plugin modularity, `DefaultPlugins`/`MinimalPlugins`, resources as globally unique data): https://bevy.org/learn/quick-start/getting-started/plugins/ and https://bevy.org/learn/quick-start/getting-started/resources/ — fetched live.
4. Bevy API docs — `Event` trait (observer triggers, immediate execution, entity events): https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html (Bevy 0.19) — fetched live. Buffered-message vs observer split per 0.16+ evolution [from training knowledge, cross-checked against the 0.19 trait docs].
5. Unofficial Bevy Cheat Book — ECS intro ("in-memory database" framing): https://bevy-cheatbook.github.io/programming/ecs-intro.html — fetched live; site self-identifies as no longer maintained, used only for the conceptual framing.
