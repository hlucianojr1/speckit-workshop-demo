# ECS World — Language-Agnostic Specification

## 1. Purpose

Manage the lifecycle of game entities: creation, destruction, and liveness queries.
Entities are lightweight opaque handles; the world owns all slot state.

## 2. Entity Model

### 2.1 Entity Handle

An entity handle is an opaque identifier composed of:

- **Index** (32-bit unsigned): slot in a fixed-capacity table
- **Generation** (32-bit unsigned): monotonically increasing version counter for that slot

Two handles with the same index but different generations refer to DIFFERENT entities.
A generation value of zero is reserved and means "invalid handle" — the default-constructed
handle is never alive.

### 2.2 Capacity

Maximum entity count is fixed at construction time (reference value: 4096).
No dynamic resizing. Exceeding capacity is a normal, recoverable condition.

## 3. Operations

| Operation | Signature (abstract)     | Complexity        | Allocates? |
| --------- | ------------------------ | ----------------- | ---------- |
| create    | `() → entity_handle`     | O(1) amortized(*) | No         |
| destroy   | `(entity_handle) → void` | O(1)              | No         |
| is_alive  | `(entity_handle) → bool` | O(1)              | No         |
| count     | `() → uint`              | O(1)              | No         |

(*) Reference implementation scans from a next-free cursor; worst case O(capacity) when
nearly full. A target implementation MAY improve this (free list) as long as handle
assignment stays deterministic for a given create/destroy sequence.

### 3.1 Create

- Returns a handle to a previously-free slot; the slot's current generation is embedded
  in the handle. A slot used for the first time gets generation 1 (never 0).
- If the pool is exhausted, returns the invalid handle (generation 0) — NOT an error
  signal via exception/panic.
- The search cursor advances past the assigned slot, so consecutive creates spread
  across slots instead of always reusing the lowest index.

### 3.2 Destroy

- Ignores handles that are out of range, already dead, or generation-mismatched (idempotent, safe).
- On success: increments the slot generation (invalidating ALL outstanding handles to
  this entity), marks the slot free, and decrements the live count.

### 3.3 Is Alive

Returns true IFF: index is in range AND slot is occupied AND handle.generation equals the
slot's current generation AND handle.generation ≠ 0.

## 4. Guarantees

| Guarantee              | Description                                                                  |
| ---------------------- | ---------------------------------------------------------------------------- |
| Generation safety      | A stale handle can never validate against a recycled slot (use-after-free guard) |
| No steady-state allocation | All slot storage is reserved at construction; create/destroy never allocate |
| Deterministic assignment | The same sequence of create/destroy calls always yields the same handle values |
| Graceful exhaustion    | Pool-full is reported by an invalid handle, not a fault                       |

## 5. Constraints (constitutional)

- No exceptions/panics on any operation path.
- Deterministic (Article 5): handle assignment is a pure function of the operation history.
- Real-time (Article 6): no heap operations after construction.
