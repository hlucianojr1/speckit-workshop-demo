# Feature Specification: Particle VFX Subsystem

**Feature Branch**: `001-particle-vfx-subsystem`
**Created**: 2026-07-05
**Status**: Draft
**Input**: User description: "Create a specification for a Particle VFX Subsystem for engine_demo. Provides visual effects (sparks, dust, explosions) for the physics sandbox. Pooled particle storage with fixed capacity, multiple emitter types (point, cone, sphere), per-particle position/velocity/lifetime/color/size, composable force applicators (gravity, wind, turbulence), deterministic seeded spawning, zero allocation in the hot path, and integration with the existing ECS World, fixed-step game loop, and frame budget system. Particles are purely visual and do not participate in physics constraint solving."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Trigger Visual Effects from Gameplay Events (Priority: P1)

As a gameplay/engine programmer, I want to spawn a burst or stream of particles (sparks on
impact, dust on contact, an explosion on destruction) so that players receive immediate
visual feedback for physics and gameplay events, without those particles influencing the
physics simulation.

**Why this priority**: This is the core value of the subsystem. Without the ability to
spawn particles and have them live, move, and expire correctly, there is no usable VFX
capability at all.

**Independent Test**: Construct a particle pool and a single point emitter, issue a burst
emit request, then advance the simulation by a fixed timestep several times. Verify the
requested number of particles become active, their position/velocity/lifetime/color/size
evolve as expected, and each particle is retired once its lifetime elapses — all without
invoking the physics constraint solver.

**Acceptance Scenarios**:

1. **Given** an emitter with free pool capacity, **When** a burst of N particles is
   requested, **Then** N new particles become active with initialized position, velocity,
   lifetime, color, and size.
2. **Given** active particles with partially elapsed lifetimes, **When** the simulation
   advances by one fixed timestep, **Then** each particle's position is updated from its
   velocity and its remaining lifetime is decreased by the timestep.
3. **Given** a particle whose remaining lifetime reaches zero during a tick, **When** that
   tick completes, **Then** the particle is retired and its slot becomes available for a
   future emission.

---

### User Story 2 - Configure Emitter Shape and Composable Forces (Priority: P2)

As an engine/effects programmer, I want to choose an emitter shape (point, cone, sphere)
and attach one or more force applicators (gravity, wind, turbulence) so that different
events can produce visually distinct effects using the same underlying subsystem.

**Why this priority**: Shape and force variety differentiate effects (a spark shower vs. a
dust puff vs. an explosion) but depend on the core spawn/update behavior from User Story 1
already working.

**Independent Test**: Configure emitters of each shape type with different force applicator
combinations, emit particles from each, and verify initial particle placement/direction
matches the configured shape and that velocities change over subsequent ticks according to
the attached forces — independent of any other emitter instance.

**Acceptance Scenarios**:

1. **Given** a cone emitter configured with a direction and spread angle, **When** particles
   are emitted, **Then** each particle's initial velocity direction falls within the
   configured cone.
2. **Given** a sphere emitter configured with a radius, **When** particles are emitted,
   **Then** each particle's initial position falls within the configured sphere volume.
3. **Given** an emitter with gravity and wind force applicators attached, **When** it is
   ticked across multiple frames, **Then** each active particle's velocity reflects the
   combined acceleration contributed by both applicators.

---

### User Story 3 - Deterministic Replay for Testing and Debugging (Priority: P3)

As a QA or engine programmer, I want particle spawning and motion to be fully deterministic
for a given seed and sequence of ticks/emit calls, so that automated tests and bug replays
produce identical results every time.

**Why this priority**: Determinism is essential for trustworthy automated testing and
debugging, but it is a correctness property layered on top of the spawning/update behavior
from User Story 1, rather than a standalone visual capability.

**Independent Test**: Construct two identically-configured emitters with the same seed, run
both through an identical sequence of fixed-timestep ticks and emit calls, and compare the
resulting particle states for exact equality.

**Acceptance Scenarios**:

1. **Given** two emitter instances constructed with the same seed and configuration,
   **When** both are driven through an identical sequence of ticks and emit calls, **Then**
   the resulting particle states (position, velocity, lifetime, color, size) are identical.
2. **Given** a fixed seed, **When** the same burst emit is requested in two separate runs,
   **Then** the sequence of spawned particles' initial attributes matches exactly between
   runs.

---

### Edge Cases

- What happens when an emit request is made against a pool that has zero free capacity?
  The system must report exhaustion rather than fail silently, crash, or throw.
- How does the system handle a tick with a zero-length timestep (a paused frame)? Particle
  positions, lifetimes, and velocities must remain unchanged.
- What happens when a particle's lifetime expires exactly on the tick that processes it?
  The particle must be retired that tick and must not be presented as active afterward.
- How does the system handle several emitters drawing from the same shared pool when
  combined demand exceeds remaining capacity within a single frame? Emitters processed
  earlier in the frame may succeed while later ones receive an exhaustion status; no
  particle state is corrupted.
- How does the system behave when force applicators (e.g., turbulence) would otherwise
  produce non-finite velocity or position values? Resulting particle state must remain
  finite and bounded.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide pooled particle storage with a fixed maximum capacity
  established once at construction time.
- **FR-002**: System MUST support emitting particles from at least three emitter shapes:
  point, cone, and sphere.
- **FR-003**: System MUST track, per particle, its position, velocity, remaining lifetime,
  color, and size.
- **FR-004**: System MUST allow zero or more force applicators (gravity, wind, turbulence)
  to be attached to an emitter and combined additively when updating particle velocities on
  each tick.
- **FR-005**: System MUST derive emitted particles' initial attributes (position within the
  emitter shape, direction, and any configured variance) from a deterministic, explicitly
  seeded random source, with no reliance on non-deterministic entropy sources.
- **FR-006**: System MUST report a distinguishable status when an emit request cannot be
  fully satisfied due to pool exhaustion, and MUST leave existing particle state unaffected
  by the failed portion of the request.
- **FR-007**: System MUST allow an emitter to be ticked using the fixed timestep of the
  existing game loop independently of, and without invoking, the physics constraint solver.
- **FR-008**: System MUST NOT perform any memory allocation during steady-state emit or
  tick operations after the pool has been constructed; all particle storage MUST come from
  the pre-allocated pool capacity.
- **FR-009**: System MUST retire a particle from the active set once its remaining lifetime
  reaches zero or below, making its slot available for a subsequent emission.
- **FR-010**: System MUST produce identical sequences of particle state across separate
  runs when given the same seed, the same emitter configuration, and the same sequence of
  ticks and emit calls.
- **FR-011**: System MUST expose per-particle state and an aggregate live-particle count in
  a form usable by a separate rendering step, without requiring particles to participate in
  physics collision or constraint resolution.
- **FR-012**: System MUST make the time cost of a tick/emit operation observable to the
  existing frame budget tracking so that VFX cost can be measured against the frame's
  allotted budget.

### Key Entities

- **Particle**: A single visual element with position, velocity, remaining lifetime, color,
  and size. Visual-only; never contributes to or is affected by physics constraint solving.
- **Particle Pool**: Fixed-capacity storage for particles, tracking which slots are live
  versus free, and reporting an exhaustion status when no free slots remain.
- **Emitter**: A configured source of particles with a shape (point, cone, or sphere),
  spawn parameters (e.g., burst size, initial variance), a seeded random source, and zero or
  more attached force applicators. Draws new particles from a particle pool.
- **Force Applicator**: A named, composable contributor to particle acceleration (gravity,
  wind, or turbulence) with its own parameters, attached to one or more emitters.
- **Emit/Tick Outcome**: The result of an emit or tick operation, distinguishing success from
  conditions such as pool exhaustion.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An emitter sustaining 500 simultaneous live particles completes its per-frame
  update in under 2 milliseconds at a fixed 60 FPS timestep.
- **SC-002**: Across a sustained run of at least 10,000 frames with continuous emission
  requests, pool exhaustion is reported on 100% of emit attempts made once capacity is
  reached, with zero crashes or unhandled failures.
- **SC-003**: 100% of emitter behavior test scenarios can be executed and validated without
  running the physics constraint solver.
- **SC-004**: Given an identical seed and an identical sequence of ticks/emit calls, repeated
  runs produce identical particle state snapshots in 100% of comparison tests.
- **SC-005**: Every publicly exposed subsystem capability has at least one automated test
  covering both a normal case and a boundary/edge case.

## Assumptions

- A single fixed-capacity particle pool is allocated once (e.g., at subsystem construction)
  and may be shared by multiple emitters; choosing a specific capacity value is a
  deployment/tuning concern outside this spec.
- Rendering or drawing particles (e.g., issuing draw calls in the sandbox application) is
  out of scope; this subsystem is responsible for simulating and exposing particle state
  only, for a separate rendering step to consume.
- Emitters advance using the same fixed timestep as the existing game loop; variable or
  interpolated sub-stepping is out of scope for this feature.
- Wind and turbulence force applicators derive any randomized contribution from the same
  deterministic, seeded random source used for spawning, to preserve overall determinism.
- Force applicators combine additively (linear superposition) each tick; ordering-dependent
  or mutually exclusive force modes are not required.
- Particles do not collide with the physics world or with each other; particle-to-particle
  and particle-to-world collision is out of scope.
