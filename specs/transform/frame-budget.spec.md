# Frame Budget — Language-Agnostic Specification

## 1. Purpose

Track recent frame timings in a fixed-size rolling window and report a rolling average,
for real-time budget monitoring (e.g. ≤ 16.67 ms at 60 FPS).

## 2. Model

- Window size is fixed at construction (default 64; hard cap 256 in the reference).
- Storage for the whole window is reserved up front; recording a sample never allocates.
- Samples are frame durations in milliseconds, 64-bit float.

## 3. Operations

| Operation       | Signature (abstract)     | Behavior                                     |
| --------------- | ------------------------ | -------------------------------------------- |
| record_sample   | `(milliseconds) → void`  | Write at cursor, advance cursor mod window   |
| rolling_average | `() → milliseconds`      | Mean of the samples recorded so far (§4)     |
| sample_count    | `() → uint`              | min(total recorded, window size)             |

## 4. Averaging Semantics

- Before the window is full: average over exactly the `count` samples recorded so far.
  Warm-up frames are counted **exactly once** (historical bug: double-counting during
  warm-up).
- Once full: average over the last `window_size` samples (oldest overwritten first).
- Zero samples ⇒ average is 0.0 (not an error, not NaN).

## 5. Guarantees

| Guarantee            | Description                                          |
| -------------------- | ---------------------------------------------------- |
| No record allocation | record_sample is allocation-free and O(1)            |
| Warm-up correctness  | Each sample contributes exactly once to the average  |
| 64-bit accumulation  | Sum/average computed in 64-bit float                 |

## 6. Constraints (constitutional)

- Real-time (Article 6): O(1) record path, zero allocation after construction.
- Determinism (Article 5): 64-bit floats only.
- No exceptions/panics.
