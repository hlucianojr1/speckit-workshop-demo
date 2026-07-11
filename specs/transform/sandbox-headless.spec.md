# Sandbox Headless Mode & CLI — Language-Agnostic Specification

> **Scope note:** This spec covers the `ea-sandbox` command-line contract and the
> headless trace writer (`apps/sandbox/main.cpp`, `apps/sandbox/headless.cpp`) — the
> machinery behind every golden-digest acceptance check in this spec set. Digest
> *algorithm* and golden *values* live in
> [sandbox-scenes.spec.md](sandbox-scenes.spec.md) §8; this spec owns the invocation,
> CSV format, and exit-code contract (Phase A5 — full-fidelity closure).

## 1. Purpose

Provide a windowless, GL-free run mode that simulates N fixed steps and emits a
per-frame CSV trace plus a final digest — the falsifiable determinism proof used by
CI, the workshop's A/B checks, and cross-implementation parity validation.

## 2. Command-Line Contract

```text
ea-sandbox [--headless --seed N --frames N --out PATH]
           [--scene rope|pendulum|cloth|storm|orbital]
           [--screenshot PATH [--warmup N]]
```

| Flag           | Default     | Meaning                                                    |
| -------------- | ----------- | ----------------------------------------------------------- |
| `--headless`   | off         | Headless mode (default is interactive)                      |
| `--seed N`     | 42          | u64 scene seed (digits only)                                |
| `--frames N`   | 600         | u32 fixed steps to simulate                                 |
| `--out PATH`   | `trace.csv` | CSV output path                                             |
| `--scene NAME` | `rope`      | Accepted names: `rope`; `pendulum` / `pendulum_tower`; `cloth`; `storm` / `particle_storm`; `orbital` / `orbital_arena` |
| `--screenshot PATH` | —      | Interactive capture mode ([controls spec](sandbox-controls.spec.md) §5) |
| `--warmup N`   | 120         | Screenshot warmup frames; valid 1 – 100000                  |
| `--help` / `-h`| —           | Print usage (including the full control map) and exit 0     |

Telemetry flags (`--debug`, `--telemetry-level`, `--telemetry-out`, `--watchdog`,
`--no-crash-handlers`) exist in the reference binary but are operational tooling —
out of scope for a port.

**Exit codes:** 0 success (incl. `--help`); 2 argument error (unknown flag, malformed
value — message + usage to stderr); 3 headless run failure.

## 3. Headless Run Semantics

1. Validate: a null output path or zero frame count is an argument error; an
   unopenable output file or a failed write is a run failure (status enums, never
   exceptions).
2. Construct the scene with the seed; switch to the requested kind if not rope.
3. For each frame i ∈ [0, N): advance exactly **one fixed step** (1/60 s — wall clock
   is never consulted), then write one CSV row.
4. Print the final digest line to stdout and exit 0.

## 4. Output Format (byte-normative)

**CSV** — header then one row per frame:

```csv
frame,sim_time,digest,rope0_x,rope0_y,ropeN_x,ropeN_y
```

| Column            | Format   | Content                                                 |
| ----------------- | -------- | -------------------------------------------------------- |
| frame             | `%u`     | 0-based frame index                                      |
| sim_time          | `%.9f`   | Accumulated sim seconds after the step                   |
| digest            | `%016llx`| State digest after the step ([scenes](sandbox-scenes.spec.md) §8) |
| rope0_x, rope0_y  | `%.9f`   | Body 0 position (0.0 if the scene has no bodies)         |
| ropeN_x, ropeN_y  | `%.9f`   | Last body's position (0.0 if none)                       |

**Stdout** on success:

```text
trace_digest=<%016llx> frames=<N> scene=<display name>
```

Display names: `rope`, `pendulum tower`, `cloth`, `particle storm`, `orbital arena`.

## 5. Acceptance Uses

- **Golden A/B:** identical (seed, frames, scene) invocations must produce
  byte-identical CSV files and digests — before/after any render-only feature, and
  across two runs of the same build. Golden values: [scenes spec](sandbox-scenes.spec.md) §8.1.
- **Cross-implementation parity:** a port matching all five golden digests is
  behaviorally identical; where bit-exact float parity is unattainable, compare the
  position columns within tolerance frame-by-frame instead.

## 6. Guarantees

| Guarantee           | Description                                              |
| -------------------- | ---------------------------------------------------------- |
| No GL / no window    | Headless never initializes graphics — CI/VM safe           |
| Wall-clock free      | Fixed-step only; run duration cannot affect results        |
| Deterministic output | (seed, frames, scene) → byte-identical CSV + digest        |
| Status-based errors  | Every failure path is an enum + exit code, never a throw   |
