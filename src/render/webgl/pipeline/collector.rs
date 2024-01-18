use std::{collections::VecDeque, iter::FromIterator};

use uuid::Uuid;

use crate::{
    bounding::Culling, entity::Entity, frustum::ViewFrustum, material::Transparency,
    render::webgl::state::FrameState, scene::Scene,
};

pub struct CollectedEntity {
    entity: *mut Entity,
    distance: f64,
}

impl CollectedEntity {
    pub unsafe fn entity<'a, 'b>(&'a self) -> &'b Entity {
        &*self.entity
    }

    pub unsafe fn entity_mut<'a, 'b>(&'a self) -> &'b mut Entity {
        &mut *self.entity
    }

    pub fn entity_raw(&self) -> *mut Entity {
        self.entity
    }

    pub fn distance(&self) -> f64 {
        self.distance
    }
}

pub struct CollectedEntities<'a> {
    id: usize,
    entities: &'a [CollectedEntity],
    opaque_entity_indices: &'a [usize],
    transparent_entity_indices: &'a [usize],
    translucent_entity_indices: &'a [usize],
}

impl<'a> CollectedEntities<'a> {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn entities(&self) -> &[CollectedEntity] {
        self.entities
    }

    pub fn opaque_entity_indices(&self) -> &[usize] {
        self.opaque_entity_indices
    }

    pub fn transparent_entity_indices(&self) -> &[usize] {
        self.transparent_entity_indices
    }

    pub fn translucent_entity_indices(&self) -> &[usize] {
        self.translucent_entity_indices
    }
}

pub struct StandardEntitiesCollector {
    enable_culling: bool,
    enable_sorting: bool,

    last_view_frustum: Option<ViewFrustum>,
    last_container: Option<Uuid>,
    last_collected_id: Option<usize>,
    last_entities: Vec<CollectedEntity>,
    last_opaque_entity_indices: Vec<usize>,
    last_transparent_entity_indices: Vec<usize>,
    last_translucent_entity_indices: Vec<usize>,
}

impl StandardEntitiesCollector {
    /// Constructs a new entities collector.
    pub fn new() -> Self {
        Self {
            enable_culling: true,
            enable_sorting: true,

            last_view_frustum: None,
            last_container: None,
            last_collected_id: None,
            last_entities: Vec::new(),
            last_opaque_entity_indices: Vec::new(),
            last_transparent_entity_indices: Vec::new(),
            last_translucent_entity_indices: Vec::new(),
        }
    }

    /// Clears previous collected result.
    pub fn clear(&mut self) {
        self.last_view_frustum = None;
        self.last_container = None;
        self.last_entities.clear();
        self.last_opaque_entity_indices.clear();
        self.last_transparent_entity_indices.clear();
        self.last_translucent_entity_indices.clear();
    }

    /// Returns `true` if entity culling enabled.
    pub fn culling_enabled(&self) -> bool {
        self.enable_culling
    }

    /// Enables culling by bounding volumes.
    pub fn enable_culling(&mut self) {
        if self.enable_culling != true {
            self.enable_culling = true;
            self.clear();
        }
    }

    /// Disables culling by bounding volumes.
    pub fn disable_culling(&mut self) {
        if self.enable_culling != false {
            self.enable_culling = false;
            self.clear();
        }
    }

    /// Returns `true` if entity distance sorting enabled.
    pub fn distance_sorting_enabled(&self) -> bool {
        self.enable_sorting
    }

    /// Enables distance sorting by bounding volumes.
    pub fn enable_distance_sorting(&mut self) {
        if self.enable_sorting != true {
            self.enable_sorting = true;
            self.clear();
        }
    }

    /// Disables distance sorting by bounding volumes.
    pub fn disable_distance_sorting(&mut self) {
        if self.enable_sorting != false {
            self.enable_sorting = false;
            self.clear();
        }
    }

    pub fn last_collected_entities(&self) -> Option<CollectedEntities> {
        match self.last_collected_id {
            Some(id) => Some(CollectedEntities {
                id,
                entities: &self.last_entities,
                opaque_entity_indices: &self.last_opaque_entity_indices,
                transparent_entity_indices: &self.last_transparent_entity_indices,
                translucent_entity_indices: &self.last_translucent_entity_indices,
            }),
            None => None,
        }
    }

    /// Collects and returns entities.
    pub fn collect_entities(&mut self, state: &FrameState, scene: &mut Scene) -> CollectedEntities {
        let view_frustum = state.camera().view_frustum();
        match (
            self.last_collected_id.as_ref(),
            self.last_container.as_ref(),
            self.last_view_frustum.as_ref(),
            scene.entity_container().is_dirty(),
        ) {
            (Some(last_collected_id), Some(last_container), Some(last_view_frustum), false) => {
                match (
                    last_container == scene.entity_container().id(),
                    last_view_frustum == &view_frustum,
                ) {
                    (true, true) => {
                        return CollectedEntities {
                            id: *last_collected_id,
                            entities: &self.last_entities,
                            opaque_entity_indices: &self.last_opaque_entity_indices,
                            transparent_entity_indices: &self.last_transparent_entity_indices,
                            translucent_entity_indices: &self.last_translucent_entity_indices,
                        }
                    }
                    _ => {
                        // recollect
                    }
                }
            }
            _ => {
                // recollect
            }
        }

        self.clear();
        scene.entity_container_mut().update();

        let view_position = state.camera().position();

        if self.enable_culling {
            let mut groups = VecDeque::from_iter([scene.entity_container_mut().root_group_mut()]);
            while let Some(group) = groups.pop_front() {
                // skips if group outside view frustum already
                if let Some(bounding) = group.bounding() {
                    let culling = bounding.cull(&view_frustum);
                    if let Culling::Outside(_) = culling {
                        continue;
                    }
                }

                for entity in group.entities_mut() {
                    let distance = match entity.bounding() {
                        Some(bounding) => {
                            let culling = bounding.cull(&view_frustum);
                            if let Culling::Outside(_) = culling {
                                continue;
                            }

                            match culling {
                                Culling::Outside(_) => unreachable!(),
                                Culling::Inside { near, .. } | Culling::Intersect { near, .. } => {
                                    near
                                }
                            }
                        }
                        None => f64::INFINITY,
                    };

                    self.last_entities
                        .push(CollectedEntity { entity, distance });

                    if let Some(transparency) =
                        entity.material().map(|material| material.transparency())
                    {
                        match transparency {
                            Transparency::Opaque => self
                                .last_opaque_entity_indices
                                .push(self.last_entities.len() - 1),
                            Transparency::Transparent => self
                                .last_transparent_entity_indices
                                .push(self.last_entities.len() - 1),
                            Transparency::Translucent(_) => self
                                .last_translucent_entity_indices
                                .push(self.last_entities.len() - 1),
                        }
                    }
                }

                groups.extend(group.subgroups_mut());
            }
        } else {
            let enable_sorting = self.enable_sorting;
            for entity in scene.entity_container_mut().entities_mut() {
                let distance = match (entity.bounding(), enable_sorting) {
                    (Some(bounding), true) => bounding.center().distance(&view_position),
                    _ => f64::INFINITY,
                };

                self.last_entities
                    .push(CollectedEntity { entity, distance });

                if let Some(transparency) =
                    entity.material().map(|material| material.transparency())
                {
                    match transparency {
                        Transparency::Opaque => self
                            .last_opaque_entity_indices
                            .push(self.last_entities.len() - 1),
                        Transparency::Transparent => self
                            .last_transparent_entity_indices
                            .push(self.last_entities.len() - 1),
                        Transparency::Translucent(_) => self
                            .last_translucent_entity_indices
                            .push(self.last_entities.len() - 1),
                    }
                }
            }
        }

        if self.enable_sorting {
            self.last_entities
                .sort_by(|a, b| a.distance.total_cmp(&b.distance));
        }

        self.last_container = Some(*scene.entity_container().id());
        self.last_view_frustum = Some(view_frustum);
        self.last_collected_id = match self.last_collected_id {
            Some(last_collected_id) => Some(last_collected_id.wrapping_add(1)),
            None => Some(0),
        };

        CollectedEntities {
            id: self.last_collected_id.unwrap(),
            entities: &self.last_entities,
            opaque_entity_indices: &self.last_opaque_entity_indices,
            transparent_entity_indices: &self.last_transparent_entity_indices,
            translucent_entity_indices: &self.last_translucent_entity_indices,
        }
    }
}
