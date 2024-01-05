use std::{collections::VecDeque, ptr::NonNull};

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
    enable_culling: bool,
    enable_sorting: bool,
    out_entities: ResourceKey<Vec<NonNull<Entity>>>,
}

impl StandardEntitiesCollector {
    /// Constructs a new standard entities collector with [`ResourceKey`]
    /// defining where to store the collected entities.
    /// Entity culling and distance sorting by is enabled by default.
    pub fn new(out_entities: ResourceKey<Vec<NonNull<Entity>>>) -> Self {
        Self {
            enable_culling: true,
            enable_sorting: true,
            out_entities,
        }
    }

    /// Disable entity culling.
    pub fn disable_culling(&mut self) {
        self.enable_culling = false;
    }

    /// Enable entity culling.
    pub fn enable_culling(&mut self) {
        self.enable_culling = true;
    }

    /// Is entity culling enabled.
    pub fn is_culling_enabled(&mut self) -> bool {
        self.enable_culling
    }

    /// Disable distance sorting.
    /// If disabled, the orderings of the entities are not guaranteed.
    pub fn disable_distance_sorting(&mut self) {
        self.enable_sorting = false;
    }

    /// Enable distance sorting.
    /// If enabled, entities are sorted from the nearest to the farthest.
    pub fn enable_distance_sorting(&mut self) {
        self.enable_sorting = true;
    }

    /// Is entity distance sorting enabled.
    pub fn is_distance_sorting_enabled(&mut self) -> bool {
        self.enable_sorting
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

        let view_position = state.camera().position();
        let view_frustum = state.camera().view_frustum();
        let mut entities = Vec::new();

        // entities collections waits for collecting. If parent model does not changed, set matrix to None.
        let mut collections = VecDeque::from([scene.entity_collection_mut()]);
        while let Some(collection) = collections.pop_front() {
            collection.update();

            // culls collection bounding
            if self.enable_culling {
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
                } else if self.enable_culling {
                    match entity.bounding() {
                        Some(bounding) => {
                            match bounding.cull(&view_frustum) {
                                Culling::Inside { near, .. } | Culling::Intersect { near, .. } => {
                                    near
                                }
                                Culling::Outside(_) => continue, // filters entity outside frustum
                            }
                        }
                        None => f64::INFINITY, // returns infinity for an entity without bounding
                    }
                } else if self.enable_sorting {
                    match entity.bounding() {
                        // returns distance between bounding center and camera position if having a bounding volume
                        Some(bounding) => bounding.center().distance(&view_position),
                        None => f64::INFINITY,
                    }
                } else {
                    f64::INFINITY
                };

                entities.push(SortEntity {
                    entity: unsafe { NonNull::new_unchecked(entity) },
                    distance,
                })
            }

            // adds child collections to list
            collections.extend(
                collection
                    .collections_mut()
                    .iter_mut()
                    .map(|collection| collection),
            );
        }

        if self.enable_sorting {
            entities.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        }

        let entities = entities
            .into_iter()
            .map(|entity| entity.entity)
            .collect::<Vec<_>>();

        resources.insert(self.out_entities.clone(), entities);

        Ok(())
    }
}
