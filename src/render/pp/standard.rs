use std::{
    any::Any,
    collections::{HashMap, VecDeque},
};

use wasm_bindgen_test::console_log;
use web_sys::WebGl2RenderingContext;

use crate::{
    bounding::Culling,
    camera::Camera,
    entity::{collection::EntityCollection, Strong},
    render::webgl::{
        draw::{bind_attributes, bind_uniforms, draw},
        error::Error,
        program::Program,
    },
    scene::Scene,
};

use super::{
    outlining::Outlining, picking::Picking, Executor, Pipeline, ResourceSource, State, Stuff,
};

pub fn create_standard_pipeline(position_key: impl Into<String>) -> Pipeline {
    let mut pipeline = Pipeline::new();
    pipeline.add_executor("__setup", StandardSetup);
    pipeline.add_executor("__update_camera", UpdateCameraFrame);
    pipeline.add_executor(
        "__collector",
        StandardEntitiesCollector::new(ResourceSource::runtime("entities")),
    );
    pipeline.add_executor(
        "__picking",
        Picking::new(
            ResourceSource::persist(position_key),
            ResourceSource::runtime("entities"),
            ResourceSource::runtime("picked"),
        ),
    );
    pipeline.add_executor(
        "__outlining",
        Outlining::new(ResourceSource::runtime("picked")),
    );
    pipeline.add_executor(
        "__drawer",
        StandardDrawer::new(ResourceSource::runtime("entities")),
    );
    pipeline.add_executor("__reset", ResetWebGLState);

    // safely unwraps
    pipeline.connect("__setup", "__update_camera").unwrap();
    pipeline.connect("__update_camera", "__collector").unwrap();
    pipeline.connect("__collector", "__picking").unwrap();
    pipeline.connect("__collector", "__drawer").unwrap();
    pipeline.connect("__picking", "__outlining").unwrap();
    pipeline.connect("__outlining", "__drawer").unwrap();
    pipeline.connect("__drawer", "__reset").unwrap();

    pipeline
}

/// Standard stuff provides [`Stuff`] data from [`Scene`].
pub struct StandardStuff<'a> {
    scene: &'a mut Scene,
}

impl<'a> StandardStuff<'a> {
    pub fn new(scene: &'a mut Scene) -> Self {
        Self { scene }
    }
}

impl<'a> Stuff for StandardStuff<'a> {
    fn camera(&self) -> &dyn Camera {
        self.scene.active_camera()
    }

    fn camera_mut(&mut self) -> &mut dyn Camera {
        self.scene.active_camera_mut()
    }

    fn entity_collection(&self) -> &EntityCollection {
        self.scene.entity_collection()
    }

    fn entity_collection_mut(&mut self) -> &mut EntityCollection {
        self.scene.entity_collection_mut()
    }
}

/// Standard drawer, draws all entities with its own material and geometry.
///
/// # Get Resources & Data Type
/// - `entities`: [`Vec<Strong>`], a list contains entities to draw.
pub struct StandardDrawer {
    entities: ResourceSource,
}

impl StandardDrawer {
    pub fn new(entities: ResourceSource) -> Self {
        Self { entities }
    }
}

impl Executor for StandardDrawer {
    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        runtime_resources: &mut HashMap<String, Box<dyn Any>>,
        persist_resources: &mut HashMap<String, Box<dyn Any>>,
    ) -> Result<(), Error> {
        let entities = match &self.entities {
            ResourceSource::Runtime(key) => runtime_resources.get(key.as_str()),
            ResourceSource::Persist(key) => persist_resources.get(key.as_str()),
        };
        let Some(entities) = entities.and_then(|resource| resource.downcast_ref::<Vec<Strong>>())
        else {
            return Ok(());
        };

        state.gl.enable(WebGl2RenderingContext::DEPTH_TEST);

        let mut last_program = None as Option<Program>;
        for entity in entities.iter() {
            unsafe {
                let mut entity = entity.borrow_mut();
                // prepare material and geometry if exists
                if let Some(geometry) = entity.geometry_raw() {
                    (*geometry).prepare(state, &entity);
                };
                if let Some(material) = entity.material_raw() {
                    (*material).prepare(state, &entity);
                };

                let (Some(geometry), Some(material)) =
                    (entity.geometry_raw(), entity.material_raw())
                else {
                    continue;
                };

                if !(*material).ready() {
                    continue;
                }

                // compile and bind program only when last program isn't equals the material
                if last_program
                    .as_ref()
                    .map(|last_program| last_program.name() != (&*material).name())
                    .unwrap_or(true)
                {
                    let p = state.program_store.use_program(&*material)?;
                    state.gl.use_program(Some(p.gl_program()));
                    last_program = Some(p.clone());
                }

                let program = last_program.as_ref().unwrap();

                // binds attributes
                bind_attributes(
                    state,
                    &entity,
                    &*geometry,
                    &*material,
                    program.attribute_locations(),
                );
                // binds uniforms
                bind_uniforms(
                    state,
                    stuff,
                    &entity,
                    &*geometry,
                    &*material,
                    program.uniform_locations(),
                );

                // before draw of material and geometry
                (&mut *material).before_draw(state, &entity);
                (&mut *geometry).before_draw(state, &entity);
                // draws
                draw(state, &*geometry, &*material);
                // after draw of material and geometry
                (&mut *material).after_draw(state, &entity);
                (&mut *geometry).after_draw(state, &entity);
            }
        }

        Ok(())
    }
}

/// Standard entities collector, collects and flatten entities from entities collection of [`Stuff`].
///
/// During collecting procedure, works list below will be done:
/// - Calculates model matrix for each entity.
/// - Culls entities which has bounding volume and it is outside the viewing frustum.
/// Entities which has no bounding volume will append to the last of the entity list.
///
/// # Provides Resources & Data Type
/// - `entities`: [`Vec<Strong>`], a list contains entities collected by this collector.
pub struct StandardEntitiesCollector {
    entities: ResourceSource,
}

impl StandardEntitiesCollector {
    pub fn new(entities: ResourceSource) -> Self {
        Self { entities }
    }
}

impl Executor for StandardEntitiesCollector {
    fn execute(
        &mut self,
        _: &mut State,
        stuff: &mut dyn Stuff,
        runtime_resources: &mut HashMap<String, Box<dyn Any>>,
        persist_resources: &mut HashMap<String, Box<dyn Any>>,
    ) -> Result<(), Error> {
        struct FilteringEntity {
            entity: Strong,
            /// Depth distance from bounding to camera
            distance: f64,
        }

        let viewing_frustum = stuff.camera().viewing_frustum();

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
                // update matrices
                if let Err(err) = entity.borrow_mut().update_frame(collection_model_matrix) {
                    // should log warning
                    console_log!("{}", err);
                    continue;
                }

                // collects to different container depending on whether having a bounding
                let distance = match entity.borrow_mut().bounding_volume() {
                    Some(bounding) => {
                        match bounding.cull(&viewing_frustum) {
                            // filters every entity outside frustum
                            Culling::Outside(_) => continue,
                            Culling::Inside { near, .. } | Culling::Intersect { near, .. } => near,
                        }
                    }
                    None => f64::INFINITY, // returns infinity for a non bounding entity
                };

                entities.push(FilteringEntity {
                    entity: entity.strong(),
                    distance,
                })
            }

            // adds sub-collections to list
            collections.extend(
                collection
                    .collections_mut()
                    .iter_mut()
                    .map(|collection| (collection_model_matrix, collection)),
            );
        }

        // do simple sorting for bounding entities, from nearest(smallest distance) to farthest(greatest distance)
        entities.sort_by(|a, b| a.distance.total_cmp(&b.distance));

        // console_log!("{}", bounding_entities.iter().map(|e| e.distance.to_string()).collect::<Vec<_>>().join(", "));
        // console_log!("entities count {}", entities.len());

        let entities = Box::new(
            entities
                .into_iter()
                .map(|entity| entity.entity)
                .collect::<Vec<_>>(),
        );
        match &self.entities {
            ResourceSource::Runtime(key) => runtime_resources.insert(key.clone(), entities),
            ResourceSource::Persist(key) => persist_resources.insert(key.clone(), entities),
        };

        Ok(())
    }
}

/// Executor update camera by current frame.
pub struct UpdateCameraFrame;

impl Executor for UpdateCameraFrame {
    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        _: &mut HashMap<String, Box<dyn Any>>,
        _: &mut HashMap<String, Box<dyn Any>>,
    ) -> Result<(), Error> {
        stuff.camera_mut().update_frame(state);
        Ok(())
    }
}

/// Executor setup to default status.
pub struct StandardSetup;

impl Executor for StandardSetup {
    fn execute(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        _: &mut HashMap<String, Box<dyn Any>>,
        _: &mut HashMap<String, Box<dyn Any>>,
    ) -> Result<(), Error> {
        state.gl.viewport(
            0,
            0,
            state.canvas.width() as i32,
            state.canvas.height() as i32,
        );
        state.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl.enable(WebGl2RenderingContext::BLEND);
        state.gl.enable(WebGl2RenderingContext::CULL_FACE);
        state.gl.cull_face(WebGl2RenderingContext::BACK);
        state.gl.blend_equation(WebGl2RenderingContext::FUNC_ADD);
        state.gl.blend_func(WebGl2RenderingContext::SRC_ALPHA, WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA);
        state.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl.clear_depth(1.0);
        state.gl.clear_stencil(0);
        state.gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT
                | WebGl2RenderingContext::DEPTH_BUFFER_BIT
                | WebGl2RenderingContext::STENCIL_BUFFER_BIT,
        );
        
        Ok(())
    }
}

/// Executor resets [`WebGl2RenderingContext`] to default state.
pub struct ResetWebGLState;

impl Executor for ResetWebGLState {
    fn execute(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        _: &mut HashMap<String, Box<dyn Any>>,
        _: &mut HashMap<String, Box<dyn Any>>,
    ) -> Result<(), Error> {
        state.gl.use_program(None);
        state
            .gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        state
            .gl
            .bind_framebuffer(WebGl2RenderingContext::DRAW_FRAMEBUFFER, None);
        state
            .gl
            .bind_framebuffer(WebGl2RenderingContext::READ_FRAMEBUFFER, None);
        state
            .gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
        state
            .gl
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        state
            .gl
            .bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);
        state
            .gl
            .bind_buffer(WebGl2RenderingContext::COPY_READ_BUFFER, None);
        state
            .gl
            .bind_buffer(WebGl2RenderingContext::COPY_WRITE_BUFFER, None);
        state
            .gl
            .bind_buffer(WebGl2RenderingContext::TRANSFORM_FEEDBACK_BUFFER, None);
        state
            .gl
            .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, None);
        state
            .gl
            .bind_buffer(WebGl2RenderingContext::PIXEL_PACK_BUFFER, None);
        state
            .gl
            .bind_buffer(WebGl2RenderingContext::PIXEL_UNPACK_BUFFER, None);
        for index in 0..32 {
            state
                .gl
                .active_texture(WebGl2RenderingContext::TEXTURE0 + index);
            state
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            state
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_CUBE_MAP, None);
        }
        state.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        state.gl.bind_vertex_array(None);
        state.gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl.disable(WebGl2RenderingContext::CULL_FACE);
        state.gl.disable(WebGl2RenderingContext::BLEND);
        state.gl.disable(WebGl2RenderingContext::DITHER);
        state
            .gl
            .disable(WebGl2RenderingContext::POLYGON_OFFSET_FILL);
        state
            .gl
            .disable(WebGl2RenderingContext::SAMPLE_ALPHA_TO_COVERAGE);
        state.gl.disable(WebGl2RenderingContext::SAMPLE_COVERAGE);
        state.gl.disable(WebGl2RenderingContext::SCISSOR_TEST);
        state.gl.disable(WebGl2RenderingContext::STENCIL_TEST);
        state.gl.disable(WebGl2RenderingContext::RASTERIZER_DISCARD);

        state.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl.clear_depth(0.0);
        state.gl.clear_stencil(0);
        state.gl.stencil_func(WebGl2RenderingContext::ALWAYS, 0, 1);
        state.gl.stencil_mask(1);
        state.gl.stencil_op(
            WebGl2RenderingContext::KEEP,
            WebGl2RenderingContext::KEEP,
            WebGl2RenderingContext::KEEP,
        );

        Ok(())
    }
}
