# R4 — Error Handling & Public-API Best Practices for Rust Game Code

> Research note for the C++ → Rust transformation (Appendix D.1, prompt R4).
> Scope: panic policy, Result/Option vs status enums, `#[must_use]`, unsafe/`// SAFETY:` policy, clippy CI tiers.
> No code changes; findings feed `rust-guidelines.md` (C2) and the Rust constitution (D.3).

## 1. Panic policy for real-time loops

- In Rust, `panic!` (and `unwrap`/`expect`/`[]` out-of-bounds/integer overflow in debug) either **unwinds** or **aborts** (`panic = "abort"` in Cargo profile). In a game loop, unwinding across a frame at best drops the frame in a corrupted state; with abort, the process dies. Neither is acceptable mid-match, so treat panics like the C++ constitution treats exceptions: **forbidden in sim/game-logic paths**.
- Policy tiers that work in practice:
  - **Startup/asset-load:** panics acceptable (fail fast before the loop starts); `expect("context")` with a message is fine here.
  - **Per-frame sim/render systems:** no `unwrap`/`expect`/`panic!`/`todo!`; recover via `Result`, `Option` defaults, clamping, or entity skip + logged diagnostic.
  - **Tests:** panics are the assertion mechanism — unrestricted.
- Rust API Guidelines [C-VALIDATE](https://rust-lang.github.io/api-guidelines/dependability.html#c-validate) and [C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html#c-failure): validate arguments, and document every panic condition in a `# Panics` doc section — an API without a documented panic must not panic.
- `panic = "abort"` in release is a legitimate shipping choice (smaller binaries, no unwind tables) but makes the "no panics in the loop" rule *harder*, not softer — it converts every stray `unwrap` into a crash-to-desktop. Fixed with lints, not with unwinding.
- Bevy note: a panicking system takes down the whole `App` by default; treat system code as loop code.

## 2. `Result`/`Option` vs C++-style status enums

- The C++ codebase returns status enums (e.g. `expected<T, status>`-style) because exceptions are off. Rust's native equivalent is `Result<T, E>` — same "error is a value" model, but with compiler-enforced consumption (`unused_must_use` fires on ignored `Result`) and `?` propagation.
- **Do not port status enums as bare return codes.** Map: C++ `status` enum → Rust error enum used as the `E` in `Result<T, E>`; C++ "null/sentinel" returns → `Option<T>`. `Option` = absence is normal; `Result` = absence is a failure the caller must handle.
- Error type guidance (API Guidelines [C-GOOD-ERR](https://rust-lang.github.io/api-guidelines/interoperability.html#c-good-err)): error types should implement `Debug + Display` (and `std::error::Error` at library boundaries). For engine-internal hot paths, a small fieldless `#[derive(Debug, Clone, Copy, PartialEq)] enum` is zero-cost — no `Box<dyn Error>`, no `anyhow` in sim code (allocation + type erasure). `thiserror` is fine for derive convenience; `anyhow` belongs in tooling/bin targets only.
- Doc examples should propagate with `?`, never `unwrap` ([C-QUESTION-MARK](https://rust-lang.github.io/api-guidelines/documentation.html#c-question-mark)).
- Fallible-but-ignorable per-frame operations (e.g. "spawn VFX, pool exhausted") can return a `#[must_use]` status enum where forcing `?` everywhere is noise — the closest true analogue of the C++ pattern — but prefer `Result<(), E>` unless profiling or ergonomics argue otherwise.

## 3. `#[must_use]` — the `[[nodiscard]]` analogue

- `#[must_use]` on a function or type triggers the `unused_must_use` lint when the value is discarded — the direct analogue of C++ `[[nodiscard]]` ([Rust Reference: diagnostic attributes](https://doc.rust-lang.org/reference/attributes/diagnostics.html#the-must_use-attribute)). Supports a message: `#[must_use = "handle leaks pool slot if dropped"]`.
- `Result` and `Option` are already `#[must_use]` in std — the constitution's "[[nodiscard]] on factories and status returns" rule is *automatic* for `Result`-returning APIs.
- Still annotate explicitly: factories returning handles/guards, builders, pure computations whose only effect is the return value, and status enums returned bare (§2). Apply to the *type* for guard/handle types (fires wherever discarded), to the *function* for computations.
- Intentional discard is spelled `let _ = f();` — greppable and reviewable, unlike C++ `(void)f()`.
- Warn-by-default `unused_must_use` should be promoted to deny in CI (via `-D warnings` or `#![deny(unused_must_use)]`).
- Clippy adds `clippy::must_use_candidate` (pedantic) to suggest missing annotations on public APIs.

## 4. `unsafe` policy and `// SAFETY:` conventions

- Baseline for a game port with no FFI: `#![forbid(unsafe_code)]` at crate root. Idiomatic ECS/physics/sim Rust needs no `unsafe`; ownership + slices replace the pointer arithmetic that motivated it in C++.
- Where `unsafe` is genuinely needed (SIMD intrinsics, GPU buffer casts, custom allocators), adopt the std convention ([std-dev-guide: safety comments](https://std-dev-guide.rust-lang.org/policy/safety-comments.html)):
  - Every `unsafe` block gets a `// SAFETY:` comment stating which invariants make it sound and where they're established.
  - Every `unsafe fn` gets a `# Safety` rustdoc section stating caller obligations ([C-FAILURE](https://rust-lang.github.io/api-guidelines/documentation.html#c-failure)).
  - Enable `unsafe_op_in_unsafe_fn` so each operation inside an `unsafe fn` needs its own block + comment.
- Enforcement: `clippy::undocumented_unsafe_blocks` (restriction tier) denies `unsafe` blocks lacking `// SAFETY:`; `clippy::missing_safety_doc` covers the doc section. `cargo-geiger` can report unsafe counts in dependencies if supply-chain auditing is wanted.
- Isolate any permitted `unsafe` into a small, heavily tested module — never inline in gameplay systems.

## 5. Clippy lint tiers worth enforcing in CI

Groups per the [Clippy book](https://doc.rust-lang.org/clippy/lints.html) (defined in the Clippy 1.0 RFC):

| Group | Default | CI recommendation |
|---|---|---|
| `correctness` | deny | Keep deny — code is "outright wrong"; do not `allow` |
| `suspicious` | warn | Deny in CI; local `#[allow]` + justification comment when intentional |
| `complexity`, `perf`, `style` | warn | Deny in CI via `cargo clippy -- -D warnings`; cheap to keep clean from day one |
| `pedantic` | allow | Do **not** enable wholesale (intentional false positives); cherry-pick e.g. `must_use_candidate`, `float_cmp` (determinism), `cast_precision_loss`, `missing_panics_doc` |
| `restriction` | allow | Never enable the group; cherry-pick the constitution enforcers (below) |
| `nursery`, `cargo` | allow | Skip `nursery` (buggy by admission); `cargo` group optional |

Constitution-enforcing restriction/pedantic picks for game code:

- `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::todo`, `clippy::unimplemented`, `clippy::indexing_slicing` — the "no panics in the loop" rule (scope to sim crates; allow in `#[cfg(test)]`).
- `clippy::undocumented_unsafe_blocks`, `clippy::missing_safety_doc` — §4.
- `clippy::arithmetic_side_effects` (optional, noisy) — overflow discipline.
- Prefer `[lints.rust]`/`[lints.clippy]` tables in `Cargo.toml` (or `#![deny(...)]` in `lib.rs`) over CI flags alone, so local builds match CI. Use `#[expect(lint, reason = "...")]` for justified exceptions — it warns when the exception becomes stale.

CI gate: `cargo clippy --all-targets --all-features -- -D warnings` plus `cargo fmt --check`.

## Sources

- Rust API Guidelines (checklist: C-VALIDATE, C-FAILURE, C-GOOD-ERR, C-QUESTION-MARK) — <https://rust-lang.github.io/api-guidelines/checklist.html> [live]
- Clippy book, "Clippy's Lints" (group semantics & recommendations) — <https://doc.rust-lang.org/clippy/lints.html> [live]
- Rust Reference, diagnostic attributes (`must_use`, lint levels, `#[expect]`, `reason =`) — <https://doc.rust-lang.org/reference/attributes/diagnostics.html> [live]
- Rust std-dev-guide, safety comments policy (`// SAFETY:`, `# Safety`, `unsafe_op_in_unsafe_fn`) — <https://std-dev-guide.rust-lang.org/policy/safety-comments.html> [live]
- Panic-abort trade-offs, Bevy system-panic behavior, `thiserror`/`anyhow` split, `cargo-geiger` — [from training knowledge]
