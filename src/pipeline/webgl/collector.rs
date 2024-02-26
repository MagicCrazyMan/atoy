use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use uuid::Uuid;

use crate::{
    bounding::Culling, clock::WebClock, entity::{Entity, Group}, frustum::ViewFrustum, material::Transparency, renderer::webgl::state::FrameState, scene::Scene
};

pub struct CollectedEntities<'a> {
    entities: &'a [Weak<RefCell<dyn Entity>>],
    opaque_entities: &'a [Weak<RefCell<dyn Entity>>],
    transparent_entities: &'a [Weak<RefCell<dyn Entity>>],
    translucent_entities: &'a [Weak<RefCell<dyn Entity>>],
}

impl<'a> CollectedEntities<'a> {
    pub fn entities(&self) -> &[Weak<RefCell<dyn Entity>>] {
        self.entities
    }

    pub fn opaque_entities(&self) -> &[Weak<RefCell<dyn Entity>>] {
        self.opaque_entities
    }

    pub fn transparent_entities(&self) -> &[Weak<RefCell<dyn Entity>>] {
        self.transparent_entities
    }

    pub fn translucent_entities(&self) -> &[Weak<RefCell<dyn Entity>>] {
        self.translucent_entities
    }
}

pub struct StandardEntitiesCollector {
    enable_culling: bool,
    enable_distance_sorting: bool,

    last_view_frustum: Option<ViewFrustum>,
    last_scene_id: Option<Uuid>,
    last_entities: Vec<Weak<RefCell<dyn Entity>>>,
    last_opaque_entities: Vec<Weak<RefCell<dyn Entity>>>,
    last_transparent_entities: Vec<Weak<RefCell<dyn Entity>>>,
    last_translucent_entities: Vec<Weak<RefCell<dyn Entity>>>,
}

impl StandardEntitiesCollector {
    /// Constructs a new entities collector.
    pub fn new() -> Self {
        Self {
            enable_culling: true,
            enable_distance_sorting: true,

            last_view_frustum: None,
            last_scene_id: None,
            last_entities: Vec::new(),
            last_opaque_entities: Vec::new(),
            last_transparent_entities: Vec::new(),
            last_translucent_entities: Vec::new(),
        }
    }

    /// Clears previous collected result.
    pub fn clear(&mut self) {
        self.last_view_frustum = None;
        self.last_scene_id = None;
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

    /// Returns last collected entities.
    pub fn last_collected_entities(&self) -> CollectedEntities {
        CollectedEntities {
            entities: &self.last_entities,
            opaque_entities: &self.last_opaque_entities,
            transparent_entities: &self.last_transparent_entities,
            translucent_entities: &self.last_translucent_entities,
        }
    }

    /// Collects and returns entities.
    pub fn collect_entities(
        &mut self,
        state: &mut FrameState,
        scene: &mut Scene<WebClock>,
    ) -> CollectedEntities {
        struct CollectedEntity {
            entity: Rc<RefCell<dyn Entity>>,
            transparency: Transparency,
            distance: f64,
        }

        let view_frustum = state.camera().view_frustum();

        let should_recollect = scene.entity_group().should_sync()
            || self
                .last_scene_id
                .as_ref()
                .map(|last_scene_id| last_scene_id != scene.entity_group().id())
                .unwrap_or(true)
            || self
                .last_view_frustum
                .as_ref()
                .map(|last_view_frustum| last_view_frustum != &view_frustum)
                .unwrap_or(true);
        if !should_recollect {
            return CollectedEntities {
                entities: &self.last_entities,
                opaque_entities: &self.last_opaque_entities,
                transparent_entities: &self.last_transparent_entities,
                translucent_entities: &self.last_translucent_entities,
            };
        }

        self.clear();

        let view_position = state.camera().position();
        let culling = self.culling_enabled();
        let distance_sorting = self.distance_sorting_enabled();
        let mut entities = Vec::new();

        scene.entity_group_mut().sync();

        if culling {
            for entity in scene.entity_group_mut().entities() {
                let distance = match entity.borrow().bounding_volume() {
                    Some(entity_bounding) => match entity_bounding.cull(&view_frustum) {
                        Culling::Outside => continue,
                        Culling::Inside { near, .. } | Culling::Intersect { near, .. } => near,
                    },
                    None => f64::INFINITY,
                };

                let transparency = entity
                    .borrow()
                    .material()
                    .map(|material| material.transparency())
                    .unwrap_or(Transparency::Transparent);

                entities.push(CollectedEntity {
                    entity,
                    transparency,
                    distance,
                });
            }

            for group in scene.entity_group_mut().sub_groups_hierarchy() {
                // culling group bounding
                if let Some(group_bounding) = group.borrow().bounding_volume() {
                    if let Culling::Outside = group_bounding.cull(&view_frustum) {
                        continue;
                    }
                }

                for entity in group.borrow().entities() {
                    let distance = match entity.borrow().bounding_volume() {
                        Some(entity_bounding) => match entity_bounding.cull(&view_frustum) {
                            Culling::Outside => continue,
                            Culling::Inside { near, .. } | Culling::Intersect { near, .. } => near,
                        },
                        None => f64::INFINITY,
                    };

                    let transparency = entity
                        .borrow()
                        .material()
                        .map(|material| material.transparency())
                        .unwrap_or(Transparency::Transparent);

                    entities.push(CollectedEntity {
                        entity,
                        transparency,
                        distance,
                    });
                }
            }
        } else {
            for entity in scene.entity_group_mut().entities_hierarchy() {
                let transparency = entity
                    .borrow()
                    .material()
                    .map(|material| material.transparency())
                    .unwrap_or(Transparency::Transparent);
                let distance = match distance_sorting {
                    true => entity
                        .borrow()
                        .bounding_volume()
                        .map(|bounding| bounding.center().distance(&view_position))
                        .unwrap_or(f64::INFINITY),
                    false => f64::INFINITY,
                };

                entities.push(CollectedEntity {
                    entity,
                    transparency,
                    distance,
                });
            }
        }

        if distance_sorting {
            entities.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        }

        for CollectedEntity {
            entity,
            transparency,
            ..
        } in entities.iter()
        {
            // checks material availability
            // prepares material if not ready yet
            {
                let mut entity = entity.borrow_mut();
                if let Some(material) = entity.material_mut() {
                    if !material.ready() {
                        material.prepare(state);
                        scene.entity_group_mut().set_resync();
                        continue;
                    }
                }
            }

            let entity = Rc::downgrade(&entity);
            self.last_entities.push(Weak::clone(&entity));
            match transparency {
                Transparency::Opaque => self.last_opaque_entities.push(Weak::clone(&entity)),
                Transparency::Transparent => {
                    self.last_transparent_entities.push(Weak::clone(&entity))
                }
                Transparency::Translucent(_) => {
                    self.last_translucent_entities.push(Weak::clone(&entity))
                }
            }
        }

        self.last_scene_id = Some(scene.entity_group().id().clone());
        self.last_view_frustum = Some(view_frustum);

        CollectedEntities {
            entities: &self.last_entities,
            opaque_entities: &self.last_opaque_entities,
            transparent_entities: &self.last_transparent_entities,
            translucent_entities: &self.last_translucent_entities,
        }
    }
}
