# Feature Specification: Sandbox VFX Visualization

**Feature Branch**: `002-sandbox-vfx-visualization`
**Created**: 2026-07-05
**Status**: Draft
**Input**: User description: "Feature 002: Sandbox VFX Visualization for engine_demo. Feature 001 delivered engine_demo::vfx (particle_pool, force_applicator, emitter) — implemented, tested, and green, but deferred wiring it into apps/sandbox (task T022). This feature closes that gap: right-mouse-button burst spawns engine_demo::vfx particles at the cursor alongside the existing 12-particle physics burst; free particles bouncing off the world bounds in non-storm scenes emit a small deterministic VFX spark burst at the contact point; VFX particles are strictly render-only and must not change scene::state_digest() or the headless golden trace; VFX sampling must use its own seeded rng stream so the scene's existing rng draw order is unchanged; live VFX particles are drawn with lifetime-based alpha fade and a live-VFX-count HUD line is added; zero heap/arena allocation per burst after scene construction. Out of scope: rope-break mechanics, including VFX state in the digest, new emitter shapes or force types."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Visual Feedback on Right-Click Burst (Priority: P1)

As a player interacting with the sandbox, when I right-click in the world, I want to see a
burst of visual particle effects at the cursor location — in addition to the existing
physics particle burst — so that the interaction feels more impactful, without changing how
the underlying physics simulation behaves or replays.

**Why this priority**: This is the most visible, most frequently triggered interaction in
the sandbox (documented in the on-screen help text as "RMB burst") and delivers the core
value of the feature on its own: a visually richer burst effect. It has no dependency on the
collision-spark story.

**Independent Test**: Launch the sandbox, right-click anywhere in the world, and observe a
fading VFX burst appear at the click location layered over the existing 12-particle physics
burst. Independently confirm via the headless golden-trace run that triggering this burst
does not change `state_digest()` output for an identical seed and frame sequence.

**Acceptance Scenarios**:

1. **Given** the sandbox is running any non-storm or storm scene, **When** the player
   right-clicks in the world, **Then** the existing 12-particle physics burst still spawns
   exactly as before, and a new VFX particle burst also spawns at the same world-space
   location.
2. **Given** a right-click VFX burst has been spawned, **When** the simulation advances
   across subsequent frames, **Then** each VFX particle's on-screen alpha fades out over its
   lifetime and it disappears once expired.
3. **Given** a headless run with a fixed seed, **When** the same sequence of right-click
   events is replayed with and without this feature's VFX burst enabled, **Then** the
   resulting `state_digest()` values at every frame are identical.

---

### User Story 2 - Collision Sparks on World-Bound Bounce (Priority: P2)

As a player watching free particles bounce off the edges of the world in the rope, pendulum
tower, or cloth scenes, I want to see a small spark effect at the point of impact so that
collisions read as physically significant events rather than silent direction changes.

**Why this priority**: This adds ambient visual polish tied to an existing physics event
(the bounce) but is not required for the feature's MVP burst effect in User Story 1, and it
depends on the same underlying VFX plumbing being in place.

**Independent Test**: Run a non-storm scene, let a free particle travel until it bounces off
a world bound, and observe a brief spark burst rendered at the contact point. Confirm via the
headless golden trace that these automatically triggered sparks do not alter `state_digest()`
for an identical seed and frame sequence.

**Acceptance Scenarios**:

1. **Given** a free particle in a non-storm scene (rope, pendulum tower, or cloth) is moving
   toward a world bound, **When** it bounces off that bound, **Then** a small VFX spark burst
   is emitted at the contact point on the same frame.
2. **Given** the particle storm scene (where free particles wrap around world bounds instead
   of bouncing), **When** a particle crosses a world bound, **Then** no spark burst is
   emitted.
3. **Given** repeated bounces occur across many frames, **When** each bounce triggers a spark
   burst, **Then** the sequence of physics bounce outcomes (position, velocity reflection) is
   unchanged from before this feature existed.

---

### User Story 3 - Observe Live VFX Load via HUD (Priority: P3)

As a developer or player monitoring sandbox performance, I want to see how many VFX
particles are currently alive so that I can understand the visual load being placed on the
frame budget at a glance.

**Why this priority**: This is an observability aid that supports the other two stories but
delivers no simulation behavior on its own; it is the smallest, most easily deferred slice.

**Independent Test**: Trigger right-click bursts and/or collision sparks repeatedly and
confirm the HUD's live-VFX-count line increases as particles spawn and decreases as they
expire, independent of any other HUD readout.

**Acceptance Scenarios**:

1. **Given** no VFX particles are currently alive, **When** the HUD is displayed, **Then**
   the live-VFX-count line reads zero.
2. **Given** one or more VFX bursts have been triggered, **When** the HUD refreshes on a
   subsequent frame, **Then** the live-VFX-count line reflects the current number of alive
   VFX particles, increasing on spawn and decreasing as particles expire.

---

### Edge Cases

- What happens when a right-click burst or a collision spark is requested while the VFX
  pool has no free capacity? The request must be handled gracefully (partial or zero spawn
  reported by the existing pool-exhaustion status), with zero crashes and no corruption of
  existing physics or VFX particle state.
- How does the system handle many bounces occurring within the same frame (e.g., several
  free particles hitting bounds simultaneously)? Each bounce must attempt its own spark
  burst independently; the frame's rng draw order for VFX must remain deterministic
  regardless of how many bounces occur.
- What happens on a zero-length or paused simulation step? No new VFX bursts or sparks are
  emitted, and existing VFX particle state (position, remaining lifetime) is unchanged.
- What happens when switching scenes or reseeding while VFX particles are still alive? All
  VFX particle state resets consistently with the rest of scene state, leaving no stale
  particles rendered from a previous scene/seed.
- What happens in the particle storm scene specifically? Free particles there wrap around
  bounds rather than bounce, so no collision-spark bursts are triggered in that scene; the
  right-click VFX burst (User Story 1) still applies to all scenes.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST spawn a burst of VFX particles at the cursor's world-space
  location whenever the player right-clicks, in addition to — and without altering — the
  existing 12-particle ad-hoc physics burst.
- **FR-002**: System MUST emit a small VFX spark burst at the contact point whenever a free
  particle bounces off a world bound in a non-storm scene (rope, pendulum tower, cloth).
- **FR-003**: System MUST NOT emit a collision-spark burst when a free particle crosses a
  world bound in the particle storm scene (where particles wrap rather than bounce).
- **FR-004**: System MUST exclude all VFX particle state from `state_digest()` and the
  headless golden trace, such that enabling or triggering VFX bursts/sparks never changes
  the digest for an identical seed and frame sequence.
- **FR-005**: System MUST source all VFX particle sampling (position/velocity/lifetime/size
  variance) from a seeded random stream that is separate from, and does not consume from,
  the scene's existing random stream, so the existing rng draw order and its outcomes are
  unchanged.
- **FR-006**: System MUST render currently-alive VFX particles with an opacity that fades
  based on each particle's remaining lifetime.
- **FR-007**: System MUST display a HUD line showing the current count of live VFX
  particles, updated every frame.
- **FR-008**: System MUST NOT perform any heap or arena allocation when spawning a
  right-click burst or a collision-spark burst, after initial scene construction completes.
- **FR-009**: System MUST handle VFX pool exhaustion during a burst or spark request without
  crashing, without throwing, and without corrupting existing particle or physics state.
- **FR-010**: System MUST maintain the sandbox's fixed 60 FPS step budget while the VFX pool
  is at full live capacity under sustained right-click bursts and collision sparks.
- **FR-011**: Every new publicly exposed function introduced by this feature MUST have at
  least one automated test covering a normal case and a boundary/edge case.

### Key Entities

- **Sandbox VFX Burst**: A player-triggered visual effect spawned at the cursor location on
  right-click, layered over the existing ad-hoc physics burst, purely for visual feedback.
- **Collision Spark**: An automatically-triggered visual effect spawned at the contact point
  when a free particle bounces off a world bound in a non-storm scene.
- **Live VFX Count**: An aggregate, per-frame count of currently-alive VFX particles, exposed
  for HUD display.
- **VFX Random Stream**: A random source dedicated to VFX particle sampling, independent of
  the scene's existing random stream used for physics/gameplay randomness.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A headless run with seed 42 over 600 frames produces a byte-identical
  `trace_digest` before and after this feature is enabled, in 100% of comparison runs.
- **SC-002**: 100% of existing automated tests continue to pass after this feature is added,
  and every new publicly exposed function has at least one passing automated test.
- **SC-003**: With the VFX pool at full live capacity under continuous right-click bursts and
  collision sparks, the sandbox sustains its fixed 60 FPS step budget with zero dropped
  fixed-step updates during a sustained manual verification session.
- **SC-004**: During manual verification, a right-click reliably produces a visibly fading
  particle burst at the cursor location layered over the existing physics burst, observable
  on the first attempt.
- **SC-005**: During manual verification in a non-storm scene, free-particle bounces off
  world bounds visibly produce a spark at the contact point in 100% of observed bounces.

## Assumptions

- The VFX particle pool capacity is an implementation/tuning detail (not specified here) that
  must comfortably fit within the sandbox scene's existing fixed memory arena alongside all
  other scene state.
- The existing 12-particle physics burst spawned on right-click (via the sandbox's ad-hoc
  particle system) is unchanged by this feature; the new VFX burst is purely additive and
  visually layered on top of it.
- No rope-break mechanic exists in the sandbox today and none is introduced by this feature;
  collision sparks apply only to the existing free-particle world-bound bounce behavior.
- The particle storm scene's wrap-around behavior for free particles is unchanged; this
  feature does not add wrap-triggered VFX.
- Burst sizes (number of VFX particles per right-click burst or per collision spark) are an
  implementation/tuning detail chosen to look visually appropriate without specified exact
  counts.
- No new emitter shapes or force applicator types are introduced beyond what Feature 001
  already delivered; this feature only wires existing VFX capability into the sandbox.
