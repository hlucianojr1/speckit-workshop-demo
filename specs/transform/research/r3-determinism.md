# R3 — Determinism in Rust Game Simulations

> Research note for the C++ → Rust transformation (Appendix D.1, prompt R3).
> Scope: seeded RNG, accumulator precision, iteration-order hazards, cross-platform floating point.
> Sources: live web (rand docs, Rust Rand Book, Bevy docs, Bevy Cheat Book) unless marked [from training knowledge].

## 1. Seeded RNG: `StdRng` vs explicit algorithm crates

- `rand::rngs::StdRng` is explicitly documented as **non-portable**: "any future library version may replace
  the algorithm and results may be platform-dependent." It is currently ChaCha12, but that is an
  implementation detail, not a contract. (Source: [docs.rs/rand StdRng](https://docs.rs/rand/latest/rand/rngs/struct.StdRng.html))
- The Rust Rand Book's reproducibility policy distinguishes **API-breaking** vs **value-breaking** changes:
  value-breaking changes (same API, different output) are permitted in *minor* versions even for portable
  items, and `StdRng`/`SmallRng` opt out of reproducibility guarantees entirely (may change in any release).
  (Source: [Rust Rand Book — Reproducibility](https://rust-random.github.io/book/crate-reprod.html))
- **Recommendation:** for replay files, lockstep networking, or golden-trace tests, depend on a named
  algorithm crate directly — e.g. `chacha20`/`rand_chacha` (`ChaCha8Rng`/`ChaCha12Rng`) or
  `rand_xoshiro` (`Xoshiro256PlusPlus`) — and pin the crate version. Portable items test value-stability
  against reference vectors. (Source: Rand Book, same chapter)
- Seeding: `SeedableRng::from_seed([u8; 32])` is the canonical portable path; `seed_from_u64` is fine for
  games (not security). Never `std::random_device`-style OS entropy in sim paths — mirrors the C++
  constitution's Article 5. Avoid sampling `usize`/`isize`: its width differs across targets, so it is
  a documented portability hole. (Source: Rand Book — "Portability of usize")
- One RNG stream per subsystem (physics, VFX, AI) so systems can't perturb each other's sequences
  [from training knowledge — standard practice, matches this repo's `engine_demo::sim::rng` design].

## 2. `f32` vs `f64` accumulators

- Accumulate time, energy, and scores in `f64` even when positions/velocities stay `f32` (Bevy's `Vec2`/
  `Vec3` are `f32`). `f32` has a 24-bit mantissa: adding 1/60 s repeatedly loses precision within minutes,
  and error growth depends on the running magnitude — a classic drift source [from training knowledge].
- Bevy exposes `Time::delta_secs_f64()` and `Time<Fixed>` for this reason; keep the fixed-timestep
  accumulator in `f64` and convert to `f32` only at the point of use [from training knowledge — verified
  against Bevy 0.16 API during the 2026-07-07 port validation].
- Note: `f32` vs `f64` is about *drift/accuracy*, not determinism per se — both are deterministic if the
  same operations run in the same order. The determinism risk is *reordering* (see §3) and platform
  differences (§4).

## 3. Nondeterministic iteration order and mitigations

| Source | Why it's nondeterministic | Mitigation |
| --- | --- | --- |
| `std::collections::HashMap`/`HashSet` | `RandomState` seeds SipHash per-process; iteration order differs every run [from training knowledge] | `BTreeMap` (ordered), `Vec` + sort, or a fixed-seed hasher (`rustc_hash::FxHashMap`, `ahash` with fixed keys) — note fixed-seed hashers trade away HashDoS resistance |
| Bevy ECS query iteration | Order follows archetype/table storage, which depends on spawn/despawn history; stable-ish within a run but not a contract [from training knowledge] | Collect query results and sort by a stable key (e.g. `Entity` index or a game-assigned ID) before order-sensitive math (constraint solvers, accumulation) |
| Bevy parallel system execution | Multi-threaded executor runs non-conflicting systems in nondeterministic order; "the order could even change every frame" (Source: [Bevy Cheat Book — System Order](https://bevy-cheatbook.github.io/programming/system-order.html)) | Explicit `.before()`/`.after()`/`.chain()` ordering on every pair of systems that touch sim state; system sets for grouping |
| Ambiguities going unnoticed | Conflicting-access system pairs with no ordering are silently ambiguous | Set `ScheduleBuildSettings::ambiguity_detection` to `LogLevel::Warn`/`Error` on sim schedules (default is `Ignore`) (Source: [docs.rs ScheduleBuildSettings](https://docs.rs/bevy/latest/bevy/ecs/schedule/struct.ScheduleBuildSettings.html)) |
| `Query::par_iter` / parallel fold | Chunked parallel reduction sums in nondeterministic order → FP results differ run-to-run [from training knowledge] | Serial iteration on determinism-critical paths, or parallel compute + serial ordered reduction |
| Command application order | `Commands` are applied at sync points; interleaving across ambiguous systems varies [from training knowledge] | Same fix: explicit ordering; consider single-threaded executor for the fixed sim schedule |

- Practical pattern: run gameplay in `FixedUpdate` with fully chained sim systems; leave rendering/VFX
  in `Update` where ambiguity is acceptable [from training knowledge].

## 4. Floating-point reproducibility across platforms

- Same binary, same machine: IEEE 754 basic ops (`+ - * /`, `sqrt`) are correctly rounded and reproducible.
  Cross-platform hazards [from training knowledge]:
  - **Transcendentals** (`sin`, `cos`, `exp`, `pow`) are implemented by the platform libm with no exactness
    guarantee — results differ between OSes/architectures. `rand_distr` prefers `libm` over `std` for this
    reason. (Source: Rand Book — "Portability of floats")
  - **FMA contraction**: hardware fused multiply-add changes rounding vs separate mul+add; Rust does not
    auto-contract by default but explicit `mul_add` differs from `a * b + c` [from training knowledge].
  - **x87 80-bit** excess precision on 32-bit x86 targets without SSE2; effectively irrelevant on modern
    x86_64/aarch64 [from training knowledge].
  - **SIMD/optimization-level reassociation** only with explicit fast-math-style intrinsics; safe Rust +
    default codegen preserves evaluation order [from training knowledge].
- Mitigations, in increasing strictness [from training knowledge]:
  1. Same-platform determinism (this project's bar): fixed seed + ordered iteration + serial sim is enough.
  2. Cross-platform bitwise: avoid std transcendentals in sim (use `libm` crate or polynomial approximations),
     avoid `mul_add`, pin toolchain, and test golden hashes on every target in CI.
  3. Fully portable lockstep: fixed-point arithmetic or software floats — usually overkill.
- Verification pattern (mirrors the C++ headless trace): run N frames twice at the same seed and compare
  a state hash; do this in CI as a regression test [from training knowledge — validated in the 07-07 port,
  identical hashes across binary runs].

## Key takeaways for the transformation

1. Use `rand_chacha`/`chacha20` (pinned) rather than `StdRng` for anything persisted or compared across runs.
2. Keep `f64` accumulators (constitution Article 5 carries over); `f32` component math is fine.
3. Determinism in Bevy is opt-in: chain sim systems, enable ambiguity detection, sort query results, ban
   `HashMap` iteration in sim paths.
4. Same-platform reproducibility is cheap; cross-platform bitwise reproducibility requires banning libm
   transcendentals — state which bar the spec targets.
