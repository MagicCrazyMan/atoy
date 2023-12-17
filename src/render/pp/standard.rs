use std::collections::VecDeque;

use gl_matrix4rust::vec3::AsVec3;
use log::warn;
use web_sys::WebGl2RenderingContext;

use crate::{
    bounding::Culling,
    entity::{BorrowedMut, Strong},
    geometry::Geometry,
    material::{Material, Transparency},
    render::{
        pp::ItemKey,
        webgl::{
            attribute::{bind_attributes, unbind_attributes},
            draw::draw,
            error::Error,
            program::ProgramItem,
            uniform::bind_uniforms,
        },
    },
};

use super::{
    outlining::Outlining, picking::Picking, Executor, Pipeline, ResourceKey, Resources, State,
    Stuff,
};

pub fn create_standard_pipeline(window_position: ResourceKey) -> Pipeline {
    let collector = ItemKey::from_uuid();
    let picking = ItemKey::from_uuid();
    let outlining = ItemKey::from_uuid();
    let drawer = ItemKey::from_uuid();

    let collected_entities = ResourceKey::runtime_uuid();
    let picked_entity = ResourceKey::runtime_uuid();
    let picked_position = ResourceKey::runtime_uuid();

    let mut pipeline = Pipeline::new();
    pipeline.add_executor(
        collector.clone(),
        StandardEntitiesCollector::new(collected_entities.clone()),
    );
    pipeline.add_executor(
        picking.clone(),
        Picking::new(
            window_position,
            collected_entities.clone(),
            picked_entity.clone(),
            picked_position.clone(),
        ),
    );
    pipeline.add_executor(outlining.clone(), Outlining::new(picked_entity));
    pipeline.add_executor(drawer.clone(), StandardDrawer::new(collected_entities));

    // safely unwraps
    pipeline.connect(&collector, &picking).unwrap();
    pipeline.connect(&collector, &drawer).unwrap();
    pipeline.connect(&picking, &outlining).unwrap();
    pipeline.connect(&outlining, &drawer).unwrap();

    pipeline
}

/// Standard drawer, draws all entities with its own material and geometry.
///
/// # Get Resources & Data Type
/// - `get_entities`: [`Vec<Strong>`], a list contains entities to draw.
pub struct StandardDrawer {
    get_entities: ResourceKey,
    last_program: Option<ProgramItem>,
}

impl StandardDrawer {
    pub fn new(get_entities: ResourceKey) -> Self {
        Self {
            get_entities,
            last_program: None,
        }
    }

    fn draw(
        &mut self,
        state: &mut State,
        stuff: &dyn Stuff,
        entity: BorrowedMut,
        geometry: *mut dyn Geometry,
        material: *mut dyn Material,
    ) -> Result<(), Error> {
        unsafe {
            // compile and bind program only when last program isn't equals the material
            if self
                .last_program
                .as_ref()
                .map(|last_program| last_program.name() != (*material).name())
                .unwrap_or(true)
            {
                let item = state.program_store.use_program(&*material)?;
                state.gl.use_program(Some(item.gl_program()));
                self.last_program = Some(item.clone());
            }

            let program = self.last_program.as_ref().unwrap();

            // binds attributes
            let items = bind_attributes(state, &entity, &*geometry, &*material, program);
            // binds uniforms
            bind_uniforms(state, stuff, &entity, &*geometry, &*material, program);

            // before draw of material and geometry
            (&mut *material).before_draw(state, &entity);
            (&mut *geometry).before_draw(state, &entity);
            // draws
            draw(state, &*geometry, &*material);
            // after draw of material and geometry
            (&mut *material).after_draw(state, &entity);
            (&mut *geometry).after_draw(state, &entity);

            unbind_attributes(state, items);
        }

        Ok(())
    }
}

impl Executor for StandardDrawer {
    fn before(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<bool, Error> {
        if !resources.contains_key(&self.get_entities) {
            return Ok(false);
        }

        state.gl.viewport(
            0,
            0,
            state.canvas.width() as i32,
            state.canvas.height() as i32,
        );
        state.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl.enable(WebGl2RenderingContext::BLEND);
        state.gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        Ok(true)
    }

    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<(), Error> {
        let Some(entities) = resources.get_downcast_ref::<Vec<Strong>>(&self.get_entities) else {
            return Ok(());
        };

        // splits opaques and translucents
        let mut opaques = Vec::new();
        let mut translucents = Vec::new();
        let state_ptr: *const State = state;
        entities.iter().for_each(|entity| unsafe {
            let mut entity = entity.borrow_mut();

            // prepare material and geometry if exists
            if let Some(geometry) = entity.geometry_raw() {
                (*geometry).prepare(&*state_ptr, &entity);
            };
            if let Some(material) = entity.material_raw() {
                (*material).prepare(&*state_ptr, &entity);
            };

            if let (Some(geometry), Some(material)) = (entity.geometry_raw(), entity.material_raw())
            {
                // filters unready material
                if !(*material).ready() {
                    return;
                }

                // filters transparent material
                if (*material).transparency() == Transparency::Transparent {
                    return;
                }

                if (*material).transparency() == Transparency::Opaque {
                    opaques.push((entity, geometry, material));
                } else {
                    translucents.push((entity, geometry, material));
                }
            }
        });

        // draws opaque enable DEPTH_TEST and disable BLEND and draws them from nearest to farthest first
        state.gl.disable(WebGl2RenderingContext::BLEND);
        state.gl.depth_mask(true);
        for (entity, geometry, material) in opaques {
            self.draw(state, stuff, entity, geometry, material)?;
        }

        // then draws translucents first with DEPTH_TEST unchangeable and enable BLEND and draws theme from farthest to nearest
        state.gl.enable(WebGl2RenderingContext::BLEND);
        state.gl.blend_equation(WebGl2RenderingContext::FUNC_ADD);
        state.gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        state.gl.depth_mask(false);
        for (entity, geometry, material) in translucents.into_iter().rev() {
            self.draw(state, stuff, entity, geometry, material)?;
        }

        self.last_program = None;

        Ok(())
    }
}

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
    set_entities: ResourceKey,
}

impl StandardEntitiesCollector {
    /// Constructs a new standard entities collector with [`ResourceKey`]
    /// defining where to store the collected entities.
    /// Entity culling and distance sorting by is enabled by default.
    pub fn new(set_entities: ResourceKey) -> Self {
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
