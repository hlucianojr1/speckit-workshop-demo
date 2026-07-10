# Arena Allocator — Language-Agnostic Specification

## 1. Purpose

Provide bounded, up-front memory for all engine containers so that steady-state
simulation performs zero system heap operations. This is a POLICY spec as much as a
component spec: in languages with ownership-based memory management, the policy
("reserve up front, never allocate in the hot path") survives even when the arena
component itself disappears.

## 2. Model (reference implementation)

- Constructed over a caller-supplied contiguous buffer of fixed capacity.
- Bump-pointer allocation with alignment; bounds check accounts for post-alignment
  padding (historical bug: padding overran the arena end).
- `deallocate` is a no-op (arena semantics — memory is reclaimed only wholesale).
- Failure mode: allocation that does not fit returns "no memory" (null / none),
  never a fault.
- Observability: `bytes_used()` and `capacity()` are exact and queryable at any time.

## 3. Operations

| Operation  | Signature (abstract)               | Behavior                              |
| ---------- | ---------------------------------- | ------------------------------------- |
| allocate   | `(size, alignment) → ptr \| none`  | Aligned bump allocation; none if full |
| deallocate | `(ptr, size) → void`               | No-op                                 |
| bytes_used | `() → uint`                        | Current offset                        |
| capacity   | `() → uint`                        | Total buffer size                     |

## 4. Guarantees

| Guarantee          | Description                                                     |
| ------------------ | ---------------------------------------------------------------- |
| Bounded memory     | Total usage can never exceed the construction-time capacity      |
| Graceful exhaustion| Out-of-memory is a value, not a fault                            |
| Alignment safety   | Returned blocks honor the requested alignment, inside the buffer |
| Steady-state zero-alloc | Containers built on the arena never touch the system heap after setup |

## 5. Transformation Note

The BEHAVIORAL requirement to preserve in a target language is §4's "bounded memory +
steady-state zero-alloc", NOT the bump-pointer mechanism. A target with ownership-based
memory management may satisfy this via capacity-reserved collections created during
initialization, with a rule (enforced by test or lint) that per-frame paths do not grow
collections.
