# Feature Specification: Particle VFX Subsystem

**Feature Branch**: `004-particle-vfx-subsystem`
**Created**: 2026-07-06
**Status**: Draft
**Input**: User description: "Create a specification for a Particle VFX Subsystem for engine_demo. Context: C++20 game engine with EASTL containers, no exceptions, no RTTI; see specs/constitution.md for binding constraints (articles 1-8); the subsystem provides visual effects (sparks, dust, explosions) for the physics sandbox; must integrate with the existing ECS World (ecs/world.h) and fixed-step game loop (sim/game_loop.h); particles are purely visual and do NOT participate in physics constraint solving. Requirements: pooled particle storage with fixed capacity set at construction; multiple emitter types (point, cone, sphere) as a tagged enum; per-particle position/velocity/lifetime/color/size; composable force applicators (gravity, wind, turbulence); deterministic spawning from seeded RNG for replay consistency; zero allocation in the update/emit hot path (Article 6); interop with the existing frame budget system for timing. Acceptance Criteria: pool exhaustion returns a status enum (not exception); emitter can be ticked independently of the physics solver; 500 simultaneous particles maintain < 2ms per frame at 60 FPS; all public functions have GTest coverage; deterministic: same seed + same frame sequence = identical particle state."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Emit a Visual Effect Burst (Priority: P1)

As an engine integrator (e.g. the sandbox scene or a gameplay system), I want to trigger a burst of visual particles (sparks, dust, explosion debris) at a specific location so that game events read as more impactful to the player, without those particles affecting physics or gameplay state.

**Why this priority**: This is the minimum viable capability — without the ability to spawn and see particles, no other requirement (composability, determinism, performance) has anything to act on. It is the demo-able MVP.

**Independent Test**: Construct an emitter with a fixed particle-pool capacity, call an emit operation for a burst of N particles, and step the simulation forward. Particles appear with the requested initial position/velocity/lifetime/color/size, age each tick, and disappear when their lifetime expires — independently verifiable via unit tests with no rendering required.

**Acceptance Scenarios**:

1. **Given** a newly constructed emitter with pool capacity of 64, **When** a burst of 20 particles is emitted, **Then** 20 particles become active with initial state matching the emitter configuration.
2. **Given** an active particle with a known lifetime, **When** the emitter is ticked past that lifetime, **Then** the particle is retired and its pool slot becomes available for reuse.
3. **Given** an emitter with pool capacity fully exhausted, **When** another emit is requested, **Then** the call returns a `pool_exhausted` status and no particle state is corrupted.

---

### User Story 2 - Shape Particle Motion with Composable Forces (Priority: P2)

As an engine integrator, I want to attach one or more force applicators (gravity, wind, turbulence) to an emitter so that particle motion looks physically plausible (falling sparks, drifting dust, chaotic explosion debris) without writing custom per-effect motion code.

**Why this priority**: Builds directly on User Story 1's emit/tick mechanics. Without composable forces, all particles would travel in straight lines, which is visually unconvincing for the target effects (sparks arc, dust drifts, explosions scatter).

**Independent Test**: Configure an emitter with a combination of forces (e.g. gravity + wind), emit particles, and step the simulation. Assert that particle velocity/position evolve according to the sum of the configured forces, independent of any other emitter's configuration.

**Acceptance Scenarios**:

1. **Given** an emitter with only a gravity force attached, **When** particles are emitted and ticked for several frames, **Then** particle vertical velocity changes at the configured gravity rate.
2. **Given** an emitter with both gravity and wind forces attached, **When** ticked, **Then** the resulting particle acceleration equals the vector sum of both forces.
3. **Given** an emitter with a turbulence force seeded deterministically, **When** the same seed and frame sequence are replayed, **Then** the turbulence contribution is bit-for-bit identical across runs.

---

### User Story 3 - Replay-Safe, Real-Time Particle Simulation at Scale (Priority: P3)

As an engine integrator working on a real-time sandbox, I want particle emission and ticking to be deterministic and to stay within a strict per-frame time budget at realistic particle counts, so that the subsystem is safe to use in replay/telemetry-sensitive contexts and never causes a frame-budget overrun.

**Why this priority**: This formalizes the non-functional guarantees (determinism, real-time performance) that make the subsystem trustworthy for production use, building on the emit/force mechanics already covered by P1/P2. It is validated last because it depends on the emit and force behavior already being correct.

**Independent Test**: Run 500 simultaneous particles across gravity/wind/turbulence forces for a fixed number of frames from a known seed twice, independently, and diff the resulting particle state — it must match exactly. Separately, measure per-frame update time at 500 particles and confirm it stays under budget.

**Acceptance Scenarios**:

1. **Given** an emitter seeded with a fixed value, **When** the same sequence of emit/tick calls is replayed in a second run, **Then** every particle's position, velocity, lifetime, color, and size match exactly between the two runs.
2. **Given** 500 simultaneous active particles with composed forces, **When** a single tick is measured, **Then** the tick completes in under 2 milliseconds.
3. **Given** an emitter ticking every frame, **When** integrated with the existing frame budget system, **Then** the emitter's per-tick cost is recorded as a sample the frame budget can report on.

---

### Edge Cases

- What happens when `emit()` is requested for more particles than remain free in the pool? (Partial emission is out of scope; the whole request is rejected with a status enum — see FR-006.)
- How does the system handle a particle whose lifetime is exactly zero or negative at emit time? (Treated as already-expired; it is never made active — see FR-013.)
- What happens when an emitter has zero forces attached? (Particles move by velocity integration alone; this is a valid, common configuration.)
- What happens when multiple emitters share the same underlying allocator/pool memory region? (Out of scope — each emitter owns its own independently sized pool, allocated once at construction.)
- How does the system behave when ticked with a zero or negative delta time? (Zero delta performs no aging/integration and is a no-op; negative delta is a caller programming error guarded by an assertion in debug builds, not a runtime status.)
- What happens to a particle exactly at the moment its lifetime reaches zero? (It is retired that same tick, before rendering would occur, so it never renders with an invalid/negative remaining lifetime.)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide a fixed-capacity particle pool per emitter, with capacity specified at emitter construction time and never resized afterward.
- **FR-002**: The system MUST support at least three emitter shapes — point, cone, and sphere — selected via a tagged enum (no runtime type identification).
- **FR-003**: Each particle MUST track, at minimum: position, velocity, remaining lifetime, color, and size.
- **FR-004**: The system MUST support emitting a burst of one or more particles in a single call, with per-particle initial state derived from the emitter's shape and configuration.
- **FR-005**: The system MUST allow an emitter to be advanced ("ticked") independently of the physics constraint solver — no dependency on physics solver state or call order.
- **FR-006**: When an emit request cannot be fully satisfied because the pool has insufficient free slots, the system MUST reject the request and return a `pool_exhausted` status without partially emitting particles or corrupting existing particle state.
- **FR-007**: The system MUST support attaching zero or more composable force applicators (at minimum: gravity, wind, turbulence) to an emitter, whose effects combine additively on active particles each tick.
- **FR-008**: The system MUST allow force applicators to be added/configured independently per emitter — one emitter's forces MUST NOT affect another emitter's particles.
- **FR-009**: Particle spawning (initial position/velocity variation within a shape, and any stochastic force behavior such as turbulence) MUST be driven by an explicitly seeded, deterministic random source.
- **FR-010**: Given the same seed and the same sequence of emit/tick calls, the system MUST produce bit-for-bit identical particle state across separate runs.
- **FR-011**: The system MUST perform zero heap/arena allocation during emit and tick operations after the emitter (and its pool) has been constructed.
- **FR-012**: Particles MUST NOT participate in the physics constraint solver in any way (no constraint registration, no collision response, no entity creation in the ECS world).
- **FR-013**: A particle emitted with zero or negative remaining lifetime MUST NOT become active; the emit call still reports success for the overall request but that individual particle is immediately retired/skipped.
- **FR-014**: When a particle's remaining lifetime reaches zero or below during a tick, the system MUST retire it that same tick and return its slot to the free pool for reuse in a subsequent emit.
- **FR-015**: The system MUST expose a way to record each tick's execution cost as a timing sample compatible with the existing frame budget system.
- **FR-016**: All emitter and pool operations that can fail MUST report failure via a status enum; the system MUST NOT use exceptions for any fallible operation (per Article 1).
- **FR-017**: Every publicly exposed function MUST have at least one automated test covering both a happy-path and a relevant edge case (per Article 7).

### Key Entities

- **Particle**: A single visual-only unit of the effect. Attributes: position, velocity, remaining lifetime, color, size. No identity beyond its pool slot; not represented as an ECS entity.
- **Particle Pool**: Fixed-capacity backing storage for particles owned by one emitter, allocated once at construction; tracks which slots are active vs. free for reuse.
- **Emitter**: Owns a particle pool, an emitter shape (point/cone/sphere), emission configuration (rate, initial velocity/lifetime/color/size ranges), a seeded deterministic random source, and zero or more attached force applicators. Exposes emit and tick operations.
- **Emitter Shape**: A tagged enum describing the geometric region/pattern used to compute each new particle's initial position and velocity (point, cone, sphere).
- **Force Applicator**: A composable, named behavior (gravity, wind, turbulence) that contributes an acceleration/velocity change to active particles each tick; multiple applicators on one emitter combine additively.
- **Emit Status**: A status enum result distinguishing success from failure conditions (e.g., pool exhaustion) for emit operations.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A burst of particles requested from an emitter appears as active, independently verifiable state (position/velocity/lifetime/color/size) within the same tick it was requested, with zero particles lost or corrupted.
- **SC-002**: An emitter holding 500 simultaneously active particles, with composed forces applied, completes a full tick in under 2 milliseconds, preserving the engine's 60 FPS (16.67 ms) frame budget with ample headroom for other subsystems.
- **SC-003**: Replaying an identical seed and identical sequence of emit/tick calls produces identical particle state 100% of the time, across at least 1,000 consecutive frames.
- **SC-004**: Requesting more particles than remain free in the pool never crashes, never corrupts existing particle state, and is reported back to the caller as a distinguishable failure outcome every time it occurs.
- **SC-005**: 100% of publicly exposed functions in the subsystem have at least one passing automated test exercising a normal case and an edge/failure case.
- **SC-006**: The subsystem can run for an extended session (equivalent to 10,000+ frames at 60 FPS) without any heap/arena allocation occurring after emitter construction.

## Assumptions

- Rendering of particles (draw calls, GPU buffers, shaders) is out of scope for this specification; it covers only simulation-side state (position, velocity, lifetime, color, size) that a rendering layer could consume.
- Particle-to-particle collision and particle-to-world collision are out of scope; particles are purely visual and do not interact with the physics constraint solver or the ECS world's entities.
- Each emitter owns and is responsible for exactly one particle pool sized at construction; pooling/sharing capacity across multiple emitters is out of scope for this feature.
- "Composable" force applicators means their per-tick contributions sum additively; more advanced blending (e.g., weighted or order-dependent composition) is out of scope unless a future spec calls for it.
- The existing frame budget system (rolling timing window) is reused as-is for recording emitter tick cost; no changes to the frame budget subsystem itself are in scope.
- Default pool capacities and default force parameters (gravity strength, wind vector, turbulence amplitude) are implementation details to be decided during planning, not fixed by this specification.
- A single logical "frame" corresponds to one fixed-step tick of the existing game loop; the emitter's tick is called once per fixed step for determinism guarantees to hold.
