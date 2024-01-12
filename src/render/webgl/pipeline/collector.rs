use std::{any::Any, collections::VecDeque, ptr::NonNull};

use gl_matrix4rust::vec3::AsVec3;

use crate::{
    bounding::Culling,
    entity::Entity,
    render::{
        pp::{Executor, ResourceKey, Resources, State},
        webgl::error::Error,
    },
    scene::Scene,
};

pub static DEFAULT_CULLING_ENABLED: bool = true;
pub static DEFAULT_SORTING_ENABLED: bool = true;

/// Standard entities collector, collects and flatten entities from entities collection of [`Stuff`].
///
/// During collecting procedure, works list below will be done:
/// - Calculates model matrix for each entity collection and entity.
/// - Culls entity which has a bounding volume and it is outside the view frustum.
/// Entity which has no bounding volume will append to the last of the entity list.
///
/// # Provides Resources & Data Type
/// - `set_entities`: [`Vec<NonNull<Entity>>`], a list contains entities collected by this collector.
pub struct StandardEntitiesCollector {
    enable_culling_key: Option<ResourceKey<bool>>,
    enable_sorting_key: Option<ResourceKey<bool>>,
    entities_key: ResourceKey<Vec<NonNull<Entity>>>,
}

impl StandardEntitiesCollector {
    /// Constructs a new standard entities collector with [`ResourceKey`]
    /// defining where to store the collected entities.
    /// Entity culling and distance sorting by is enabled by default.
    pub fn new(
        entities_key: ResourceKey<Vec<NonNull<Entity>>>,
        enable_culling_key: Option<ResourceKey<bool>>,
        enable_sorting_key: Option<ResourceKey<bool>>,
    ) -> Self {
        Self {
            entities_key,
            enable_culling_key,
            enable_sorting_key,
        }
    }

    /// Returns `true` if entity culling enabled.
    pub fn culling_enabled(&self, resources: &Resources) -> bool {
        self.enable_culling_key
            .as_ref()
            .and_then(|key| resources.get(key))
            .copied()
            .unwrap_or(DEFAULT_CULLING_ENABLED)
    }

    /// Returns `true` if entity distance sorting enabled.
    pub fn distance_sorting_enabled(&self, resources: &Resources) -> bool {
        self.enable_sorting_key
            .as_ref()
            .and_then(|key| resources.get(key))
            .copied()
            .unwrap_or(DEFAULT_SORTING_ENABLED)
    }
}

impl Executor for StandardEntitiesCollector {
    type Error = Error;

    fn execute(
        &mut self,
        state: &mut State,
        scene: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        struct SortEntity {
            entity: NonNull<Entity>,
            /// Depth distance from sorting entities, from nearest to farthest
            distance: f64,
        }

        let culling_enabled = self.culling_enabled(resources);
        let sorting_enabled = self.distance_sorting_enabled(resources);

        let view_position = state.camera().position();
        let view_frustum = state.camera().view_frustum();
        let mut entities = Vec::new();

        // entities collections waits for collecting. If parent model does not changed, set matrix to None.
        unsafe {
            let mut collections = VecDeque::from([scene.entity_collection_mut()]);
            while let Some(collection) = collections.pop_front() {
                let collection = &mut *collection;
                collection.update();

                // culls collection bounding
                if culling_enabled {
                    if let Some(collection_bounding) = collection.bounding() {
                        if let Culling::Outside(_) = collection_bounding.cull(&view_frustum) {
                            continue;
                        }
                    }
                }

                // travels each entity
                for entity in collection.entities_mut() {
                    entity.update();

                    let distance = if entity.material().and_then(|m| m.instanced()).is_some() {
                        // never apply culling to an instanced material
                        f64::INFINITY
                    } else if culling_enabled {
                        match entity.bounding() {
                            Some(bounding) => {
                                match bounding.cull(&view_frustum) {
                                    Culling::Inside { near, .. }
                                    | Culling::Intersect { near, .. } => near,
                                    Culling::Outside(_) => continue, // filters entity outside frustum
                                }
                            }
                            None => f64::INFINITY, // returns infinity for an entity without bounding
                        }
                    } else if sorting_enabled {
                        match entity.bounding() {
                            // returns distance between bounding center and camera position if having a bounding volume
                            Some(bounding) => bounding.center().distance(&view_position),
                            None => f64::INFINITY,
                        }
                    } else {
                        f64::INFINITY
                    };

                    entities.push(SortEntity {
                        entity: NonNull::new_unchecked(entity),
                        distance,
                    })
                }

                // adds child collections to list
                collections.extend(collection.collections_mut().iter_mut());
            }
        }

        if sorting_enabled {
            entities.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        }

        let entities = entities
            .into_iter()
            .map(|entity| entity.entity)
            .collect::<Vec<_>>();

        resources.insert(self.entities_key.clone(), entities);

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
