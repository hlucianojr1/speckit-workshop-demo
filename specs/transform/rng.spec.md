# Deterministic RNG — Language-Agnostic Specification

## 1. Purpose

Provide an explicitly seeded, deterministic random stream for simulation paths.
Non-deterministic entropy sources (OS randomness, time-based seeds) are forbidden.

## 2. Seeding Model

- The public seed is a **64-bit unsigned integer**, carried at full width.
- The full 64 bits MUST influence the stream: two seeds differing only in their high
  32 bits MUST produce different streams. (Reference implementation XOR-folds
  `high32 ^ low32` when the underlying engine takes a narrower seed. A target whose
  engine accepts 64-bit seeds natively satisfies this trivially.)

## 3. Operations

| Operation        | Signature (abstract) | Behavior                                        |
| ---------------- | -------------------- | ----------------------------------------------- |
| construct        | `(seed_u64) → rng`   | Deterministic; no entropy sources               |
| next_u32         | `() → uint32`        | Next engine output                              |
| next_double_unit | `() → float64`       | `next_u32() / 2^32` — uniform in [0, 1)         |

The generator is a value type: copyable, and a copy continues the stream identically
from the point of copy.

## 4. Guarantees

| Guarantee           | Description                                                     |
| ------------------- | ---------------------------------------------------------------- |
| Reproducibility     | Same seed ⇒ identical stream, every run, every platform          |
| Full-width seeding  | All 64 seed bits contribute to the stream                        |
| Unit-interval bound | next_double_unit ∈ [0, 1), 64-bit float precision                |
| No hidden entropy   | Never reads OS randomness, clocks, or global state               |

## 5. Constraints (constitutional)

- Determinism (Article 5): explicit seeding only.
- The choice of underlying engine is an implementation detail per language; the
  BEHAVIORAL contract is reproducibility + full-width seeding, not a specific
  bit stream. Cross-language implementations are NOT required to produce identical
  numeric sequences.
- Floating-point outputs are 64-bit.
