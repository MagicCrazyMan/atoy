use crate::{
    bounding::Culling,
    entity::{Container, Entity},
    frustum::ViewFrustum,
    material::Transparency,
    render::webgl::state::FrameState,
    scene::Scene,
};

#[derive(Clone, Copy)]
pub struct CollectedEntity {
    entity: *mut Entity,
    transparency: Transparency,
    distance: f64,
}

impl CollectedEntity {
    #[inline]
    pub unsafe fn entity<'a, 'b>(&'a self) -> &'b Entity {
        &*self.entity
    }

    #[inline]
    pub unsafe fn entity_mut<'a, 'b>(&'a self) -> &'b mut Entity {
        &mut *self.entity
    }

    #[inline]
    pub fn entity_raw(&self) -> *mut Entity {
        self.entity
    }

    #[inline]
    pub fn distance(&self) -> f64 {
        self.distance
    }
}

pub struct CollectedEntities<'a> {
    id: usize,
    entities: &'a [CollectedEntity],
    opaque_entities: &'a [CollectedEntity],
    transparent_entities: &'a [CollectedEntity],
    translucent_entities: &'a [CollectedEntity],
}

impl<'a> CollectedEntities<'a> {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn entities(&self) -> &[CollectedEntity] {
        self.entities
    }

    pub fn opaque_entities(&self) -> &[CollectedEntity] {
        self.opaque_entities
    }

    pub fn transparent_entities(&self) -> &[CollectedEntity] {
        self.transparent_entities
    }

    pub fn translucent_entities(&self) -> &[CollectedEntity] {
        self.translucent_entities
    }
}

pub struct StandardEntitiesCollector {
    enable_culling: bool,
    enable_distance_sorting: bool,

    last_view_frustum: Option<ViewFrustum>,
    last_container_ptr: Option<*const Container>,
    last_collected_id: Option<usize>,
    last_entities: Vec<CollectedEntity>,
    last_opaque_entities: Vec<CollectedEntity>,
    last_transparent_entities: Vec<CollectedEntity>,
    last_translucent_entities: Vec<CollectedEntity>,
}

impl StandardEntitiesCollector {
    /// Constructs a new entities collector.
    pub fn new() -> Self {
        Self {
            enable_culling: true,
            enable_distance_sorting: true,

            last_view_frustum: None,
            last_container_ptr: None,
            last_collected_id: None,
            last_entities: Vec::new(),
            last_opaque_entities: Vec::new(),
            last_transparent_entities: Vec::new(),
            last_translucent_entities: Vec::new(),
        }
    }

    /// Clears previous collected result.
    pub fn clear(&mut self) {
        self.last_view_frustum = None;
        self.last_container_ptr = None;
        self.last_entities.clear();
        self.last_opaque_entities.clear();
        self.last_transparent_entities.clear();
        self.last_translucent_entities.clear();
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
        self.enable_distance_sorting
    }

    /// Enables distance sorting by bounding volumes.
    pub fn enable_distance_sorting(&mut self) {
        if self.enable_distance_sorting != true {
            self.enable_distance_sorting = true;
            self.clear();
        }
    }

    /// Disables distance sorting by bounding volumes.
    pub fn disable_distance_sorting(&mut self) {
        if self.enable_distance_sorting != false {
            self.enable_distance_sorting = false;
            self.clear();
        }
    }

    pub fn last_collected_entities(&self) -> Option<CollectedEntities> {
        match self.last_collected_id {
            Some(id) => Some(CollectedEntities {
                id,
                entities: &self.last_entities,
                opaque_entities: &self.last_opaque_entities,
                transparent_entities: &self.last_transparent_entities,
                translucent_entities: &self.last_translucent_entities,
            }),
            None => None,
        }
    }

    /// Collects and returns entities.
    pub fn collect_entities(&mut self, state: &FrameState, scene: &mut Scene) -> CollectedEntities {
        let view_frustum = state.camera().view_frustum();
        match (
            self.last_collected_id.as_ref(),
            self.last_container_ptr.as_ref(),
            self.last_view_frustum.as_ref(),
            scene.entity_container().is_dirty(),
        ) {
            (Some(last_collected_id), Some(last_container), Some(last_view_frustum), false) => {
                match (
                    std::ptr::eq(*last_container, scene.entity_container()),
                    last_view_frustum == &view_frustum,
                ) {
                    (true, true) => {
                        return CollectedEntities {
                            id: *last_collected_id,
                            entities: &self.last_entities,
                            opaque_entities: &self.last_opaque_entities,
                            transparent_entities: &self.last_transparent_entities,
                            translucent_entities: &self.last_translucent_entities,
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

        let view_position = state.camera().position();
        let culling = self.culling_enabled();
        let distance_sorting = self.distance_sorting_enabled();

        scene.entity_container_mut().refresh();
        for entity in scene.entity_container_mut().entities_mut() {
            let transparency = match entity.material().map(|material| material.transparency()) {
                Some(transparency) => transparency,
                None => continue,
            };

            let collected = match (culling, distance_sorting) {
                (true, true) => {
                    let distance = match entity.bounding() {
                        Some(bounding) => match bounding.cull(&view_frustum) {
                            Culling::Outside => continue,
                            Culling::Inside { near, .. } | Culling::Intersect { near, .. } => near,
                        },
                        None => f64::INFINITY,
                    };

                    CollectedEntity {
                        entity,
                        transparency,
                        distance,
                    }
                }
                (true, false) => {
                    let distance = match entity.bounding() {
                        Some(bounding) => match bounding.cull(&view_frustum) {
                            Culling::Outside => continue,
                            Culling::Inside { near, .. } | Culling::Intersect { near, .. } => near,
                        },
                        None => f64::INFINITY,
                    };

                    CollectedEntity {
                        entity,
                        transparency,
                        distance,
                    }
                }
                (false, true) => {
                    let distance = match entity.bounding() {
                        Some(bounding) => bounding.center().distance(&view_position),
                        None => f64::INFINITY,
                    };

                    CollectedEntity {
                        entity,
                        transparency,
                        distance,
                    }
                }
                (false, false) => CollectedEntity {
                    entity,
                    transparency,
                    distance: f64::INFINITY,
                },
            };

            self.last_entities.push(collected);
            if !distance_sorting {
                match transparency {
                    Transparency::Opaque => self.last_opaque_entities.push(collected),
                    Transparency::Transparent => self.last_transparent_entities.push(collected),
                    Transparency::Translucent(_) => self.last_translucent_entities.push(collected),
                };
            }
        }

        if distance_sorting {
            self.last_entities
                .sort_by(|a, b| a.distance.total_cmp(&b.distance));
            for collected in self.last_entities.iter() {
                match collected.transparency {
                    Transparency::Opaque => self.last_opaque_entities.push(*collected),
                    Transparency::Transparent => self.last_transparent_entities.push(*collected),
                    Transparency::Translucent(_) => self.last_translucent_entities.push(*collected),
                }
            }
        }

        self.last_container_ptr = Some(scene.entity_container());
        self.last_view_frustum = Some(view_frustum);
        self.last_collected_id = match self.last_collected_id {
            Some(last_collected_id) => Some(last_collected_id.wrapping_add(1)),
            None => Some(0),
        };

        CollectedEntities {
            id: self.last_collected_id.unwrap(),
            entities: &self.last_entities,
            opaque_entities: &self.last_opaque_entities,
            transparent_entities: &self.last_transparent_entities,
            translucent_entities: &self.last_translucent_entities,
        }
    }
}
