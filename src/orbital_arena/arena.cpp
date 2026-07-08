// Orbital Arena aggregate root — see include/orbital_arena/arena.h for the contract.
//
// Tick pipeline order (data-model.md §Tick Pipeline — the determinism backbone):
//   1. clamp + log inputs (every state)          5. field replenishment (salted rng)
//   2. match state machine                       6. 60-tick snapshot hash sample
//   3. playing systems (a..h, fixed order)       7. tick counter increment
//   4. particle age_and_retire (captures leave)
//
// Design note: the countdown->playing transition tick performs placement/reset side
// effects ONLY; gameplay systems run from the following tick. This keeps "scores are
// zero when play begins" (US2 scenario 2) exact even if a spawned particle happens
// to sit inside a freshly placed well's capture radius.

#include "orbital_arena/arena.h"

// Interop boundary (constitution Article 3): std::chrono::steady_clock is used for
// frame_budget self-measurement only (never for gameplay), mirroring the vfx
// subsystem's documented exception; placement new comes from <new> (non-allocating).
#include <chrono>
#include <cmath>
#include <new>

namespace orbital_arena {

namespace {

// Initial speed cap for field particles: slow drift so radial infall dominates.
constexpr float kFieldParticleSpeedMax = 0.1f;

// Writes the k-th of `n` rotationally symmetric copies of `in` to `out`.
// n == 2 and n == 4 use exact sign/index forms (no trig) so mirrored worlds evolve
// bit-identically (Article 9); n == 3 uses constants for cos/sin(2*pi/3).
void rotate_copy(std::uint8_t k, std::uint8_t n, const float in[2], float out[2]) noexcept {
    if (k == 0) {
        out[0] = in[0];
        out[1] = in[1];
        return;
    }
    if (n == 2) {  // exact point reflection
        out[0] = -in[0];
        out[1] = -in[1];
        return;
    }
    if (n == 4) {  // exact quarter turns
        switch (k) {
            case 1:
                out[0] = -in[1];
                out[1] = in[0];
                return;
            case 2:
                out[0] = -in[0];
                out[1] = -in[1];
                return;
            default:  // 3
                out[0] = in[1];
                out[1] = -in[0];
                return;
        }
    }
    // n == 3: rotation by k * 120 degrees.
    constexpr float kCos120 = -0.5f;
    constexpr float kSin120 = 0.86602540378f;
    const float c = kCos120;
    const float s = k == 1 ? kSin120 : -kSin120;
    out[0] = c * in[0] - s * in[1];
    out[1] = s * in[0] + c * in[1];
}

[[nodiscard]] engine_demo::vfx::emitter_config make_flood_emitter_config(
    const arena_config& cfg) noexcept {
    engine_demo::vfx::emitter_config ec{};
    ec.shape = engine_demo::vfx::sphere_shape{{0.0f, 0.0f, 0.0f}, cfg.half_extent};
    ec.seed = cfg.seed ^ kParticleSeedSalt;  // salted sub-seed (research.md D6)
    ec.speed_min = 0.0f;
    ec.speed_max = kFieldParticleSpeedMax;
    ec.lifetime_min_seconds = kFieldParticleLifetimeSeconds;
    ec.lifetime_max_seconds = kFieldParticleLifetimeSeconds;
    return ec;
}

}  // namespace

eastl::optional<arena> arena::create(engine_demo::allocator& alloc,
                                     const arena_config& cfg) noexcept {
    if (cfg.player_count < kMinPlayers || cfg.player_count > kMaxPlayers) {
        return eastl::nullopt;
    }
    if (!(cfg.half_extent > 0.0f)) {
        return eastl::nullopt;
    }
    if (cfg.particle_capacity == 0 || cfg.particle_capacity > kMaxSnapshotParticles) {
        return eastl::nullopt;
    }

    // The pool lives at an allocator-provided stable address: the emitter member
    // keeps an internal pointer to it, which must survive arena moves.
    void* raw = alloc.allocate(sizeof(engine_demo::vfx::particle_pool),
                               alignof(engine_demo::vfx::particle_pool));
    if (raw == nullptr) {
        return eastl::nullopt;
    }
    auto* pool = new (raw) engine_demo::vfx::particle_pool{alloc, cfg.particle_capacity};
    return eastl::optional<arena>{arena{alloc, cfg, pool}};
}

arena::arena(engine_demo::allocator& alloc, const arena_config& cfg,
             engine_demo::vfx::particle_pool* pool) noexcept
    : m_alloc{&alloc},
      m_cfg{cfg},
      m_pool{pool},
      m_emitter{alloc, *pool, make_flood_emitter_config(cfg)},
      m_rng{cfg.seed},
      m_field_rng{cfg.seed ^ kFieldSeedSalt},
      m_budget{alloc},
      m_powerups{alloc},
      m_log{alloc, kMaxRecordedTicks},
      m_hashes{engine_demo::eastl_allocator_ref{alloc}} {
    m_hashes.reserve(kMaxHashHistory);  // Article 6: reserved once, never regrows
}

arena::arena(arena&& other) noexcept
    : m_alloc{other.m_alloc},
      m_cfg{other.m_cfg},
      m_pool{other.m_pool},
      m_emitter{eastl::move(other.m_emitter)},
      m_rng{other.m_rng},
      m_field_rng{other.m_field_rng},
      m_budget{eastl::move(other.m_budget)},
      m_match{other.m_match},
      m_scores{other.m_scores},
      m_powerups{eastl::move(other.m_powerups)},
      m_log{eastl::move(other.m_log)},
      m_hashes{eastl::move(other.m_hashes)},
      m_tick{other.m_tick} {
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        m_wells[i] = other.m_wells[i];
    }
    other.m_pool = nullptr;  // ownership of the placement-new'd pool moved here
}

arena& arena::operator=(arena&& other) noexcept {
    if (this != &other) {
        destroy_pool();
        m_alloc = other.m_alloc;
        m_cfg = other.m_cfg;
        m_pool = other.m_pool;
        m_emitter = eastl::move(other.m_emitter);
        m_rng = other.m_rng;
        m_field_rng = other.m_field_rng;
        m_budget = eastl::move(other.m_budget);
        m_match = other.m_match;
        m_scores = other.m_scores;
        m_powerups = eastl::move(other.m_powerups);
        m_log = eastl::move(other.m_log);
        m_hashes = eastl::move(other.m_hashes);
        for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
            m_wells[i] = other.m_wells[i];
        }
        m_tick = other.m_tick;
        other.m_pool = nullptr;
    }
    return *this;
}

arena::~arena() {
    destroy_pool();
}

void arena::destroy_pool() noexcept {
    if (m_pool != nullptr) {
        m_pool->~particle_pool();
        m_alloc->deallocate(m_pool, sizeof(engine_demo::vfx::particle_pool));
        m_pool = nullptr;
    }
}

arena_status arena::join(std::uint8_t player) noexcept {
    if (player >= m_cfg.player_count) {
        return arena_status::invalid_argument;  // dense identity player->well mapping
    }
    return m_match.join(player);
}

arena_status arena::set_ready(std::uint8_t player, bool ready) noexcept {
    if (player >= m_cfg.player_count) {
        return arena_status::invalid_argument;
    }
    return m_match.set_ready(player, ready);
}

void arena::leave(std::uint8_t player) noexcept {
    if (player >= m_cfg.player_count) {
        return;
    }
    m_match.leave(player);
    if (m_match.state() != match_state::playing) {
        return;
    }
    m_wells[player].active = false;  // FR-013: the well deactivates immediately
    if (m_match.active_count() < kMinPlayers) {
        // Last remaining active player wins by forfeit (FR-013 edge case).
        for (std::uint8_t i = 0; i < m_cfg.player_count; ++i) {
            if (m_wells[i].active) {
                m_scores.winner = static_cast<std::int8_t>(i);
                break;
            }
        }
        m_match.enter_game_over();
        m_powerups.clear_all();  // FR-011: all effects end at game_over
    }
}

arena_status arena::acknowledge_results() noexcept {
    const arena_status status = m_match.acknowledge_results();
    if (status == arena_status::ok) {
        reset_scores(m_scores);  // FR-012: scores reset on return to lobby
    }
    return status;
}

arena_status arena::tick(const tick_inputs& inputs) noexcept {
    const std::chrono::steady_clock::time_point t0 = std::chrono::steady_clock::now();

    // 1. Ingest + clamp + log (every state — FR-015/FR-016).
    tick_inputs clamped{};
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        clamped.players[i] = clamp_input(inputs.players[i]);
    }
    const arena_status log_status = m_log.append(clamped);

    // 2. Match state machine; transition side effects run exactly once (match.md).
    bool entered_playing = false;
    if (const eastl::optional<match::transition> tr = m_match.tick()) {
        if (tr->to == match_state::playing) {
            on_playing_entry();
            entered_playing = true;
        }
    }

    // 3. Gameplay systems (playing only — FR-014; skipped on the transition tick,
    // see file header note).
    if (!entered_playing && m_match.state() == match_state::playing) {
        run_playing_systems(clamped);
    }

    // 4. Captured particles (zeroed lifetimes, research.md D2) physically leave.
    m_pool->age_and_retire(kTickSeconds);

    // 5. Deterministic replenishment schedule (salted stream, symmetric groups).
    replenish_field();

    // 6. Article 10: 60-tick state hash for drift detection.
    if ((m_tick % kStateHashIntervalTicks) == 0 && m_hashes.size() < kMaxHashHistory) {
        m_hashes.push_back(state_hash(capture_snapshot()));
    }

    // 7. Advance the fixed-step clock.
    ++m_tick;

    const std::chrono::steady_clock::time_point t1 = std::chrono::steady_clock::now();
    m_budget.record_sample(
        std::chrono::duration<double, std::milli>(t1 - t0).count());
    return log_status;
}

void arena::on_playing_entry() noexcept {
    reset_scores(m_scores);   // US2 scenario 2
    m_powerups.clear_all();   // fresh spawn timer, no stale effects
    const std::uint8_t n = m_cfg.player_count;
    const float base[2] = {m_cfg.half_extent * 0.5f, 0.0f};
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        m_wells[i] = gravity_well{};
        m_wells[i].active = false;
    }
    for (std::uint8_t i = 0; i < n; ++i) {
        rotate_copy(i, n, base, m_wells[i].position);  // FR-001 rotational symmetry
        m_wells[i].active = true;
        m_wells[i].strength = 0.0f;
    }
}

void arena::run_playing_systems(const tick_inputs& in) noexcept {
    const std::uint8_t n = m_cfg.player_count;

    // 3a. Inputs -> well velocity/strength; integrate; clamp to bounds.
    for (std::uint8_t i = 0; i < n; ++i) {
        step_well(m_wells[i], in.players[i].steer, in.players[i].strength,
                  m_cfg.half_extent, kTickSeconds);
    }

    // 3b + 3c. Radial attraction (well index order, distance-only) then particle
    // integration with boundary reflection — one fixed-order pass (research.md D1).
    const eastl::span<engine_demo::vfx::particle> live = m_pool->live_particles();
    const float dt = static_cast<float>(kTickSeconds);
    for (engine_demo::vfx::particle& p : live) {
        float accel[2] = {0.0f, 0.0f};
        const float pos[2] = {p.position[0], p.position[1]};
        for (std::uint8_t i = 0; i < n; ++i) {
            gravity_well pulled = m_wells[i];  // POD copy: apply Strength Surge
            pulled.strength *= m_powerups.strength_multiplier(i);
            float a[2];
            radial_acceleration(pulled, pos, a);
            accel[0] += a[0];
            accel[1] += a[1];
        }
        p.velocity[0] += accel[0] * dt;
        p.velocity[1] += accel[1] * dt;
        p.position[0] += p.velocity[0] * dt;
        p.position[1] += p.velocity[1] * dt;
        for (int axis = 0; axis < 2; ++axis) {  // reflect at arena bounds
            if (p.position[axis] > m_cfg.half_extent) {
                p.position[axis] = 2.0f * m_cfg.half_extent - p.position[axis];
                p.velocity[axis] = -p.velocity[axis];
            } else if (p.position[axis] < -m_cfg.half_extent) {
                p.position[axis] = -2.0f * m_cfg.half_extent - p.position[axis];
                p.velocity[axis] = -p.velocity[axis];
            }
        }
    }

    // 3d. Capture contention + same-tick scoring (FR-007, SC-004).
    bool double_points[kMaxPlayers] = {};
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        double_points[i] = m_powerups.points_multiplier(i) > 1u;
    }
    const std::uint32_t scored_mask =
        resolve_captures({m_wells, n}, live, m_scores, double_points);

    // 3e. Pickup consumption; flood requests forward to the salted emitter.
    if (const eastl::optional<flood_request> flood =
            m_powerups.consume_pickups({m_wells, n}, m_rng)) {
        spawn_flood(flood->count);
    }

    // 3f. Power-up spawn schedule (game-rule rng draws only on spawn — FR-019).
    m_powerups.tick_spawn(m_rng, m_tick, m_cfg.half_extent);

    // 3g. Effect timers.
    m_powerups.tick_effects();

    // 3h. Win / sudden-death evaluation (FR-006/FR-008).
    const std::int8_t win = evaluate_win(m_scores, n, scored_mask);
    if (win >= 0) {
        m_match.enter_game_over();
        m_powerups.clear_all();  // FR-011: all effects end at game_over
    }
}

void arena::replenish_field() noexcept {
    if ((m_tick % kReplenishIntervalTicks) != 0) {
        return;
    }
    const std::uint8_t group = m_cfg.player_count;
    std::size_t deficit = m_cfg.particle_capacity - m_pool->live_count();

    engine_demo::vfx::spawn_params params[kMaxPlayers];
    while (deficit >= group) {
        // Fixed draw order per group: x, y, heading, speed (Article 5).
        const float base_pos[2] = {
            static_cast<float>((m_field_rng.next_double_unit() * 2.0 - 1.0) *
                               m_cfg.half_extent),
            static_cast<float>((m_field_rng.next_double_unit() * 2.0 - 1.0) *
                               m_cfg.half_extent)};
        const double heading = m_field_rng.next_double_unit() * 6.283185307179586;
        const float speed =
            static_cast<float>(m_field_rng.next_double_unit()) * kFieldParticleSpeedMax;
        const float base_vel[2] = {speed * static_cast<float>(std::cos(heading)),
                                   speed * static_cast<float>(std::sin(heading))};

        for (std::uint8_t k = 0; k < group; ++k) {
            params[k] = engine_demo::vfx::spawn_params{};
            rotate_copy(k, group, base_pos, params[k].position);
            rotate_copy(k, group, base_vel, params[k].velocity);
            params[k].position[2] = 0.0f;  // 2D game plane
            params[k].velocity[2] = 0.0f;
            params[k].lifetime_seconds = kFieldParticleLifetimeSeconds;
        }
        (void)m_pool->try_spawn({params, group});
        deficit -= group;
    }
}

void arena::spawn_flood(std::uint32_t count) noexcept {
    const std::size_t before = m_pool->live_count();
    (void)m_emitter.try_emit(count);  // partial fill on pool exhaustion is fine
    const eastl::span<engine_demo::vfx::particle> live = m_pool->live_particles();
    for (std::size_t i = before; i < live.size(); ++i) {
        live[i].position[2] = 0.0f;  // project the emitter's 3D sample onto the plane
        live[i].velocity[2] = 0.0f;
    }
}

match_snapshot arena::capture_snapshot() const noexcept {
    match_snapshot s{};
    s.state = m_match.state();
    s.tick = m_tick;
    s.seed = m_cfg.seed;
    s.player_count = m_cfg.player_count;
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        s.scores[i] = m_scores.scores[i];
    }
    s.winner = m_scores.winner;
    s.sudden_death = m_scores.sudden_death;
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        s.wells[i].position[0] = m_wells[i].position[0];
        s.wells[i].position[1] = m_wells[i].position[1];
        s.wells[i].velocity[0] = m_wells[i].velocity[0];
        s.wells[i].velocity[1] = m_wells[i].velocity[1];
        s.wells[i].strength = m_wells[i].strength;
        s.wells[i].active = m_wells[i].active;
    }
    const eastl::span<const active_effect> effects = m_powerups.effects();
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        s.effects[i].kind = effects[i].kind;
        s.effects[i].remaining_ticks = effects[i].remaining_ticks;
    }
    const eastl::span<const pickup> field = m_powerups.field_pickups();
    for (std::uint32_t i = 0; i < kMaxFieldPickups; ++i) {
        s.pickups[i].kind = field[i].kind;
        s.pickups[i].position[0] = field[i].position[0];
        s.pickups[i].position[1] = field[i].position[1];
        s.pickups[i].spawn_tick = field[i].spawn_tick;
        s.pickups[i].alive = field[i].alive;
    }
    s.powerup_timer_ticks = m_powerups.spawn_timer_ticks();

    const eastl::span<const engine_demo::vfx::particle> live = m_pool->live_particles();
    const std::size_t count =
        live.size() < kMaxSnapshotParticles ? live.size() : kMaxSnapshotParticles;
    s.live_particle_count = static_cast<std::uint32_t>(count);
    for (std::size_t i = 0; i < count; ++i) {
        s.particles[i].position[0] = live[i].position[0];
        s.particles[i].position[1] = live[i].position[1];
        s.particles[i].velocity[0] = live[i].velocity[0];
        s.particles[i].velocity[1] = live[i].velocity[1];
    }
    return s;
}

arena_status arena::restore(const match_snapshot& s) noexcept {
    // The snapshot must belong to this arena's configuration: the rng streams are
    // derived from the seed and are not snapshot state (see header contract).
    if (s.seed != m_cfg.seed || s.player_count != m_cfg.player_count) {
        return arena_status::invalid_argument;
    }
    if (s.live_particle_count > m_cfg.particle_capacity) {
        return arena_status::invalid_argument;
    }

    m_tick = s.tick;
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        m_scores.scores[i] = s.scores[i];
    }
    m_scores.winner = s.winner;
    m_scores.sudden_death = s.sudden_death;

    bool departed[kMaxPlayers] = {};
    const bool roster_fixed =
        s.state == match_state::playing || s.state == match_state::game_over;
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        m_wells[i] = gravity_well{};
        m_wells[i].position[0] = s.wells[i].position[0];
        m_wells[i].position[1] = s.wells[i].position[1];
        m_wells[i].velocity[0] = s.wells[i].velocity[0];
        m_wells[i].velocity[1] = s.wells[i].velocity[1];
        m_wells[i].strength = s.wells[i].strength;
        m_wells[i].active = s.wells[i].active;
        departed[i] = roster_fixed && i < s.player_count && !s.wells[i].active;
    }
    m_match.restore(s.state, s.player_count, departed);

    pickup field[kMaxFieldPickups];
    for (std::uint32_t i = 0; i < kMaxFieldPickups; ++i) {
        field[i].kind = s.pickups[i].kind;
        field[i].position[0] = s.pickups[i].position[0];
        field[i].position[1] = s.pickups[i].position[1];
        field[i].spawn_tick = s.pickups[i].spawn_tick;
        field[i].alive = s.pickups[i].alive;
    }
    active_effect effects[kMaxPlayers];
    for (std::uint8_t i = 0; i < kMaxPlayers; ++i) {
        effects[i].kind = s.effects[i].kind;
        effects[i].remaining_ticks = s.effects[i].remaining_ticks;
    }
    m_powerups.restore({field, kMaxFieldPickups}, {effects, kMaxPlayers},
                       s.powerup_timer_ticks);

    // Rebuild the particle field in snapshot order (pool order is deterministic).
    clear_particles();
    for (std::uint32_t i = 0; i < s.live_particle_count; ++i) {
        engine_demo::vfx::spawn_params sp{};
        sp.position[0] = s.particles[i].position[0];
        sp.position[1] = s.particles[i].position[1];
        sp.position[2] = 0.0f;
        sp.velocity[0] = s.particles[i].velocity[0];
        sp.velocity[1] = s.particles[i].velocity[1];
        sp.velocity[2] = 0.0f;
        sp.lifetime_seconds = kFieldParticleLifetimeSeconds;
        (void)m_pool->try_spawn({&sp, 1});
    }
    return arena_status::ok;
}

void arena::clear_particles() noexcept {
    for (engine_demo::vfx::particle& p : m_pool->live_particles()) {
        p.remaining_lifetime_seconds = 0.0;
    }
    m_pool->age_and_retire(1.0e-9);  // any positive dt retires the zeroed particles
}

}  // namespace orbital_arena
