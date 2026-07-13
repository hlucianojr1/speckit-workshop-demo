
### Agent Loop — Batching the Remaining Reverse-Specs

> **⏱ ~10 minutes** | One prompt generates all 5 remaining engine reverse-specs — and the same technique scales to every other reverse-spec batch in this part.

#### What an Agent Loop Is

An **agent loop** is a single Agent Mode prompt that contains four things:

1. A **work manifest** — the ordered list of items to process
2. A **per-item task template** — the same instructions applied to each item
3. **Per-item acceptance criteria** — mechanical checks the agent runs on its own output before advancing
4. A **completion report** — a summary the human reviews at the end

The agent then iterates autonomously: pick the next manifest item → execute the template → self-verify against the criteria → mark done → continue. You watch the rhythm instead of retyping the prompt five times.

**Why this task qualifies.** Not every batch of work should be looped. The remaining reverse-specs pass all three qualification tests:

| Test                         | Reverse-specs                                                               |
| ---------------------------- | --------------------------------------------------------------------------- |
| Items are independent        | Each spec reads its own header/source pair; no spec depends on another      |
| Items are identical in shape | Same 4-point extraction template, same 5-section output structure           |
| Verification is mechanical   | File exists, sections present, zero C++ syntax — checkable without judgment |

**When NOT to loop.** Never loop `/speckit.implement` tasks. Implementation tasks produce code, depend on each other, and require per-item human judgment — that's why §1.3 mandates one-task-at-a-time with a HITL gate between each. The loop here is acceptable **only because reverse-specs are read-only prose artifacts**: a bad one costs a regeneration, not a broken build. The HITL gate doesn't disappear — it moves from per-item to per-batch (see the gate below).

**How this differs from the CLI batch in §4.6 Scenario B.** Both produce the same 5 files; the mechanics and the outcome quality differ:

| Dimension         | CLI batch (§4.6 Scenario B)                   | Agent Loop (this section)                                        |
| ----------------- | --------------------------------------------- | ---------------------------------------------------------------- |
| Mechanics         | Shell loop; N isolated `copilot -p` processes | One Agent Mode session; the agent iterates internally            |
| Shared context    | None — each process starts cold               | Full — every spec sees the ecs-world template and prior outputs  |
| Output style      | Can drift between specs                       | Consistent structure and terminology across all 5                |
| Self-verification | None built in                                 | Per-item acceptance criteria checked before advancing            |
| Best for          | CI, headless environments, true parallelism   | Interactive sessions where consistency matters                   |

#### The Agent Loop Prompt

Paste this into Copilot Chat **Agent Mode** (not a `/speckit.*` command — this is a plain agent-mode prompt):

```text
You are executing an AGENT LOOP: a work manifest, a per-item task template,
per-item acceptance criteria, and a completion report.

MANIFEST — process strictly in this order, one item at a time:
1. include/engine_demo/physics/constraint.h + src/engine_demo/physics/constraint.cpp
   → specs/transform/physics-constraint.spec.md
2. include/engine_demo/sim/game_loop.h + src/engine_demo/sim/game_loop.cpp
   → specs/transform/game-loop.spec.md
3. include/engine_demo/sim/rng.h + src/engine_demo/sim/rng.cpp
   → specs/transform/rng.spec.md
4. include/engine_demo/frame_budget.h + src/engine_demo/frame_budget.cpp
   → specs/transform/frame-budget.spec.md
5. include/engine_demo/allocator.h + src/engine_demo/allocator.cpp
   → specs/transform/allocator.spec.md

PER-ITEM TASK TEMPLATE — for each manifest item:
Read both source files. Extract a language-agnostic behavioral specification
that captures:
1. What the abstraction IS (behavioral contract, not C++ class definition)
2. What operations are supported, with complexity and allocation behavior
3. What guarantees the system provides
4. What constraints are constitutional (determinism, real-time, no allocation
   in inner loops)
Use specs/transform/ecs-world.spec.md as the structural template — mirror its
section layout (Purpose / Model / Operations / Guarantees / Constraints).
Output must be implementable in ANY language: no C++ syntax, no EASTL
references, no pointer semantics.

PER-ITEM ACCEPTANCE CRITERIA — verify each item BEFORE advancing to the next:
- File written at the exact manifest path
- Contains all five sections (Purpose / Model / Operations / Guarantees /
  Constraints)
- Operations table includes Complexity and Allocates? columns
- Zero C++ syntax, EASTL references, or pointer semantics anywhere in the file
If an item fails a criterion, fix it before moving on. Do NOT stop to ask
questions mid-loop.

COMPLETION REPORT — after all 5 items, output a summary table:
| Subsystem | Output file | Line count | Criteria pass/fail |
```

**What to watch for while it runs:** the loop rhythm — read files → write spec → self-check → next item. If the agent skips the self-check or batches multiple items into one pass, interrupt it and restate the loop protocol.

**🚨 HITL GATE — batch review.** The per-item gates were delegated to the agent's self-checks; your review now happens once, at batch level:

- [ ] Completion report shows all 5 items passing all criteria
- [ ] Spot-check two specs by hand — one subsystem you know well (does the behavior match reality?) and one you don't (is it understandable without the C++?)
- [ ] Run the mechanical C++-leak check — it must return nothing:

```bash
grep -rlE 'eastl::|std::|->' specs/transform/*.spec.md
```

❌ **If any spec fails**, reject only that item: re-run the loop prompt with the manifest cut down to the failing entries. The passing specs stay.

> **Facilitator says:** "Notice how the loop reused every Spec-Kit habit you already have. The manifest is a contextual anchor (Practice 6). The per-item criteria are falsifiable objectives (Practice 1). And the human gate never disappeared — it moved to where it's cheapest: one batch review instead of five identical ones (Practice 10). That trade is safe here ONLY because specs are prose. Never make it for code."

#### Reusing the Loop for the Extension Track

The manifest above is an example, not the only shape this technique takes. The same
four-part structure (manifest / per-item template / per-item criteria / completion
report) applies directly to every batch of reverse-specs in the
[Part 4 Extension Track](part4-extension-track.md) — swap the manifest and, where the
target subsystem differs enough, the per-item template's extraction questions:

| Extension                            | Manifest (swap in)                                                                | Template changes                                                       |
| ------------------------------------ | --------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| E.1 (game layer)                     | `orbital-arena-gravity-well`, `orbital-arena-scoring`, `orbital-arena-match`      | Add the fairness/no-player-index check to the acceptance criteria       |
| E.2 (HUD + control)                  | `sandbox-hud`, `orbital-arena-interactive-control`                                 | Tag the interactive-control item **New**, not reverse-specced           |
| E.3 (VFX + particles)                | `vfx-particle-system`, `sandbox-free-particles`                                   | Add the rng-isolation and rule-isolation checks                         |
| E.4 (full closure)                   | The five gap-closing specs in that extension's table                              | Add the descope-reversal and golden-digest checks to the criteria       |

The one exception worth restating: **only reverse-specs loop.** The moment any phase's
work turns into `/speckit.implement` (extending the Rust code for A2–A5), go back to
one-task-at-a-time with a HITL gate per task — the same rule from §1.3, unchanged by any
of this.

---
