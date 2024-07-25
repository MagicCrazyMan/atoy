use super::{
    archetype::{Archetype, AsArchetype},
    component::Component,
    entity::Entity,
};

pub struct Chunk {
    archetype: Archetype,
    components: Vec<Box<dyn Component>>,
    versions: Vec<usize>,
}

impl Chunk {
    pub fn new(archetype: Archetype) -> Self {
        Self {
            archetype,
            components: Vec::new(),
            versions: Vec::new(),
        }
    }

    pub fn with_capacity(archetype: Archetype, capacity: usize) -> Self {
        let components_capacity = capacity * archetype.components_per_entity();
        Self {
            archetype,
            components: Vec::with_capacity(components_capacity),
            versions: Vec::with_capacity(capacity),
        }
    }

    pub fn from_as_archetypes<A: AsArchetype>() -> Self {
        Self::new(A::as_archetype())
    }

    pub fn from_as_archetypes_with_capacity<A: AsArchetype>(capacity: usize) -> Self {
        Self::with_capacity(A::as_archetype(), capacity)
    }
}

impl Chunk {
    pub fn archetype(&self) -> &Archetype {
        &self.archetype
    }

    pub fn components(&self, entity_index: usize) -> Option<ChunkComponentsRef> {
        let components_per_entity = self.archetype.components_per_entity();
        let si = entity_index * components_per_entity;
        let ei = si + components_per_entity;

        if ei > self.components.len() {
            return None;
        } else {
            Some(ChunkComponentsRef {
                archetype: &self.archetype,
                components: &self.components[si..ei],
                version: &self.versions[entity_index],
            })
        }
    }

    pub fn components_mut(&mut self, entity_index: usize) -> Option<ChunkComponentsMut> {
        let components_per_entity = self.archetype.components_per_entity();
        let si = entity_index * components_per_entity;
        let ei = si + components_per_entity;

        if ei > self.components.len() {
            return None;
        } else {
            Some(ChunkComponentsMut {
                archetype: &self.archetype,
                components: &mut self.components[si..ei],
                version: &self.versions[entity_index],
            })
        }
    }

    pub(super) fn components_by_entity(&self, entity: &Entity) -> Option<ChunkComponentsRef> {
        let r = self.components(entity.index)?;
        if *r.version == entity.version {
            Some(r)
        } else {
            None
        }
    }

    pub(super) fn components_mut_by_entity(&mut self, entity: &Entity) -> Option<ChunkComponentsMut> {
        let r = self.components_mut(entity.index)?;
        if *r.version == entity.version {
            Some(r)
        } else {
            None
        }
    }
}

pub struct ChunkComponentsRef<'a> {
    pub archetype: &'a Archetype,
    pub components: &'a [Box<dyn Component>],
    pub version: &'a usize,
}

pub struct ChunkComponentsMut<'a> {
    pub archetype: &'a Archetype,
    pub components: &'a mut [Box<dyn Component>],
    pub version: &'a usize,
}
