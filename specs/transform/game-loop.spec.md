# Fixed-Step Game Loop — Language-Agnostic Specification

## 1. Purpose

Convert variable real-time deltas into a deterministic sequence of fixed-duration
simulation substeps using the classic accumulator pattern.

## 2. Configuration

| Field              | Type          | Default  | Notes                                |
| ------------------ | ------------- | -------- | ------------------------------------ |
| fixed_step_seconds | 64-bit float  | 1/60     | Duration of one simulation substep   |
| max_substeps       | number        | 8        | Upper bound of substeps per advance  |

## 3. Operations

| Operation           | Signature (abstract)        | Behavior                                   |
| ------------------- | --------------------------- | ------------------------------------------ |
| advance             | `(delta_seconds) → substeps`| See §4                                     |
| accumulator_seconds | `() → seconds`              | Current unconsumed time (read-only)        |

## 4. Advance Semantics

1. Add `delta_seconds` to the internal accumulator.
2. While `accumulator ≥ fixed_step_seconds` AND `substeps < max_substeps`:
   subtract `fixed_step_seconds` from the accumulator; increment `substeps`.
3. Return `substeps`.

Consequences the spec REQUIRES:

- `advance(d)` with `d < fixed_step_seconds` (from empty) returns 0; time is banked.
- `advance(fixed_step_seconds)` returns exactly 1.
- Spiral-of-death protection: no more than `max_substeps` substeps per call; excess
  time REMAINS in the accumulator (it is not discarded).

## 5. Guarantees

| Guarantee        | Description                                                                  |
| ---------------- | ---------------------------------------------------------------------------- |
| Determinism      | The substep sequence is a pure function of (config, sequence of deltas)      |
| No drift         | Accumulator is 64-bit float; long runs (hours of sim time) do not drift      |
| Statelessness    | No global/ambient state; the loop is a value type over (config, accumulator) |
| No allocation    | advance performs zero heap operations                                        |

## 6. Constraints (constitutional)

- Accumulator MUST be 64-bit float. (Historical bug: a 32-bit accumulator drifted after
  ~30 s of simulated time, causing replay divergence.)
- No exceptions/panics.
