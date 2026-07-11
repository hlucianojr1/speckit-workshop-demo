# GitHub Copilot Spec-Kit: Comprehensive Workshop Training

> **Workshop Series** | EA × Insight Copilot Workshop
> **Audience:** Intermediate developers new to specification-driven AI development
> **Prerequisites:** GitHub Copilot subscription (Pro+ or Enterprise), VS Code with Copilot extension, basic familiarity with version control
> **Total Duration:** 3 sessions × 90 minutes each
> **Format:** Demo-first — watch experts use Spec-Kit live, then learn the concepts behind it

---

## Table of Contents

- [Part 0: Live Demo — Spec-Kit in Action](#part-0-live-demo--spec-kit-in-action)
  - [0.1 What You're About to See](#01-what-youre-about-to-see)
  - [0.2 Demo Setup](#02-demo-setup)
  - [0.2a Workshop Branching Model: demo-0x](#02a-workshop-branching-model-demo-0x)
  - [0.3 Demo Step 1: Ground the Constitution](#03-demo-step-1-ground-the-constitution)
  - [0.4 Demo Step 2: Generate the Specification](#04-demo-step-2-generate-the-specification)
  - [0.5 Demo Step 3: Produce the Architecture Plan](#05-demo-step-3-produce-the-architecture-plan)
  - [0.6 Demo Step 4: Decompose into Tasks](#06-demo-step-4-decompose-into-tasks)
  - [0.7 Demo Step 5: Implement with Human Review](#07-demo-step-5-implement-with-human-review)
  - [0.8 What Just Happened?](#08-what-just-happened)
- [Part 1: Foundations — What is Spec-Kit?](#part-1-foundations--what-is-spec-kit)
  - [1.1 The Problem Spec-Kit Solves](#11-the-problem-spec-kit-solves)
  - [1.1a The Why and the Tradeoffs](#11a-the-why-and-the-tradeoffs)
  - [1.1b Context Windows and Hallucination Mitigation](#11b-context-windows-and-hallucination-mitigation)
  - [1.2 The Five-Stage Spec-Kit Flow](#12-the-five-stage-spec-kit-flow)
  - [1.2a Optional Quality Commands: Clarify, Analyze, Checklist](#12a-optional-quality-commands-clarify-analyze-checklist)
  - [1.3 Core Principles](#13-core-principles)
  - [1.3a Top 10 Best Practices for Spec-Driven Copilot Workflows](#13a-top-10-best-practices-for-spec-driven-copilot-workflows)
  - [1.4 GitHub Copilot Ecosystem Integration](#14-github-copilot-ecosystem-integration)
  - [1.5 Copilot Custom Instructions Architecture](#15-copilot-custom-instructions-architecture)
  - [1.6 GitHub Copilot CLI — The Terminal Agent](#16-github-copilot-cli--the-terminal-agent)
  - [1.7 The Reference Project: engine_demo](#17-the-reference-project-engine_demo)
- [Part 2: Hands-On Practice — Adding More Features](#part-2-hands-on-practice--adding-more-features)
  - [2.1 Session Overview](#21-session-overview)
  - [2.2 Rejection and Re-Specification Flow](#22-rejection-and-re-specification-flow)
  - [2.3 Feature 2: Lockless Ring Buffer (Condensed)](#23-feature-2-lockless-ring-buffer-condensed)
  - [2.4 Feature 3: Fixed String (Condensed)](#24-feature-3-fixed-string-condensed)
  - [2.5 Reflection and Key Takeaways](#25-reflection-and-key-takeaways)
- [Part 3: Use Case — Designing a New Game from an Existing Foundation](#part-3-use-case--designing-a-new-game-from-an-existing-foundation)
  - [3.1 Session Overview](#31-session-overview)
  - [3.2 Analyzing the Existing Engine](#32-analyzing-the-existing-engine)
  - [3.3 Writing a New Constitution](#33-writing-a-new-constitution)
  - [3.4 /speckit.specify — New Game Features](#34-speckitspecify--new-game-features)
  - [3.5 /speckit.plan — Mapping Foundation to New Architecture](#35-speckitplan--mapping-foundation-to-new-architecture)
  - [3.6 /speckit.tasks — Delta Decomposition](#36-speckittasks--delta-decomposition)
  - [3.7 /speckit.implement — Building on the Foundation](#37-speckitimplement--building-on-the-foundation)
  - [3.7a Seeing It Run — Visualization and Screenshots](#37a-seeing-it-run--visualization-and-screenshots)
  - [3.8 How Spec-Kit Prevents Scope Creep](#38-how-spec-kit-prevents-scope-creep)
  - [3.9 Reflection and Key Takeaways](#39-reflection-and-key-takeaways)
- [Part 4: Use Case — Cross-Language Game Transformation](#part-4-use-case--cross-language-game-transformation)
  - [4.1 Session Overview](#41-session-overview)
  - [4.2 Phase A: Reverse-Specification](#42-phase-a-reverse-specification)
  - [4.2a Phase A2 (Extension): Reverse-Specifying the Orbital Arena Game Layer](#42a-phase-a2-extension-reverse-specifying-the-orbital-arena-game-layer)
  - [4.2b Phase A3 (Further Extension): HUD Parity and Interactive Control](#42b-phase-a3-further-extension-hud-parity-and-interactive-control)
  - [4.2c Phase A4 (Further Extension): The Default Sandbox Stage — VFX and Free Particles](#42c-phase-a4-further-extension-the-default-sandbox-stage--vfx-and-free-particles)
  - [4.2d Phase A5 (Completion): Full-Fidelity Closure — Every Spec to Recreate the Game](#42d-phase-a5-completion-full-fidelity-closure--every-spec-to-recreate-the-game)
  - [4.3 Phase B: Target Constitution (Rust/Bevy)](#43-phase-b-target-constitution-rustbevy)
  - [4.4 Phase C: Transformation Plan](#44-phase-c-transformation-plan)
  - [4.5 Phase D: Tasks and Implementation](#45-phase-d-tasks-and-implementation)
  - [4.6 Copilot CLI Scenario: Terminal-Driven Transformation](#46-copilot-cli-scenario-terminal-driven-transformation)
  - [4.7 Best Practices for Cross-Language Transformation](#47-best-practices-for-cross-language-transformation)
  - [4.8 Reflection and Key Takeaways](#48-reflection-and-key-takeaways)
- [Appendix A: Spec-Kit Quick Reference Card](#appendix-a-spec-kit-quick-reference-card)
- [Appendix B: Copilot CLI Command Reference](#appendix-b-copilot-cli-command-reference)
- [Appendix C: Custom Instructions File Templates](#appendix-c-custom-instructions-file-templates)
- [Appendix D: Rust Game-Dev Research and Guidelines Prompt Series](#appendix-d-rust-game-dev-research-and-guidelines-prompt-series)
- [Self-Study Lab: Visualize the VFX Subsystem in the Sandbox](#self-study-lab-visualize-the-vfx-subsystem-in-the-sandbox)
  - [S.1 Mission and Context](#s1-mission-and-context)
  - [S.2 Prerequisites](#s2-prerequisites)
  - [S.3 Step 1: /speckit.specify — Feature 002](#s3-step-1-speckitspecify--feature-002)
  - [S.4 Step 2: /speckit.clarify (Optional)](#s4-step-2-speckitclarify-optional)
  - [S.5 Step 3: /speckit.plan — Review Checklist](#s5-step-3-speckitplan--review-checklist)
  - [S.6 Step 4: /speckit.tasks — Sizing Gate](#s6-step-4-speckittasks--sizing-gate)
  - [S.7 Step 5: /speckit.implement — One Task at a Time](#s7-step-5-speckitimplement--one-task-at-a-time)
  - [S.8 Final Verification](#s8-final-verification)
  - [S.9 Stretch Goals and Reflection](#s9-stretch-goals-and-reflection)
- [Workshop Finale: Two Features, One Branch](#workshop-finale-two-features-one-branch)

---

## Part 0: Live Demo — Spec-Kit in Action

### 0.1 What You're About to See

You're about to watch an expert drive GitHub Copilot through a complete feature build — from a blank problem statement to compiled, tested C++ code — in under 20 minutes. No slides. No theory. Just Spec-Kit doing what it does: converting vague intent into governed, reviewable, working software.

**The scenario:** We're adding a **Particle VFX Subsystem** to an existing C++20 game engine. This is a real subsystem with real constraints — no heap allocation in hot loops, deterministic behavior, explicit allocators, full test coverage. Watch how Spec-Kit ensures every line of generated code respects those rules.

**The game engine you'll be working with:**

![ENGINE_DEMO showcase — Rope scene with verlet-integrated particles, physics telemetry HUD, and frame budget monitor](../screenshot.png)

_The `engine_demo` physics sandbox: a C++20 engine with EASTL containers, constraint-based physics, and a real-time frame budget system. This is the codebase we'll be extending with Spec-Kit._

**What to watch for:**

- How a 5-sentence problem statement becomes a formal specification with measurable acceptance criteria
- How the AI generates architecture that's _constrained_ by binding rules (not hallucinated)
- How tasks are right-sized for human review (≤ 150 lines each)
- How the first implementation compiles and passes tests on the first try — because the spec was precise

**The demo flow at a glance:**

```mermaid
flowchart LR
    A["🔒 Step 1\nGround the\nConstitution"] -->|"HITL ✅"| B["📋 Step 2\nGenerate the\nSpecification"]
    B -->|"HITL ✅"| C["🏗️ Step 3\nProduce the\nArchitecture Plan"]
    C -->|"HITL ✅"| D["✂️ Step 4\nDecompose\ninto Tasks"]
    D -->|"HITL ✅"| E["⚙️ Step 5\nImplement with\nHuman Review"]

    style A fill:#1a1a2e,stroke:#00d4ff,color:#fff
    style B fill:#1a1a2e,stroke:#00d4ff,color:#fff
    style C fill:#1a1a2e,stroke:#00d4ff,color:#fff
    style D fill:#1a1a2e,stroke:#00d4ff,color:#fff
    style E fill:#1a1a2e,stroke:#00d4ff,color:#fff
```

_Each arrow is a Human-In-The-Loop (HITL) gate — the demo pauses for approval before advancing. This is the core discipline that makes AI-generated code governable._

> **Command naming:** this workspace installs the Spec-Kit prompts under the `speckit.` prefix.
> Wherever this document says `/constitution`, `/specify`, `/plan`, `/tasks`, or `/implement`
> as a stage name, the actual command you type in Copilot Chat is `/speckit.constitution`,
> `/speckit.specify`, `/speckit.plan`, `/speckit.tasks`, or `/speckit.implement`.
> The same prefix applies to the **optional quality commands** covered in §1.2a:
> `/speckit.clarify`, `/speckit.analyze`, and `/speckit.checklist`.
> (The GitHub Copilot **CLI**'s `/plan` command in §1.6 and Appendix B is a different,
> CLI-native feature and keeps its short name.)

<!-- markdownlint-disable-next-line MD028 -->

> **Facilitator says:** "Don't worry about understanding every concept yet. Just watch. We'll unpack the WHY in Part 1. Right now, absorb the WHAT."

---

### 0.2 Demo Setup

| Requirement                                            | Status                |
| ------------------------------------------------------ | --------------------- |
| `speckit-workshop-demo` repository cloned and building | `ctest` green ✅      |
| Spec-Kit CLI installed locally ([install guide](local-build-guide.md#6-install-spec-kit-specify-cli-locally)) | `specify check` ✅    |
| VS Code with Copilot Chat (Agent Mode enabled)         | Connected ✅          |
| `specs/constitution.md` open in editor                 | 8 articles visible ✅ |
| `AGENTS.md` open in editor                             | Hard rules visible ✅ |
| Demo branch created and checked out (§0.2a)            | `demo-01` ✅          |
| Fallback branch available (`git branch -a`)            | `demo-fallback` ✅    |

> **Stage insurance:** if live generation stalls mid-demo, `git checkout demo-fallback`
> contains the completed `specs/particle-vfx/` artifacts and the implemented, tested
> subsystem — you can jump to any step's finished state instantly.

**Build verification (run this before demo starts):**

```bash
cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

> **Facilitator says:** "Everything's green. Now watch what happens when we feed a problem to Spec-Kit."

---

### 0.2a Workshop Branching Model: demo-0x

Everything in this workshop happens on a disposable **demo branch** — `demo-01` for your first run, `demo-02` if you restart, and so on. `main` is never touched.

```mermaid
gitGraph
    commit id: "baseline"
    branch demo-01
    checkout demo-01
    commit id: "setup"
    branch feature-001-particle-vfx
    checkout feature-001-particle-vfx
    commit id: "001 tasks"
    checkout demo-01
    merge feature-001-particle-vfx
    branch feature-002-sandbox-vfx
    checkout feature-002-sandbox-vfx
    commit id: "002 tasks"
    checkout demo-01
    merge feature-002-sandbox-vfx
```

**Create the demo branch (before the demo starts):**

```bash
git checkout main && git pull
git checkout -b demo-01
```

**The three rules:**

1. **Feature branches base off `demo-0x`, not `main`.** Spec-Kit's `create-new-feature.sh` branches from the **current HEAD** — so simply be checked out on `demo-0x` whenever you run `/speckit.specify`, and the feature branch (e.g., `001-particle-vfx-subsystem`) is automatically based on it.
2. **Features merge back to `demo-0x`, never to `main`.** When a feature's tasks are all approved: `git checkout demo-0x && git merge --no-ff <feature-branch>`, then re-run `ctest` as the merge gate.
3. **Restart by incrementing, not repairing.** If a run goes off the rails, abandon the branch entirely and cut `demo-0(x+1)` fresh from `main`. A restart costs one `git checkout -b`; untangling a broken branch mid-workshop costs the session.

By the end of the workshop, your `demo-0x` branch carries **two merged features** — Feature 001 (Particle VFX, Part 0) and Feature 002 (Sandbox VFX Visualization, Self-Study Lab) — demonstrated running together in the [Workshop Finale](#workshop-finale-two-features-one-branch).

---

### 0.3 Demo Step 1: Ground the Constitution

> **⏱ ~2 minutes**

**What you'll see:** Before writing ANY specification, we ground ourselves in the existing rules. The constitution is not a guideline — it's a **binding contract** that gates everything downstream.

**Action:** Open `specs/constitution.md` and scan the 8 articles that will constrain our particle system.

For the **Particle VFX Subsystem**, the relevant constraints are:

| Article                  | What It Means for Our Feature                                   |
| ------------------------ | --------------------------------------------------------------- |
| Art. 1 — No exceptions   | Allocation failures return status enums, never throw            |
| Art. 2 — No RTTI         | No `dynamic_cast`; use tagged enums for particle types          |
| Art. 3 — EASTL-first     | `eastl::vector` with explicit allocator, never `std::vector`    |
| Art. 4 — Allocator-aware | Emitter constructor takes `engine_demo::allocator&`             |
| Art. 5 — Determinism     | Seeded RNG; positions reproducible across runs                  |
| Art. 6 — Real-time       | **Critical**: Zero allocation in emit/update loop; pre-allocate |
| Art. 7 — Test-first      | GTest for every public function                                 |
| Art. 8 — HITL gates      | Human approval between every stage                              |

> **Facilitator says:** "These 8 rules are the guardrails. Every single thing Copilot generates must satisfy ALL of them. Watch how this shapes every downstream decision."

---

### 0.4 Demo Step 2: Generate the Specification

> **⏱ ~5 minutes**

**What you'll see:** A natural-language problem statement goes in. A fully-formed specification with data models, behavioral contracts, and measurable acceptance criteria comes out — all grounded in the constitution.

> **Branch check:** you're on `demo-01` (§0.2a). `/speckit.specify` will create the feature
> branch `001-particle-vfx-subsystem` from it and switch to it automatically — all
> Feature 001 work lands there until the merge-back in §0.7.

**Prompt to Copilot:**

```text
/speckit.specify

Create a specification for a Particle VFX Subsystem for engine_demo.

Context:
- This is a C++20 game engine with EASTL containers, no exceptions, no RTTI
- See specs/constitution.md for binding constraints (articles 1-8)
- The subsystem provides visual effects (sparks, dust, explosions) for the physics sandbox
- Must integrate with the existing ECS World (ecs/world.h) and fixed-step game loop (sim/game_loop.h)
- Particles are purely visual — they do NOT participate in physics constraint solving

Requirements:
- Pooled particle storage with fixed capacity set at construction
- Multiple emitter types (point, cone, sphere) as a tagged enum
- Per-particle: position, velocity, lifetime, color, size
- Force applicators (gravity, wind, turbulence) composable per-emitter
- Deterministic spawning from seeded RNG for replay consistency
- Zero allocation in the update/emit hot path (Article 6)
- Interop with the existing frame budget system for timing

Acceptance Criteria:
- Pool exhaustion returns a status enum (not exception)
- Emitter can be ticked independently of the physics solver
- 500 simultaneous particles maintain < 2ms per frame at 60 FPS
- All public functions have GTest coverage
- Deterministic: same seed + same frame sequence = identical particle state
```

**Watch Copilot produce `specs/particle-vfx/spec.md`:**

The output includes:

- **§1 Purpose** — one-paragraph scope statement
- **§2 Scope** — explicit in/out boundaries (no GPU rendering, no particle collisions)
- **§3 Constitutional Constraints** — table showing how EACH article is addressed
- **§4 Data Model** — concrete `struct particle` (32-byte aligned), `enum class emitter_shape`, `enum class emit_status`
- **§5 Behavioral Contracts** — emitter lifecycle, force application protocol, determinism guarantee
- **§6 Performance Budget** — 500 particles < 2ms, emit(100) < 0.1ms, ≤ 24KB memory per emitter
- **§7 Test Plan** — 6 named tests with what each validates

> **Facilitator says:** "Notice what just happened. A vague 'add particle effects' became a 7-section document with checkable criteria. The AI can't hallucinate its way past 'pool exhaustion returns pool_exhausted enum' — that's falsifiable. Let's approve it and move on."

**🚨 HITL GATE:** Review the spec. All 8 articles addressed? Acceptance criteria measurable? Scope boundaries clear?

✅ **Approve** → proceed to `/speckit.plan`

> **Optional refinement:** in a real run, this is where you'd insert `/speckit.clarify` — a
> structured Q&A pass that hunts for underspecified areas and records the answers in a
> Clarifications section of `spec.md`. The demo skips it because the prompt above was
> deliberately precise; the Self-Study Lab (§S.4) runs it for real. See §1.2a.

---

### 0.5 Demo Step 3: Produce the Architecture Plan

> **⏱ ~5 minutes**

**What you'll see:** The spec (WHAT) becomes an architecture plan (HOW). File layout, memory decisions, algorithms, test ordering, and task sizing — all derived from the specification.

**Prompt to Copilot:**

```text
/speckit.plan

Based on specs/particle-vfx/spec.md and specs/constitution.md, produce an implementation
plan for the Particle VFX Subsystem.

Requirements for the plan:
- File layout (which headers, which .cpp files, where in the directory tree)
- Memory layout decisions (pool structure, alignment)
- Algorithm choices for emit/tick/recycle
- Test-first ordering (what gets tested before what)
- Integration points with existing subsystems (ECS world, game loop, frame budget)
- Estimated task count and sizing (each task < 150 lines of diff)
```

**Watch Copilot produce the plan with:**

- **File Layout** — `include/engine_demo/vfx/particle.h`, `emitter.h`, `forces.h` + matching `.cpp` + test file
- **Memory Layout** — flat `eastl::vector<particle>` (cache-friendly) + free-list index stack. Explains WHY: "~4× faster than pointer-chasing a linked list at 500 particles"
- **Algorithm: tick()** — zero-allocation loop advancing particles, recycling dead slots
- **Algorithm: emit()** — pop from free-list, spawn with seeded RNG
- **Test-First Ordering** — creation → exhaustion → lifetime → determinism → force composition → zero-allocation assertion
- **Task Decomposition** — 5 tasks, largest is 120 lines

> **Facilitator says:** "Look at the memory layout decision. It chose a flat array over a linked list and EXPLAINS WHY — cache locality, Article 6 compliance. This isn't random code generation. It's architecture reasoning backed by the constitution."

**🚨 HITL GATE:** Plan review.

- Memory layout satisfies Article 6 (real-time)? ✅ Flat array, pre-allocated pool
- Test ordering satisfies Article 7 (test-first)? ✅ Tests before force implementation
- Each task < 150 lines? ✅ Largest is 120 lines

✅ **Approve** → proceed to `/speckit.tasks`

---

### 0.6 Demo Step 4: Decompose into Tasks

> **⏱ ~3 minutes**

**What you'll see:** The plan becomes a prioritized list of discrete, reviewable work items. Each task has explicit files, deliverables, and acceptance gates.

**Prompt to Copilot:**

```text
/speckit.tasks

Decompose specs/particle-vfx/plan.md into implementable tasks. Each task must:
- Be < 150 lines of diff
- Include the test BEFORE or WITH the implementation (Article 7)
- State which files are created or modified
- State the acceptance gate (what must be true for the task to be approved)
```

**Watch Copilot produce 5 tasks:**

| #   | Task                                                  | Lines | Key Gate                                       |
| --- | ----------------------------------------------------- | ----- | ---------------------------------------------- |
| 1   | Data types header (`particle.h`)                      | ~60   | `static_assert(sizeof(particle) == 32)` passes |
| 2   | Emitter core: constructor + tick + emit               | ~120  | Zero allocations in tick/emit                  |
| 3   | Core tests (4 GTest cases)                            | ~100  | All 4 pass with `ctest`                        |
| 4   | Force system: gravity, wind, turbulence               | ~80   | Deterministic turbulence given same seed       |
| 5   | Integration test: force composition + zero-allocation | ~60   | All 6 tests green                              |

> **Facilitator says:** "Five tasks. The largest is 120 lines. A human can review each one in under 5 minutes. Compare that to a single 'add particle system' prompt producing 600+ lines in one shot that nobody can review."

**🚨 HITL GATE:** Task sizing review. Dependencies ordered correctly? Test-first maintained?

✅ **Approve** → proceed to `/speckit.implement`

> **Optional gate:** between `/speckit.tasks` and `/speckit.implement` you can run
> `/speckit.analyze` — a read-only consistency check across `spec.md`, `plan.md`, and
> `tasks.md` that flags coverage gaps and contradictions before any code exists. It's the
> last cheap place to catch a drifted artifact. See §1.2a.

---

### 0.7 Demo Step 5: Implement with Human Review

> **⏱ ~5 minutes**

**What you'll see:** Copilot generates code for ONE task. We review the diff, verify it compiles, and approve it. This is the HITL gate in action.

**Prompt to Copilot:**

```text
/speckit.implement task 1

Implement the particle data types header as specified in specs/particle-vfx/tasks.md Task 1.
File: include/engine_demo/vfx/particle.h

Constraints (from constitution):
- No #include of std:: headers except <cstdint>, <cstddef>
- No exceptions, no RTTI
- 32-byte aligned particle struct
- All enums are `enum class` with explicit underlying type
```

**Watch Copilot produce `include/engine_demo/vfx/particle.h`:**

```cpp
#pragma once

#include <cstdint>
#include <cstddef>
#include <engine_demo/allocator.h>

namespace engine_demo::vfx {

/// Per-particle visual state. 32-byte aligned for cache-line efficiency.
struct alignas(32) particle {
    float   position[2];         // x, y
    float   velocity[2];         // vx, vy
    float   lifetime_remaining;  // seconds; dead when <= 0
    uint8_t color[4];            // RGBA
    float   size;
    uint8_t _pad[4];             // pad to 32 bytes
};
static_assert(sizeof(particle) == 32, "particle must be exactly 32 bytes");

/// Emitter shape discrimination (Article 2: no RTTI, use enum)
enum class emitter_shape : uint8_t { point, cone, sphere };

/// Status returned by emit operations (Article 1: no exceptions)
enum class emit_status : uint8_t { ok, pool_exhausted, lifetime_expired };

/// Configuration for emitter construction
struct emitter_config {
    emitter_shape   shape;
    uint32_t        max_particles;
    float           emit_rate;          // particles per second
    float           particle_lifetime;  // seconds
    float           origin[2];          // x, y
    float           spread_angle;       // radians (cone shape only)
    float           radius;             // sphere shape only
    uint64_t        seed;               // RNG seed for determinism (Article 5)
};

} // namespace engine_demo::vfx
```

**HITL Review Checklist:**

- [x] No `std::` includes beyond allowed subset
- [x] `static_assert(sizeof(particle) == 32)` present
- [x] `enum class` with explicit underlying type
- [x] No default constructors
- [x] Compiles with `-fno-exceptions -fno-rtti`

**Run the build:**

```bash
cmake --build --preset default-debug
# ✅ Compiles clean
```

> **Facilitator says:** "60 lines. Clean compile. Every constitutional article respected. Task 1 done. We'd commit this, then proceed to Task 2. Each subsequent task follows the same cycle: prompt → generate → review → approve/reject."

✅ **Task 1 Approved** → commit and proceed

**After all 5 tasks are approved — merge back to the demo branch:**

```bash
git checkout demo-01
git merge --no-ff 001-particle-vfx-subsystem
ctest --preset default-debug --output-on-failure   # merge gate: everything green
```

Feature 001 is now on `demo-01`; `main` never moved. This is the first of the two merges
your demo branch collects (see the [Workshop Finale](#workshop-finale-two-features-one-branch)).

---

### 0.8 What Just Happened?

You just watched Spec-Kit drive a feature from idea to working code in 5 stages:

```text
┌─────────────┐    ┌──────────┐    ┌────────┐    ┌────────┐    ┌─────────────┐
│/constitution│───▶│ /specify  │───▶│ /plan  │───▶│ /tasks │───▶│ /implement  │
└─────────────┘    └──────────┘    └────────┘    └────────┘    └─────────────┘
       │                 │               │              │               │
    Binding          Formal         Architecture     Right-sized      Code +
    rules            spec with      decisions +      work items       tests
                     acceptance     memory layout    (≤150 lines)
                     criteria
```

**What made this different from "just asking Copilot to code something":**

| Without Spec-Kit                                      | With Spec-Kit (what you just saw)                    |
| ----------------------------------------------------- | ---------------------------------------------------- |
| "Add a particle system" → 600+ line monolithic diff   | 5 tasks × ~80 lines = reviewed, tested increments    |
| Copilot might use `std::vector` (violating Article 3) | Constitution caught at spec stage, never generated   |
| No performance guarantee                              | Spec mandates < 2ms/500 particles — falsifiable      |
| No way to know if output matches intent               | Acceptance criteria are checkable yes/no             |
| Rollback means deleting everything                    | Rollback means fixing ONE spec line and regenerating |

**The key insight:** Specifications constrain generation. The tighter the spec, the less room for hallucination. You just saw zero constitutional violations because the AI had no room to violate them — the constraints were explicit, measurable, and gated by human review at every transition.

> **Up next in Part 1:** We'll unpack the principles behind what you just watched — WHY this works, how context windows and hallucination mitigation make specifications essential, and the full Copilot ecosystem that powers the workflow.

---

## Part 1: Foundations — What is Spec-Kit?

> Now that you've seen Spec-Kit in action, let's unpack the methodology, principles, and tooling behind what you observed.

### 1.1 The Problem Spec-Kit Solves

Modern AI coding assistants can generate code at remarkable speed, but without discipline they produce:

- **Drift** — code that gradually diverges from architectural intent
- **Compounding errors** — a vague requirement at the start cascades into dozens of bugs downstream
- **Ungovernable output** — no human can review a 2000-line PR generated in one shot

**Spec-Kit** is a methodology for specification-driven AI development. It structures the interaction between a human architect and an AI coding agent into five discrete, reviewable stages — each with a human approval gate. The result is code that is:

1. **Traceable** — every line connects back to a specification
2. **Reviewable** — diffs are right-sized (< 150 lines) and contextual
3. **Deterministic** — the same spec produces consistent output
4. **Recoverable** — when something goes wrong, you roll back to the spec (not the code)

### 1.1a The Why and the Tradeoffs

#### Pros

| Benefit                     | How Spec-Kit Delivers It                                                                                 |
| --------------------------- | -------------------------------------------------------------------------------------------------------- |
| **Focused output**          | Each `/implement` task targets a single concern with explicit acceptance criteria — no meandering code   |
| **Architectural alignment** | The constitution binds every stage; drift is structurally impossible if gates are respected              |
| **Reviewability**           | Right-sized diffs (≤ 150 lines) mean a human can actually read and reason about every change             |
| **Onboarding ramp**         | New team members read the spec and constitution to understand _why_ code exists, not just _what_ it does |
| **Deterministic outcomes**  | The same spec + constitution produces consistent output across different sessions and developers         |
| **Cheap rollback**          | When something is wrong, you fix the spec (cheap) not the code (expensive)                               |

#### Cons

| Cost                           | Mitigation                                                                                                                                         |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Upfront time investment**    | Writing a spec takes 10-20 min. But a single vague prompt followed by 2 hours of debugging costs more. The ROI is 3-5× on any feature > 100 lines. |
| **Learning curve**             | The five-stage flow takes 2-3 repetitions to internalize. Part 2's hands-on features provide that practice.                                        |
| **Overhead for trivial tasks** | A 20-line bug fix doesn't need five stages. Scale the methodology to the task.                                                                     |
| **Discipline fatigue**         | HITL gates feel slow. That friction is intentional — it catches errors when they're cheapest to fix.                                               |

#### When NOT to Use Full Spec-Kit

Not every task needs all five stages. The methodology scales:

- **One-off scripts (< 50 lines):** Skip to `/implement` with a one-sentence prompt
- **Exploratory prototypes:** Use `/specify` loosely — skip the constitution check
- **Well-understood bug fixes:** A failing test IS the spec; go straight to `/implement`
- **Multi-system features (> 500 lines):** Use the full five-stage flow — this is where Spec-Kit pays dividends

> **Rule of thumb:** If a feature would produce a PR you'd struggle to review in one sitting, it needs Spec-Kit.

### 1.1b Context Windows and Hallucination Mitigation

#### The Context Window Problem

AI coding agents operate within a **context window** — a finite token budget (typically 100K-200K tokens) that represents their working memory. Every file, conversation turn, and instruction competes for space in that window. When context overflows:

- The AI "forgets" earlier instructions and constraints
- Output quality degrades as architectural decisions made 10 prompts ago fade from memory
- The AI confabulates — filling knowledge gaps with plausible-sounding but incorrect code

```mermaid
block-beta
    columns 1
    block:window["🧠 AI Context Window (100K–200K tokens)"]
        columns 4
        A["📂 Open Files\n& Codebase\n~40K tokens"]:2
        B["💬 Conversation\nHistory\n~30K tokens"]:2
        C["📜 System\nInstructions\n~10K tokens"]
        D["📋 Specs &\nConstitution\n~5K tokens"]
        E["🔧 Tool Outputs\n& Results\n~15K tokens"]
        F["⚠️ OVERFLOW\nForgotten\nContext"]:1
    end
    space
    block:overflow["❌ What Gets Lost When Context Overflows"]
        columns 3
        G["Earlier\nArchitectural\nDecisions"]
        H["Constraint\nRules from\n10 Prompts Ago"]
        I["Data Model\nDetails &\nNaming"]
    end

    window --> overflow

    style A fill:#2d5a87,stroke:#4a90d9,color:#fff
    style B fill:#8b5e3c,stroke:#d4a574,color:#fff
    style C fill:#5c4b8a,stroke:#9b7ed9,color:#fff
    style D fill:#1a6b3c,stroke:#4db876,color:#fff
    style E fill:#7a5c00,stroke:#d4a017,color:#fff
    style F fill:#8b1a1a,stroke:#ff4444,color:#fff
    style G fill:#4a1a1a,stroke:#ff6666,color:#fff
    style H fill:#4a1a1a,stroke:#ff6666,color:#fff
    style I fill:#4a1a1a,stroke:#ff6666,color:#fff
```

_The context window is a zero-sum game. As open files, conversation history, and tool outputs grow, earlier architectural decisions and constraints get pushed out — leading to drift, hallucination, and inconsistency._

#### How Specs Manage Context

Spec-Kit solves context management structurally:

| Mechanism                       | How It Helps                                                                                                                     |
| ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| **Constitution as anchor**      | A short, high-signal document (< 500 tokens) that stays in context and prevents architectural drift                              |
| **Stage isolation**             | Each `/implement` task carries ONLY its task description + spec reference — not the entire codebase history                      |
| **Markdown as source of truth** | The `.md` plan file is a persistent, external memory — the AI re-reads it each stage rather than relying on conversation history |
| **Right-sized tasks**           | Small tasks complete within a single context window — no multi-turn degradation                                                  |

#### How Specs Mitigate Hallucination

AI hallucination occurs when the model generates plausible but incorrect output because it lacks grounding constraints. Specifications provide that grounding:.

1. **Verifiable assertions** — Acceptance criteria like "returns `pool_exhausted` when capacity is exceeded" are checkable. The AI cannot fabricate its way past a failing test.
2. **Constitutional constraints** — Hard rules ("no `std::` containers") give the AI clear boundaries. Violations are immediately detectable at review.
3. **Concrete data models** — When the spec defines `struct particle { float position[2]; ... }`, the AI has no room to hallucinate an incompatible structure.
4. **Measurable performance budgets** — "< 2ms for 500 particles" is a falsifiable claim. The AI must produce code that actually meets it.

> **Key insight:** Specs convert open-ended generation ("add a particle system") into constrained completion ("implement this exact API with these exact guarantees"). Constrained completion hallucinates far less than open-ended generation.

#### Many Approaches to Planning

There is no single "correct" plan shape. The approach depends on the task:

- **Greenfield feature** → Full 5-stage flow (constitution → specify → plan → tasks → implement)
- **Extending existing system** → Start at `/specify` with existing code as context anchor
- **Bug fix with repro** → The failing test IS the spec; jump to `/implement`
- **Cross-cutting refactor** → Heavy `/plan` stage with module mapping; lighter `/specify`
- **Cross-language port** → Reverse-spec first (extract WHAT from source), then forward-implement in target

The methodology provides the stages. Professional judgment decides which stages a given task needs.

### 1.2 The Five-Stage Spec-Kit Flow

```text
┌─────────────┐    ┌──────────┐    ┌────────┐    ┌────────┐    ┌─────────────┐
│ /constitution│───▶│ /specify │───▶│ /plan  │───▶│ /tasks │───▶│ /implement  │
└─────────────┘    └──────────┘    └────────┘    └────────┘    └─────────────┘
       │                 │               │              │               │
       ▼                 ▼               ▼              ▼               ▼
  [Ground Truth]   [Requirements]  [Architecture]  [Work Items]   [Code + Tests]
                                        │              │               │
                                   ┌────┴────┐    ┌────┴────┐    ┌────┴────┐
                                   │HITL GATE│    │HITL GATE│    │HITL GATE│
                                   └─────────┘    └─────────┘    └─────────┘
```

| Stage                   | Input                            | Output                                                      | Gate                                                        |
| ----------------------- | -------------------------------- | ----------------------------------------------------------- | ----------------------------------------------------------- |
| `/speckit.constitution` | Project values, hard constraints | Binding articles document                                   | Human reads aloud, confirms                                 |
| `/speckit.specify`      | Problem statement + constitution | Formal specification (`spec.md`)                            | Human reviews acceptance criteria                           |
| `/speckit.plan`         | Specification + constitution     | Architecture decisions, data structures, test strategy      | **HITL: Human approves before decomposition**               |
| `/speckit.tasks`        | Plan + constitution              | Ordered list of implementable work items (< 150 lines each) | **HITL: Human approves sizing**                             |
| `/speckit.implement`    | One task at a time               | Code diff + test                                            | **HITL: Human reads diff, runs ctest, approves or rejects** |

### 1.2a Optional Quality Commands: Clarify, Analyze, Checklist

The five core stages are the spine of Spec-Kit, but the toolkit ships three **optional
quality commands** that slot between them. They are not extra bureaucracy — each one is a
cheap gate positioned exactly where the Compounding-Error Principle (§1.3) says errors are
cheapest to fix:

```text
┌─────────────┐   ┌──────────┐   ┌────────┐   ┌────────┐   ┌─────────────┐
│/constitution│──▶│ /specify │──▶│ /plan  │──▶│ /tasks │──▶│ /implement  │
└─────────────┘   └──────────┘   └────────┘   └────────┘   └─────────────┘
                         │    ▲              │    ▲
                         ▼    │              ▼    │
                    ┌──────────┐        ┌──────────┐
                    │ /clarify │        │ /analyze │
                    └──────────┘        └──────────┘

                    /checklist — run at any point after /specify
```

| Command              | When to Run                             | What It Produces                              |
| -------------------- | --------------------------------------- | --------------------------------------------- |
| `/speckit.clarify`   | After `/specify`, **before** `/plan`    | Clarifications section written into `spec.md` |
| `/speckit.analyze`   | After `/tasks`, **before** `/implement` | Cross-artifact consistency & coverage report  |
| `/speckit.checklist` | Any point after `/specify`              | Quality checklist ("unit tests for English")  |

**Why each one matters:**

- **`/speckit.clarify`** asks up to 5 targeted questions probing underspecified areas and records the answers in a **Clarifications** section of `spec.md` (formerly `/quizme`). A vague word in the spec becomes 3 wrong tasks downstream (§1.3) — clarify catches it at the cheapest stage.
- **`/speckit.analyze`** is non-destructive: it reads `spec.md`, `plan.md`, and `tasks.md`, then flags requirements with no covering task, tasks with no spec basis, and constitution conflicts. It is the last cheap gate before any code exists.
- **`/speckit.checklist`** generates a custom checklist validating requirements **completeness, clarity, and consistency** — "unit tests for English." It turns spec review from a vibe check into a falsifiable pass/fail list (compare the hand-written gates in §S.5).

**How they interact with the HITL gates:** the optional commands don't replace human
approval — they arm it. `/speckit.clarify` makes the §0.4 spec-review gate sharper by
eliminating ambiguity before you sign off; `/speckit.analyze` gives the §0.6 sizing gate a
machine-checked consistency report to read alongside the task list; `/speckit.checklist`
turns any gate's "does this look right?" into a checklist you can tick.

**When to skip them:** the official guidance is to run `/speckit.clarify` before every
`/speckit.plan` unless you explicitly state you're skipping it (e.g., a spike or throwaway
prototype) — otherwise the agent may block on missing clarifications. For a workshop demo
with a deliberately precise prompt (§0.4), skipping is fine; for real feature work, the
clarify pass routinely pays for itself in avoided rework.

> **Also in the box:** Spec-Kit ships two more workflow commands you may see in command
> listings — `/speckit.taskstoissues` (convert `tasks.md` into GitHub issues for tracking)
> and `/speckit.converge` (assess the codebase against spec/plan/tasks and append the
> remaining work as new tasks). They extend the workflow rather than gate it, so this
> workshop doesn't use them.

### 1.3 Core Principles

#### The Compounding-Error Principle

> Errors introduced early in the spec cascade exponentially downstream.

A vague word in `/specify` becomes an ambiguous architecture choice in `/plan`, which becomes 3 wrong tasks in `/tasks`, which becomes 15 broken lines in `/implement`. **Fix errors at the earliest possible stage.**

#### Human-In-The-Loop (HITL)

Every stage transition requires human approval. This is not a suggestion — it is the methodology. The friction is the feature. HITL gates:

- Catch misunderstandings before code is written
- Maintain architectural coherence across features
- Ensure the AI's interpretation matches the human's intent
- Provide natural checkpoints for rollback

#### Stage Roll-Back Discipline

> If a task is rejected 3× in a row, the **spec** is wrong — not the code.

When implementation repeatedly fails to satisfy intent, the problem is upstream:

1. First rejection → re-prompt with clearer acceptance criteria
2. Second rejection → examine the plan for misaligned architecture
3. Third rejection → **roll back to `/specify`** and tighten the specification

#### Right-Sizing

Tasks must produce diffs of **≤ 150 lines**. Larger tasks are rejected and split. This ensures:

- Every diff is humanly reviewable in < 5 minutes
- Test coverage is granular and meaningful
- Rollback cost is low (one small commit, not a monolithic PR)

### 1.3a Top 10 Best Practices for Spec-Driven Copilot Workflows

These practices distill the Spec-Kit methodology into actionable habits. Each maps directly to a stage or principle covered in this workshop.

| #   | Practice                        | What It Means                                                                                                                                                                  | Where Demonstrated                    |
| --- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------- |
| 1   | **Define Clear Objectives**     | State the exact goal before writing any code. Use `/speckit.specify` with measurable acceptance criteria — not "make it fast" but "< 2ms for 500 particles at 60 FPS."         | §0.4 `/specify` demo                  |
| 2   | **Modularize the Plan**         | Break work into small, digestible chunks. Every task produces a diff of ≤ 150 lines — right-sized for human review and cheap to roll back.                                     | §0.6 `/tasks` decomposition           |
| 3   | **Establish Constraints**       | Explicitly state performance, stylistic, or architectural boundaries. The `/speckit.constitution` stage encodes these as binding articles that gate every downstream decision. | §0.3 constitution grounding           |
| 4   | **Draft a Markdown Plan**       | Always maintain a central `.md` file as the source of truth. The `spec.md`, `plan.md`, and `tasks.md` files ARE the project memory — persistent, versionable, diffable.        | §0.4–0.6 spec/plan/tasks outputs      |
| 5   | **Iterate on the Spec**         | Review and refine the plan with Copilot before executing it. Stage roll-back discipline: if implementation fails 3×, the spec is wrong — fix upstream.                         | §2.2 Rejection flow                   |
| 6   | **Provide Contextual Anchors**  | Point Copilot to relevant existing files or APIs. Use `copilot-instructions.md`, `.instructions.md` files, and explicit file references in prompts to keep the AI grounded.    | §1.5 Custom instructions architecture |
| 7   | **Sequential Execution**        | Have the agent tackle one stage at a time. `/speckit.implement` runs one task, pauses for review, then proceeds. Never batch multiple tasks into a single generation.          | §0.7 `/implement` with HITL           |
| 8   | **Implement Strict Guardrails** | Define what the AI should not touch or modify. `AGENTS.md` declares hard rules; `--deny-tool` in CLI prevents dangerous operations. The AI knows its boundaries.               | §1.5 AGENTS.md; Appendix B CLI flags  |
| 9   | **Test-Driven Prompts**         | Include unit testing requirements within the spec itself. Every spec has a §Test Plan; every task has acceptance gates that include running tests.                             | §0.4 spec §7 Test Plan                |
| 10  | **Human-in-the-Loop Review**    | Validate each completed stage before moving to the next. HITL gates between every stage transition are non-negotiable — the friction catches errors at their cheapest.         | §1.3 HITL principle; every stage gate |

#### Applying the Top 10 in Practice

**Before coding:** Practices 1, 3, 4 (define objectives, constraints, and plan document)
**During planning:** Practices 2, 5, 6 (modularize, iterate, anchor context)
**During execution:** Practices 7, 8, 9 (sequential, guardrails, test-driven)
**At every transition:** Practice 10 (human review)

> **The workflow in one sentence:** Write a markdown plan (4) with clear objectives (1) and constraints (3), iterate on it (5) with contextual anchors (6), then release the agent to execute sequentially (7) within guardrails (8), testing at every step (9), with human approval at every gate (10), in right-sized chunks (2).

### 1.4 GitHub Copilot Ecosystem Integration

Spec-Kit leverages multiple GitHub Copilot surfaces:

| Surface                    | Role in Spec-Kit                                                                                                                                     |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Copilot Chat (VS Code)** | Primary interface for `/specify`, `/plan`, `/tasks`, `/implement` stages. Agent mode with workspace context.                                         |
| **Copilot Cloud Agent**    | Autonomous task execution from GitHub Issues. Respects `AGENTS.md` and `copilot-instructions.md`. Creates PRs for HITL review.                       |
| **Copilot CLI**            | Terminal-based agentic interface. Plan mode for architecture analysis. Programmatic mode for scripted spec generation.                               |
| **Custom Instructions**    | `.github/copilot-instructions.md` encodes the constitution. `AGENTS.md` enforces hard rules. `.instructions.md` files target specific file types.    |
| **MCP Servers**            | Extend Copilot with external tools (linters, build systems, test runners) for validation during `/implement`.                                        |
| **Prompt Files**           | `.prompt.md` files encode reusable spec-kit workflows as parameterized templates.                                                                    |
| **Custom Agents**          | Specialized personas (this repo ships the `speckit.*.agent.md` stage agents in `.github/agents/`) for spec validation, planning, and implementation. |

### 1.5 Copilot Custom Instructions Architecture

The custom instructions hierarchy enforces constitutional rules at every level:

```text
Repository Root
├── .github/
│   ├── copilot-instructions.md        ← Repository-wide rules (applies to ALL Copilot interactions)
│   ├── instructions/
│   │   ├── cpp-impl.instructions.md   ← Applies to src/**/*.cpp, include/**/*.h
│   │   └── tests.instructions.md      ← Applies to **/test_*.cpp
│   ├── prompts/
│   │   └── speckit.*.prompt.md        ← The /speckit.* slash commands
│   └── agents/
│       └── speckit.*.agent.md         ← Spec-Kit stage agents
├── AGENTS.md                          ← Agent-level hard rules (Cloud Agent + CLI)
├── .specify/
│   └── memory/constitution.md         ← Machine-readable constitution (read by /speckit.*)
├── specs/
│   └── constitution.md                ← The Spec-Kit ground truth (human-facing)
└── src/
    └── ...
```

**Key relationships:**

- `copilot-instructions.md` — Tells Copilot HOW to work (build commands, project structure, coding standards)
- `AGENTS.md` — Tells autonomous agents WHAT they must never violate (hard rules, files not to edit)
- `constitution.md` — The spec-kit ground truth that binds every stage output
- `.instructions.md` files — Path-specific rules (e.g., "all test files must use GoogleTest table-driven patterns")

### 1.6 GitHub Copilot CLI — The Terminal Agent

GitHub Copilot CLI is a powerful agentic interface that operates directly in your terminal. Key capabilities relevant to Spec-Kit:

#### Interactive Mode

```bash
$ copilot
# Opens interactive session with full agentic capabilities
# Press Shift+Tab to toggle PLAN MODE
```

#### Plan Mode

In plan mode, Copilot analyzes your request, asks clarifying questions, and builds a structured implementation plan **before writing any code**. This maps directly to the `/plan` stage of Spec-Kit.

#### Programmatic Mode

```bash
copilot -p "Generate a spec.md for the particle system" --allow-tool='write'
```

Pass a single prompt on the command line. Copilot completes the task and exits. Ideal for scripting spec-kit workflows.

#### Key Commands

| Command     | Purpose                                 |
| ----------- | --------------------------------------- |
| `/plan`     | Switch to plan mode (or Shift+Tab)      |
| `/fleet`    | Parallelize independent tasks for speed |
| `/delegate` | Hand off a task to run autonomously     |
| `/pr`       | Create/manage pull requests from CLI    |
| `/compact`  | Compress conversation context           |
| `/context`  | View token usage breakdown              |
| `/model`    | Switch AI model                         |

#### Tool Permissions

```bash
# Allow specific tools for spec generation
copilot -p "Analyze the ECS and generate migration spec" \
  --allow-tool='shell(cat)' \
  --allow-tool='shell(find)' \
  --allow-tool='write'

# Full autonomy (use with caution)
copilot --allow-all-tools --deny-tool='shell(rm)' --deny-tool='shell(git push)'
```

### 1.7 The Reference Project: engine_demo

All three use cases in this workshop are built around `engine_demo` (the `speckit-workshop-demo` repository) — a synthetic C++20 game engine designed for teaching:

**What it is:** A 2D physics sandbox with 4 playable scenes (Rope, Pendulum Tower, Cloth, Particle Storm) using verlet integration, EASTL containers, and a custom arena allocator.

**Architecture (6 subsystems):**

| Subsystem    | Header                 | Purpose                                          |
| ------------ | ---------------------- | ------------------------------------------------ |
| Allocator    | `allocator.h`          | Arena-style linear allocator, EASTL-compatible   |
| ECS World    | `ecs/world.h`          | Generational entity handles, O(1) create/destroy |
| Physics      | `physics/constraint.h` | Position-projection constraint solver            |
| Sim Loop     | `sim/game_loop.h`      | Fixed-step accumulator (60 FPS default)          |
| RNG          | `sim/rng.h`            | Seeded Mersenne Twister for determinism          |
| Frame Budget | `frame_budget.h`       | Rolling 64-frame timing window                   |

**Constitutional Rules (8 articles):**

1. No exceptions (`-fno-exceptions`)
2. No RTTI (`-fno-rtti`)
3. EASTL-first containers
4. Allocator-aware (explicit allocator at construction)
5. Determinism (explicit seeds, `double` accumulators)
6. Real-time (no allocation in inner loops, ≤16.67ms frame budget)
7. Test-first (GTest for every public function)
8. HITL gates (human approval between spec-kit stages)

**Build:**

```bash
cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

---

## Part 2: Hands-On Practice — Adding More Features

> You saw the full Spec-Kit flow in the demo (Part 0). You understand the principles from Part 1. Now apply the methodology yourself with two additional features and learn what happens when things go wrong.

### 2.1 Session Overview

|                  |                                                                                       |
| ---------------- | ------------------------------------------------------------------------------------- |
| **Objective**    | Practice the Spec-Kit flow independently and learn the rejection/roll-back discipline |
| **Duration**     | 60 minutes                                                                            |
| **Features**     | (1) Lockless Ring Buffer, (2) Fixed String — pre-staged in `specs/`                   |
| **Key Learning** | "When implementation fails 3×, the spec is wrong — not the code"                      |

**Prerequisites:**

- Completed the live demo (Part 0) — you've seen the full flow end-to-end
- Completed Part 1 — you understand the principles behind HITL gates, right-sizing, and stage roll-back
- speckit-workshop-demo repository cloned and building (`ctest` green)
- Checked out on your `demo-0x` branch (§0.2a) — each Part 2 feature branches off it and merges back to it, never to `main`

---

### 2.2 Rejection and Re-Specification Flow

**Scenario:** During Task 2 implementation, Copilot generates code that uses `std::function` instead of `eastl::function` for the force applicator:

```cpp
// ❌ VIOLATION: Article 3 — EASTL-first
using force_fn = std::function<vec2(particle const&, double)>;
```

**First Rejection:**

> "Rejected. Article 3 requires EASTL-first. Replace `std::function` with an EASTL-compatible callable per Article 4."

Copilot regenerates with `eastl::fixed_function` (EASTL's `eastl::function` has no allocator template parameter; `fixed_function` stores the callable inline, which also satisfies Article 6):

```cpp
// ✅ Corrected (inline storage, no heap, no allocator needed)
using force_fn = eastl::fixed_function<32, vec2(particle const&, double)>;
```

**What if rejected 3×?**

If the third attempt still violates (e.g., Copilot keeps reaching for heap-backed callables):

> **Roll back to `/speckit.specify`.**

The facilitator identifies: "The spec says forces are callables but doesn't specify HOW they are stored without allocating. We need to tighten §5.2."

**Tightened spec §5.2:**

```markdown
### 5.2 Force Application (Revised)

Forces are applied during `tick()`. Each force is a stateless function pointer (not a
capturing lambda) to avoid callable-storage complexity entirely:

    using force_fn = vec2(*)(particle const&, double t);

Emitter holds `eastl::vector<force_fn, engine_demo::allocator>` added via `add_force()`.
Stateful forces (e.g., turbulence with seed) use a pre-configured closure stored
outside the emitter and referenced via function pointer.
```

This is the **stage roll-back discipline** in action: the spec was imprecise, so we fix the spec, not the code.

---

### 2.3 Feature 2: Lockless Ring Buffer (Condensed)

> Pre-staged skeleton: [`specs/lockless-ring-buffer/`](../specs/lockless-ring-buffer/) — run the
> five-stage flow live against its brief.

**Spec Summary:**

A single-producer / single-consumer lockless ring buffer for cross-thread message hand-off
in `engine_demo` (e.g., sim thread → audio/render thread). Capacity is fixed at
construction; storage comes from an `engine_demo::allocator&`. Backpressure policy is
**drop-oldest** when full (configurable as a stretch goal).

**Constitution Application:**

- Article 6 is paramount: no allocation and no locks after construction
- Article 5: event ordering must be deterministic (FIFO, single producer)
- Article 1: overflow returns `ring_status::full` (or drops oldest), never throws
- Article 4: backing storage allocated once from the explicit allocator

**Key Spec Excerpt:**

```markdown
## Lockless Ring Buffer — Specification

### Data Model

enum class ring_status : uint8_t { ok, full, empty };

template <typename T>
class ring_buffer; // capacity fixed at construction, power-of-two

### Behavioral Contract

- Fixed-capacity ring buffer (default 256 slots)
- `push(T const&)` → `ring_status` (producer thread only)
- `pop()` → `eastl::optional<T>` (consumer thread only)
- Atomics protocol: acquire/release on head/tail indices; no mutexes
- Backpressure: drop-oldest when full (configurable)
```

**Task Count:** 4–6 tasks (data types + config, atomics protocol, push/pop core, tests)

---

### 2.4 Feature 3: Fixed String (Condensed)

> Pre-staged skeleton: [`specs/fixed-string/`](../specs/fixed-string/) — same five-stage
> cadence.

**Spec Summary:**

A stack-allocated, fixed-capacity string type for `engine_demo` debug labels and
small-string scenarios. Capacity is a non-type template parameter. Never allocates;
interoperates with `eastl::string_view`.

**Key Spec Excerpt:**

```markdown
## Fixed String — Specification

### Data Model

template <size_t Capacity>
class fixed_string; // stack storage: char data[Capacity + 1]

enum class append_status : uint8_t { ok, truncated };

### Behavioral Contract

- `append(eastl::string_view)` → `append_status` (truncates at capacity, never throws)
- `view()` → `eastl::string_view` (interop boundary, Article 3)
- `size()`, `capacity()`, `clear()` — all noexcept, all O(1)
- Compiles under -fno-exceptions -fno-rtti; zero heap usage (Articles 1, 2, 6)
```

**Task Count:** 4 tasks (class skeleton + storage, append/truncation semantics,
string_view interop, tests + stretch goal wiring)

---

### 2.5 Reflection and Key Takeaways

| Question                                              | Expected Answer                                                                     |
| ----------------------------------------------------- | ----------------------------------------------------------------------------------- |
| "How much code did we write manually?"                | Zero. All code was generated by Copilot within spec constraints.                    |
| "How many constitutional violations slipped through?" | Zero (or very few caught by HITL gates).                                            |
| "What was the largest single diff?"                   | < 150 lines — every task was right-sized.                                           |
| "When implementation failed, what did we fix?"        | The spec, not the code (stage roll-back discipline).                                |
| "Could we have gotten this quality without Spec-Kit?" | Unlikely — a single "add particle system" prompt would produce ungovernable output. |

**Key Insight:** Spec-Kit converts the problem of "review 1000 lines of AI-generated code" into "review 5 × 100-line diffs, each backed by a specification and constitutional constraint."

---

## Part 3: Use Case — Designing a New Game from an Existing Foundation

### 3.1 Session Overview

|                  |                                                                                                      |
| ---------------- | ---------------------------------------------------------------------------------------------------- |
| **Objective**    | Use Spec-Kit to design a complete new game ("Orbital Arena") using the existing engine as foundation |
| **Duration**     | 90 minutes                                                                                           |
| **Scenario**     | Competitive 2D physics game: players control orbital gravity wells to capture particles              |
| **Key Learning** | Spec-Kit prevents scope creep and ensures new designs respect existing architecture                  |

**Prerequisites:**

- Completed Parts 0–2 (understand the 5-stage flow and practiced it hands-on)
- Familiarity with engine_demo subsystems (ECS, physics, RNG, frame budget)
- Understanding of the constitutional model
- **Features 001 (Particle VFX) and 002 (Sandbox VFX Visualization) merged into your working branch** — Part 3 reuses `vfx::emitter`/`vfx::particle_pool` as the particle field, and the sandbox scene infrastructure for visualization. On a fresh clone, merge them first and confirm a green `ctest` baseline before starting.

> **Field-tested (2026-07-07):** this entire Part was executed end-to-end on branch `part3-orbital-arena` (feature branch `006-orbital-arena`, artifacts in `specs/003-orbital-arena/`). Callouts marked **Field note** below record where reality differed from the original script. Net result: a complete, playable Orbital Arena — 20 tasks, 148 tests, zero unplanned compile errors, tick cost 0.08 ms against the 16.67 ms Article 6 budget.


**Value Proposition:** Starting a new game without Spec-Kit leads to "blank page paralysis" followed by ad-hoc decisions that conflict with the engine's design principles. Spec-Kit forces you to explicitly state what you're building BEFORE you build it, ensuring the new game inherits the engine's architectural strengths.

---

### 3.2 Analyzing the Existing Engine

**Before writing any specification, catalog what already exists:**

| Subsystem                    | Reusable As-Is? | Adaptation Needed?                                   |
| ---------------------------- | --------------- | ---------------------------------------------------- |
| `engine_demo::allocator`     | ✅ Yes          | None — arena allocator is game-agnostic              |
| `ecs::world`                 | ✅ Yes          | None — generational handles work for any entity type |
| `physics::constraint_solver` | ✅ Yes          | None — it stays a verlet body/constraint system      |
| `vfx::particle_pool/emitter` | ✅ Yes          | Reused as the free-particle field; gravity wells apply forces over its spans |
| `sim::game_loop`             | ✅ Yes          | None — fixed-step accumulator is universal           |
| `sim::rng`                   | ✅ Yes          | None — seeded RNG for replay works for competitive   |
| `frame_budget`               | ✅ Yes          | None — timing telemetry is game-agnostic             |
| Sandbox scenes               | ❌ No           | Replace with Orbital Arena scenes                    |
| Sandbox HUD                  | ⚠️ Partial      | Replace HUD content, keep rendering infrastructure   |

> **Field note:** an earlier draft of this table mapped gravity wells onto `physics::constraint_solver` ("add a new force type"). The real `/speckit.plan` run corrected this: `constraint_solver` is a verlet **body/constraint** system, while the free-floating particles actually live in `vfx::particle_pool`. Gravity wells became a **new module** (`orbital_arena::gravity_well`) that applies radial forces over particle-pool spans — the same pattern the sandbox's `particle_storm` scene already uses. Cataloging is a hypothesis; the plan stage is where Copilot verifies it against real headers.

**Key Insight:** ~70% of the engine is reusable. Spec-Kit helps us focus the new specification on the **delta** — only what's new or changed.

---

### 3.3 Writing a New Constitution

**The new game inherits articles 1-6 unchanged** (they are engine-level rules). We ADD game-specific articles:

```markdown
# Orbital Arena — Constitution

## Inherited Articles (from engine_demo)

Articles 1–6 apply unchanged:

1. No exceptions
2. No RTTI
3. EASTL-first
4. Allocator-aware
5. Determinism
6. Real-time budgets

## Article 7 — Test-First (Modified)

Every public function has at least one GTest. Additionally, every **game rule** has a
deterministic replay test that validates correct scoring across 1000 simulated frames.

## Article 8 — HITL Gates (Unchanged)

Spec-Kit pauses for human approval between stages.

## Article 9 — Competitive Fairness (NEW)

No game mechanic may provide asymmetric advantage based on player index. All players
have identical starting conditions. Randomization (power-up spawning) uses a shared
seed visible to all players for verifiability.

## Article 10 — Lockstep Replay (NEW)

The game state must be fully deterministic from initial seed + input sequence. Any
desync between two clients replaying the same input log is a critical bug. State
hashing every 60 frames for drift detection.

## Article 11 — Spectator-Safe State (NEW)

All game state is representable as a flat serializable snapshot (no pointers, no handles
crossing frame boundaries). This enables replay, spectating, and rollback netcode.
```

**Why a new constitution?** The original constitution governs a physics sandbox demo. Orbital Arena is a competitive game with fairness, replay, and networking concerns that don't exist in a single-player sandbox. The constitution captures these new non-negotiable constraints.

> **Field note — where the constitution lives.** The original script never said. What worked: the human-facing copy at `specs/orbital-arena/constitution.md`, **and** the new articles merged into `.specify/memory/constitution.md` (the file the `/speckit.*` agents actually read), scoped with "applies to `orbital_arena` code only." If you skip the machine-copy sync, `/speckit.plan`'s constitution check silently runs against articles 1–8 only and Articles 9–11 are never enforced.

---

### 3.4 /speckit.specify — New Game Features

**Prompt to Copilot:**

```text
/speckit.specify

Design "Orbital Arena" — a competitive 2D physics game built on the engine_demo foundation.

Game concept: 2-4 players each control a gravity well. Free-floating particles are
attracted to wells. Players score by capturing particles into their well. Power-ups
spawn periodically. First to 100 points wins.

Use the Orbital Arena constitution (articles 1-11). Reference existing engine_demo
subsystems where reusable. Only specify the DELTA — what's new.

Features to specify:
1. Gravity Well entity (player-controlled, variable strength)
2. Scoring system (capture detection, point values, win condition)
3. Power-up system (spawn logic, types, duration)
4. Match state machine (lobby → countdown → playing → game_over)
5. Input system (player input → well control mapping)
```

> **Field note — branch vs. spec-dir numbering can diverge.** On a repo with leftover feature branches, the git hook numbers the **branch** by scanning `git branch -a` (here: `006-orbital-arena`), while the specify workflow numbers the **spec directory** by scanning `specs/` (here: `specs/003-orbital-arena/`). Downstream commands follow `.specify/feature.json`, but the helper scripts (`setup-plan.sh`, `setup-tasks.sh`, `check-prerequisites.sh`) recompute the path from the branch name and will error or create a stray `specs/006-…` directory. Don't panic, don't "fix" the numbers — point the command at the directory recorded in `.specify/feature.json` and delete any stray dir.

**Expected Output — Full Specification (excerpt for Scoring System):**

```markdown
# Orbital Arena — Scoring System Specification

## 1. Purpose

Track particle captures per player, assign point values, detect win conditions,
and maintain a deterministic score history for replay verification.

## 2. Constitutional Constraints

| Article             | Application                                                                                                                    |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| 5 — Determinism     | Score state derived purely from physics state; no floating-point comparison for capture detection (use distance² < threshold²) |
| 9 — Fairness        | All wells have identical capture radius; point values are symmetric                                                            |
| 10 — Lockstep       | Score state included in frame hash; any desync is critical                                                                     |
| 11 — Spectator-Safe | Score snapshot is a flat array: `uint32_t scores[MAX_PLAYERS]`                                                                 |

## 3. Data Model

struct capture_event {
uint32_t player_index;
uint32_t particle_id; // entity handle from ECS
uint32_t point_value; // base: 1; power-up modified
uint64_t frame_number; // for replay verification
};

struct score_state {
uint32_t scores[4]; // MAX_PLAYERS = 4
uint32_t capture_count[4];
uint32_t target_score; // win condition (default: 100)
};

enum class match_result : uint8_t { in_progress, player_won, draw_timeout };

## 4. Behavioral Contracts

### 4.1 Capture Detection

A particle is "captured" when `distance²(particle.pos, well.pos) < well.capture_radius²`.
Capture detection runs AFTER physics tick, BEFORE score update. Captured particles are
destroyed (entity removed from ECS world).

### 4.2 Scoring Rules

- Base capture: 1 point
- Double-point power-up active: 2 points
- Chain bonus (3+ captures in 1 frame): +1 bonus per capture beyond 2nd

### 4.3 Win Condition

- `scores[i] >= target_score` → `match_result::player_won`
- Frame count exceeds timeout → `match_result::draw_timeout` (highest score wins; tie = draw)

## 5. Test Plan

| Test                        | Validates                                                   |
| --------------------------- | ----------------------------------------------------------- |
| `test_single_capture`       | One particle enters well → score increments by 1            |
| `test_simultaneous_capture` | Two particles same frame → both counted                     |
| `test_chain_bonus`          | 3 captures in 1 frame → 1 + 1 + 2 = 4 points                |
| `test_win_condition`        | Score reaches target → match_result::player_won             |
| `test_fairness`             | Symmetric setup → player index doesn't affect capture logic |
| `test_replay_determinism`   | Same seed + inputs → identical score history                |
```

---

### 3.5 /speckit.plan — Mapping Foundation to New Architecture

**Key planning insight:** Show which existing modules map directly and which need new code.

````markdown
# Orbital Arena — Architecture Plan

## Module Mapping

| New Feature         | Builds On                       | New Code                                    |
| ------------------- | ------------------------------- | ------------------------------------------- |
| Gravity Well        | `physics::constraint_solver`    | New force type: `radial_attractor`          |
| Scoring             | `ecs::world` (entity queries)   | `scoring.h/cpp` — capture detection + state |
| Power-ups           | `sim::rng` + `ecs::world`       | `powerup.h/cpp` — spawn logic + types       |
| Match State         | New (no equivalent)             | `match.h/cpp` — state machine               |
| Input               | New (sandbox used direct mouse) | `input.h/cpp` — player→well mapping         |
| Particles (targets) | `vfx::emitter` from UC1         | Reuse emitter as particle spawner           |

## Dependency Graph

```

                    ┌───────────────┐
                    │ match (state) │
                    └───────┬───────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
    ┌─────────▼───┐  ┌─────▼─────┐  ┌───▼────────┐
    │   input     │  │  scoring   │  │  powerups   │
    └─────────────┘  └─────┬─────┘  └───┬────────┘
                            │            │
                    ┌───────▼────────────▼───┐
                    │   physics + gravity    │
                    │   (existing solver)    │
                    └────────────┬───────────┘
                                 │
                    ┌────────────▼───────────┐
                    │   ECS world + RNG      │
                    │   (existing, unchanged)│
                    └────────────────────────┘

```

## Task Decomposition (8 tasks)

| #   | Task                                  | Lines | Deps             |
| --- | ------------------------------------- | ----- | ---------------- |
| 1   | Gravity well component + radial force | ~100  | Existing physics |
| 2   | Capture detection (spatial query)     | ~80   | Task 1           |
| 3   | Scoring state + rules                 | ~90   | Task 2           |
| 4   | Power-up types + spawn logic          | ~110  | Existing RNG     |
| 5   | Match state machine                   | ~100  | Tasks 3, 4       |
| 6   | Input mapping (player → well)         | ~60   | Task 1           |
| 7   | Integration: full match loop          | ~120  | Tasks 5, 6       |
| 8   | Replay determinism test               | ~80   | Task 7           |
````

> **Field note — three things the 8-row table omits, which the real plan had to add:**
>
> 1. **CMake wiring.** A new game library is 4 build-file changes: `src/orbital_arena/CMakeLists.txt` (new static lib), `tests/orbital_arena/CMakeLists.txt` (new CTest dir, **no `GTest::gmock`** — see §S.7's registry gotcha), plus one-line `add_subdirectory` edits in `src/` and `tests/`. There's also a knock-on: any existing test target that compiles `apps/sandbox/scene.cpp` directly must now link the new library. Make this an explicit first task.
> 2. **Snapshot/state-hash module.** Articles 10–11 (lockstep replay, spectator-safe state) need a home — a flat POD `match_snapshot` + FNV-1a `state_hash()`. The plan grew a 9th module the training table never listed.
> 3. **A visualization task** (§3.7a). Without a sandbox scene, the "complete game" is invisible — it exists only as green test output.
>
> The real plan produced **11 work units**; strict Article 7 test-first splitting turned those into **20 tasks** (test task before each impl task). Expect roughly 2× the unit count, not 8.

---

### 3.6 /speckit.tasks — Delta Decomposition

Each task specifies ONLY what's new — never re-implementing existing subsystems.

**Example Task — Gravity Well Component:**

```markdown
## Task 1: Gravity Well Component + Radial Force

**Files:**

- `include/orbital_arena/gravity_well.h` (new)
- `src/orbital_arena/gravity_well.cpp` (new)
- `tests/orbital_arena/test_gravity_well.cpp` (new)

**Deliverables:**

- `struct gravity_well_config { float strength; float capture_radius; vec2 position; uint8_t player_index; }`
- `vec2 radial_force(particle const& p, gravity_well const& well)` — returns force toward well center
- Force magnitude: `strength / distance²` (inverse-square falloff)
- Integrates with existing `physics::constraint_solver` as a new force type

**Acceptance Gate:**

- [ ] `radial_force` is deterministic (Article 5)
- [ ] No allocation in force computation (Article 6)
- [ ] Test: particle at known distance produces expected force magnitude
- [ ] Test: force is symmetric regardless of player_index (Article 9)
- [ ] Compiles with `-fno-exceptions -fno-rtti`
```

---

### 3.7 /speckit.implement — Building on the Foundation

When implementing gravity wells, Copilot can reference the existing physics solver:

```cpp
// Integration point: gravity_well force plugs into existing solver
// File: src/orbital_arena/gravity_well.cpp

#include <orbital_arena/gravity_well.h>
#include <engine_demo/physics/constraint.h>  // existing solver interface
#include <cmath>  // allowed: non-allocating standard math

namespace orbital_arena {

[[nodiscard]] vec2 radial_force(
    engine_demo::vfx::particle const& p,
    gravity_well const& well
) noexcept {
    float const dx = well.position[0] - p.position[0];
    float const dy = well.position[1] - p.position[1];
    float const dist_sq = dx * dx + dy * dy;

    // Avoid division by zero; clamp minimum distance
    float const safe_dist_sq = dist_sq < 1.0f ? 1.0f : dist_sq;
    float const magnitude = well.strength / safe_dist_sq;

    // Normalize direction
    float const dist = std::sqrt(safe_dist_sq);
    return vec2{ (dx / dist) * magnitude, (dy / dist) * magnitude };
}

[[nodiscard]] bool is_captured(
    engine_demo::vfx::particle const& p,
    gravity_well const& well
) noexcept {
    float const dx = well.position[0] - p.position[0];
    float const dy = well.position[1] - p.position[1];
    // Integer-safe comparison: distance² < radius² (Article 5: no float equality)
    return (dx * dx + dy * dy) < (well.capture_radius * well.capture_radius);
}

} // namespace orbital_arena
```

> **Field note:** the sketch above is illustrative — `vec2` doesn't exist in this repo. The real implementation follows house style (`float pos[2]`, matching `vfx::particle`), and every module takes an explicit `engine_demo::allocator&` (Article 4). The measured full-arena tick (500 particles + wells + captures + scoring + power-ups + state machine) came in at **0.08 ms** against the 16.67 ms Article 6 budget — in a Debug build.
>
> **The one real bug of the run** wasn't in generated game logic at all — it was destruction order: the sandbox scene's destructor freed its arena buffer in the destructor *body*, which runs **before** member destructors; the embedded arena's placement-new'd pool then tore down inside freed memory (access violation in 6 tests). Fix: reset the arena member first. Generated code respected the constitution; the integration seam with pre-existing code is where the crash lived — exactly what per-task `ctest` gates are for.

---

### 3.7a Seeing It Run — Visualization and Screenshots

The 8-task decomposition produces a complete, tested game **that you cannot see**. For a workshop, add one final task: a sandbox scene (`scene_kind::orbital_arena`, modeled on the existing `particle_storm` scene) with a scripted 2-player autopilot, well/particle rendering, and a HUD panel showing per-player scores, match state, and winner.

Countdown (tick 120) — neutral particle field, scores 0:

![Orbital Arena — countdown, neutral 500-particle field](screenshots/orbital-arena-early.png)

Game over (tick 1600) — P0 wins 102 : 80, particles tinted by owning well:

![Orbital Arena — game over, P0 winner HUD](screenshots/orbital-arena-late.png)

Run it yourself: `ea-sandbox --scene orbital --seed 42` (key `5` switches interactively; `--screenshot <relative-path> --warmup N` for captures).

**Constraints that made this task honest:**

- Autopilot inputs derive purely from the arena tick index — zero draws from the scene's existing rng, so **all pre-existing scene digests are byte-identical** (verified: rope digest at seed 42 unchanged before/after).
- The orbital scene contributes its own digest (headless runs at the same seed produce identical traces — Article 10 at the app layer).
- Rendering is float-boundary code (Article 5 allows it); game state stays in the deterministic core.

---

### 3.8 How Spec-Kit Prevents Scope Creep

**Without Spec-Kit:** "Let's also add networking... and a level editor... and achievements..."

**With Spec-Kit:** Every feature must:

1. Be specified against the constitution
2. Have measurable acceptance criteria
3. Fit within the task budget (< 150 lines per task)
4. Pass HITL review

If someone says "add networking," the response is: "Write a `/specify` for it. Which constitutional articles apply? What are the acceptance criteria? How many tasks?" This process naturally constrains scope to what's well-defined and achievable.

**The constitution is the scope boundary.** Article 10 (Lockstep Replay) implies networking-readiness but does NOT require a network implementation. The spec explicitly states "replay" not "live multiplayer." Scope is bounded by what the constitution mandates.

> **Field note — the constitution bites back in fun ways.** The first demo autopilot gave player 1 the exact *negation* of player 0's steering. Result: a permanent 495–495 sudden-death tie. Why? The arena implements Article 9 fairness **bit-exactly** — mirrored inputs are guaranteed to produce mirrored outcomes, so no leader could ever emerge. The "bug" was the constitution working perfectly; the fix was detuning the two players' steering frequencies. When a constitutional article is implemented as a hard invariant, even your demo script has to respect it.

---

### 3.9 Reflection and Key Takeaways

| Insight                                                                       | Evidence                                               |
| ----------------------------------------------------------------------------- | ------------------------------------------------------ |
| "70% of the engine is reusable without modification"                          | Field-verified: 6 of 7 subsystems consumed unchanged   |
| "The new constitution adds game-specific rules without breaking engine rules" | Articles 1-6 inherited; 9-11 are additive              |
| "Task count is proportional to actual new code, not total codebase size"      | 20 test-first tasks for a complete game, because foundation exists |
| "Spec-Kit makes the 'use existing code' decision explicit and documented"     | Plan shows module mapping table                        |
| "Detailed contracts before implementation eliminate iteration"                | Field run: zero unplanned compile errors across all 20 tasks; every impl green on first build |
| "The plan stage corrects the analysis stage"                                  | §3.2's constraint-solver mapping was wrong; `/speckit.plan` fixed it against real headers |

---

## Part 4: Use Case — Cross-Language Game Transformation

### 4.1 Session Overview

|                  |                                                                                                  |
| ---------------- | ------------------------------------------------------------------------------------------------ |
| **Objective**    | Transform the **`engine_demo` engine foundation** (ECS, physics constraint solver, game loop, RNG, frame budget, allocator) from C++20 to Rust/Bevy ECS using Spec-Kit as the translation layer |
| **Duration**     | 90 minutes (core) + 45–60 minutes for the optional Phase A2 game-layer extension                |
| **Approach**     | Reverse-spec the C++ → write Rust constitution → plan transformation → implement in Rust         |
| **Key Learning** | Specs are language-agnostic; the same spec can drive implementation in any language              |

> **⚠️ Scope — read this before you start.** This part transforms the reusable **engine
> foundation** living in `include/engine_demo/` and `src/engine_demo/` — the same six
> subsystems reverse-specced in §4.2. It does **NOT** transform the **Orbital Arena game**
> itself (`include/orbital_arena/`: gravity wells, capture/scoring, match lifecycle,
> power-ups, input replay) that Part 3 built on top of that foundation. If you run only
> §4.2–§4.5, the resulting Rust program will look like a generic physics demo (a rigid-link
> "rope" orbiting an anchor) — it will **not** resemble the two-player gravity-well capture
> game from Part 3's screenshots. That is expected, not a bug: "engine" and "game" are
> different layers, and Part 4 as written only exercises the engine layer.
>
> Want the Rust program to actually look like the Part 3 game? Complete the optional
> [§4.2a Phase A2 extension](#42a-phase-a2-extension-reverse-specifying-the-orbital-arena-game-layer)
> after §4.2, which reverse-specs the game layer's visually-defining mechanics (gravity
> wells, capture, scoring) and extends the Rust implementation to match.

**Prerequisites:**

- Completed Parts 1–3
- Basic Rust familiarity (ownership, traits, lifetimes)
- Bevy ECS conceptual understanding (Entities, Components, Systems, Resources)
- GitHub Copilot CLI installed (`copilot --version`)

**Value Proposition:** Rewriting a game in a new language without Spec-Kit means "read the C++ and translate line-by-line" — which produces non-idiomatic target code that doesn't leverage the new language's strengths. Spec-Kit separates WHAT (the specification) from HOW (the implementation), allowing the same spec to produce idiomatic code in any language.

**The transformation pipeline:**

```text
┌──────────────────┐     ┌─────────────────┐     ┌──────────────────┐
│  C++20 / raylib  │────▶│  Spec Documents │────▶│  Rust / Bevy ECS │
│  (source impl)   │     │  (language-free) │     │  (target impl)   │
└──────────────────┘     └─────────────────┘     └──────────────────┘
     Phase A                 Phase B + C              Phase D + E
  (reverse-spec)          (new constitution         (implement in
                           + transform plan)          Rust idioms)
```

---

### 4.2 Phase A: Reverse-Specification

**Goal:** Extract language-agnostic specifications from the existing C++ codebase. We're documenting WHAT the code does, not HOW it does it in C++.

**Prompt to Copilot (VS Code Agent Mode):**

```text
/speckit.specify (reverse)

Analyze the ECS World subsystem in include/engine_demo/ecs/world.h and
src/engine_demo/ecs/world.cpp. Extract a language-agnostic specification that
captures:

1. What entities ARE (behavioral contract, not C++ class definition)
2. What operations are supported (create, destroy, query, iterate)
3. What guarantees the system provides (generation safety, capacity limits)
4. What constraints are constitutional (determinism, real-time, no allocation in inner loops)

Output as a spec document that could be implemented in ANY language — no C++ syntax,
no EASTL references, no pointer semantics. Pure behavioral specification.
```

**Expected Output — `specs/transform/ecs-world.spec.md`:**

```markdown
# ECS World — Language-Agnostic Specification

## 1. Purpose

Manage the lifecycle of game entities: creation, destruction, component attachment,
and iteration. Entities are lightweight handles; components are the data.

## 2. Entity Model

### 2.1 Entity Handle

An entity handle is an opaque identifier composed of:

- **Index** (32-bit unsigned): slot in a fixed-capacity array
- **Generation** (32-bit unsigned): monotonically increasing version counter

Two handles with the same index but different generations refer to DIFFERENT entities.
This prevents use-after-free: destroying entity (index=5, gen=1) and creating a new entity
that reuses slot 5 produces (index=5, gen=2) — the old handle is now invalid.

### 2.2 Capacity

Maximum entity count is fixed at construction (default: 4096). No dynamic resizing.

## 3. Operations

| Operation | Signature (abstract)       | Complexity | Allocates?            |
| --------- | -------------------------- | ---------- | --------------------- |
| create    | `() → entity_handle`       | O(1)       | No (reuses pool slot) |
| destroy   | `(entity_handle) → status` | O(1)       | No                    |
| is_alive  | `(entity_handle) → bool`   | O(1)       | No                    |
| count     | `() → uint32`              | O(1)       | No                    |

### 3.1 Create

- Returns a new entity handle with the next available slot index and current generation
- If pool is full, returns an error status (not an exception)
- Created entities start with no components

### 3.2 Destroy

- Increments the generation counter for the slot (invalidates all existing handles to this entity)
- Returns the slot to the free pool for reuse
- Returns error if handle is already invalid (generation mismatch)

### 3.3 Is Alive

- Returns true IFF handle.index is occupied AND handle.generation matches current slot generation

## 4. Guarantees

| Guarantee                     | Description                                                               |
| ----------------------------- | ------------------------------------------------------------------------- |
| Generation safety             | Stale handles never accidentally reference new entities                   |
| O(1) operations               | Create/destroy are constant time regardless of entity count               |
| No allocation in steady state | After initial pool creation, zero heap operations                         |
| Deterministic ordering        | Iteration order is deterministic given same creation/destruction sequence |

## 5. Constraints (from constitution)

- Pool allocated once at construction from an explicit allocator
- No exceptions — errors return status enums
- Deterministic: same sequence of create/destroy always produces same handle assignments
- Real-time: O(1) with no branches in hot path (create/destroy)
```

**Repeat for each subsystem:**

- `specs/transform/physics-constraint.spec.md`
- `specs/transform/game-loop.spec.md`
- `specs/transform/rng.spec.md`
- `specs/transform/frame-budget.spec.md`
- `specs/transform/allocator.spec.md`

These six specs fully cover the **engine foundation**. If you stop here and go straight to
Phase B, the Rust program you build in §4.5 will be a faithful port of the constraint
solver — not the Orbital Arena game. Continue to §4.2a only if you want the game layer too.

---

### 4.2a Phase A2 (Extension): Reverse-Specifying the Orbital Arena Game Layer

**Goal:** Extend Phase A to the Orbital Arena GAME built in Part 3 (`include/orbital_arena/`)
— specifically the mechanics that define the game's visual identity: player-controlled
gravity wells, capture contention, and scoring/win condition. This is what makes the Rust
screenshot actually look like a two-player capture game instead of a generic physics demo.

**Why this is a separate phase, not part of §4.2:** `include/orbital_arena/` is a
substantially larger surface than the six engine subsystems — it also includes a full
lobby/countdown/game-over state machine (`match.h`), timed power-ups (`powerup.h`), an
input replay log (`input.h`), and a POD snapshot/state-hash for determinism verification
(`snapshot.h`). Reverse-specifying and re-implementing **all** of it is a multi-day
undertaking, not a workshop extension. Phase A2 deliberately reverse-specs only the
mechanics a screenshot can show — gravity wells, capture, scoring — and explicitly
documents the rest as descoped. Treat the omitted subsystems as a "further work" exercise
for after the workshop, not as things this exercise claims to have covered.

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
Phase A) — they do not share formulas.
```

Repeat the same prompt shape against `include/orbital_arena/scoring.h` (capture contention,
award, win evaluation) and `include/orbital_arena/match.h` (lobby/countdown/playing/
game-over state machine — spec it fully even though the Rust port will simplify it; see
the Scope Reduction note below).

**Expected outputs:**

- `specs/transform/orbital-arena-gravity-well.spec.md`
- `specs/transform/orbital-arena-scoring.spec.md`
- `specs/transform/orbital-arena-match.spec.md`

**Documented Scope Reduction (put this in each spec's Constraints section, don't skip it):**
Every Phase A2 spec should explicitly name what it is choosing NOT to cover and why —
power-ups (`powerup.h`), the input replay log (`input.h`), and the snapshot/state-hash
serialization (`snapshot.h`) are real subsystems with real value, but are descoped here so
the exercise stays focused on the mechanics a screenshot can prove were ported correctly.
An honest spec says what it left out; a spec that silently narrows scope teaches students
the wrong lesson about what "done" means. (These particular descopes are later closed by
[Phase A5](#42d-phase-a5-completion-full-fidelity-closure--every-spec-to-recreate-the-game),
which supersedes them **in place, with markers** — the honest-descope discipline is what
makes that later closure auditable.)

**Extending the Rust implementation:** feed these three specs into the same
`/speckit.specify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.implement` pipeline as
§4.5, targeting a new `game` module (gravity wells + a particle field they attract +
capture/scoring) alongside the existing constraint-solver module from §4.2–§4.5 — don't
delete the original module, since it's still a valid, separately-tested demonstration of
Phase A's reverse-specs. A reasonable scope for the extension:

- Two (or more) gravity wells, each driven by a simple deterministic motion pattern (a
  pure function of the fixed tick counter — NOT wall-clock time, per Article 5) since a
  screenshot-capture exercise has no interactive player input to record
- A capacity-reserved field of particles attracted by every active well's
  `radial_acceleration`, integrated the same way as any other sim state
- Capture resolution + scoring exactly per `orbital-arena-scoring.spec.md`, including the
  index-independent tie rule
- A HUD showing each player's score (and the winner, once latched)

**Acceptance gate (in addition to §4.5's per-task gates):**

- [ ] `cargo test` covers the acceleration formula, the capture/tie rule, and the win-latch
      sequence (score freeze after a winner is set) as unit tests — not just visual
      inspection
- [ ] The rendered scene visibly resembles Part 3's reference screenshots: colored wells,
      an attracted particle field, and a score readout
- [ ] The original constraint-solver scene/tests from §4.2–§4.5 still build and pass
      unmodified

---

### 4.2b Phase A3 (Further Extension): HUD Parity and Interactive Control

**Goal:** Close the remaining visual gap between the Rust port and the reference C++
screenshots — the on-screen telemetry HUD and interactive mouse control — surfaced by
directly comparing running screenshots side by side after completing §4.2a. This is the
workshop's second worked example of **"compare the actual output to the reference, then
identify what spec is missing"** rather than declaring victory once *some* screenshot
exists.

**What comparison revealed:** §4.2a's Rust screenshots showed wells, particles, and a bare
score readout — but the reference C++ sandbox (`apps/sandbox/app.cpp`) also renders a rich
telemetry HUD (seed/tick/frame-time with three-tier color coding, a state-digest line, a
top-right counts panel, a bottom control-hint legend, and a frame-budget histogram widget)
on **every** scene, plus a match panel (score + winner) specifically for the orbital arena
scene. None of that was reverse-specced by Phase A or Phase A2 — both of those phases
targeted *simulation* logic, not *display* logic. That is a real, previously-undocumented
gap, found only by looking at the actual pixels.

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

**The interactive-control gap is different in kind, not just in coverage.** Checking the
reference C++ orbital arena scene's actual input handling (`scene.cpp`'s
`make_orbital_inputs`) shows both players are driven by a **scripted sinusoidal
autopilot** — the reference implementation has no mouse-driven well control either. What
the base sandbox DOES have is a generic, scene-agnostic "click and drag the nearest
object" mechanic (`app.cpp`'s `grab_nearest_rope_node` / `drag_held_node`), used for
dragging rope/pendulum/cloth bodies. Wanting the Rust *default scene* to be interactively
explorable — "click on the well and move it around" — is therefore a **new capability**,
not a reverse-spec of anything that already exists. Write it as a spec anyway, tagged
explicitly as New, so its origin is as auditable as every reverse-specced subsystem:

```text
Design a NEW specification (not a reverse-spec — state this explicitly) adapting the
sandbox's generic "press near an object, drag it to the cursor" interaction pattern to
one gravity well in the Rust port, so the default scene is interactively explorable.
Define: grab radius, what happens while held (position follows cursor, clamped to arena
bounds), what happens on release (resumes normal driving logic), and failure-mode
guarantees (missing window/camera/cursor must never panic). Output to
specs/transform/orbital-arena-interactive-control.spec.md.
```

**Extending the Rust implementation:**

- HUD: recreate the CONTENT of `sandbox-hud.spec.md` using the target UI framework's
  native widgets (Bevy: `Text`/`Node` UI, not the reference's immediate-mode drawing
  calls) — the spec's §5 explicitly descopes pixel-perfect layout so the port isn't
  fighting the wrong constraint.
- Interactive control: a small system reading mouse position/buttons, converting screen
  space to world space via the engine's own camera API (never hand-rolled), gated by a
  small state resource so the driving system (whatever moves wells normally) can skip its
  own update for the well currently being dragged.

**Acceptance gate:**

- [ ] Every color-coded metric in the HUD spec has a corresponding three-tier check in
      the Rust implementation (not just "some color changes somewhere")
- [ ] The interactive-control spec is tagged **New** in its own text, not silently
      presented as ported from a C++ header that doesn't have this behavior
- [ ] Cursor/window/camera queries that can legitimately be absent (no primary window, no
      camera, cursor outside the window) are handled without `unwrap()`/panic (Article I)
- [ ] `cargo test` covers the interaction's PURE logic (grab-radius check, position/
      velocity update, bounds clamping) as unit tests — the ECS system that wires mouse
      input to that logic is glue code and, consistent with the reference C++ app's own
      input-handling layer, is verified by manual/visual testing, not GTest/`#[test]`

---

### 4.2c Phase A4 (Further Extension): The Default Sandbox Stage — VFX and Free Particles

**Goal:** Reverse-spec and port the actual *default* base sandbox stage (`ea-sandbox`'s
"rope" scene, the very first screenshot in this training's §0.1) — not the Orbital
Arena game. Compared side by side against the Rust port's `--scene constraint` demo,
two more subsystems turn out to be visually dominant in the reference screenshot and
were never reverse-specced by any prior phase: the pink/blue spark trails (VFX particle
system, "vfx=N" in the HUD) and the bouncing background dots ("particles=N" in the HUD).
This is the workshop's THIRD worked example of the same lesson: a passing screenshot
comparison at a glance is not the same as an audited one.

**What comparison revealed, precisely:**

| HUD counter                | Subsystem                              | Reverse-specced before this phase? |
| ---------------------------- | ----------------------------------------- | --------------------------------------- |
| `bodies=` / `edges=`         | `physics::constraint_solver`              | ✅ Phase A (§4.2)                       |
| `frame_avg` / telemetry text | Sandbox HUD                               | ✅ Phase A3 (§4.2b) — but only wired to the Rust port's `arena` scene, not `constraint` |
| `particles=`                 | Scene-local free-particle ballistic sim   | ❌ Never — genuinely distinct from both the rope solver and VFX |
| `vfx=`                       | `engine_demo::vfx` (Feature 001/002)      | ❌ Never — the original workshop's OWN Part 0/Self-Study Lab feature, never carried into Part 4 at all |

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

**A second, related gap needs its own spec — don't fold it into the VFX spec above, they
are genuinely different subsystems:**

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

**Extending the Rust implementation — this is the workshop's largest single addition,
scope it deliberately:**

- Port the particle pool + gravity-only force application (wind/turbulence explicitly
  descoped per the VFX spec's own §7 — not needed for visual parity with the reference
  screenshots) as a small, independently-tested module.
- Port ONLY the default variant's free-particle force/boundary rule (light gravity +
  bounce + spark-on-hit) — the twin-well and delegated variants are explicitly descoped
  by `sandbox-free-particles.spec.md` §6, since the delegated variant's role is already
  served (differently) by the Orbital Arena game's own particle field from §4.2a.
- Extend the ORIGINAL `--scene constraint` demo (§4.2's rope port) with both: it is the
  Rust equivalent of the exact screenshot this phase targets. Do not add this to the
  `arena` scene — that scene already has its own particle field serving an analogous
  role, and blending the two would contradict `sandbox-free-particles.spec.md`'s rule-
  isolation guarantee.
- Wire the §4.2b HUD onto this scene too, closing the "only wired to `arena`" gap noted
  in the table above, so BOTH Rust scenes show full telemetry.
- Add an interactive burst (mouse click → physics + VFX burst at the cursor), reusing
  the camera/cursor conversion already built for §4.2b's well-drag feature rather than
  re-deriving it — factor that conversion into a small shared helper the first time it's
  needed by a second feature.

**Acceptance gate:**

- [ ] The VFX pool and free-particle force rule are each covered by `#[test]`s
      independent of any rendering (spawn-cap partial success, age/retire, gravity
      integration, bounce reflection, spark trigger) — consistent with every other
      pure-logic module in this port
- [ ] The `--scene constraint` screenshot now visually matches the reference "rope"
      screenshot's defining features: bouncing background particles, spark trails on
      bounce, and the full telemetry HUD
- [ ] The `arena` scene (§4.2a/§4.2b) is unmodified and its tests still pass — this
      phase touches only the `constraint` scene
- [ ] Both spec documents state explicitly which fields/rules are in scope vs. descoped,
      per this training's running convention (§4.2a, §4.2b) of never silently narrowing

---

### 4.2d Phase A5 (Completion): Full-Fidelity Closure — Every Spec to Recreate the Game

**Goal:** Close **every** remaining gap between the C++ game and `specs/transform/` so
the spec set alone is sufficient to recreate the game **100%** — same colors, same
gameplay, same scenes, same controls, same headless determinism contract. Phases A–A4
each closed the gap a screenshot happened to surface; A5 inverts the method: instead of
comparing outputs and speccing what looks different, **audit the entire code surface
feature-by-feature against the spec inventory** and spec everything that has no home.
This is the workshop's fourth — and final — worked example of the audit lesson, and the
only one that produces a provably complete result rather than a visibly improved one.

**What the full audit revealed (2026-07-11):** reviewing every function in
`apps/sandbox/` and `include/orbital_arena/` against the then-13 specs found five
categories of uncovered behavior — roughly 40% of what defines the game on screen:

| Gap category                        | Code source                                   | Prior coverage                        | Closed by (new spec)                          |
| ----------------------------------- | ---------------------------------------------- | -------------------------------------- | ---------------------------------------------- |
| Game-layer orchestration (tick pipeline, rng stream topology, symmetric replenishment, constants) | `orbital_arena/arena.h`, `types.h` | ❌ never specced as a whole | `orbital-arena-orchestration.spec.md` |
| Power-ups / input replay log / snapshot+hash | `powerup.h`, `input.h`, `snapshot.h`  | ❌ A2's documented descope             | `orbital-arena-powerups.spec.md`, `orbital-arena-input-log.spec.md`, `orbital-arena-snapshot.spec.md` |
| The five scenes: exact geometry, rng draw order, per-scene force rules, digest algorithm | `scene.cpp` builds + `substep()`  | ❌ only the default free-particle rule (A4) | `sandbox-scenes.spec.md` |
| Visual identity: every RGBA, gradient, bloom layer, animation formula, effect | `app.cpp` draw functions | ❌ explicitly descoped by A3's HUD spec | `sandbox-visual-identity.spec.md` |
| Full control map + app loop + headless CLI/CSV contract | `app.cpp` input block, `main.cpp`, `headless.cpp` | ❌ only the LMB-drag adaptation (A3)  | `sandbox-controls.spec.md`, `sandbox-headless.spec.md` |

**The descope-reversal convention (the teaching point):** A2–A4's scope reductions were
honest and explicit — which is precisely what made A5 cheap. Each descope was reversed
**in place**: the original paragraph stays, amended with a *“superseded for full
fidelity”* marker and a link to the closing spec (see the amended §6 of
`sandbox-hud.spec.md`, §5 of `orbital-arena-scoring.spec.md`, §6/§7 of
`sandbox-free-particles.spec.md` / `vfx-particle-system.spec.md`). A silently-narrowed
spec would have required re-auditing everything; an explicitly-narrowed one is a to-do
list.

**The decisive acceptance artifact — golden digests for all five scenes** (seed 42,
600 frames, two independent runs byte-identical; recorded in
`sandbox-scenes.spec.md` §8.1):

| Scene           | `trace_digest`     |
| --------------- | ------------------ |
| rope            | `9dc3bd72a4f7f31a` |
| pendulum tower  | `dee045cb412df634` |
| cloth           | `3cbd246289e0cf63` |
| particle storm  | `fd2df9d9c889a7fc` |
| orbital arena   | `919d2feba5bdbeac` |

> **Audit catch worth teaching:** the reference source itself contained a stale golden
> value — a comment in `scene.cpp` still cites rope digest `33a6319d856d4869`, which
> pre-dates the rope scene's 32 free particles. Code comments rot; recorded, re-run
> acceptance values don't. The spec table above is ground truth.

**Prompt shape (one per gap row — the same reverse-spec pattern as §4.2–§4.2c, now
demanding exact values):**

```text
/speckit.specify (reverse)

Analyze <files>. Extract a language-agnostic specification at FULL FIDELITY: every
constant, color, formula, and rng draw order is normative — nothing is "implementation
detail" unless it is genuinely invisible (platform presentation workarounds, telemetry
logging). Cross-reference the existing specs in specs/transform/ instead of restating
them. State explicitly which prior spec's descope this closes, if any. Output to
specs/transform/<name>.spec.md.
```

**Acceptance gate (how you know the spec set is actually complete):**

- [ ] **Traceability:** every literal constant in `scene.h`/`scene.cpp`/`app.cpp` and
      `include/orbital_arena/*.h` maps to exactly one spec section (spot-check with
      grep; zero unmapped gameplay/visual literals)
- [ ] **Golden digests:** all five scenes' digests recorded in the spec from two
      byte-identical runs of the reference build
- [ ] **Descope audit:** `grep -i descoped specs/transform/` returns only (a) telemetry
      / platform-workaround exclusions and (b) historical descopes carrying a
      "superseded for full fidelity" marker with a link to the closing spec
- [ ] **Config vs. constant:** values that legitimately vary (e.g. arena `half_extent`:
      game default 5.0 vs. sandbox embedding 2.0) are documented as *configuration*
      with both values, never silently hardcoded to one
- [ ] Every new spec names the constitutional articles it inherits and follows the
      house format (Purpose / Data Model / Operations / Guarantees / Constraints)

**What A5 deliberately still excludes** (and why that's a scope decision, not a gap):
platform presentation workarounds (DWM/Metal/compositor shims), the JSONL telemetry
pipeline, crash handlers, and the hang watchdog — operational tooling around the game,
not the game. The F1 crash key is *listed* in the controls spec (it's player-visible)
but its mechanism is not specced.

---

### 4.3 Phase B: Target Constitution (Rust/Bevy)

The target constitution is **generated, not hand-written**. Before running the prompt below,
complete the research and consolidation prompt series in
[Appendix D](#appendix-d-rust-game-dev-research-and-guidelines-prompt-series) — it produces
`specs/transform/rust-guidelines.md`, the consolidated Rust game-development guidelines that
carry EASTL's *concepts* (explicit memory ownership, fixed capacity, no hidden allocation,
allocation observability) into Rust idioms without attempting a 1:1 EASTL→Rust mapping.

#### 4.3.1 Constitution Generation Prompt

**Prompt to Copilot (VS Code Agent Mode or Copilot CLI):**

```text
/speckit.constitution

Generate the target constitution for the Rust/Bevy port of engine_demo and write it
to specs/transform/rust-constitution.md.

Inputs — read ALL of these before drafting a single article:
1. specs/transform/rust-guidelines.md — consolidated Rust game-dev guidelines
   (produced by the Appendix D prompt series)
2. specs/transform/*.spec.md — the language-agnostic behavioral specs from Phase A
3. specs/constitution.md — the C++ constitution, for the list of behavioral
   guarantees that must survive the language change

Rules:
- Preserve behavioral GUARANTEES (determinism, frame budget, test-first, HITL
  gates), never C++ MECHANISMS. If a C++ article exists only to work around a C++
  limitation (e.g. "EASTL-first" exists because std:: containers hide allocation),
  replace it with the Rust-idiomatic rule that achieves the same guarantee, or
  drop it with a one-line justification.
- Where the guidelines borrow an EASTL concept, express it in Rust idioms — do
  NOT invent EASTL-shaped APIs in Rust. The goal is a solid foundation for Rust
  game development, not a port of EASTL.
- Every article must be enforceable: name the tool or test pattern that enforces
  it (clippy lint, grep gate in CI, #[test] pattern, cargo deny, headless App
  integration test).
- Tag every article as Inherited / Adapted / New with a one-line rationale.
- 8–12 articles maximum. List any guideline you deliberately did NOT promote
  into the constitution, and why.

Gate (HITL): I will review each article against the Phase C pattern-mapping table
before this constitution is committed. Do not proceed to /speckit.plan.
```

**Review checklist before accepting the generated constitution:**

- [ ] Every C++ behavioral guarantee is covered by exactly one Rust article (no orphans, no duplicates)
- [ ] No article prescribes an EASTL mechanism dressed in Rust syntax (e.g. a custom allocator trait where `Vec::with_capacity()` + a zero-alloc test gives the same guarantee)
- [ ] Every article names its enforcement tool — an unenforceable article is a wish, not a law
- [ ] The Inherited/Adapted/New tags match your expectations from the §4.3.2 reference below

#### 4.3.2 Reference Constitution

**A well-generated constitution should land close to this reference — it replaces C++-specific rules with Rust/Bevy idioms while preserving behavioral guarantees:**

```markdown
# engine_demo (Rust/Bevy) — Constitution

## Article 1 — No Panics in Game Logic

Game logic must not use `unwrap()`, `expect()`, or `panic!()`. Use `Result<T, E>` or
`Option<T>` with explicit error handling. Panics are reserved for truly unrecoverable
states in initialization only.

## Article 2 — Bevy ECS Architecture

All game state lives in Bevy Components, Resources, and Events. No global mutable state.
No `Mutex` or `RwLock` in game systems. Cross-system communication uses Bevy Events.

## Article 3 — Standard Library + Bevy Ecosystem

Use Rust standard library types (`Vec`, `HashMap`, `String`) and Bevy types (`Entity`,
`Component`, `Resource`, `Query`, `Commands`). External crates require justification in
the spec document and must be listed in `Cargo.toml` with pinned versions.

## Article 4 — Ownership-Aware Design

All data has clear ownership. Shared references use `&` or `Res<T>`. Mutable access uses
`&mut` or `ResMut<T>`. No `Rc<RefCell<T>>` in game logic — this is a code smell indicating
wrong ECS design.

## Article 5 — Determinism (Inherited)

Simulation paths are deterministic across runs at fixed seeds:

- RNG via `rand` crate with explicit `StdRng::seed_from_u64()`
- Time accumulators are `f64` (not `f32`)
- Iteration order over entities is deterministic (use `Query` with explicit ordering)

## Article 6 — Performance Budget (Inherited)

Frame budgets: ≤16.67 ms at 60 FPS. No heap allocation in per-frame systems.
Pre-allocate collections in startup systems. Use `Vec::with_capacity()` during init.

## Article 7 — Test-First (Adapted)

Every public function has at least one `#[test]`. Integration tests use Bevy's
`App::new()` headless runner for system-level testing.

## Article 8 — HITL Gates (Unchanged)

Spec-Kit pauses for human approval between stages.

## Article 9 — Idiomatic Rust

Code passes `clippy` with default lints. No `unsafe` without a `// SAFETY:` comment
explaining the invariant. Prefer iterators over index loops. Use `derive` macros for
Component/Resource traits.
```

**Critical Differences from C++ Constitution:**

| C++ Rule                     | Rust Equivalent            | Rationale                                               |
| ---------------------------- | -------------------------- | ------------------------------------------------------- |
| No exceptions → status enums | No panics → `Result<T, E>` | Rust's `Result` is zero-cost; no need for status enums  |
| No RTTI → tagged unions      | Bevy ECS → Component trait | ECS replaces RTTI with compile-time component queries   |
| EASTL-first                  | std + Bevy                 | Rust's std is allocation-aware; no need for alternative |
| Explicit allocator           | Ownership system           | Rust's ownership replaces manual allocator management   |
| Lockless ring buffer         | Bevy Events                | Bevy's event system IS the cross-system message bus     |

---

### 4.4 Phase C: Transformation Plan

**The plan maps C++ patterns to Rust/Bevy patterns:**

## C++ → Rust/Bevy — Transformation Plan

### Pattern Mapping

| C++ Pattern                         | Rust/Bevy Pattern                                                     |
| ----------------------------------- | --------------------------------------------------------------------- |
| `engine_demo::allocator` (arena)    | Bevy `Resource` + `Vec::with_capacity()`                              |
| `ecs::world` (generational handles) | Bevy `World` + `Entity` (built-in generational)                       |
| `physics::constraint_solver`        | Bevy System with `Query<(&Position, &Velocity, &Constraint)>`         |
| `sim::game_loop` (fixed-step)       | Bevy `FixedUpdate` schedule                                           |
| `sim::rng` (seeded MT)              | `rand::rngs::StdRng` as a Bevy `Resource`                             |
| `frame_budget` (rolling window)     | Bevy `Diagnostics` plugin + custom `Resource`                         |
| `eastl::function<force_fn>`         | Rust trait: `trait ForceApplicator { fn apply(&self, ...) -> Vec2; }` |
| `eastl::vector<T, alloc>`           | `Vec<T>` (Rust's allocator is implicit, zero-cost)                    |

## Architecture Comparison

```text
C++ Architecture Rust/Bevy Architecture
───────────────── ──────────────────────
allocator.h (manual arena) → Bevy Resource (automatic)
ecs/world.h (custom ECS) → Bevy World (built-in)
physics/constraint.h → PhysicsPlugin (System)
sim/game_loop.h → FixedUpdate schedule
sim/rng.h → Resource<StdRng>
frame_budget.h → DiagnosticsPlugin
```

## File Layout (Target)

```text
orbital-arena-rs/
├── Cargo.toml
├── src/
│ ├── main.rs ← App builder + plugin registration
│ ├── physics/
│ │ ├── mod.rs
│ │ ├── components.rs ← Position, Velocity, Constraint
│ │ └── systems.rs ← constraint_solver_system
│ ├── sim/
│ │ ├── mod.rs
│ │ ├── rng.rs ← DeterministicRng resource
│ │ └── frame_budget.rs ← FrameBudget resource
│ └── ecs/
│ └── mod.rs ← Entity lifecycle utilities
└── tests/
├── test_physics.rs
├── test_determinism.rs
└── test_frame_budget.rs
```

## Task Decomposition (6 tasks, ordered by dependency)

| #   | Task                                             | C++ Source                 | Rust Target                 | Lines |
| --- | ------------------------------------------------ | -------------------------- | --------------------------- | ----- |
| 1   | Project scaffold + Bevy app                      | `CMakeLists.txt`           | `Cargo.toml`, `main.rs`     | ~50   |
| 2   | Deterministic RNG resource                       | `sim/rng.h/cpp`            | `sim/rng.rs`                | ~60   |
| 3   | Physics components + solver system               | `physics/constraint.h/cpp` | `physics/*.rs`              | ~140  |
| 4   | Fixed-step simulation (FixedUpdate)              | `sim/game_loop.h/cpp`      | Bevy schedule config        | ~40   |
| 5   | Frame budget diagnostics                         | `frame_budget.h/cpp`       | `sim/frame_budget.rs`       | ~80   |
| 6   | Integration test: determinism across 1000 frames | `test_game_loop.cpp`       | `tests/test_determinism.rs` | ~100  |

---

### 4.5 Phase D: Tasks and Implementation

**Example — Task 3: Physics Components + Solver System:**

````markdown
## Task 3: Physics Components + Constraint Solver System

**Source Analysis:** `include/engine_demo/physics/constraint.h` + `src/engine_demo/physics/constraint.cpp`
**Target:** `src/physics/components.rs` + `src/physics/systems.rs`

**C++ Behavioral Spec (from Phase A):**

- Bodies have position and velocity (vec2 each)
- Constraints are distance constraints between body pairs
- Solver iterates constraints N times (default 4), projecting positions to satisfy distance
- Iteration order is deterministic (sorted by constraint ID)

**Rust Implementation Plan:**

```rust
// components.rs
#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct DistanceConstraint {
    target: Entity,
    rest_length: f64,
}

// systems.rs — runs in FixedUpdate schedule
fn constraint_solver_system(
    mut query: Query<(Entity, &mut Position, &DistanceConstraint)>,
    positions: Query<&Position>,
) {
    // 4 iterations of position projection
    for _iter in 0..4 {
        // ... project positions to satisfy constraints
    }
}
```
````

**Acceptance Gate:**

- [ ] `cargo clippy` clean (Article 9)
- [ ] No `unwrap()` in solver (Article 1)
- [ ] Deterministic: same initial state → same result after N iterations (Article 5)
- [ ] No allocation in solver system (Article 6)
- [ ] `#[test]` validates solver converges for known constraint setup

---

### 4.6 Copilot CLI Scenario: Terminal-Driven Transformation

This section demonstrates using **GitHub Copilot CLI** for the transformation workflow — showing how the terminal-based agent can analyze existing code and generate transformation specs.

#### Scenario A: Interactive Plan Mode — Analyzing Existing Architecture

```bash
cd /path/to/speckit-workshop-demo
copilot
```

**In the interactive session, switch to Plan Mode (Shift+Tab):**

```text
You: I need to transform this C++20 game engine to Rust/Bevy ECS. Analyze the
architecture in include/engine_demo/ and src/engine_demo/ and create a
transformation plan that maps each C++ subsystem to its Rust/Bevy equivalent.

Focus on:
1. The ECS world (ecs/world.h) → Bevy World
2. The physics solver (physics/constraint.h) → Bevy System
3. The game loop (sim/game_loop.h) → Bevy FixedUpdate
4. The allocator pattern → Rust ownership

For each mapping, explain:
- What C++ pattern is being replaced
- What Rust/Bevy pattern replaces it
- Why the Rust version is idiomatic (not a transliteration)
```

**Copilot CLI Plan Mode Output:**

Copilot enters plan mode, reads the source files, asks clarifying questions:

```text
Copilot [Plan Mode]: I've analyzed the codebase. Before I create the plan, let me
confirm a few things:

1. The ECS in ecs/world.h uses generational handles with slot reuse. Bevy's Entity
   type already provides this. Should the plan preserve the 4096 entity cap, or use
   Bevy's dynamic capacity?

2. The physics solver in constraint.h iterates constraints in a deterministic order
   (sorted by hash key). Bevy queries don't guarantee order. Should I plan for
   explicit sorting in the Bevy system?

3. The allocator pattern (explicit allocator& at every construction) maps to Rust's
   implicit ownership. Should I maintain any arena-style allocation for hot paths,
   or rely entirely on Rust's default allocator + Vec::with_capacity()?
```

**Human answers (steering the agent):**

```text
You:
1. Use Bevy's dynamic capacity — the 4096 cap was a C++ optimization we don't need
2. Yes, plan for explicit sorting — determinism (Article 5) must be preserved
3. Use Vec::with_capacity() in init systems — no custom arena needed
```

**Copilot produces the plan** and saves it to a file.

#### Scenario B: Programmatic Mode — Batch Spec Generation

Use Copilot CLI programmatically to generate reverse-specs for each subsystem:

```bash
# Generate reverse-spec for the ECS subsystem
copilot -p "Read include/engine_demo/ecs/world.h and src/engine_demo/ecs/world.cpp. \
  Generate a language-agnostic behavioral specification (not C++ code) that captures \
  WHAT the ECS does: entity lifecycle, generational handles, capacity model, \
  guarantees. Output as markdown to specs/transform/ecs-world.spec.md" \
  --allow-tool='shell(cat)' \
  --allow-tool='shell(find)' \
  --allow-tool='write'

# Generate reverse-spec for the physics solver
copilot -p "Read include/engine_demo/physics/constraint.h and \
  src/engine_demo/physics/constraint.cpp. Generate a language-agnostic behavioral \
  specification for the constraint solver. Output to specs/transform/physics.spec.md" \
  --allow-tool='shell(cat)' \
  --allow-tool='write'

# Generate reverse-spec for the game loop
copilot -p "Read include/engine_demo/sim/game_loop.h and \
  src/engine_demo/sim/game_loop.cpp. Generate a language-agnostic specification \
  for the fixed-step accumulator pattern. Output to specs/transform/game-loop.spec.md" \
  --allow-tool='shell(cat)' \
  --allow-tool='write'
```

#### Scenario C: Using /fleet for Parallel Task Implementation

Once specs are generated, use `/fleet` to parallelize independent tasks:

```bash
$ copilot
You: /fleet

I have 6 transformation tasks. Tasks 1, 2, and 4 are independent (project scaffold,
RNG resource, and FixedUpdate config). Execute all three in parallel:

Task 1: Create Cargo.toml with bevy 0.13, rand 0.8 dependencies. Create src/main.rs
with App::new() + DefaultPlugins + our custom plugin stubs.

Task 2: Create src/sim/rng.rs implementing DeterministicRng as a Bevy Resource wrapping
StdRng::seed_from_u64(). Include #[test] for determinism.

Task 4: Configure Bevy's FixedUpdate schedule at 60Hz in main.rs. Add a placeholder
system that logs frame timing.
```

#### Scenario D: Using /delegate for Autonomous Spec-to-Code

For well-specified tasks, delegate to Copilot to work autonomously:

```bash
$ copilot
You: /delegate

Work on this task autonomously. The spec is at specs/transform/physics.spec.md and
the target constitution is at specs/transform/rust-constitution.md.

Create src/physics/components.rs and src/physics/systems.rs implementing the constraint
solver as a Bevy system. Ensure:
- No unwrap() or panic!() (Article 1)
- Uses Query<> for entity access (Article 2)
- Deterministic iteration order (Article 5)
- No allocation in the solver system (Article 6)
- Include #[test] module with at least 3 tests

When done, create a PR with a description referencing the spec.
```

---

### 4.7 Best Practices for Cross-Language Transformation

Based on GitHub's official documentation and the Copilot ecosystem capabilities:

#### 1. Separate Semantics from Syntax

> **Never transliterate.** A line-by-line translation from C++ to Rust produces non-idiomatic, brittle code. Instead:
>
> - Extract the BEHAVIORAL specification (what the code does)
> - Re-implement using TARGET LANGUAGE idioms (how it should be done in Rust)

| Anti-Pattern                                               | Best Practice                                                     |
| ---------------------------------------------------------- | ----------------------------------------------------------------- |
| `eastl::unique_ptr<T>` → `Box<T>`                          | Think: "Who owns this?" → Use Bevy's ownership model              |
| `eastl::vector<T, alloc>` → `Vec<T>` with custom allocator | Think: "When is this allocated?" → `Vec::with_capacity()` in init |
| `enum class + switch` → `match` on Rust enum               | ✅ This IS idiomatic — enums map naturally                        |
| Manual RAII → Drop trait                                   | Think: "Does Bevy manage this resource?" → Usually yes            |

#### 2. Use the Constitution as the Transformation Contract

The spec is the bridge between languages. Both the source and target constitutions should produce implementations that satisfy the **same behavioral specification**. If the C++ passes a determinism test and the Rust passes the same determinism test, the transformation is correct.

#### 3. Leverage Copilot's Multi-File Context

When using VS Code Agent Mode or Copilot CLI for transformation:

- Keep BOTH the C++ source AND the Rust target open in the workspace
- Reference the spec document as the shared truth
- Let Copilot see the C++ for "what" and the Rust constitution for "how"

#### 4. Custom Instructions for the Target Language

Create `.github/instructions/rust-bevy.instructions.md`:

```markdown
---
applyTo: "**/*.rs"
---

## Rust/Bevy Conventions

- All game state is Components, Resources, or Events — never global mutable state
- Systems take Query<> parameters — never access World directly
- Use `#[derive(Component)]`, `#[derive(Resource)]` — never manual trait impl
- Error handling: `Result<T, GameError>` — never unwrap() in game logic
- Testing: use `App::new()` headless for system integration tests
- Performance: pre-allocate in startup systems, zero allocation in Update/FixedUpdate
```

#### 5. Validate Equivalence with Deterministic Tests

The ultimate validation of a cross-language transformation:

```text
C++ (seed=42, 1000 frames) → state hash = 0xABCD1234
Rust (seed=42, 1000 frames) → state hash = 0xABCD1234  ← Must match!
```

Write a "golden file" test that captures the C++ output at a known seed and frame count, then validates the Rust implementation produces identical results.

---

### 4.8 Reflection and Key Takeaways

| Question                                                    | Answer                                                                                                                                    |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| "Why not just rewrite from scratch?"                        | Specs preserve institutional knowledge. The WHAT doesn't change when the HOW changes.                                                     |
| "What role does the constitution play in transformation?"   | It's the contract renegotiation. C++ constraints (no exceptions) → Rust constraints (no panics). The behavioral guarantees stay the same. |
| "When is Copilot CLI better than VS Code?"                  | For batch operations (generating 6 reverse-specs), headless/CI environments, and when you want plan mode's structured approach.           |
| "What's the biggest risk in cross-language transformation?" | Transliteration — writing "C++ in Rust syntax." The constitution prevents this by encoding idiomatic target-language rules.               |
| "How does `/fleet` help?"                                   | Independent tasks (scaffold, RNG, config) can run in parallel, cutting transformation time by ~40%.                                       |

---

## Appendix A: Spec-Kit Quick Reference Card

### The Five Stages

| Stage        | Command                 | Input                  | Output                       | Gate                           |
| ------------ | ----------------------- | ---------------------- | ---------------------------- | ------------------------------ |
| Ground Truth | `/speckit.constitution` | Project values         | Binding articles             | Human confirms                 |
| Requirements | `/speckit.specify`      | Problem + constitution | `spec.md`                    | Human reviews criteria         |
| Architecture | `/speckit.plan`         | Spec + constitution    | Design document              | **HITL: Approve before tasks** |
| Work Items   | `/speckit.tasks`        | Plan + constitution    | Task list (< 150 lines each) | **HITL: Approve sizing**       |
| Code         | `/speckit.implement`    | One task               | Diff + tests                 | **HITL: Review diff + ctest**  |

### Optional Commands (§1.2a)

| Command              | When                                | Output                                              | Gate                                       |
| -------------------- | ----------------------------------- | --------------------------------------------------- | ------------------------------------------ |
| `/speckit.clarify`   | After `/specify`, before `/plan`    | Clarifications section appended to `spec.md`        | Human answers up to 5 targeted questions   |
| `/speckit.analyze`   | After `/tasks`, before `/implement` | Cross-artifact consistency & coverage report        | Human resolves flagged gaps before coding  |
| `/speckit.checklist` | Any time after `/specify`           | Custom quality checklist ("unit tests for English") | Human ticks every item or rejects upstream |

Also available: `/speckit.taskstoissues` (tasks → GitHub issues) and `/speckit.converge` (audit codebase vs. artifacts, append remaining work as tasks).

### Recovery Patterns

| Situation                    | Action                                                  |
| ---------------------------- | ------------------------------------------------------- |
| Task rejected once           | Re-prompt with explicit constitutional citation         |
| Task rejected twice          | Examine plan for misaligned architecture                |
| Task rejected 3×             | **Roll back to `/speckit.specify`** — the spec is wrong |
| Task too large (> 150 lines) | Reject and ask for split                                |
| Constitution violated        | Reject immediately; cite article number                 |
| Tests fail                   | Reject; ask Copilot to fix while preserving spec intent |
| Demo run unrecoverable       | Abandon `demo-0x`; cut `demo-0(x+1)` from `main` (§0.2a) |

### Anti-Patterns

| Don't                               | Do Instead                                  |
| ----------------------------------- | ------------------------------------------- |
| Skip `/speckit.constitution`        | Always establish ground truth first         |
| Plan on a vague spec                | Run `/speckit.clarify` before planning      |
| Approve without reading diff        | Read every line; run every test             |
| Let scope grow mid-session          | New features → new `/speckit.specify` cycle |
| Fix the code when the spec is wrong | Fix the spec, regenerate the code           |
| Generate > 150 lines in one task    | Split into right-sized tasks                |
| Auto-approve all HITL gates         | The friction is the feature                 |

---

## Appendix B: Copilot CLI Command Reference

### Installation

```bash
# macOS
brew install gh
gh auth login
gh extension install github/gh-copilot  # Legacy — now use:
# Install Copilot CLI (standalone)
# See: https://docs.github.com/en/copilot/how-tos/set-up/install-copilot-cli
```

### Interactive Mode (CLI)

```bash
copilot                          # Start interactive session
copilot --model claude-sonnet    # Start with specific model
```

### Key Slash Commands

| Command      | Purpose                                            |
| ------------ | -------------------------------------------------- |
| Shift+Tab    | Toggle between Ask/Execute mode and Plan mode      |
| `/plan`      | Enter plan mode explicitly                         |
| `/fleet`     | Parallelize independent subtasks                   |
| `/delegate`  | Hand off task for autonomous execution             |
| `/pr`        | Create/manage pull requests                        |
| `/compact`   | Compress conversation context                      |
| `/context`   | Show token usage breakdown                         |
| `/model`     | Switch AI model                                    |
| `/mcp`       | List configured MCP servers                        |
| `/allow-all` | Allow all tools without approval (current session) |
| `/feedback`  | Submit feedback to GitHub                          |

### Programmatic Mode (CLI)

```bash
# Single prompt execution
copilot -p "Describe your task here" --allow-tool='write'

# With specific tool permissions
copilot -p "Analyze code and create spec" \
  --allow-tool='shell(cat)' \
  --allow-tool='shell(find)' \
  --allow-tool='write' \
  --deny-tool='shell(rm)'

# Full autonomy (dangerous — use in sandboxed environments)
copilot -p "Implement the full task" --allow-all-tools --deny-tool='shell(git push)'
```

### Custom Instructions for CLI

Place in `.github/copilot-instructions.md`:

```markdown
## Build & Test

- Always run `cargo clippy` before committing
- Always run `cargo test` after any code change
- Use `cargo fmt` for formatting

## Spec-Kit Workflow

- Before implementing, check specs/ for existing specification
- Reference the constitution before making architectural decisions
- Keep diffs under 150 lines per task
```

---

## Appendix C: Custom Instructions File Templates

### Repository-Wide: `.github/copilot-instructions.md`

```markdown
# Project: engine_demo (C++20 Game Engine)

## Build Commands

- Configure: `cmake --preset default-debug`
- Build: `cmake --build --preset default-debug`
- Test: `ctest --preset default-debug --output-on-failure`
- Lint: `clang-tidy --config-file=.clang-tidy src/**/*.cpp`

## Architecture

- Language: C++20 with `-fno-exceptions -fno-rtti`
- Containers: EASTL (never std:: containers)
- Dependencies: vcpkg (EASTL pinned), raylib (optional for sandbox)
- Tests: GoogleTest, wired via CTest

## Spec-Kit Constitution

Ground truth: `specs/constitution.md`
Every code change must satisfy all 8 articles.

## Key Constraints

- No heap allocation in inner loops (Article 6)
- Every public function must have a GTest (Article 7)
- All containers take explicit allocator (Article 4)
```

### Path-Specific: `.github/instructions/cpp-impl.instructions.md`

```markdown
---
applyTo: "src/**/*.cpp"
---

## C++ Implementation Rules

1. Include the corresponding header first: `#include <engine_demo/subsystem/file.h>`
2. Use `[[nodiscard]]` on all factory functions and status-returning functions
3. Use `noexcept` on move constructors, move assignment, and swap
4. Never use `auto` for return types — be explicit
5. Prefer `eastl::span` over pointer+size pairs
6. Assert preconditions with `EA_ASSERT()` not `assert()`
```

### Path-Specific: `.github/instructions/tests.instructions.md`

```markdown
---
applyTo: "**/test_*.cpp"
---

## Test File Rules

1. Use GoogleTest (not Catch2 or doctest)
2. Test fixture pattern: `class SubsystemTest : public ::testing::Test { ... }`
3. Every test constructs its own allocator — never share state between tests
4. Use explicit seeds for any RNG-dependent test
5. Name pattern: `TEST(SubsystemName, BehaviorUnderTest)`
6. Include at least one happy-path and one edge-case test per public function
```

### Agent Instructions: `AGENTS.md`

````markdown
# AGENTS.md — engine_demo workspace

## Mission

Synthetic C++20 game-engine workspace. Treat as real game-engine subsystem:
deterministic, real-time, allocator-aware, no exceptions, no RTTI.

## Hard Rules

1. No `std::` containers in committed code
2. No exceptions (`-fno-exceptions`)
3. No RTTI (`-fno-rtti`)
4. Explicit allocator on every container
5. `[[nodiscard]]` on factories and status returns
6. `noexcept` on move operations
7. Deterministic sim paths (seeded RNG, double accumulators)
8. No allocation in inner loops

## Spec-Kit

Constitution: `specs/constitution.md` — ground truth.
If a task requires violating an article, THE SPEC IS WRONG.

## Build & Validate

```bash
cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

## Off-The-Rails Recovery

If you draft `std::vector` or `try`/`catch`, STOP and re-read this file.
````

---

## Appendix D: Rust Game-Dev Research and Guidelines Prompt Series

This appendix supports [§4.3 Phase B](#43-phase-b-target-constitution-rustbevy). It is a
three-step prompt pipeline: **research → consolidate → generate**. Steps D.1 and D.2 live
here; the final constitution-generation prompt (step D.3) is part of the training body at
§4.3.1.

```text
┌────────────────────┐     ┌──────────────────────────┐     ┌───────────────────────────┐
│  D.1 Research      │────▶│  D.2 Consolidation       │────▶│  D.3 Constitution (§4.3.1)│
│  5 focused prompts │     │  EASTL concepts + merge  │     │  /speckit.constitution    │
│  research/*.md     │     │  rust-guidelines.md      │     │  rust-constitution.md     │
└────────────────────┘     └──────────────────────────┘     └───────────────────────────┘
```

All artifacts land under `specs/transform/`. Run the prompts in VS Code Agent Mode, or
batch them with `copilot -p` (see Appendix B) — each research prompt is independent, so
they parallelize cleanly.

### D.1 Research Prompt Series

One prompt per topic. Each produces a short, evidence-backed research note — demand
sources and trade-offs, not just recommendations.

**R1 — Architecture and ECS:**

```text
Research current best practices for structuring game code in Rust with an ECS
(Bevy in particular). Cover: Components vs Resources vs Events decision rules;
system ordering and schedules (Update vs FixedUpdate); plugin decomposition;
when NOT to use ECS. For each practice, state the failure mode it prevents.
Cite sources (Bevy book/docs, established community references). Write the note
to specs/transform/research/r1-architecture.md. Do not write any Rust code.
```

**R2 — Memory and Allocation:**

```text
Research memory-management best practices for real-time Rust games. Cover:
pre-allocation patterns (Vec::with_capacity, object pools, arenas — bumpalo and
friends); how to detect and prevent per-frame heap allocation; fixed-capacity
collection crates (arrayvec, smallvec, heapless) and their trade-offs; when
Rust's ownership model makes a C++-style custom allocator unnecessary, and the
rare cases where it doesn't. Write to specs/transform/research/r2-memory.md.
```

**R3 — Determinism:**

```text
Research determinism in Rust game simulations. Cover: seeded RNG choices (rand's
StdRng vs explicit algorithm crates and their stability-across-versions
guarantees); f32 vs f64 accumulators; sources of nondeterministic iteration order
(HashMap, ECS query order, parallel system execution) and their mitigations;
floating-point reproducibility across platforms. Write to
specs/transform/research/r3-determinism.md.
```

**R4 — Error Handling and API Design:**

```text
Research error-handling and public-API best practices for Rust game code. Cover:
panic policy for real-time loops (panic = dropped frame or crashed process);
Result/Option patterns vs C++-style status enums; #[must_use] as the analogue of
[[nodiscard]]; unsafe policy and // SAFETY: conventions; clippy lint tiers worth
enforcing in CI. Write to specs/transform/research/r4-errors-api.md.
```

**R5 — Testing and Tooling:**

```text
Research testing and CI practices for Rust games. Cover: headless Bevy App tests
(MinimalPlugins) for system-level integration; deterministic replay tests; frame
budget/perf assertions in tests vs criterion benchmarks; cargo clippy/fmt/deny in
CI; detecting per-frame allocations in tests. Write to
specs/transform/research/r5-testing.md.
```

### D.2 Consolidation Prompts

**C1 — Extract EASTL's design concepts (not its APIs):**

```text
Read specs/constitution.md, include/engine_demo/allocator.h, and the EASTL usage
across include/ and src/. Extract the DESIGN CONCEPTS that EASTL brings to this
codebase, stated language-neutrally — e.g.: explicit memory ownership; allocation
is visible and budgeted, never hidden; fixed capacity decided up front; container
behavior is deterministic; allocation observability (bytes_used is queryable).
For each concept: why it matters for games, and what breaks without it.
Explicitly EXCLUDE EASTL mechanics that are C++ workarounds (allocator template
parameters, fixed_vector overflow flags). Write to
specs/transform/research/c1-eastl-concepts.md.
```

**C2 — Consolidate into proposed guidelines:**

```text
Read all of specs/transform/research/*.md. Consolidate them into a single
proposed guidelines document: specs/transform/rust-guidelines.md.

Goal: a solid foundation for Rust game development that HONORS the EASTL
concepts from c1-eastl-concepts.md — NOT a 1:1 mapping of EASTL to Rust. Where
Rust's ownership model already delivers a concept, say so and stop; where it
doesn't (e.g. hidden Vec growth in a frame loop), propose the Rust-idiomatic
practice that restores the guarantee.

Format: 10–15 numbered guidelines. Each has: the guideline (one sentence), the
EASTL concept or research note it derives from, the Rust idiom that implements
it, and how to enforce it (lint/test/CI). Flag conflicts between research notes
rather than silently resolving them — conflicts are HITL review items.

Gate (HITL): I will review and edit these guidelines before they feed the
constitution prompt in §4.3.1.
```

### D.3 Constitution Generation

With `rust-guidelines.md` reviewed and approved, run the constitution-generation prompt in
[§4.3.1](#43-phase-b-target-constitution-rustbevy) — that step is part of the training
proper, because generating (and gating) a constitution from researched guidelines is the
repeatable skill; the research pipeline in this appendix is the reusable scaffolding.

**Batch variant (Copilot CLI, requires authentication):**

```bash
# D.1 research prompts are independent — run them in parallel shells or /fleet
copilot -p "<R1 prompt>" --allow-tool='write' --allow-tool='shell(cat)'
copilot -p "<R2 prompt>" --allow-tool='write' --allow-tool='shell(cat)'
# ... R3–R5, then sequentially:
copilot -p "<C1 prompt>" --allow-tool='write' --allow-tool='shell(cat)'
copilot -p "<C2 prompt>" --allow-tool='write' --allow-tool='shell(cat)'
# D.3 runs interactively — the HITL gate on the constitution is the point.
```

---

## Self-Study Lab: Visualize the VFX Subsystem in the Sandbox

> **⏱ 60–90 minutes, solo** | **Difficulty:** Intermediate
> **You drive every stage yourself.** No facilitator, no answer key — just you, Copilot, and the Spec-Kit gates you watched in Part 0.

### S.1 Mission and Context

Part 0's demo produced the `engine_demo::vfx` subsystem — `particle_pool`, `force_applicator`, and `emitter` — fully implemented and tested under `specs/001-particle-vfx-subsystem/`. But open the sandbox (`apps/sandbox/`) and you'll see... nothing new. Task T022 of that feature deliberately deferred wiring VFX into the sandbox scene:

> _"`apps/sandbox` does not construct an `engine_demo::vfx::emitter` yet, so listing it there would be inaccurate until a follow-up feature wires VFX into the sandbox scene."_

**Your mission:** close that gap as **Feature 002 — Sandbox VFX Visualization**, driving the complete Spec-Kit cycle (`/speckit.specify` → `/speckit.plan` → `/speckit.tasks` → `/speckit.implement`) yourself. When you're done, right-clicking in the sandbox spawns a visible burst of VFX particles, and free particles throw sparks when they bounce off the world bounds.

**Why this feature is a great solo exercise:**

- It spans a **library change** (a small addition to `emitter`), an **app integration** (scene + renderer), and a **determinism constraint** (the headless golden trace must not change) — three very different pressures on your spec.
- It has a built-in falsifiable acceptance criterion: the headless `trace_digest` **must be byte-identical** before and after your change.
- It will tempt Copilot to violate Article 6 in a subtle way. Your plan review (§S.5) is where you catch it.

### S.2 Prerequisites

1. Feature 001 is merged into your `demo-0x` branch (§0.2a): `specs/001-particle-vfx-subsystem/` exists on it and all its tasks are `[x]`.
2. A green baseline — run all three and confirm zero failures before you begin:

```bash
cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

3. Record your baseline golden digest (you will diff against this in §S.8):

```bash
./build/apps/sandbox/ea-sandbox --headless --seed 42 --frames 600 --out baseline-trace.csv
# Note the printed trace_digest — this number must NOT change.
```

4. Check out your `demo-0x` branch. When you run `/speckit.specify` in §S.3, Spec-Kit creates the feature branch (e.g., `002-sandbox-vfx-visualization`) from it automatically — implement there, then merge back to `demo-0x` in §S.8, never to `main`.

### S.3 Step 1: /speckit.specify — Feature 002

Paste this prompt, then **stop and review** the generated spec at the HITL gate below.

```text
/speckit.specify

Create a specification for Feature 002: Sandbox VFX Visualization for engine_demo.

Context:
- Feature 001 (specs/001-particle-vfx-subsystem/) delivered engine_demo::vfx
  (particle_pool, force_applicator, emitter) — implemented, tested, and green.
- Its task T022 explicitly deferred wiring VFX into apps/sandbox. This feature closes
  that gap.
- The sandbox scene (apps/sandbox/scene.cpp) already has: an RMB handler that calls
  scene::spawn_particle_burst (12 ad-hoc physics particles), and world-bounds bounce
  branches for free particles in scene::substep (non-storm scenes).
- scene::state_digest() feeds the headless golden trace (apps/sandbox/headless.cpp).
  Motion trails are the existing precedent for render-only state that is excluded
  from the digest.
- See specs/constitution.md for binding constraints (articles 1-8).

Requirements:
- Right-mouse-button burst: in addition to the existing 12-particle physics burst,
  spawn a burst of engine_demo::vfx particles at the cursor (do NOT remove or alter
  the existing physics burst behavior).
- Collision sparks: when a free particle bounces off the world bounds in non-storm
  scenes, emit a small deterministic VFX spark burst at the contact point.
- VFX particles are strictly render-only: scene::state_digest() and the headless CSV
  trace are unchanged — byte-identical golden traces for identical seeds.
- The scene's existing rng draw order must not change (VFX must use its own seeded
  rng stream).
- Render pass: draw live VFX particles in the raylib renderer with lifetime-based
  alpha fade; add a live-VFX-count line to the HUD.
- Zero heap/arena allocation per burst after scene construction (Article 6).

Acceptance Criteria:
- Headless run (seed 42, 600 frames) produces the identical trace_digest before and
  after this feature.
- All existing tests stay green; every new public function has a GTest (Article 7).
- Frame budget: the sandbox holds its 60 FPS fixed-step budget with a full VFX pool.
- Out of scope: rope-break mechanics (none exist today), including VFX state in the
  digest, new emitter shapes or force types.
```

**🚨 HITL GATE — Spec review.** Before approving, verify:

- [ ] User stories are independently testable and prioritized (RMB burst should be P1 — it's the demo-able MVP).
- [ ] "Digest byte-identical" appears as a **measurable** success criterion, not a vague "should stay deterministic."
- [ ] Scope explicitly excludes rope-break and digest changes. If Copilot invented a rope-break mechanic, that's hallucinated scope — reject and re-specify.

✅ **Approve** → proceed. ❌ **Reject** → refine the prompt and re-run (see §2.2 for the rejection flow).

### S.4 Step 2: /speckit.clarify (Optional)

If your Spec-Kit install provides `/speckit.clarify`, run it now (see §1.2a for where it and the other optional quality commands fit in the flow). Good clarifying questions it might ask (and the answers this lab intends):

| Likely question                                        | Intended answer                                             |
| ------------------------------------------------------ | ----------------------------------------------------------- |
| Should storm-scene wrap-around also spark?             | No — a wrap is not a bounce/collision                       |
| VFX pool capacity?                                     | Implementer's choice; ~2048 fits the scene's 4 MiB arena    |
| Replace or augment the RMB physics burst?              | Augment — existing behavior is untouched                    |
| Should sparks fire in the particle_storm scene?        | No — its particles wrap instead of bouncing                 |

### S.5 Step 3: /speckit.plan — Review Checklist

Run `/speckit.plan` against your approved spec. **This is the stage where this lab is won or lost.** A plausible-looking plan can hide two constitutional traps. Do not approve until the plan explicitly addresses all four items:

- [ ] **The emitter-shape trap (Article 6).** `emitter`'s shape is fixed at construction, but bursts happen at arbitrary cursor/contact positions. Reconstructing an `emitter` per burst re-allocates its internal scratch buffer from the scene's bump arena on every click — an allocation leak in disguise. A good plan proposes a **small library addition** instead (e.g., `emitter::set_shape(emitter_shape) noexcept` in `include/engine_demo/vfx/emitter.h`), with its own GTest per Article 7.
- [ ] **The rng-stream trap (Article 5).** If the scene's existing `m_rng` feeds VFX sampling, every draw shifts the physics rng sequence and the golden digest changes. A good plan gives the emitter its **own seeded rng stream** (the `emitter` already owns one — the plan just must not route scene rng into it).
- [ ] **Test strategy.** `scene.cpp` is raylib-free by design — a good plan proposes compiling it into a new test target (e.g., `test_scene_vfx.cpp`) that asserts: bursts populate the pool, particles age/retire across `step()`, and **digest-with-bursts == digest-without-bursts**.
- [ ] **Arena headroom math.** The scene arena is 4 MiB. The plan should show the pool + scratch cost (~2048 particles ≈ 200 KB total) fits.

> **If the plan misses any of these, reject it** and re-prompt with the missing constraint spelled out. Catching a bad plan here costs one re-prompt; catching it in code review costs a rework cycle. This is the §2.2 rejection flow in real life.

**🚨 HITL GATE:** All four boxes checked? ✅ **Approve** → proceed to `/speckit.tasks`

### S.6 Step 4: /speckit.tasks — Sizing Gate

Run `/speckit.tasks`. Compare the generated decomposition against this expected shape (yours may differ in detail — that's fine, but the *structure* should match):

| Phase                | Expected tasks                                                                  | Test-first?                          |
| -------------------- | ------------------------------------------------------------------------------- | ------------------------------------ |
| Library              | `set_shape` test (RED) → `set_shape` impl in `emitter.h`/`.cpp` (GREEN)          | Yes — test task precedes impl task   |
| Scene integration    | `test_scene_vfx.cpp` (RED) → pool/emitter members, burst + spark emission, tick  | Yes                                  |
| Renderer             | Draw pass + HUD count in `app.cpp`, RMB handler wiring                           | Manual/screenshot verification       |
| Docs & validation    | README key table, digest A/B run, full ctest gate                                | —                                    |

**🚨 HITL GATE:** Every implementation task preceded by its test task? Each task small enough to review in ~5 minutes? Digest A/B check present as an explicit task? ✅ **Approve** → proceed.

### S.7 Step 5: /speckit.implement — One Task at a Time

Run `/speckit.implement` **one task at a time**. Between every task:

```bash
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

**Off-the-rails recovery** (from `AGENTS.md`): if Copilot drafts `std::vector`, `try`/`catch`, or an unseeded rng — stop, cite the violated article, and re-run the task. If the same task is rejected 3×, the spec is wrong: roll back to `/speckit.specify`.

### S.8 Final Verification

1. **Full gate:** `ctest --preset default-debug --output-on-failure` — everything green, including your new `set_shape` and scene-VFX tests.
2. **Golden digest A/B** — the decisive check:

```bash
./build/apps/sandbox/ea-sandbox --headless --seed 42 --frames 600 --out feature-trace.csv
# trace_digest must equal the baseline you recorded in S.2. If it differs, VFX state
# leaked into the digest or the scene rng stream shifted — find which trap you hit.
```

3. **Visual smoke:** run the sandbox, right-click — you should see the VFX burst layered over the familiar 12-particle physics burst; watch free particles spark when they hit the bounds. (In a VM without hardware GL, use `--screenshot <relative-path>` with a software renderer.)
4. **Merge back to the demo branch:**

```bash
git checkout demo-01          # your demo-0x branch
git merge --no-ff 002-sandbox-vfx-visualization
ctest --preset default-debug --output-on-failure   # merge gate
```

Both features now live on `demo-0x` — continue to the [Workshop Finale](#workshop-finale-two-features-one-branch).

### S.9 Stretch Goals and Reflection

**Stretch goals** (each is a new mini Spec-Kit cycle — resist the urge to bolt them on):

- Align bounce sparks with the collision normal using `cone_shape` instead of `point_shape`.
- Attach a `turbulence_force` to the spark emitter — then re-run the digest A/B and explain why it still passes.
- Add a per-frame emission cap so busy scenes cannot exhaust the pool.

**Reflection questions:**

1. Why did keeping VFX render-only preserve the golden digest? What exactly would including it in the digest have cost the project?
2. Where did the constitution *force* a design decision that a "just make it work" prompt would have gotten wrong? (Hint: §S.5's first two checkboxes.)
3. The `pool_exhausted` status made spark-flooding a *graceful* failure instead of a crash. Which article made that behavior inevitable, and when was it locked in — during 001 or 002?
4. Compare your total prompt count against a hypothetical single "add particle visuals to the sandbox" prompt. Where did the extra prompts buy you reviewability?

---

## Workshop Finale: Two Features, One Branch

> **⏱ ~15 minutes** | Requires Feature 001 (Part 0) and Feature 002 (Self-Study Lab) both merged into your `demo-0x` branch.

The workshop closes by proving the branching model paid off: two features, built as separate Spec-Kit cycles on separate feature branches, now run together on one demo branch — while `main` never moved.

### F.1 Verify the Merged State

```bash
git checkout demo-01                       # your demo-0x branch
git log --oneline --graph --merges -n 10   # two --no-ff merge commits visible
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure   # full gate: 001 + 002 tests green
```

The golden digest A/B (Feature 002's decisive check, §S.8) must still pass on the merged branch:

```bash
./build/apps/sandbox/ea-sandbox --headless --seed 42 --frames 600 --out finale-trace.csv
# trace_digest must equal the baseline recorded in §S.2
```

### F.2 The Visual Payoff

Run the sandbox and demonstrate both features live in a single session:

```bash
./build/apps/sandbox/ea-sandbox
```

| Action                                       | What you should see                                                    | Feature   |
| -------------------------------------------- | ---------------------------------------------------------------------- | --------- |
| Right-click anywhere                         | 12-particle physics burst **plus** a VFX particle burst at the cursor  | 001 + 002 |
| Watch free particles hit the world bounds    | Deterministic spark bursts at each contact point                       | 002       |
| Check the HUD                                | Live-VFX-count line updating as particles spawn and retire             | 002       |
| Watch the frame budget monitor               | 60 FPS fixed-step budget held with a full VFX pool (Article 6)         | 001       |

### F.3 Optional: Competition Mode

Run the finale as a head-to-head race:

1. Pair up. Each participant cuts their **own** demo branch from `main` (`demo-01`, `demo-02`, …).
2. Both run the full relay solo: Feature 001 (Part 0 flow) → merge → Feature 002 (Self-Study Lab) → merge.
3. First to a fully green finale wins: `ctest` green + digest A/B pass + both visual behaviors demonstrated side-by-side (F.2).
4. Judge's checklist:
   - [ ] Two `--no-ff` merge commits on `demo-0x`; zero new commits on `main`
   - [ ] HITL gates actually reviewed (spot-check: ask for one rejected artifact and how it was fixed)
   - [ ] Golden digest unchanged from the §S.2 baseline
   - [ ] Both visual behaviors demonstrated in one sandbox run

> **Restart rule:** if a run goes sideways at any point, don't repair the branch — abandon it and cut `demo-0(x+1)` from `main` (§0.2a). Restarts are cheap by design; that's the point of the model.

---

> **End of Workshop Training Document**
>
> This document provides complete, self-contained training material for a demo-first workshop demonstrating the value of GitHub Copilot Spec-Kit for game development. Part 0 hooks participants with a live demo, Part 1 unpacks the principles, Part 2 provides hands-on practice, Part 3 designs a new game from an existing foundation, and Part 4 transforms a game across programming languages.
>
> **Next Steps:**
>
> - Session delivery: Part 0 (20 min demo) + Parts 1–4 map to facilitated sessions
> - Hands-on exercises: Participants replicate the demonstrated flows on their own `demo-0x` branch (§0.2a)
> - Self-study: The [Self-Study Lab](#self-study-lab-visualize-the-vfx-subsystem-in-the-sandbox) has participants run the full Spec-Kit cycle solo on Feature 002 (Sandbox VFX Visualization)
> - Assessment: Use the reflection questions at the end of each section for group discussion
