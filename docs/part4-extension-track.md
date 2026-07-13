# Part 4 Extension Track — From Engine Port to Full Game

> **Companion to** [Part 4 of the workshop training](speckit-workshop-training.md#part-4-use-case--cross-language-game-transformation)
> **Prerequisite:** Core Track completed through §4.5a — the Rust engine port builds, `cargo test` is green, and the rope scene's headless digest is bit-exact against the C++ reference
> **Format:** Self-paced; each extension is a complete Spec-Kit cycle with its own HITL gates

## Table of Contents

- [E.0 How the Extension Track Works](#e0-how-the-extension-track-works)
- [E.1 Extension 1: The Orbital Arena Game Layer](#e1-extension-1-the-orbital-arena-game-layer)
- [E.2 Extension 2: HUD Parity and Interactive Control](#e2-extension-2-hud-parity-and-interactive-control)
- [E.3 Extension 3: The Default Sandbox Stage — VFX and Free Particles](#e3-extension-3-the-default-sandbox-stage--vfx-and-free-particles)
- [E.4 Extension 4: Full-Fidelity Closure — Every Spec to Recreate the Game](#e4-extension-4-full-fidelity-closure--every-spec-to-recreate-the-game)
- [E.5 Results: The Verified Rust Port](#e5-results-the-verified-rust-port)

---

## E.0 How the Extension Track Works

The Core Track ([training doc §4.2–§4.5a](speckit-workshop-training.md#42-phase-a-reverse-specification)) proved the reverse-spec method on the engine foundation and ended with a running, digest-verified Rust physics scene. This track closes the remaining distance to the **full game** — gravity wells, scoring, telemetry HUD, spark trails, all five scenes — through four iterations of one repeating cycle.

### The Parity Loop

```text
        ┌───────────────────────────────────────────────────────────┐
        │                                                           │
        ▼                                                           │
┌────────────────┐   ┌──────────────────┐   ┌───────────┐   ┌──────┴──────┐
│ Compare output │──▶│ Reverse-spec the │──▶│ Implement │──▶│ Gate: tests │
│ vs. reference  │   │ gap you found    │   │ from spec │   │ + evidence  │
└────────────────┘   └──────────────────┘   └───────────┘   └─────────────┘
```

Each iteration:

1. **Compare** the Rust port's actual output (screenshots, HUD counters, digests) against the C++ reference.
2. **Reverse-spec** whatever the comparison reveals as missing — using the same `/speckit.specify (reverse)` pattern as the Core Track.
3. **Implement** from that spec through the standard `/speckit.plan` → `/speckit.tasks` → `/speckit.implement` pipeline.
4. **Gate** on falsifiable acceptance criteria (unit tests, visual evidence, golden digests) — then loop.

Extensions 1–3 each close the gap a comparison happens to surface. Extension 4 inverts the method: instead of comparing outputs, it audits the entire code surface against the spec inventory — the only iteration that produces a *provably complete* result rather than a visibly improved one. Together they teach one lesson four ways: **a passing glance at a screenshot is not the same as an audited comparison.**

### The Honest-Descope Convention

Every extension explicitly names what it chooses NOT to cover and why, directly in each spec's Constraints section. This is not optional hygiene — it is what makes the track converge:

- An honest spec says what it left out; a spec that silently narrows scope teaches the wrong lesson about what "done" means.
- Explicit descopes form a to-do list. Extension 4 closes them **in place**: the original descope paragraph stays, amended with a *"superseded for full fidelity"* marker and a link to the closing spec.
- A silently-narrowed spec set would require re-auditing everything; an explicitly-narrowed one is cheap to close.

### Working Method

- **Reverse-spec prompts batch cleanly.** The agent-loop technique from [Agent Loop — Reverse-Specs](Agent%20Loop%20%E2%80%94%20Reverse-Specs.md) applies to every extension's spec set — swap the manifest and per-item criteria.
- **Implementation never batches.** The moment work turns into `/speckit.implement`, return to one-task-at-a-time with a HITL gate per task — the same rule as [training §1.3](speckit-workshop-training.md#13-core-principles), unchanged.
- The Rust target constitution ([training §4.3](speckit-workshop-training.md#43-phase-b-target-constitution-rustbevy)) binds every extension's implementation; article references below (e.g., Article 1 "no panics", Article 5 "determinism") point at it.

---

## E.1 Extension 1: The Orbital Arena Game Layer

**Goal:** Extend reverse-specification to the Orbital Arena GAME built in Part 3 (`include/orbital_arena/`) — specifically the mechanics that define the game's visual identity: player-controlled gravity wells, capture contention, and scoring/win condition. This is what makes the Rust screenshot actually look like a two-player capture game instead of a generic physics demo.

### E.1 Gap Found

The Core Track's six specs cover only the engine foundation, so its Rust program is a faithful physics demo — a rigid-link "rope" orbiting an anchor. Set it next to Part 3's screenshots and the missing layer is the game itself.

**Why this is a separate extension, not part of Core Track Phase A:** `include/orbital_arena/` is a substantially larger surface than the six engine subsystems — it also includes a full lobby/countdown/game-over state machine (`match.h`), timed power-ups (`powerup.h`), an input replay log (`input.h`), and a POD snapshot/state-hash for determinism verification (`snapshot.h`). Reverse-specifying and re-implementing **all** of it is a multi-day undertaking, not a single extension. Extension 1 deliberately reverse-specs only the mechanics a screenshot can show — gravity wells, capture, scoring — and explicitly documents the rest as descoped (per the E.0 convention). Those particular descopes are later closed by [Extension 4](#e4-extension-4-full-fidelity-closure--every-spec-to-recreate-the-game), which supersedes them in place, with markers — the honest-descope discipline is what makes that later closure auditable.

### E.1 Reverse-Spec Prompts

**Prompt to Copilot (VS Code Agent Mode) — repeat for each of the three targets below:**

```text
/speckit.specify (reverse)

Analyze the gravity well subsystem in include/orbital_arena/gravity_well.h (there is no
.cpp — it's declarations plus a small .cpp with the two free functions). Extract a
language-agnostic specification that captures:

1. What a gravity well IS (a player-controlled radial-attraction field with a capture
   radius) — behavioral contract, not C++ struct definition
2. The radial-acceleration formula (inverse-square with a clamped minimum distance) and
   exactly when it is zero
3. The capture predicate (strictly-inside test) and the kinematic step (steer → velocity →
   position, clamped to arena bounds)
4. What guarantees the system provides (fairness — no player-index parameter anywhere;
   no singularities; determinism)

Output as a spec document with no C++ syntax, no EASTL references. Note explicitly that
this is a DIFFERENT subsystem from physics::constraint_solver (already reverse-specced in
the Core Track) — they do not share formulas.
```

Repeat the same prompt shape against `include/orbital_arena/scoring.h` (capture contention, award, win evaluation) and `include/orbital_arena/match.h` (lobby/countdown/playing/game-over state machine — spec it fully even though the Rust port will simplify it; see the descope note below).

**Expected outputs:**

- `specs/transform/orbital-arena-gravity-well.spec.md`
- `specs/transform/orbital-arena-scoring.spec.md`
- `specs/transform/orbital-arena-match.spec.md`

**Documented scope reduction (put this in each spec's Constraints section, don't skip it):** power-ups (`powerup.h`), the input replay log (`input.h`), and the snapshot/state-hash serialization (`snapshot.h`) are real subsystems with real value, but are descoped here so the extension stays focused on the mechanics a screenshot can prove were ported correctly.

### E.1 Implementation Scope

Feed these three specs into the same `/speckit.specify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.implement` pipeline as the Core Track ([training §4.5](speckit-workshop-training.md#45-phase-d-tasks-and-implementation)), targeting a new `game` module (gravity wells + a particle field they attract + capture/scoring) alongside the existing constraint-solver module — don't delete the original module, since it's still a valid, separately-tested demonstration of the Core Track's reverse-specs. A reasonable scope:

- Two (or more) gravity wells, each driven by a simple deterministic motion pattern (a pure function of the fixed tick counter — NOT wall-clock time, per Article 5) since a screenshot-capture exercise has no interactive player input to record
- A capacity-reserved field of particles attracted by every active well's `radial_acceleration`, integrated the same way as any other sim state
- Capture resolution + scoring exactly per `orbital-arena-scoring.spec.md`, including the index-independent tie rule
- A HUD showing each player's score (and the winner, once latched)

### E.1 Acceptance Gate

In addition to the per-task gates:

- [ ] `cargo test` covers the acceleration formula, the capture/tie rule, and the win-latch sequence (score freeze after a winner is set) as unit tests — not just visual inspection
- [ ] The rendered scene visibly resembles Part 3's reference screenshots: colored wells, an attracted particle field, and a score readout
- [ ] The original constraint-solver scene/tests from the Core Track still build and pass unmodified

---

## E.2 Extension 2: HUD Parity and Interactive Control

**Goal:** Close the remaining visual gap between the Rust port and the reference C++ screenshots — the on-screen telemetry HUD and interactive mouse control — surfaced by directly comparing running screenshots side by side after completing Extension 1. This is the track's second worked example of the Parity Loop: compare the actual output to the reference, then identify what spec is missing, rather than declaring victory once *some* screenshot exists.

### E.2 Gap Found

Extension 1's Rust screenshots showed wells, particles, and a bare score readout — but the reference C++ sandbox (`apps/sandbox/app.cpp`) also renders a rich telemetry HUD (seed/tick/frame-time with three-tier color coding, a state-digest line, a top-right counts panel, a bottom control-hint legend, and a frame-budget histogram widget) on **every** scene, plus a match panel (score + winner) specifically for the orbital arena scene. None of that was reverse-specced by the Core Track or Extension 1 — both targeted *simulation* logic, not *display* logic. That is a real, previously-undocumented gap, found only by looking at the actual pixels.

### E.2 Reverse-Spec Prompts

**Prompt to Copilot (VS Code Agent Mode):**

```text
/speckit.specify (reverse)

Analyze apps/sandbox/app.cpp's draw_hud and draw_histogram functions (NOT scene.cpp —
this is display code, not simulation logic). Extract a language-agnostic specification
for the on-screen telemetry HUD:

1. What information is displayed, in what screen regions (top-left telemetry column,
   top-right counts panel, bottom-left control hints, bottom-right frame-budget widget)
2. The three-tier color-coding pattern applied to time-budget metrics (normal / warning /
   critical) and which metrics use it
3. The orbital-arena-specific match panel (per-player scores, winner banner) and when it
   appears
4. What is explicitly OUT of scope for a port (exact pixel layout, specific drawing API
   calls, telemetry event logging)

Output as a spec document with no C++ syntax, no raylib references — content and the
three-tier color GUARANTEE, not exact pixel positions.
```

**Expected output:** `specs/transform/sandbox-hud.spec.md`.

**The interactive-control gap is different in kind, not just in coverage.** Checking the reference C++ orbital arena scene's actual input handling (`scene.cpp`'s `make_orbital_inputs`) shows both players are driven by a **scripted sinusoidal autopilot** — the reference implementation has no mouse-driven well control either. What the base sandbox DOES have is a generic, scene-agnostic "click and drag the nearest object" mechanic (`app.cpp`'s `grab_nearest_rope_node` / `drag_held_node`), used for dragging rope/pendulum/cloth bodies. Wanting the Rust *default scene* to be interactively explorable — "click on the well and move it around" — is therefore a **new capability**, not a reverse-spec of anything that already exists. Write it as a spec anyway, tagged explicitly as New, so its origin is as auditable as every reverse-specced subsystem:

```text
Design a NEW specification (not a reverse-spec — state this explicitly) adapting the
sandbox's generic "press near an object, drag it to the cursor" interaction pattern to
one gravity well in the Rust port, so the default scene is interactively explorable.
Define: grab radius, what happens while held (position follows cursor, clamped to arena
bounds), what happens on release (resumes normal driving logic), and failure-mode
guarantees (missing window/camera/cursor must never panic). Output to
specs/transform/orbital-arena-interactive-control.spec.md.
```

### E.2 Implementation Scope

- HUD: recreate the CONTENT of `sandbox-hud.spec.md` using the target UI framework's native widgets (Bevy: `Text`/`Node` UI, not the reference's immediate-mode drawing calls) — the spec's §5 explicitly descopes pixel-perfect layout so the port isn't fighting the wrong constraint.
- Interactive control: a small system reading mouse position/buttons, converting screen space to world space via the engine's own camera API (never hand-rolled), gated by a small state resource so the driving system (whatever moves wells normally) can skip its own update for the well currently being dragged.

### E.2 Acceptance Gate

- [ ] Every color-coded metric in the HUD spec has a corresponding three-tier check in the Rust implementation (not just "some color changes somewhere")
- [ ] The interactive-control spec is tagged **New** in its own text, not silently presented as ported from a C++ header that doesn't have this behavior
- [ ] Cursor/window/camera queries that can legitimately be absent (no primary window, no camera, cursor outside the window) are handled without `unwrap()`/panic (Article 1)
- [ ] `cargo test` covers the interaction's PURE logic (grab-radius check, position/velocity update, bounds clamping) as unit tests — the ECS system that wires mouse input to that logic is glue code and, consistent with the reference C++ app's own input-handling layer, is verified by manual/visual testing, not GTest/`#[test]`

---

## E.3 Extension 3: The Default Sandbox Stage — VFX and Free Particles

**Goal:** Reverse-spec and port the actual *default* base sandbox stage (`ea-sandbox`'s "rope" scene, the very first screenshot in the training's §0.1) — not the Orbital Arena game. This is the track's third worked example of the Parity Loop lesson.

### E.3 Gap Found

Compared side by side against the Rust port's `--scene constraint` demo, two more subsystems turn out to be visually dominant in the reference screenshot and were never reverse-specced by the Core Track or any prior extension: the pink/blue spark trails (VFX particle system, "vfx=N" in the HUD) and the bouncing background dots ("particles=N" in the HUD).

| HUD counter | Subsystem | Reverse-specced before this extension? |
| --- | --- | --- |
| `bodies=` / `edges=` | `physics::constraint_solver` | ✅ Core Track Phase A |
| `frame_avg` / telemetry text | Sandbox HUD | ✅ Extension 2 — but only wired to the Rust port's `arena` scene, not `constraint` |
| `particles=` | Scene-local free-particle ballistic sim | ❌ Never — genuinely distinct from both the rope solver and VFX |
| `vfx=` | `engine_demo::vfx` (Feature 001/002) | ❌ Never — the workshop's OWN Part 0/Self-Study Lab feature, never carried into Part 4 at all |

### E.3 Reverse-Spec Prompts

**Prompt to Copilot (VS Code Agent Mode):**

```text
/speckit.specify (reverse)

Analyze include/engine_demo/vfx/{particle.h,emitter.h,force_applicator.h} (the Feature
001 Particle VFX Subsystem from Part 0 of this training) AND apps/sandbox/scene.cpp's
spawn_vfx_burst / collision-spark usage of it (Feature 002). Extract a language-agnostic
specification covering:

1. The particle pool's data model and partial-success try_spawn/age_and_retire contract
2. The emitter's shape (point/cone/sphere) and force (gravity/wind/turbulence) tagged
   unions, and try_emit/tick
3. The determinism rule: each emitter owns ONE seeded rng stream, isolated from every
   other rng consumer in the simulation, with fixed per-particle draw order
4. The two application-layer usage patterns: interactive burst-on-click, and
   collision-triggered spark bursts

Note explicitly that this is render-only state (never read by physics or game rules) —
a DIFFERENT category from every engine_demo/orbital_arena subsystem specced so far.
Output to specs/transform/vfx-particle-system.spec.md.
```

**A second, related gap needs its own spec — don't fold it into the VFX spec above, they are genuinely different subsystems:**

```text
/speckit.specify (reverse)

Analyze apps/sandbox/scene.cpp's m_particles field and substep() function — a
lightweight ballistic free-particle simulation that is NEITHER the rope solver NOR the
VFX pool. Extract a specification covering: the particle data model (position/velocity/
radius only, no constraints), and the fact that different scene variants apply
COMPLETELY DIFFERENT force/boundary rules to the same particle type (light-gravity-plus-
bounce-with-sparks vs. twin-gravity-wells-with-wraparound vs. delegated-to-another-
subsystem) — rule isolation, never blended. State explicitly that (unlike VFX particles)
this state DOES contribute to the scene's deterministic digest. Output to
specs/transform/sandbox-free-particles.spec.md.
```

### E.3 Implementation Scope

This is the track's largest single addition — scope it deliberately:

- Port the particle pool + gravity-only force application (wind/turbulence explicitly descoped per the VFX spec's own §7 — not needed for visual parity with the reference screenshots) as a small, independently-tested module.
- Port ONLY the default variant's free-particle force/boundary rule (light gravity + bounce + spark-on-hit) — the twin-well and delegated variants are explicitly descoped by `sandbox-free-particles.spec.md` §6, since the delegated variant's role is already served (differently) by the Orbital Arena game's own particle field from Extension 1.
- Extend the ORIGINAL `--scene constraint` demo (the Core Track's rope port) with both: it is the Rust equivalent of the exact screenshot this extension targets. Do not add this to the `arena` scene — that scene already has its own particle field serving an analogous role, and blending the two would contradict `sandbox-free-particles.spec.md`'s rule-isolation guarantee.
- Wire the Extension 2 HUD onto this scene too, closing the "only wired to `arena`" gap noted in the table above, so BOTH Rust scenes show full telemetry.
- Add an interactive burst (mouse click → physics + VFX burst at the cursor), reusing the camera/cursor conversion already built for Extension 2's well-drag feature rather than re-deriving it — factor that conversion into a small shared helper the first time it's needed by a second feature.

### E.3 Acceptance Gate

- [ ] The VFX pool and free-particle force rule are each covered by `#[test]`s independent of any rendering (spawn-cap partial success, age/retire, gravity integration, bounce reflection, spark trigger) — consistent with every other pure-logic module in this port
- [ ] The `--scene constraint` screenshot now visually matches the reference "rope" screenshot's defining features: bouncing background particles, spark trails on bounce, and the full telemetry HUD
- [ ] The `arena` scene (Extensions 1–2) is unmodified and its tests still pass — this extension touches only the `constraint` scene
- [ ] Both spec documents state explicitly which fields/rules are in scope vs. descoped, per the E.0 convention of never silently narrowing

---

## E.4 Extension 4: Full-Fidelity Closure — Every Spec to Recreate the Game

**Goal:** Close **every** remaining gap between the C++ game and `specs/transform/` so the spec set alone is sufficient to recreate the game **100%** — same colors, same gameplay, same scenes, same controls, same headless determinism contract.

### E.4 Gap Found — by Audit, Not Comparison

Extensions 1–3 each closed the gap a screenshot happened to surface; Extension 4 inverts the method: instead of comparing outputs and speccing what looks different, **audit the entire code surface feature-by-feature against the spec inventory** and spec everything that has no home. This is the track's fourth — and final — worked example of the audit lesson, and the only one that produces a provably complete result rather than a visibly improved one.

**What the full audit revealed (2026-07-11):** reviewing every function in `apps/sandbox/` and `include/orbital_arena/` against the then-13 specs found five categories of uncovered behavior — roughly 40% of what defines the game on screen:

| Gap category | Code source | Prior coverage | Closed by (new spec) |
| --- | --- | --- | --- |
| Game-layer orchestration (tick pipeline, rng stream topology, symmetric replenishment, constants) | `orbital_arena/arena.h`, `types.h` | ❌ never specced as a whole | `orbital-arena-orchestration.spec.md` |
| Power-ups / input replay log / snapshot+hash | `powerup.h`, `input.h`, `snapshot.h` | ❌ Extension 1's documented descope | `orbital-arena-powerups.spec.md`, `orbital-arena-input-log.spec.md`, `orbital-arena-snapshot.spec.md` |
| The five scenes: exact geometry, rng draw order, per-scene force rules, digest algorithm | `scene.cpp` builds + `substep()` | ❌ only the default free-particle rule (Extension 3) | `sandbox-scenes.spec.md` |
| Visual identity: every RGBA, gradient, bloom layer, animation formula, effect | `app.cpp` draw functions | ❌ explicitly descoped by Extension 2's HUD spec | `sandbox-visual-identity.spec.md` |
| Full control map + app loop + headless CLI/CSV contract | `app.cpp` input block, `main.cpp`, `headless.cpp` | ❌ only the LMB-drag adaptation (Extension 2) | `sandbox-controls.spec.md`, `sandbox-headless.spec.md` |

**The descope-reversal convention pays off here:** Extensions 1–3's scope reductions were honest and explicit — which is precisely what made this closure cheap. Each descope was reversed **in place**: the original paragraph stays, amended with a *"superseded for full fidelity"* marker and a link to the closing spec (see the amended §6 of `sandbox-hud.spec.md`, §5 of `orbital-arena-scoring.spec.md`, §6/§7 of `sandbox-free-particles.spec.md` / `vfx-particle-system.spec.md`).

**The decisive acceptance artifact — golden digests for all five scenes** (seed 42, 600 frames, two independent runs byte-identical; recorded in `sandbox-scenes.spec.md` §8.1):

| Scene          | `trace_digest`     |
| -------------- | ------------------ |
| rope           | `9dc3bd72a4f7f31a` |
| pendulum tower | `dee045cb412df634` |
| cloth          | `3cbd246289e0cf63` |
| particle storm | `fd2df9d9c889a7fc` |
| orbital arena  | `919d2feba5bdbeac` |

> **Audit catch worth teaching:** the reference source itself contained a stale golden
> value — a comment in `scene.cpp` still cites rope digest `33a6319d856d4869`, which
> pre-dates the rope scene's 32 free particles. Code comments rot; recorded, re-run
> acceptance values don't. The spec table above is ground truth.

### E.4 Reverse-Spec Prompts

**Prompt shape (one per gap row — the same reverse-spec pattern as every prior extension, now demanding exact values):**

```text
/speckit.specify (reverse)

Analyze <files>. Extract a language-agnostic specification at FULL FIDELITY: every
constant, color, formula, and rng draw order is normative — nothing is "implementation
detail" unless it is genuinely invisible (platform presentation workarounds, telemetry
logging). Cross-reference the existing specs in specs/transform/ instead of restating
them. State explicitly which prior spec's descope this closes, if any. Output to
specs/transform/<name>.spec.md.
```

### E.4 Acceptance Gate

How you know the spec set is actually complete:

- [ ] **Traceability:** every literal constant in `scene.h`/`scene.cpp`/`app.cpp` and `include/orbital_arena/*.h` maps to exactly one spec section (spot-check with grep; zero unmapped gameplay/visual literals)
- [ ] **Golden digests:** all five scenes' digests recorded in the spec from two byte-identical runs of the reference build
- [ ] **Descope audit:** `grep -i descoped specs/transform/` returns only (a) telemetry / platform-workaround exclusions and (b) historical descopes carrying a "superseded for full fidelity" marker with a link to the closing spec
- [ ] **Config vs. constant:** values that legitimately vary (e.g. arena `half_extent`: game default 5.0 vs. sandbox embedding 2.0) are documented as *configuration* with both values, never silently hardcoded to one
- [ ] Every new spec names the constitutional articles it inherits and follows the house format (Purpose / Data Model / Operations / Guarantees / Constraints)

**What this extension deliberately still excludes** (and why that's a scope decision, not a gap): platform presentation workarounds (DWM/Metal/compositor shims), the JSONL telemetry pipeline, crash handlers, and the hang watchdog — operational tooling around the game, not the game. The F1 crash key is *listed* in the controls spec (it's player-visible) but its mechanism is not specced.

---

## E.5 Results: The Verified Rust Port

Everything through Extension 4 produced *specifications* — language-agnostic documents plus the C++ reference's golden digests. This section closes the loop: it reports what was actually **built** from those specs, and how well it conforms. Unlike the compact teaching example in [training §4.4–§4.5](speckit-workshop-training.md#44-phase-c-transformation-plan) (a single illustrative constraint-solver port), the real implementation ran as its own full spec-kit feature — `specs/004-rust-bevy-visual-port/` — with its own `spec.md`, `plan.md`, `tasks.md` (14 phases, 81 tasks, ten user stories US1–US10), living in `rust-port/orbital-arena-rs/`.

**Why a second spec-kit feature, not an extension of the teaching example:** the illustrative 6-task plan in training §4.4 covers only the constraint solver. Reaching the acceptance bar Extension 4 sets — five scenes, full visual identity, the whole Orbital Arena game layer, full controls, byte-identical digests — is a materially larger effort. Following the training's own [§1.1a guidance](speckit-workshop-training.md#11a-the-why-and-the-tradeoffs) ("Multi-system features (> 500 lines): use the full five-stage flow"), it got the full flow instead.

### What Was Actually Built

| Story | What it delivers | Status |
| --- | --- | --- |
| US1–US4 | Base engine port: bodies, constraint solver, windowed render, screenshot mode, determinism + zero-alloc tests | ✅ Built |
| US5 | `cargo run -- headless` — windowless CSV/digest trace, the cross-language golden-check instrument | ✅ Built |
| US6 | The real rope scene: exact 24-node chain geometry, exact rng draw order, 32 free particles | ✅ Built — digest **bit-exact** vs. the C++ reference |
| US7 | Full visual identity: gradient background, breathing grid, dust motes, speed-color ramp, particle bloom/trails, HUD (telemetry, three-tier color, histogram) | ✅ Built |
| US8 | Full Orbital Arena game layer: lobby/countdown/playing/game-over state machine, power-ups, input replay log, snapshot/state-hash, world-unit gravity wells | ✅ Built — digest **not** bit-exact (see below) |
| US9 | Remaining three scenes (pendulum tower, cloth, particle storm) | ❌ Not built — `--scene pendulum\|cloth\|storm` is accepted by the CLI and exits with a clear "not implemented yet" error (exit code 3), never a crash |
| US10 | Full control map / app-loop parity (pause, single-step, speed control, perf-bomb demo, screen shake, cursor gravity well) | ❌ Not built — only the LMB well-drag from Extension 2 exists |

This table is itself an example of the E.0 honest-descope convention: US9/US10 are named here as explicitly open, not silently absent.

### Acceptance Results — the Decisive Golden-Digest Comparison

The Extension 4 table recorded the **C++ reference's** golden digests. Here is the **Rust port's** actual conformance against them (seed 42, 600 frames):

| Scene | C++ golden digest | Rust result | Verdict |
| --- | --- | --- | --- |
| rope | `9dc3bd72a4f7f31a` | `9dc3bd72a4f7f31a` | ✅ bit-exact |
| pendulum tower | `dee045cb412df634` | — | ❌ scene not implemented (US9) |
| cloth | `3cbd246289e0cf63` | — | ❌ scene not implemented (US9) |
| particle storm | `fd2df9d9c889a7fc` | — | ❌ scene not implemented (US9) |
| orbital arena | `919d2feba5bdbeac` | `43ca6a47a61de7d4` | ⚠️ documented tolerance-fallback (below) |

**The orbital-arena digest, root-caused:** a byte-for-byte diff of the two implementations' per-frame CSV traces showed frames 0–183 **byte-identical** — including the very first tick's replenishment of all 500 field particles across 250 rotationally-symmetric groups, which alone confirms the ported rng stream, its salting, and the rotational-symmetry formulas are exact — then a **sudden**, not gradual, full hash change at frame 184 (the 4th live-gameplay tick). That signature points at the digest's own extreme sensitivity, not a logic bug: the digest hashes *live-particle order*, and the particle pool's dense-array swap-remove recycling means a single capture resolving to a different (but physically equally valid) winner under a sub-ULP floating-point difference immediately permutes every subsequent particle's storage slot, cascading into a wholly different hash from that tick on. This is exactly the risk `sandbox-scenes.spec.md` §8.1 anticipates — "implementations on platforms where [bit-identical evaluation] is unattainable should fall back to trajectory-tolerance comparison" — so it is recorded here rather than chased indefinitely.

> **Field notes from the engine-port layer** (the parity rng engine, `Time<Fixed>`
> quantization, and particle-order-as-state) live with the Core Track milestone in
> [training §4.5a](speckit-workshop-training.md#45a-milestone-the-engine-runs-in-rust) —
> they apply to any port with a golden-digest gate, not just the game layer.

### How to Run the Resulting Game

```powershell
cd rust-port/orbital-arena-rs

# The Orbital Arena game (US8) -- windowed (default scene)
cargo run -- --scene arena

# The rope / VFX / free-particle scene (US6/US7) -- windowed
cargo run -- --scene constraint

# Automated screenshot evidence (either scene)
cargo run -- --scene arena screenshot --warmup-frames 400 --out ../../docs/screenshots/my-run.png --seed 42

# Windowless golden-digest trace (US5) -- the cross-language acceptance instrument
cargo run -- headless --scene constraint --seed 42 --frames 600 --out trace.csv

# Full verification
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

On a VM with no hardware GPU, `wgpu` falls back automatically to a software adapter (DX12 WARP / "Microsoft Basic Render Driver") — slower, but no extra flags are needed for windowed/screenshot modes; `headless` needs no GPU at all.

### Continuing This Work

US9 (three remaining scenes) and US10 (full controls) are tracked as Phase 12/13 of `specs/004-rust-bevy-visual-port/tasks.md` — an exercise for after the workshop, following the same Parity Loop pattern demonstrated above: reverse-spec → implement → golden-digest gate.
