// Orbital Arena — aggregate root implementation (T001 stub).
//
// The full arena integration tick lands in T017; this translation unit anchors the
// static library target so the CMake scaffolding builds from the first task.

#include "orbital_arena/arena.h"

namespace orbital_arena {

// Library link anchor: keeps this TU non-empty (avoids LNK4221) until the real
// arena implementation replaces it in T017.
[[nodiscard]] arena_status arena_lib_anchor() noexcept;
[[nodiscard]] arena_status arena_lib_anchor() noexcept {
    return arena_status::ok;
}

}  // namespace orbital_arena
