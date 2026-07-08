# Quickstart: Orbital Arena

**Feature**: 006-orbital-arena | spec dir `specs/003-orbital-arena/`

## Prerequisites

- The `engine_demo::vfx` subsystem (feature 005) must be present on the working branch —
  Orbital Arena reuses `particle_pool`/`emitter` directly.
- Windows: run inside a VS dev shell with `VCPKG_ROOT` set (see
  [docs/local-build-guide.md](../../docs/local-build-guide.md)).

## Build & test

```bash
cmake --preset default-debug
cmake --build --preset default-debug
ctest --preset default-debug --output-on-failure
```

New CTest targets (after implementation): `test_gravity_well`, `test_scoring`,
`test_powerup`, `test_match`, `test_input`, `test_arena_integration`,
`test_replay_determinism`.

Verify the tests actually registered (repo gotcha — a gmock-polluted link produces
false-positive "100% passed" with zero tests):

```bash
ctest --preset default-debug -R test_replay_determinism -V | grep "\[ RUN \]"
```

## Run only the Orbital Arena tests

```bash
ctest --preset default-debug -R "test_(gravity_well|scoring|powerup|match|input|arena_integration|replay_determinism)" --output-on-failure
```

## Minimal library usage

```cpp
#include <orbital_arena/arena.h>

alignas(16) static unsigned char buffer[1u << 20];
engine_demo::allocator alloc(buffer, sizeof(buffer));

auto a = orbital_arena::arena::create(alloc, {.seed = 42, .player_count = 2});
(void)a->join(0);  (void)a->join(1);
(void)a->set_ready(0, true);  (void)a->set_ready(1, true);

orbital_arena::tick_inputs in{};             // zero input
for (int i = 0; i < 180 + 600; ++i) {        // countdown + 10 s of play
    in.players[0] = {.steer = {1.0f, 0.0f}, .strength = 1.0f};
    (void)a->tick(in);
}
// a->score(0), a->state(), a->hash_history() ...
```

## Replay verification (Article 10)

```cpp
// Run the same seed + input sequence twice; hash histories must be identical.
ASSERT_TRUE(eastl::equal(run1.hash_history().begin(), run1.hash_history().end(),
                         run2.hash_history().begin()));
```

## Sandbox visualization (screenshots)

```bash
# after the visualization task lands:
./build/apps/sandbox/ea-sandbox.exe                       # press key 5 / cycle to orbital_arena scene
./build/apps/sandbox/ea-sandbox.exe --screenshot shot.png --warmup 600   # RELATIVE path!
```

HUD shows per-player scores, active effects, and the match state banner. The scene is
render-only: headless `--headless` digests are unaffected by HUD/trails. On this VM,
interactive/screenshot runs need the Mesa llvmpipe DLLs in `build/apps/sandbox/`
(recipe in [docs/local-build-guide.md](../../docs/local-build-guide.md)).
