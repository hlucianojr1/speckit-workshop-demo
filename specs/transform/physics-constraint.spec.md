# Physics Constraint Solver — Language-Agnostic Specification

## 1. Purpose

Resolve distance constraints between point bodies by iterative position projection
(Gauss–Seidel style), with fully deterministic iteration order.

## 2. Data Model

### 2.1 Body

| Field        | Type              | Notes                                    |
| ------------ | ----------------- | ---------------------------------------- |
| id           | 64-bit unsigned   | Unique key; caller-assigned              |
| position     | 3 × 64-bit float  | x, y, z                                  |
| inverse_mass | 64-bit float      | 0 ⇒ infinite mass (immovable)            |

### 2.2 Distance Constraint

| Field       | Type            | Notes                                  |
| ----------- | --------------- | -------------------------------------- |
| a, b        | 64-bit unsigned | Body ids; order does not matter        |
| rest_length | 64-bit float    | Target separation                      |

All arithmetic is 64-bit floating point (constitutional determinism rule).

## 3. Operations

| Operation      | Signature (abstract)                  | Behavior                                        |
| -------------- | ------------------------------------- | ----------------------------------------------- |
| add_body       | `(body) → void`                       | Insert or overwrite by id                       |
| add_constraint | `(constraint) → void`                 | Insert, kept sorted (see §4.1)                  |
| solve          | `(max_iterations) → iterations_spent` | Run projection passes; returns max_iterations   |
| try_get_body   | `(id) → body \| none`                 | Read-only lookup; "none" if id unknown          |

## 4. Solve Semantics

For each iteration (exactly `max_iterations` of them), visit every constraint in
canonical order and project the two bodies:

1. `d = pos(b) − pos(a)`; `len = |d|`. Skip if `len < 1e-12`.
2. `diff = (len − rest_length) / len`.
3. `w = inv_mass(a) + inv_mass(b)`. Skip if `w < 1e-12`.
4. Move `a` by `+(inv_mass(a)/w) · d · diff`; move `b` by `−(inv_mass(b)/w) · d · diff`.
5. Constraints referencing unknown body ids are skipped silently.

### 4.1 Deterministic Ordering (critical)

- Bodies are stored in a key-sorted structure (sorted by id).
- Constraints are kept sorted by their **canonical key** `(min(a,b), max(a,b))`,
  established at insertion time.
- Therefore projection order — and the converged positions — are identical across runs
  AND across different construction orders of the same body/constraint set.

## 5. Guarantees

| Guarantee            | Description                                                          |
| -------------------- | -------------------------------------------------------------------- |
| Determinism          | Same body/constraint set (any insertion order) ⇒ bitwise-same result |
| Convergence          | Two-body constraint approaches rest_length as iterations increase    |
| Infinite-mass respect| Bodies with inverse_mass 0 never move                                 |
| No solve allocation  | `solve` performs zero heap operations                                 |

## 6. Constraints (constitutional)

- No exceptions/panics; missing bodies and degenerate geometry are skipped, not faulted.
- 64-bit float arithmetic only in the solver path.
- Storage may allocate on add_body/add_constraint (setup phase) but never during solve.
