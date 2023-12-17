use std::collections::VecDeque;

use gl_matrix4rust::vec3::AsVec3;
use log::warn;

use crate::{
    bounding::Culling,
    entity::Strong,
    render::pp::{error::Error, Executor, ResourceKey, Resources, State, Stuff},
};

/// Standard entities collector, collects and flatten entities from entities collection of [`Stuff`].
///
/// During collecting procedure, works list below will be done:
/// - Calculates model matrix for each entity collection and entity.
/// - Culls entity which has a bounding volume and it is outside the view frustum.
/// Entity which has no bounding volume will append to the last of the entity list.
///
/// # Provides Resources & Data Type
/// - `set_entities`: [`Vec<Strong>`], a list contains entities collected by this collector.
pub struct StandardEntitiesCollector {
    enable_culling: bool,
    enable_sorting: bool,
    set_entities: ResourceKey<Vec<Strong>>,
}

impl StandardEntitiesCollector {
    /// Constructs a new standard entities collector with [`ResourceKey`]
    /// defining where to store the collected entities.
    /// Entity culling and distance sorting by is enabled by default.
    pub fn new(set_entities: ResourceKey<Vec<Strong>>) -> Self {
        Self {
            enable_culling: true,
            enable_sorting: true,
            set_entities,
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
    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<(), Error> {
        struct SortEntity {
            entity: Strong,
            /// Depth distance from sorting entities, from nearest to farthest
            distance: f64,
        }

        stuff.camera_mut().update_frame(state);
        let view_position = stuff.camera().position();
        let view_frustum = stuff.camera().view_frustum();
        let mut entities = Vec::new();

        // entities collections waits for collecting. If parent model does not changed, set matrix to None.
        let mut collections = VecDeque::from([(None, stuff.entity_collection_mut())]);
        while let Some((parent_model_matrix, collection)) = collections.pop_front() {
            // update frame for collection
            let mut collection_model_matrix = None;
            if collection.update_frame(parent_model_matrix) {
                collection_model_matrix = Some(*collection.model_matrix());
            }

            // travels each entity
            for entity in collection.entities_mut() {
                let mut entity_mut = entity.borrow_mut();

                // update entity frame
                if let Err(err) = entity_mut.update_frame(collection_model_matrix) {
                    warn!(
                        target: "StandardEntitiesCollector",
                        "entity {} update frame failed: {}, entity ignored", entity_mut.id(), err
                    );
                    continue;
                }

                let distance = if self.enable_culling {
                    match entity_mut.bounding_volume_mut() {
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
                } else {
                    match entity_mut.bounding_volume() {
                        // returns distance between bounding center and camera position if having a bounding volume
                        Some(bounding) => bounding.center().distance(&view_position),
                        None => f64::INFINITY,
                    }
                };

                entities.push(SortEntity {
                    entity: entity.strong(),
                    distance,
                })
            }

            // adds child collections to list
            collections.extend(
                collection
                    .collections_mut()
                    .iter_mut()
                    .map(|collection| (collection_model_matrix, collection)),
            );
        }

        if self.enable_sorting {
            entities.sort_by(|a, b| a.distance.total_cmp(&b.distance));
        }

        // console_log!("{}", entities.iter().map(|e| e.distance.to_string()).collect::<Vec<_>>().join(", "));
        // console_log!("entities count {}", entities.len());

        let entities = entities
            .into_iter()
            .map(|entity| entity.entity)
            .collect::<Vec<_>>();

        resources.insert(self.set_entities.clone(), entities);

        Ok(())
    }
}
