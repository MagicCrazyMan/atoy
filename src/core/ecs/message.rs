use uuid::Uuid;

use super::archetype::Archetype;

#[derive(Debug, Clone)]
pub enum Message {
    AddComponent {
        entity_id: Uuid,
        old_archetype: Archetype,
        new_archetype: Archetype,
    },
    RemoveComponent {
        entity_id: Uuid,
        old_archetype: Archetype,
        new_archetype: Archetype,
    },
    AddEntity {
        entity_id: Uuid,
    },
    RemoveEntity {
        entity_id: Uuid,
    },
}
