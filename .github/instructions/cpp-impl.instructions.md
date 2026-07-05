---
applyTo: "src/**/*.cpp,include/**/*.h"
---

## C++ Implementation Rules

1. Include the corresponding header first: `#include "engine_demo/subsystem/file.h"`
2. Use `[[nodiscard]]` on all factory functions and status-returning functions
3. Use `noexcept` on move constructors, move assignment, and swap
4. Never use `auto` for return types — be explicit
5. Prefer `eastl::span` over pointer+size pairs
6. EASTL containers only (`eastl::vector`, `eastl::string`, …); `std::` containers require
   an explicit `// interop boundary` comment
7. Every container takes an explicit allocator (`engine_demo::allocator` /
   `engine_demo::eastl_allocator_ref`) — default-constructed containers are forbidden
8. No exceptions (`-fno-exceptions`), no RTTI (`-fno-rtti`); fallible operations return a
   status enum
9. No allocation in inner loops — pool or arena up front (constitution Article 6)
10. Time accumulators are `double`; RNG is explicitly seeded (constitution Article 5)
