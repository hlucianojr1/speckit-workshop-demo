// Orbital Arena power-up system — see include/orbital_arena/powerup.h for the contract.

#include "orbital_arena/powerup.h"

#include "orbital_arena/scoring.h"

namespace orbital_arena {

powerup_system::powerup_system(engine_demo::allocator& /*alloc*/) noexcept {
    // v1 storage is fixed slot arrays only; the allocator parameter exists for
    // constructor-signature uniformity across allocator-aware modules (Article 4).
}

void powerup_system::tick_spawn(engine_demo::sim::rng& rng, std::uint64_t current_tick,
                                float half_extent) noexcept {
    if (--m_spawn_timer > 0) {
        return;
    }
    m_spawn_timer = kPowerupSpawnIntervalTicks;

    // FR-009 cap: at kMaxFieldPickups alive the spawn is skipped and — critically for
    // stream auditability (FR-019) — no rng draws occur.
    std::uint32_t alive = 0;
    pickup* free_slot = nullptr;
    for (pickup& slot : m_pickups) {
        if (slot.alive) {
            ++alive;
        } else if (free_slot == nullptr) {
            free_slot = &slot;
        }
    }
    if (alive >= kMaxFieldPickups || free_slot == nullptr) {
        return;
    }

    // Exactly 3 draws in fixed order: x, y, kind (FR-019, research.md D6).
    const float x = static_cast<float>((rng.next_double_unit() * 2.0 - 1.0) * half_extent);
    const float y = static_cast<float>((rng.next_double_unit() * 2.0 - 1.0) * half_extent);
    const powerup_kind kind = static_cast<powerup_kind>(rng.next_u32() % 3u);

    free_slot->kind = kind;
    free_slot->position[0] = x;
    free_slot->position[1] = y;
    free_slot->spawn_tick = current_tick;
    free_slot->alive = true;
}

eastl::optional<flood_request>
powerup_system::consume_pickups(eastl::span<const gravity_well> wells,
                                engine_demo::sim::rng& /*rng*/) noexcept {
    std::uint32_t flood_total = 0;

    // Fixed slot-index iteration; contention per pickup is scoring's distance-only
    // rule (FR-007 applied to pickups — exact tie leaves the pickup on the field).
    for (pickup& slot : m_pickups) {
        if (!slot.alive) {
            continue;
        }
        const capture_result result = resolve_capture(wells, slot.position);
        if (result.winner_index < 0) {
            continue;
        }
        const std::uint8_t player = static_cast<std::uint8_t>(result.winner_index);
        slot.alive = false;
        switch (slot.kind) {
            case powerup_kind::strength_surge:
                m_effects[player] = active_effect{powerup_kind::strength_surge,
                                                  kStrengthSurgeTicks};
                break;
            case powerup_kind::double_points:
                m_effects[player] = active_effect{powerup_kind::double_points,
                                                  kDoublePointsTicks};
                break;
            case powerup_kind::particle_flood:
                flood_total += kFloodBurstCount;  // instant: never occupies a slot
                break;
        }
    }

    if (flood_total == 0) {
        return eastl::nullopt;
    }
    return flood_request{flood_total};
}

void powerup_system::tick_effects() noexcept {
    for (active_effect& effect : m_effects) {
        if (effect.remaining_ticks > 0) {
            --effect.remaining_ticks;  // expires the exact tick it reaches 0 (SC-006)
        }
    }
}

void powerup_system::clear_all() noexcept {
    for (active_effect& effect : m_effects) {
        effect = active_effect{};
    }
    for (pickup& slot : m_pickups) {
        slot = pickup{};
    }
    m_spawn_timer = kPowerupSpawnIntervalTicks;
}

bool powerup_system::is_active(std::uint8_t player, powerup_kind kind) const noexcept {
    if (player >= kMaxPlayers) {
        return false;
    }
    const active_effect& effect = m_effects[player];
    return effect.remaining_ticks > 0 && effect.kind == kind;
}

float powerup_system::strength_multiplier(std::uint8_t player) const noexcept {
    return is_active(player, powerup_kind::strength_surge) ? 2.0f : 1.0f;
}

std::uint32_t powerup_system::points_multiplier(std::uint8_t player) const noexcept {
    return is_active(player, powerup_kind::double_points) ? 2u : 1u;
}

}  // namespace orbital_arena
