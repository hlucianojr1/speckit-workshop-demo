# CLI Contract: `orbital-arena-rs`

This is the external interface the port exposes (FR-009, FR-010, FR-011). Parsed with `clap`
derive (research.md R3).

## Modes

### Windowed (default)

```text
orbital-arena-rs [--seed <u64>]
```

- Opens a window; simulation and rendering run indefinitely (FR-009).
- No exit condition tied to frame count; closing the window exits normally.
- `--seed` defaults to `42` (FR-007).

### Screenshot-capture

```text
orbital-arena-rs screenshot --warmup-frames <u32> --out <path> [--seed <u64>]
```

- Runs the fixed-timestep simulation; after exactly `warmup_frames` `FixedUpdate` ticks have
  elapsed, writes a single image file to `<path>` depicting the current scene (FR-010).
- **Precondition enforced**: no file is written before `warmup_frames` ticks have elapsed
  (Acceptance Scenario US2#2). `warmup_frames = 0` is valid — capture happens at the first
  post-setup render (Edge Cases).
- **On success**: process exits with status `0` after the file is confirmed written
  (Acceptance Scenario US2#4, SC-002).
- **On failure to write** (e.g. unwritable directory): process prints a clear error to stderr
  and exits with a non-zero status. No panic (Edge Cases, Article I/II).

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Windowed mode closed normally, or screenshot mode wrote its file successfully |
| non-zero | CLI argument parse error (from `clap`), or screenshot write failure |

## Determinism contract (not a CLI flag — test-only entry point)

`tests/determinism.rs` calls into `src/lib.rs` helpers directly (headless `App` construction),
not through the CLI binary, since FR-012's "run twice, compare" check is exercised via
`cargo test`, not by shelling out to the built binary.
