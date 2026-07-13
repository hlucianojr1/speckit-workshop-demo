# **The Rise of Spec-Driven Development: An Evaluative Architecture Report on Spec-Driven Development Frameworks**

> [!NOTE]
> **Why you're reading this.** This report is optional background reading for the Spec-Kit Workshop — it isn't required for the hands-on labs. It situates GitHub Spec Kit, the tool used throughout this workshop, inside the wider landscape of Spec-Driven Development (SDD) tooling, and collects the real, ongoing critiques of the methodology (the "Waterfall Strikes Back" debate) so you can form your own opinion instead of taking SDD on faith. Short on time? Skim the [Comparative Tool Matrix](#comparative-tool-matrix) and the [Comprehensive Trade-Off Analysis](#comprehensive-trade-off-analysis) first.

The software development lifecycle (SDLC) is undergoing an unprecedented paradigm shift. The integration of generative artificial intelligence has progressed from conversational, ad-hoc "vibe coding" to structured, agentic engineering workflows1.

> [!TIP]
> **Vibe coding** means prompting an AI agent conversationally and accepting whatever it produces, with no formal specification, plan, or review gate in between. It's fast for throwaway prototypes, but leaves nothing durable behind: no one can explain *why* the code looks the way it does once the chat history scrolls away.

While vibe coding—characterized by rapid, unconstrained conversational prompting—served as an effective mechanism for disposable prototyping, it introduces unsustainable friction in production environments2. When deployed without systematic constraints, intelligent coding agents frequently construct superficially plausible code that is architecturally incoherent, plagued by API hallucinations, and prone to silent regressions3.  
To bridge this "intent-to-code chasm," the industry has converged on a structured methodology known as Spec-Driven Development (SDD)3. This methodology treats high-level specifications as executable, version-controlled blueprints that guide autonomous agents through rigid, phase-gated implementations6. This report delivers an exhaustive analysis of GitHub's open-source Spec Kit, performs a comparative evaluation against five competing SDD frameworks (AWS Kiro, Tessl, cc-sdd, BMAD-METHOD, and OpenSpec), and contextualizes the broader systemic trade-offs, historical lineages, and security implications of the SDD movement.

## **Technical Deep Dive: GitHub Spec Kit**

GitHub Spec Kit represents a formalized attempt to bring rigorous engineering discipline to agentic coding workflows1. By treating specifications as active, living agreements rather than static documentation, Spec Kit structures the interaction space for 30+ supported AI coding agents, both CLI tools and IDE-based assistants, including GitHub Copilot, Claude Code, and Gemini CLI6, 9.

### **Repository Scaffolding and File Architecture**

Spec Kit is initialized using a Python-based command-line interface (CLI) managed by uv, a fast Python package manager1. Developers bootstrap an SDD-ready environment inside an existing or new project using the following command9:

```bash
uv tool install specify-cli --from git+https://github.com/github/spec-kit.git@vX.Y.Z
```

The execution of the specify init <PROJECT_NAME> command automatically configures two primary directories within the root of the repository1. The structure and role of these directories are mapped out in the following table:

| Directory Path | Component File | Structural and Operational Role in SDD |
| :---- | :---- | :---- |
| **.github/** | prompts/speckit.\<cmd\>.prompt.md | Contains pre-configured, system-level prompts loaded by the IDE coding assistant to guide agent behavior during specific steps of the workflow8. |
| **.specify/** | memory/constitution.md | Serves as the repository's foundational ruleset, establishing project-wide invariants, design patterns, coding styles, and non-negotiable principles1. |
|  | templates/ | Stores markdown templates for generating standardized specifications, technical plans, and task breakdowns1. |
|  | scripts/ | Houses POSIX-compliant shell scripts or PowerShell scripts that orchestrate directory management, branch creation, and git automation1. |

Beyond this core scaffolding, Spec Kit ships a three-layer customization system9: **extensions** add new commands (for example, a Jira integration or a post-implementation code-review step), **presets** override the format of existing templates and commands (for example, enforcing a compliance-oriented spec structure), and **bundles** package a curated set of extensions, presets, and workflows into a single role-based setup (product manager, security researcher, developer, and so on) that a team installs with one command. Templates resolve through a priority stack, project-local overrides first, then presets, then extensions, then Spec Kit's own core templates, so organizations can standardize the SDD workflow without forking the tool9.

### **The Spec Kit Phase-Gated Lifecycle**

Rather than immediately prompting an agent to write code, the developer guides the assistant through a deterministic sequence of state transitions via specialized slash commands within the agent's chat interface8:

\[Constitution\] ──\> \[Specify\] ──\> \[Clarification & Checklist\] ──\> \[Plan\] ──\> \[Tasks\] ──\> \[Analysis\] ──\> \[Implementation\] ──\> \[Issues / Converge (optional)\]

1. **Establishing Invariants (/speckit.constitution):** The agent generates or reads the project's core constitution document9. This file outlines systemic rules—such as enforcing test-driven development, conventional commit guidelines, or specific dependency management schemes—that must govern all subsequent code generation1.  
2. **Functional Specification (/speckit.specify):** The developer provides a natural-language description outlining the "what" and "why" of a new feature8. The agent parses this input and generates a formal, high-level functional specification (spec.md) that explicitly omits technical stack choices or implementation details8.  
3. **Ambiguity Clarification (/speckit.clarify):** The agent runs a dedicated gap-analysis pass on the generated spec.md8. It surfaces implicit assumptions, architectural edge cases, and missing requirements, prompting the developer with targeted, sequential questions to refine the specification before moving forward8.  
4. **Validation Checklisting (/speckit.checklist):** The agent generates automated quality checklists13. Often described as "unit tests for English," these checklists establish objective verification criteria to ensure the spec is logically cohesive, complete, and verifiable13.  
5. **Technical Blueprinting (/speckit.plan):** With the functional spec approved, the developer inputs the desired tech stack, performance constraints, and architectural patterns9. The agent outputs plan.md, documenting database schemas, API contracts, folder hierarchies, and library dependencies9.  
6. **Task Decomposition (/speckit.tasks):** The technical plan is parsed and decomposed into an ordered, dependency-aware graph of atomic, testable, and isolated coding tasks stored in tasks.md9.  
7. **Consistency Analysis (/speckit.analyze):** An optional but recommended quality gate where the agent scans for alignment gaps across all generated artifacts13. It flags inconsistencies, such as a user story in spec.md with no corresponding task in tasks.md, or a database schema reference in plan.md that is missing from the underlying data model documentation13.  
8. **Automated Execution (/speckit.implement):** The agent iterates through the task list, generating code files, writing corresponding unit tests, executing local test suites, and committing successful changes incrementally9.
9. **Issue Handoff (/speckit.taskstoissues):** An optional command that converts the generated task list into GitHub Issues for teams that track work in Issues rather than in tasks.md directly, preserving dependency ordering9.
10. **Continuous Convergence (/speckit.converge):** Run after an initial implementation pass, this command re-checks the current codebase against spec.md, plan.md, and tasks.md, and appends any remaining or drifted work as new tasks, useful for brownfield loops where code and spec keep evolving after the first release9.

By splitting development into separate planning and execution phases, Spec Kit isolates the stable "what" of a business requirement from the highly volatile "how" of its implementation6. This separation of concerns allows developers to reuse specs to prototype multiple parallel technical architectures, validate requirements before committing capital, and safely refactor or rebuild legacy software without carrying forward historical technical debt6.

## **Competitive Landscape: Kiro, Tessl, cc-sdd, BMAD-METHOD, and OpenSpec**

The SDD movement is characterized by a rapid expansion of competing frameworks, each offering distinct interpretations of how to manage agentic interactions11. To understand GitHub Spec Kit's position in the industry, it must be evaluated alongside alternative frameworks such as AWS Kiro, Tessl, the open-source cc-sdd harness, BMAD-METHOD, and OpenSpec11, 53, 54.

### **AWS Kiro: Formal Verification and the Sunset of Q Developer**

AWS Kiro represents a major enterprise-level push toward specification-centric environments7. Following a strategic re-platforming, AWS reportedly blocked new Amazon Q Developer signups on May 15, 2026, with a full end-of-support date targeted for April 30, 2027, to transition its developer ecosystem to Kiro21; confirm these dates against AWS's own Amazon Q Developer end-of-life notice before relying on them for migration planning. Unlike standard IDE plug-ins, Kiro is a customized, Code OSS-based IDE and CLI toolchain built on Amazon Bedrock, designed around spec-driven development as its core primitive2.  
The defining technical advantage of AWS Kiro is its first-party integration of the Easy Approach to Requirements Syntax (EARS) notation23.

> [!TIP]
> **EARS notation** structures a requirement into a fixed grammar so it can be parsed and checked for contradictions the way code is: `WHEN <optional trigger> THE <system> SHALL <response>`. It was developed decades before AI coding agents existed, originally to reduce ambiguity in safety-critical requirements documents. SDD tools like Kiro borrow it because a machine-checkable grammar is exactly what an LLM needs to catch a contradictory or missing requirement before it writes any code.

This structured syntax allows Kiro to run automated reasoning engines over requirements documents to identify contradictions and logical gaps before code is generated24. Kiro leverages this formal requirements structure to automatically generate property-based tests, such as fuzz testing, validating that architectural invariants hold true across the entire input space24.  
AWS introduced a "Kiro Pro Max" tier in June 2026, granting developers high-volume access to frontier reasoning models27. Kiro also reportedly offers a companion mobile app for monitoring and approving agentic sessions28; as with the Q Developer sunset dates, verify current tier names and mobile feature availability against Kiro's own release notes, since AWS revises pricing and packaging frequently.

### **The Tessl Framework: From "Spec-as-Source" to Skills Governance**

If Spec Kit and Kiro focus on "spec-first" planning to guide manual or semi-autonomous development, Tessl originally staked out the most ambitious rung of the SDD ladder: "spec-as-source"11. Founded by Guy Podjarny, Tessl reportedly secured over $125 million in seed and Series A funding at a $750 million valuation before launching its commercial product, reflecting strong early industry interest in the approach29.  
Tessl's original framework operated on the premise that in an AI-native world, source code is an impermanent, compiled artifact, and that the natural-language specification is the only canonical file maintained by human developers29. It established a strict 1:1 mapping between spec files and generated files11, prefixing generated code with a warning comment:

```text
// GENERATED FROM SPEC - DO NOT EDIT
```

To execute a change under this model, the developer updates the natural-language specification31; Tessl's compiler then regenerates the affected code and runs a localized verification suite scoped by metadata tags like `@generate` and `@test`11.

> [!NOTE]
> **This section has aged since the underlying research was gathered.** As of mid-2026, Tessl's own marketing has shifted away from the spec-to-code compiler pitch above and toward governing the "skills" (reusable, versioned instructions) that agentic coding tools like Claude Code, Cursor, and Copilot consume. Its current products (Tessl Registry, Tessl Agent, Tessl Academy) are framed around security scanning, adoption tracking, and evaluating those skills at enterprise scale, rather than 1:1 spec-to-code generation55. Treat the paragraph above as a record of Tessl's original positioning, not its current product line, and check the current docs before recommending any tool to a team; this space moves fast enough that a report like this one is a snapshot, not a permanent map.

The registry concept persists across both eras of Tessl's positioning: what was pitched as the "Tessl Spec Registry" of over 10,000 version-accurate, pre-built specs for open-source libraries4 is now described as a searchable registry of 3,000+ skills55. In either framing, the goal is the same: give agents a version-accurate description of a library's API so they stop hallucinating integration code against outdated training weights4.

### **Collaborative Spec-Driven Development (cc-sdd)**

For teams seeking lightweight, open-source alternatives that integrate across multiple commercial coding agents (such as Cursor, Windsurf, Copilot, and Claude Code), the cc-sdd harness provides a highly functional framework19. Rather than treating the specification as an authoritarian command document, cc-sdd views it as an explicit, machine-readable contract between systems, allowing code to remain the ultimate source of truth19.  
The cc-sdd toolchain implements a 17-skill agentic workflow structured around two key commands19:

* /kiro-discovery: Routes incoming requirements into discovery pipelines, automatically generating a lightweight product brief (brief.md) and roadmap (roadmap.md) to establish scope without overloading context windows19.  
* /kiro-impl: Executes an autonomous, multi-agent development loop19. It creates a dedicated context for each individual task, running a test-driven development (TDD) cycle (Red → Green → Refactor) behind feature flags19. A separate reviewer agent validates the output, and a specialized auto-debugger is triggered to resolve compile-time blockers19.

### **BMAD-METHOD: Agile AI-Driven Development**

BMAD-METHOD (Breakthrough Method of Agile AI-Driven Development) takes a role-based rather than phase-gated approach to SDD53. Instead of a single agent working through a linear spec-plan-tasks-implement pipeline, BMAD installs a roster of specialized AI personas, an Analyst, Product Manager, Architect, Scrum Master, Developer, and Test Architect among them, each with its own guided workflow and deliverable template53. A "scale-adaptive" planning layer routes a one-line bug fix through a lightweight path and a new platform build through the full analysis-to-architecture sequence, using the same underlying agent roster53.  
BMAD is free and open source (MIT-licensed) and ships a plugin and module marketplace so teams can add domain-specific workflows, including a dedicated game-development module targeting Unity, Unreal, and Godot, on top of the core Agile suite53. Because BMAD leans on structured upfront interviews to build its planning artifacts before any code is written, it is also the primary target of the Domain-Driven Design critique discussed in [Domain-Driven Design and Upfront vs. Continuous Discovery](#domain-driven-design-and-upfront-vs-continuous-discovery) below44.

### **OpenSpec: Change-Driven Development**

OpenSpec inverts the relationship between specs and change requests that most other SDD frameworks assume54. Rather than treating a single spec.md as the perpetual source of truth, OpenSpec keeps a specs/ directory that reflects what is currently built and deployed, and a separate changes/ directory holding proposals, proposal.md, tasks.md, an optional design.md, and a set of spec deltas, for what should change next54. Once a change is implemented and approved, its delta specs are merged into the main specs/ tree and the change folder is archived, leaving a permanent, auditable history of how the system's understood behavior evolved54.  
This change-driven framing is a direct response to one of the sharpest critiques of SDD: by keeping specs anchored to deployed behavior and routing all proposed changes through small, reviewable diffs, OpenSpec avoids asking teams to front-load a complete domain model before writing any code. OpenSpec is free, open source, requires no API keys, and its CLI integrates with a range of agents including Claude Code, Cursor, Cline, and Crush54.

### **Comparative Tool Matrix**

The operational mechanics, capabilities, and system requirements of these frameworks are structured for comparison in the following matrix:

| Architectural Vector | GitHub Spec Kit | AWS Kiro | Tessl Framework | cc-sdd Harness | BMAD-METHOD | OpenSpec |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| **Development Scope** | Greenfield & Brownfield6 | Greenfield AWS focus23 | Skills/registry governance across any stack (formerly isolated CRUD & API generation)31, 55 | Multi-Agent Pipelines19 | Bug fix to enterprise platform, scale-adaptive53 | Any stack; change-proposal driven brownfield loops54 |
| **Maturity Level** | Spec-First to Spec-Anchored11 | Spec-Anchored2 | Spec-as-Source (original) / skills-as-governed-artifacts (current)11, 55 | Spec-Anchored / TDD-first19 | Spec-First, role-based upfront planning53 | Spec-Anchored, change-driven54 |
| **Verification Strategy** | Static consistency checks (/speckit.analyze)13 | Formal EARS parsing and property-based fuzz tests24 | Registry-level skill security scans and evals (formerly @test annotations)14, 55 | Multi-agent review passes and TDD validation loops19 | Dedicated Test Architect agent generates tests from the existing framework53 | `openspec validate --strict` structural checks on change proposals54 |
| **Host Environment** | Agnostic; CLI integrates with 30+ agents1, 9 | Code OSS-based standalone desktop IDE plus companion mobile app23 | CLI/registry integrated via Model Context Protocol (MCP); skills consumed by Claude Code, Cursor, Copilot, Gemini3, 55 | Portable shell integration across 8+ major agents19 | Works alongside Claude, Cursor, GitHub Copilot, and other assistants53 | CLI integrates with Cline, Crush, and other agents; no API keys required54 |
| **Context Management** | Static memory files, template variables, plus presets/extensions/bundles1, 9 | Dynamic "steering files" (product.md, tech.md)22 | Registry spec/skill packs loaded per project4, 55 | Boundary-first roadmap and implementation note propagation19 | Persona-specific context scoped to the active workflow step53 | Change-scoped context; each proposal's tasks/design/deltas travel together54 |
| **License / Cost Model** | Free, open source (MIT)9 | Commercial, tiered (Free/Pro/Pro Max) via AWS26, 27 | Commercial (enterprise) with a free registry tier55 | Free, open source19 | Free, open source (MIT)53 | Free, open source, no API keys required54 |
| **Primary Artifact Unit** | Task file (tasks.md) within a numbered feature branch9 | EARS-notated requirement plus steering file23 | Versioned skill/spec package in the registry55 | Per-task agent context plus TDD cycle artifacts19 | Role-specific deliverable (PRD, architecture doc, story) per persona53 | Change proposal folder (proposal.md, tasks.md, design.md, spec deltas)54 |

## **Conceptual Frameworks: Historical Lineages and the SDD Ladder**

To assess the impact of Spec-Driven Development, we must examine its theoretical placement within software engineering history34. Rather than representing entirely new computer science paradigms, SDD is a reconfiguration of historical concepts designed to address the unique capabilities of modern generative models32.

> [!TIP]
> **Reading the ladder below:** think of it as three answers to the same question, "which file do I edit when behavior needs to change?" Spec-First: you edit code; the spec was just a starting point. Spec-Anchored: you edit the spec and the code, in that order, every time. Spec-as-Source: you edit only the spec, and the code is regenerated for you.

                \[SPEC-AS-SOURCE\] ── (Code is a compiled, ephemeral artifact)  
                      ▲  
                      │  
               \[SPEC-ANCHORED\]  ── (Specs and code are versioned in lockstep)  
                      ▲  
                      │  
                \[SPEC-FIRST\]    ── (Upfront spec guides initial code draft)

The SDD movement operates across three levels of maturity along a "ladder of ambition"31:

* **Spec-First:** The specification is drafted to align the developer and the agent before code generation11. Once the implementation is drafted, the code becomes the primary maintained artifact17.  
* **Spec-Anchored:** The specification remains an active, version-controlled document residing alongside the codebase11. Any functional modification requires updating the spec first, then updating the code16.  
* **Spec-As-Source:** The logical end-state where developers only modify specifications31. The source code is generated and validated by agents, meaning humans never manually edit code14.

### **The Model-Driven Development Parallel**

> [!NOTE]
> **MDD and CASE, briefly.** Model-Driven Development (MDD) and Computer-Aided Software Engineering (CASE) were 1990s-2000s attempts to generate production code directly from visual models (UML diagrams) or domain-specific languages, rather than from natural-language prose. SDD is attempting to solve the same problem MDD tried to solve, with LLMs standing in for rigid template parsers.

The ambition of the Spec-as-Source paradigm shares a direct lineage with the Model-Driven Development (MDD) and Computer-Aided Software Engineering (CASE) initiatives of the late 1990s and early 2000s5. MDD attempted to use Unified Modeling Language (UML) diagrams or Domain-Specific Languages (DSLs) to compile production software directly from abstract specifications5.  
MDD ultimately struggled to gain mainstream traction due to fundamental limitations31:

1. **Leaky Abstractions:** Abstract specifications could not easily account for niche, low-level implementation details, forcing developers to manually patch generated code31.  
2. **Generator Overhead:** Maintaining custom code generators and parser templates often introduced more operational complexity than writing the code manually11.  
3. **Regeneration Variance:** Minor updates to a UML model often caused massive, chaotic changes to the generated code files, rendering standard debugging and git-diff tracking unusable31.

Modern SDD attempts to resolve these problems by replacing rigid, deterministic template parsers with probabilistic, reasoning-capable LLMs15. The natural-language processing capabilities of modern models allow specifications to be authored in clean, human-readable markdown or structured EARS notation4. This eliminates the need for highly specialized, visual modeling tools while maintaining logical alignment between intent and execution7.

## **Causal Analysis: The "Waterfall Strikes Back" Critique**

The rapid adoption of SDD has sparked an ideological debate, with several prominent software engineering leaders critiquing the methodology as a regression toward outdated, rigid planning models39.

### **The Return to Upfront Documentation**

In a widely discussed critique, François Zaninotto argued that SDD "revives the old idea of heavy documentation before coding, an echo of the Waterfall era"41. The Waterfall model—characterized by upfront requirements gathering followed by sequential execution gates—was largely replaced by Agile methodologies because software requirements are fundamentally fluid39. Agile recognized that teams discover the true constraints of a system only during implementation, and that freezing requirements months in advance invites failure40.  
Zaninotto's critique highlights an emerging pattern: as developers rely on agents to handle larger volumes of unsupervised work, they spend more time writing functional design specifications40. In an empirical trial using Spec Kit to implement a simple date-display component, Marmelab documented that the task generated 8 distinct specification files comprising over 1,300 lines of markdown38. Critics suggest that this shift transforms software developers into bureaucratic requirements clerks, performing the exact administrative work many chose to avoid40.

### **Domain-Driven Design and Upfront vs. Continuous Discovery**

This critique is shared by practitioners of Domain-Driven Design (DDD)44. SDD tools like BMAD-METHOD (see [BMAD-METHOD: Agile AI-Driven Development](#bmad-method-agile-ai-driven-development) above) operate on the assumption that software discovery can be front-loaded through automated, upfront AI interview sessions44.  
However, DDD principles state that a truly accurate domain model cannot be isolated from the physical constraints of implementation44. The conceptual frictions and edge cases that emerge during active coding must continuously inform and reshape the domain model44. Because SDD places planning before implementation, it risks decoupling the domain experts from the active feedback loop, creating a rigid structure where agents build against an incomplete, idealized model44.

### **The Counter-Perspective: Agile on Fast-Forward**

Proponents of SDD argue that the Waterfall comparison misses a critical architectural shift: the feedback loop has collapsed from months to minutes45.

Waterfall Loop:  \[Upfront Design\] ───────────────────────────\> \[Code Delivery (6+ Months)\]  
                                                                     │  
                                                                     ▼  
Agile Loop:      \[Sprint Planning\] ──\> \[Two-Week Cycle\] ───────\> \[Shipped Code (14 Days)\]  
                                                                     │  
                                                                     ▼  
SDD Loop:        \[Markdown Spec\]   ──\> \[Agent Execution\] ──────\> \[Code Run (20 Minutes)\]

In the Waterfall model, discovering a requirements mismatch during late-stage testing was financially and operationally catastrophic43. In an SDD workflow, the developer can update the specification, trigger a complete reconstruction of the code, and evaluate the generated implementation within a single afternoon45.  
Furthermore, agents have no professional ego46. In traditional teams, refactoring a system to accommodate a late requirement is often met with resistance due to human sunk-cost bias43. An agent will discard thousands of lines of generated code and reconstruct them from scratch without hesitation if the underlying specification is updated6. This rapid cycle transforms the spec from a bureaucratic contract into an active, highly responsive feedback mechanism45.

## **Comprehensive Trade-Off Analysis**

Organizations evaluating the adoption of Spec-Driven Development must balance its structural benefits against several operational risks10. The primary trade-offs of the SDD methodology are compiled and analyzed below:

### **Advantages of Spec-Driven Development**

* **Drastic Reduction in AI Drift:** Constraining agents with an immutable constitution.md and detailed functional specs limits the model's search space1. This prevents the AI from introducing unwanted code patterns or choosing inappropriate implementation paths8.  
* **Self-Updating Documentation:** In a Spec-Anchored workflow, the specifications serve as the living documentation of the system1. Because specifications must be updated to drive code changes, the documentation naturally stays in sync with the codebase, avoiding the drift common in standard wikis1.  
* **Systemic Compliance Enforcement:** Complex architectural rules, accessibility requirements, and security policies are documented as invariants in the constitution1. The agent is forced to validate all generated code against these rules, ensuring that compliance is treated as an active, continuous quality gate1.  
* **Reduced Context Rot:** By dividing the implementation into small, task-based files, the agent processes localized, short-lived sessions33. This keeps the agent's context window clean and prevents performance degradation caused by bloated chat histories47.

### **Disadvantages of Spec-Driven Development**

* **The Markdown Review Burden:** Under an SDD model, developers are often forced to review long, repetitive, and verbose markdown files generated by AI11. This can introduce cognitive fatigue, occasionally leading developers to approve specs with hidden logic flaws that require intensive debugging later10.  
* **Instruction Bloat and Context Decay:** As a project grows, the cumulative set of instructions, constitutions, and specifications can overwhelm the agent's context window10. This can cause the agent to ignore critical constraints, generate redundant code, or fail to follow its own rules10.  
* **Extreme Friction for Small Tasks:** For quick bug fixes or simple code changes, the administrative overhead of writing a specification, generating a plan, and decomposing tasks introduces unnecessary friction11. In these scenarios, traditional manual coding is often faster and more efficient11.  
* **False Sense of Security:** Even with highly detailed specs and strict constraints, agents remain probabilistic engines37. Studies have shown instances where agents marked verification tasks as complete without writing any unit tests, highlighting that manual verification remains necessary20.

## **Future Outlook: Harness Engineering and Agentic Security**

As the SDD movement matures, the responsibilities of software engineers are shifting from manual implementation to architectural oversight and system validation24. This transition has driven the emergence of two new disciplines: **Harness Engineering** and **Agent Enablement**49.

### **Harness Engineering and Agent Enablement**

> [!NOTE]
> **Harness engineering**, here, means building the automated verification rig (the "harness") that an autonomous agent's output must pass before a human ever looks at it: linters, property-based tests, sandboxed execution, and the like. The term borrows from hardware-in-the-loop testing, where a physical test harness validates a device before it ships.

Harness Engineering is the practice of designing robust, isolated testing and verification environments specifically tailored for autonomous coding agents38. Because agents cannot determine "intent" on their own, the harness serves as the objective gatekeeper3. It validates generated code against the spec using advanced property-based tests, static analysis tools, and sandboxed execution runs before any changes are committed to the main codebase24.  
Concurrently, organizations are establishing dedicated Agent Enablement teams49. Sitting at the intersection of Platform Engineering and DevOps, these teams are responsible for managing the organization's agentic infrastructure49. They define standard guidelines for skills, maintain the enterprise's private spec registries, and manage the prompt architectures that ensure agents work safely and consistently across the company30.

### **The Security Shift: Securing the Builder, Not the Code**

The rise of autonomous agentic development challenges traditional security practices37. When agents can generate and refactor thousands of lines of code in minutes, static code analysis scanners cannot keep pace37. As Guy Podjarny summarized, the industry must transition from "securing the code" to "securing the coder"37.  
This security paradigm recognizes that the agent's instructions—its skills, custom tools, and context files—are now the primary target for attacks37. Security vulnerabilities are emerging in these agentic systems:

* **Malicious Skills:** Attackers can seed public package and spec registries with compromised skills37. For example, a skill designed as a standard API helper can contain hidden instructions that direct the agent to download compromised dependencies during code execution37.  
* **Vulnerable Custom Tools:** Agents configured with open Model Context Protocol (MCP) servers can be manipulated via prompt injection, potentially allowing external parties to execute unauthorized commands on the developer's machine or access sensitive repository variables37.

To address these risks, platforms are adopting "agentic security" models37. Tools like AWS Continuum automatically generate threat models from design specs, run continuous security scans on pull requests, and validate agent-suggested remediations in sandboxed environments before they reach production28.

## **Strategic Conclusions and Recommendations**

Spec-Driven Development represents a major milestone in software engineering's adaptation to generative AI1. While the methodology introduces some administrative overhead and challenges with document maintenance, it provides the structured boundaries necessary to run autonomous agents safely and predictably in enterprise environments4.  
To implement SDD successfully without drowning in unnecessary documentation, organizations should adopt the following structured guidelines:

### **Establishing an SDD Hierarchy based on Project Context**

Organizations should select the appropriate SDD maturity level based on project duration, team size, and compliance requirements, as outlined in the following table:

| Maturity Level | Target Use Case and Project Type | Operational Implementation Guide |
| :---- | :---- | :---- |
| **Spec-First** | Low-risk prototypes, small features, personal scripts, and non-critical bug fixes31. | Write a simple, one-page markdown document outlining the feature's core goals, non-goals, and basic success criteria before prompting the agent38. Let the agent generate the implementation, then maintain the code manually going forward11. |
| **Spec-Anchored** | Core business services, features slated for long-term maintenance, and collaborative team projects31. | Maintain specifications in the repository alongside the code36. Enforce a strict policy that the specification must be updated and approved *before* the agent is allowed to modify the implementation31. |
| **Spec-as-Source** | Standard CRUD screens, repetitive database integrations, boilerplate code, and clean-slate migrations31. | Treat the specification as the primary source file31. Prevent developers from manually editing the generated files, relying on automated test suites and compiler validation to verify regenerated code11. |

### **Practical Guidelines for Minimizing Documentation Overhead**

To prevent SDD workflows from becoming overly bureaucratic, engineering teams should follow these key practices:

1. **Keep Steering Files Lean:** Ensure that foundational rulesets like constitution.md and persistent steering files remain under 300 lines10. Focus exclusively on core architectural boundaries, naming conventions, and security rules to avoid instruction bloat1.  
2. **Use Markdown Checklists for Fast Reviews:** Do not force developers to read through verbose, AI-generated design prose42. Require agents to output standard markdown checklists that explicitly map user stories directly to verification tests13.  
3. **Implement Direct Skip-Paths:** Allow developers to bypass the full SDD workflow for minor fixes and small changes, saving the formal planning phases for more complex feature work20.

Ultimately, the goal of Spec-Driven Development is not to produce more documentation, but to build a stable, reliable framework where human engineers set the strategic direction and autonomous agents safely handle the execution6.

#### **Works cited**

1. Diving Into Spec-Driven Development With GitHub Spec Kit \- Microsoft for Developers, [https://developer.microsoft.com/blog/spec-driven-development-spec-kit](https://developer.microsoft.com/blog/spec-driven-development-spec-kit)  
2. Amazon Kiro AI IDE: Spec-Driven Development \- Tutorials Dojo, [https://tutorialsdojo.com/amazon-kiro-ai-ide-spec-driven-development/](https://tutorialsdojo.com/amazon-kiro-ai-ide-spec-driven-development/)  
3. Spec-Driven Development with Tessl, [https://docs.tessl.io/use/spec-driven-development-with-tessl](https://docs.tessl.io/use/spec-driven-development-with-tessl)  
4. How Tessl's Products Pioneer Spec-Driven Development, [https://tessl.io/blog/how-tessls-products-pioneer-spec-driven-development/](https://tessl.io/blog/how-tessls-products-pioneer-spec-driven-development/)  
5. Spec-Driven Development vs Vibe Coding: Tools & Guide (2026) \- Turing Post, [https://www.turingpost.com/p/sdd](https://www.turingpost.com/p/sdd)  
6. Spec-driven development with AI: Get started with a new open source toolkit \- The GitHub Blog, [https://github.blog/ai-and-ml/generative-ai/spec-driven-development-with-ai-get-started-with-a-new-open-source-toolkit/](https://github.blog/ai-and-ml/generative-ai/spec-driven-development-with-ai-get-started-with-a-new-open-source-toolkit/)  
7. Comprehensive Guide to Spec-Driven Development Kiro, GitHub Spec Kit, and BMAD-METHOD | by Vishal Mysore | Medium, [https://medium.com/@visrow/comprehensive-guide-to-spec-driven-development-kiro-github-spec-kit-and-bmad-method-5d28ff61b9b1](https://medium.com/@visrow/comprehensive-guide-to-spec-driven-development-kiro-github-spec-kit-and-bmad-method-5d28ff61b9b1)  
8. GitHub Spec-Kit: From Vibe Coding to Spec-Driven Development \- DEV Community, [https://dev.to/petersaktor/github-spec-kit-from-vibe-coding-to-spec-driven-development-1pgd](https://dev.to/petersaktor/github-spec-kit-from-vibe-coding-to-spec-driven-development-1pgd)  
9. GitHub \- github/spec-kit: Toolkit to help you get started with Spec-Driven Development, [https://github.com/github/spec-kit](https://github.com/github/spec-kit)  
10. GitHub Spec Kit | Technology Radar | Thoughtworks United States, [https://www.thoughtworks.com/en-us/radar/languages-and-frameworks/github-spec-kit](https://www.thoughtworks.com/en-us/radar/languages-and-frameworks/github-spec-kit)  
11. Understanding Spec-Driven-Development: Kiro, spec-kit, and Tessl \- Martin Fowler, [https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html](https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html)  
12. I cannot say enough good things about github's spec-kit, [https://www.reddit.com/r/vibecoding/comments/1sykoln/i\_cannot\_say\_enough\_good\_things\_about\_githubs/](https://www.reddit.com/r/vibecoding/comments/1sykoln/i_cannot_say_enough_good_things_about_githubs/)  
13. Meet GitHub Spec-Kit: An Open Source Toolkit for Spec-Driven Development with AI Coding Agents \- MarkTechPost, [https://www.marktechpost.com/2026/05/08/meet-github-spec-kit-an-open-source-toolkit-for-spec-driven-development-with-ai-coding-agents/](https://www.marktechpost.com/2026/05/08/meet-github-spec-kit-an-open-source-toolkit-for-spec-driven-development-with-ai-coding-agents/)  
14. 2.0.1 • spec-driven-development • tessl-labs • Registry, [https://tessl.io/registry/tessl-labs/spec-driven-development](https://tessl.io/registry/tessl-labs/spec-driven-development)  
15. Spec driven development: a guide to moving beyond vibe-coding with AI | SparkFabrik, [https://www.sparkfabrik.com/en/blog/spec-driven-development-guide/](https://www.sparkfabrik.com/en/blog/spec-driven-development-guide/)  
16. Exploring Spec Driven Development (SDD)- A Practical Guide with GitHub SpecKit and Copilot | by JingJing (Chris) Bao | Level Up Coding, [https://levelup.gitconnected.com/exploring-spec-driven-development-sdd-a-practical-guide-with-github-speckit-and-copilot-72fd9a70535a](https://levelup.gitconnected.com/exploring-spec-driven-development-sdd-a-practical-guide-with-github-speckit-and-copilot-72fd9a70535a)  
17. 4 cutting-edge tools for spec-driven development \- InfoWorld, [https://www.infoworld.com/article/4171332/4-cutting-edge-tools-for-spec-driven-development.html](https://www.infoworld.com/article/4171332/4-cutting-edge-tools-for-spec-driven-development.html)  
18. Spec-driven development | Technology Radar | Thoughtworks United States, [https://www.thoughtworks.com/en-us/radar/techniques/spec-driven-development](https://www.thoughtworks.com/en-us/radar/techniques/spec-driven-development)  
19. GitHub \- gotalab/cc-sdd: Turn approved specs into long-running autonomous implementation. A minimal, adaptable SDD harness with Agent Skills for Claude Code, Codex, Cursor, Copilot, Windsurf, OpenCode, Gemini CLI, and Antigravity., [https://github.com/gotalab/cc-sdd](https://github.com/gotalab/cc-sdd)  
20. Understanding Spec-Driven-Development: Kiro, spec-kit, and Tessl \- daily.dev, [https://daily.dev/posts/understanding-spec-driven-development-kiro-spec-kit-and-tessl-wbu9w3aj6](https://daily.dev/posts/understanding-spec-driven-development-kiro-spec-kit-and-tessl-wbu9w3aj6)  
21. Amazon Q to Kiro: Complete Migration Playbook for 2026 \- Digital Applied, [https://www.digitalapplied.com/blog/amazon-q-to-kiro-migration-playbook](https://www.digitalapplied.com/blog/amazon-q-to-kiro-migration-playbook)  
22. Kiro Documentation \- AWS \- Amazon.com, [https://aws.amazon.com/documentation-overview/kiro/](https://aws.amazon.com/documentation-overview/kiro/)  
23. Kiro vs Intent (2026): AWS Spec-Driven IDE vs Living Specs Platform — Which Wins?, [https://www.augmentcode.com/tools/intent-vs-kiro](https://www.augmentcode.com/tools/intent-vs-kiro)  
24. DEV Track Spotlight: Spec-driven development with Kiro (DEV314), [https://dev.to/aws/dev-track-spotlight-spec-driven-development-with-kiro-dev314-45e8](https://dev.to/aws/dev-track-spotlight-spec-driven-development-with-kiro-dev314-45e8)  
25. Kiro: Move beyond AI coding to agentic engineering, [https://kiro.dev/](https://kiro.dev/)  
26. Blog \- Kiro, [https://kiro.dev/blog/](https://kiro.dev/blog/)  
27. AWS Weekly Roundup: AWS FinOps Agent in preview, Gemma 4 on Bedrock, Kiro Pro Max, and more (June 15, 2026\) | AWS News Blog, [https://aws.amazon.com/blogs/aws/aws-weekly-roundup-aws-finops-agent-in-preview-gemma-4-on-bedrock-kiro-pro-max-and-more-june-15-2026/](https://aws.amazon.com/blogs/aws/aws-weekly-roundup-aws-finops-agent-in-preview-gemma-4-on-bedrock-kiro-pro-max-and-more-june-15-2026/)  
28. Top announcements of the AWS Summit in New York, 2026, [https://aws.amazon.com/blogs/aws/top-announcements-of-the-aws-summit-in-new-york-2026/](https://aws.amazon.com/blogs/aws/top-announcements-of-the-aws-summit-in-new-york-2026/)  
29. Tessl Review (2026): The Spec-as-Source Bet \- CodeMySpec, [https://codemyspec.com/blog/tessl-review](https://codemyspec.com/blog/tessl-review)  
30. Tessl launches spec-driven development tools for reliable AI coding agents, [https://tessl.io/blog/tessl-launches-spec-driven-framework-and-registry/](https://tessl.io/blog/tessl-launches-spec-driven-framework-and-registry/)  
31. The Three Levels of SDD \- The AI Agent Factory \- Panaversity, [https://agentfactory.panaversity.org/docs/General-Agents-Foundations/spec-driven-development/three-levels-of-sdd](https://agentfactory.panaversity.org/docs/General-Agents-Foundations/spec-driven-development/three-levels-of-sdd)  
32. Spec-Driven Development for AI Agents, Done Right: Specs as Governed Artifacts, [https://www.truefoundry.com/blog/spec-driven-development-ai-agents](https://www.truefoundry.com/blog/spec-driven-development-ai-agents)  
33. Experience with Kiro's spec driven development methodology | by Kanai Dutta | Medium, [https://medium.com/@kanaiduttaiem/experience-with-kiros-spec-driven-development-methodology-1e57af895fd7](https://medium.com/@kanaiduttaiem/experience-with-kiros-spec-driven-development-methodology-1e57af895fd7)  
34. Spec-Driven Development is Better With Core Architecture, [https://www.architectureandgovernance.com/applications-technology/spec-driven-development-is-better-with-core-architecture/](https://www.architectureandgovernance.com/applications-technology/spec-driven-development-is-better-with-core-architecture/)  
35. Spec-Driven Development: workflow, narzędzia i ryzyka \- Selleo, [https://selleo.com/blog/spec-driven-development](https://selleo.com/blog/spec-driven-development)  
36. Part 1: One Spec To Rule Them All \- DEV Community, [https://dev.to/ttypic/part-1-one-spec-to-rule-them-all-hf5](https://dev.to/ttypic/part-1-one-spec-to-rule-them-all-hf5)  
37. Securing the Coder, Not the Code: Notes on Agentic Development and Security \- Tessl, [https://tessl.io/blog/securing-the-coder-not-the-code-notes-on-agentic-development-and-security/](https://tessl.io/blog/securing-the-coder-not-the-code-notes-on-agentic-development-and-security/)  
38. Spec-Driven Development: Structure Beats Vibes \- DEV Community, [https://dev.to/remybuilds/spec-driven-development-structure-beats-vibes-4oma](https://dev.to/remybuilds/spec-driven-development-structure-beats-vibes-4oma)  
39. Spec-Driven Development: Just Waterfall 2.0? \- byteiota, [https://byteiota.com/spec-driven-development-just-waterfall-2-0/](https://byteiota.com/spec-driven-development-just-waterfall-2-0/)  
40. The Waterfall Strikes Back \- Test Pappy \- WordPress.com, [https://testpappy.wordpress.com/2026/04/16/the-waterfall-strikes-back/](https://testpappy.wordpress.com/2026/04/16/the-waterfall-strikes-back/)  
41. The agents are here and we're all doing waterfall again \- MakerX, [https://blog.makerx.com.au/the-agents-are-here-and-were-all-doing-waterfall-again/](https://blog.makerx.com.au/the-agents-are-here-and-were-all-doing-waterfall-again/)  
42. Spec-Driven Development: The Waterfall Strikes Back \- Marmelab, [https://marmelab.com/blog/2025/11/12/spec-driven-development-waterfall-strikes-back.html](https://marmelab.com/blog/2025/11/12/spec-driven-development-waterfall-strikes-back.html)  
43. Spec-Driven Development: The Waterfall Strikes Back | Hacker News, [https://news.ycombinator.com/item?id=45935763](https://news.ycombinator.com/item?id=45935763)  
44. Spec-Driven Development is Domain-Driven Design's Impatient Cousin \- INNOQ, [https://www.innoq.com/en/blog/2026/03/sdd-ddd-why-bmad-wont-save-you/](https://www.innoq.com/en/blog/2026/03/sdd-ddd-why-bmad-wont-save-you/)  
45. Spec-Driven Development vs Waterfall: Key Differences | Augment Code, [https://www.augmentcode.com/guides/spec-driven-development-vs-waterfall](https://www.augmentcode.com/guides/spec-driven-development-vs-waterfall)  
46. Spec-Driven Development Isn't Waterfall: Why the AI Coding Bottleneck Changed Everything \- Allstacks, [https://www.allstacks.com/blog/spec-driven-development-isnt-waterfall-why-the-ai-coding-bottleneck-changed-everything](https://www.allstacks.com/blog/spec-driven-development-isnt-waterfall-why-the-ai-coding-bottleneck-changed-everything)  
47. Spec-Driven Development and the Ralph Loop: The Good, the Bad, and the Ugly, [https://www.abrahamberg.com/blog/spec-driven-development-and-the-ralph-loop-the-good-the-bad-and-the-ugly/](https://www.abrahamberg.com/blog/spec-driven-development-and-the-ralph-loop-the-good-the-bad-and-the-ugly/)  
48. From spec to production: a three-week drug discovery agent using Kiro | AWS for Industries, [https://aws.amazon.com/blogs/industries/from-spec-to-production-a-three-week-drug-discovery-agent-using-kiro/](https://aws.amazon.com/blogs/industries/from-spec-to-production-a-three-week-drug-discovery-agent-using-kiro/)  
49. AI Native DevCon'26: The London conference for developers building with AI \- Tessl, [https://tessl.io/blog/ai-native-devcon26-the-london-conference-for-developers-building-with-ai/](https://tessl.io/blog/ai-native-devcon26-the-london-conference-for-developers-building-with-ai/)  
50. AI Native DevCon Day 1: Making AI Agents Ready for Enterprise \- Tessl, [https://tessl.io/blog/ai-native-devcon-day-1-making-ai-agents-ready-for-enterprise/](https://tessl.io/blog/ai-native-devcon-day-1-making-ai-agents-ready-for-enterprise/)  
51. AWS Security Agent adds threat modeling, Kiro power and Claude Code plugin, and more, [https://aws.amazon.com/blogs/aws/aws-security-agent-adds-threat-modeling-kiro-power-and-claude-code-plugin-and-more/](https://aws.amazon.com/blogs/aws/aws-security-agent-adds-threat-modeling-kiro-power-and-claude-code-plugin-and-more/)  
52. AWS Summit New York 2026: New ways to make AI agents more effective at work, [https://www.aboutamazon.com/news/aws/aws-summit-nyc-2026-ai-agents](https://www.aboutamazon.com/news/aws/aws-summit-nyc-2026-ai-agents)

[image1]: <data:image/png;base64,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>

[image2]: <data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABUAAAAYCAYAAAAVibZIAAAAYUlEQVR4XmNgGAWjYFCAQnQBaoCFQKyKLkgpsAbibeiC1ADZQJyGLogMhIBYigy8FIjXQtlUASpAvJcBEhRUARxAfAWIZdAlKAEpQFyMLkgp2A/ELOiClAJJdIFRMApoCABrsQmRQJAtvwAAAABJRU5ErkJggg==>