// ECS world. Slot-based, generational handles.
//
// FIX BUG-003 (CWE-416 use-after-free): destroy_entity() now bumps slot.generation so
// that stale handles captured before the destroy can never validate against a recycled
// slot.

#include "engine_demo/ecs/world.h"

namespace engine_demo::ecs {

world::world(allocator& alloc) noexcept : m_alloc{&alloc} {}

entity_handle world::create_entity() noexcept {
    // Linear scan for the first free slot from m_next_free.
    for (std::size_t i = 0; i < kMaxEntities; ++i) {
        const std::size_t idx = (m_next_free + i) % kMaxEntities;
        if (!m_slots[idx].alive) {
            m_slots[idx].alive = true;
            if (m_slots[idx].generation == 0) {
                m_slots[idx].generation = 1;
            }
            m_next_free = (idx + 1) % kMaxEntities;
            ++m_live_count;
            return entity_handle{static_cast<std::uint32_t>(idx), m_slots[idx].generation};
        }
    }
    return entity_handle{};  // pool exhausted
}

void world::destroy_entity(entity_handle h) noexcept {
    if (h.index >= kMaxEntities)
        return;
    auto& s = m_slots[h.index];
    if (!s.alive || s.generation != h.generation)
        return;
    // Invalidate every outstanding handle to this slot before recycling it (CWE-416 guard).
    ++s.generation;
    s.alive = false;
    if (m_live_count > 0)
        --m_live_count;
}

bool world::is_alive(entity_handle h) const noexcept {
    if (h.index >= kMaxEntities)
        return false;
    const auto& s = m_slots[h.index];
    return s.alive && s.generation == h.generation && h.generation != 0;
}

}  // namespace engine_demo::ecs
