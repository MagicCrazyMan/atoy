use std::{
    any::Any,
    collections::{HashMap, VecDeque},
};

use gl_matrix4rust::{mat4::AsMat4, vec3::AsVec3};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::console_log;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlUniformLocation};

use crate::{
    bounding::Culling,
    camera::Camera,
    entity::{collection::EntityCollection, BorrowedMut, Strong},
    geometry::Geometry,
    material::Material,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        buffer::BufferTarget,
        conversion::{GLint, GLuint, ToGlEnum},
        draw::Draw,
        error::Error,
        program::Program,
        uniform::{UniformBinding, UniformValue},
    },
    scene::Scene,
};

use super::{Executor, Pipeline, State, Stuff};

pub fn create_standard_pipeline() -> Pipeline {
    let mut pipeline = Pipeline::new();
    pipeline.add_executor(UpdateCameraFrame::new("update_camera"));
    pipeline.add_executor(StandardEntitiesCollector::new(
        "entities_collector",
        "input_entities",
    ));
    pipeline.add_executor(StandardDrawer::new("drawer", "input_entities"));
    pipeline.add_executor(ResetWebGLState::new("reset"));

    // safely unwraps
    pipeline.connect("update_camera", "entities_collector").unwrap();
    pipeline.connect("entities_collector", "drawer").unwrap();
    pipeline.connect("drawer", "reset").unwrap();

    pipeline
}

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

pub struct StandardDrawer {
    name: String,
    resource_name: String,
}

impl StandardDrawer {
    pub fn new(name: impl Into<String>, resource_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            resource_name: resource_name.into(),
        }
    }

    /// Binds attributes of the entity.
    unsafe fn bind_attributes(
        &mut self,
        state: &mut State,
        entity: &BorrowedMut,
        geometry: *const dyn Geometry,
        material: *const dyn Material,
        attribute_locations: &HashMap<AttributeBinding, GLuint>,
    ) {
        for (binding, location) in attribute_locations {
            let value = match binding {
                AttributeBinding::GeometryPosition => (*geometry).vertices(),
                AttributeBinding::GeometryTextureCoordinate => (*geometry).texture_coordinates(),
                AttributeBinding::GeometryNormal => (*geometry).normals(),
                AttributeBinding::FromGeometry(name) => (*geometry).attribute_value(name, entity),
                AttributeBinding::FromMaterial(name) => (*material).attribute_value(name, entity),
                AttributeBinding::FromEntity(name) => entity.attribute_values().get(*name).cloned(),
            };
            let Some(value) = value else {
                // should log warning
                console_log!("3");
                continue;
            };

            match value {
                AttributeValue::Buffer {
                    descriptor,
                    target,
                    component_size,
                    data_type,
                    normalized,
                    bytes_stride,
                    bytes_offset,
                } => {
                    let buffer = match state.buffer_store.use_buffer(descriptor, target) {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log error
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    state.gl.bind_buffer(target.gl_enum(), Some(&buffer));
                    state.gl.vertex_attrib_pointer_with_i32(
                        *location,
                        component_size as GLint,
                        data_type.gl_enum(),
                        normalized,
                        bytes_stride,
                        bytes_offset,
                    );
                    state.gl.enable_vertex_attrib_array(*location);
                }
                AttributeValue::InstancedBuffer {
                    descriptor,
                    target,
                    component_size,
                    data_type,
                    normalized,
                    component_count_per_instance: components_length_per_instance,
                    divisor,
                } => {
                    let buffer = match state.buffer_store.use_buffer(descriptor, target) {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log error
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    state.gl.bind_buffer(target.gl_enum(), Some(&buffer));

                    let component_size = component_size as GLint;
                    // binds each instance
                    for i in 0..components_length_per_instance {
                        let offset_location = *location + (i as GLuint);
                        state.gl.vertex_attrib_pointer_with_i32(
                            offset_location,
                            component_size,
                            data_type.gl_enum(),
                            normalized,
                            data_type.bytes_length()
                                * component_size
                                * components_length_per_instance,
                            i * data_type.bytes_length() * component_size,
                        );
                        state.gl.enable_vertex_attrib_array(offset_location);
                        state.gl.vertex_attrib_divisor(offset_location, divisor);
                    }
                }
                AttributeValue::Vertex1f(x) => state.gl.vertex_attrib1f(*location, x),
                AttributeValue::Vertex2f(x, y) => state.gl.vertex_attrib2f(*location, x, y),
                AttributeValue::Vertex3f(x, y, z) => state.gl.vertex_attrib3f(*location, x, y, z),
                AttributeValue::Vertex4f(x, y, z, w) => {
                    state.gl.vertex_attrib4f(*location, x, y, z, w)
                }
                AttributeValue::Vertex1fv(v) => {
                    state.gl.vertex_attrib1fv_with_f32_array(*location, &v)
                }
                AttributeValue::Vertex2fv(v) => {
                    state.gl.vertex_attrib2fv_with_f32_array(*location, &v)
                }
                AttributeValue::Vertex3fv(v) => {
                    state.gl.vertex_attrib3fv_with_f32_array(*location, &v)
                }
                AttributeValue::Vertex4fv(v) => {
                    state.gl.vertex_attrib4fv_with_f32_array(*location, &v)
                }
            };
        }
    }

    /// Binds uniform data of the entity.
    unsafe fn bind_uniforms(
        &mut self,
        state: &mut State,
        stuff: &dyn Stuff,
        entity: &BorrowedMut,
        geometry: *const dyn Geometry,
        material: *const dyn Material,
        uniform_locations: &HashMap<UniformBinding, WebGlUniformLocation>,
    ) {
        for (binding, location) in uniform_locations {
            let value = match binding {
                UniformBinding::FromGeometry(name) => (*geometry).uniform_value(name, entity),
                UniformBinding::FromMaterial(name) => (*material).uniform_value(name, entity),
                UniformBinding::FromEntity(name) => entity.uniform_values().get(*name).cloned(),
                UniformBinding::ModelMatrix
                | UniformBinding::ViewMatrix
                | UniformBinding::ProjMatrix
                | UniformBinding::NormalMatrix
                | UniformBinding::ViewProjMatrix => {
                    let mat = match binding {
                        UniformBinding::ModelMatrix => entity.model_matrix().to_gl(),
                        UniformBinding::NormalMatrix => entity.normal_matrix().to_gl(),
                        UniformBinding::ViewMatrix => stuff.camera().view_matrix().to_gl(),
                        UniformBinding::ProjMatrix => stuff.camera().proj_matrix().to_gl(),
                        UniformBinding::ViewProjMatrix => stuff.camera().view_proj_matrix().to_gl(),
                        _ => unreachable!(),
                    };

                    Some(UniformValue::Matrix4 {
                        data: mat,
                        transpose: false,
                    })
                }
                UniformBinding::ActiveCameraPosition | UniformBinding::ActiveCameraCenter => {
                    let vec = match binding {
                        UniformBinding::ActiveCameraPosition => stuff.camera().position().to_gl(),
                        UniformBinding::ActiveCameraCenter => stuff.camera().center().to_gl(),
                        _ => unreachable!(),
                    };

                    Some(UniformValue::FloatVector3(vec))
                }
                UniformBinding::CanvasSize => state
                    .gl
                    .canvas()
                    .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
                    .map(|canvas| {
                        UniformValue::UnsignedIntegerVector2([canvas.width(), canvas.height()])
                    }),
            };
            let Some(value) = value else {
                // should log warning
                continue;
            };

            match value {
                UniformValue::UnsignedInteger1(x) => state.gl.uniform1ui(Some(location), x),
                UniformValue::UnsignedInteger2(x, y) => state.gl.uniform2ui(Some(location), x, y),
                UniformValue::UnsignedInteger3(x, y, z) => {
                    state.gl.uniform3ui(Some(location), x, y, z)
                }
                UniformValue::UnsignedInteger4(x, y, z, w) => {
                    state.gl.uniform4ui(Some(location), x, y, z, w)
                }
                UniformValue::FloatVector1(data) => {
                    state.gl.uniform1fv_with_f32_array(Some(location), &data)
                }
                UniformValue::FloatVector2(data) => {
                    state.gl.uniform2fv_with_f32_array(Some(location), &data)
                }
                UniformValue::FloatVector3(data) => {
                    state.gl.uniform3fv_with_f32_array(Some(location), &data)
                }
                UniformValue::FloatVector4(data) => {
                    state.gl.uniform4fv_with_f32_array(Some(location), &data)
                }
                UniformValue::IntegerVector1(data) => {
                    state.gl.uniform1iv_with_i32_array(Some(location), &data)
                }
                UniformValue::IntegerVector2(data) => {
                    state.gl.uniform2iv_with_i32_array(Some(location), &data)
                }
                UniformValue::IntegerVector3(data) => {
                    state.gl.uniform3iv_with_i32_array(Some(location), &data)
                }
                UniformValue::IntegerVector4(data) => {
                    state.gl.uniform4iv_with_i32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector1(data) => {
                    state.gl.uniform1uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector2(data) => {
                    state.gl.uniform2uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector3(data) => {
                    state.gl.uniform3uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector4(data) => {
                    state.gl.uniform4uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::Matrix2 { data, transpose } => state
                    .gl
                    .uniform_matrix2fv_with_f32_array(Some(location), transpose, &data),
                UniformValue::Matrix3 { data, transpose } => state
                    .gl
                    .uniform_matrix3fv_with_f32_array(Some(location), transpose, &data),
                UniformValue::Matrix4 { data, transpose } => state
                    .gl
                    .uniform_matrix4fv_with_f32_array(Some(location), transpose, &data),
                UniformValue::Texture {
                    descriptor,
                    params,
                    texture_unit,
                } => {
                    // active texture
                    state.gl.active_texture(texture_unit.gl_enum());

                    let (target, texture) = match state.texture_store.use_texture(&descriptor) {
                        Ok(texture) => texture,
                        Err(err) => {
                            // should log warning
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    // binds texture
                    state.gl.bind_texture(target, Some(texture));
                    // setups sampler parameters
                    params
                        .iter()
                        .for_each(|param| param.tex_parameteri(&state.gl, target));
                    // binds to shader
                    state
                        .gl
                        .uniform1i(Some(location), texture_unit.unit_index());
                }
            };
        }
    }

    unsafe fn draw(
        &mut self,
        state: &mut State,
        geometry: *const dyn Geometry,
        material: *const dyn Material,
    ) {
        // draws entity
        if let Some(num_instances) = (*material).instanced() {
            // draw instanced
            match (*geometry).draw() {
                Draw::Arrays {
                    mode,
                    first,
                    count: num_vertices,
                } => state.gl.draw_arrays_instanced(
                    mode.gl_enum(),
                    first,
                    num_vertices,
                    num_instances,
                ),
                Draw::Elements {
                    mode,
                    count: num_vertices,
                    element_type,
                    offset,
                    indices,
                } => {
                    let buffer = match state
                        .buffer_store
                        .use_buffer(indices, BufferTarget::ElementArrayBuffer)
                    {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log warning
                            console_log!("{}", err);
                            return;
                        }
                    };

                    state
                        .gl
                        .bind_buffer(BufferTarget::ElementArrayBuffer.gl_enum(), Some(&buffer));
                    state.gl.draw_elements_instanced_with_i32(
                        mode.gl_enum(),
                        num_vertices,
                        element_type.gl_enum(),
                        offset,
                        num_instances,
                    );
                }
            }
        } else {
            // draw normally!
            match (*geometry).draw() {
                Draw::Arrays {
                    mode,
                    first,
                    count: num_vertices,
                } => state.gl.draw_arrays(mode.gl_enum(), first, num_vertices),
                Draw::Elements {
                    mode,
                    count: num_vertices,
                    element_type,
                    offset,
                    indices,
                } => {
                    let buffer = match state
                        .buffer_store
                        .use_buffer(indices, BufferTarget::ElementArrayBuffer)
                    {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log warning
                            console_log!("{}", err);
                            return;
                        }
                    };

                    state
                        .gl
                        .bind_buffer(BufferTarget::ElementArrayBuffer.gl_enum(), Some(&buffer));
                    state.gl.draw_elements_with_i32(
                        mode.gl_enum(),
                        num_vertices,
                        element_type.gl_enum(),
                        offset,
                    );
                }
            }
        }
    }
}

impl Executor for StandardDrawer {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut HashMap<String, Box<dyn Any>>,
    ) -> Result<(), Error> {
        let Some(entities) = resources
            .get(&self.resource_name)
            .and_then(|resource| resource.downcast_ref::<Vec<Strong>>())
        else {
            return Ok(());
        };

        state.gl.viewport(
            0,
            0,
            state.canvas.width() as i32,
            state.canvas.height() as i32,
        );
        state.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl.clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl.clear_depth(1.0);
        state.gl.clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

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
                    state.gl.use_program(Some(p.program()));
                    last_program = Some(p.clone());
                }

                let program = last_program.as_ref().unwrap();

                // binds attributes
                self.bind_attributes(
                    state,
                    &entity,
                    geometry,
                    material,
                    program.attribute_locations(),
                );
                // binds uniforms
                self.bind_uniforms(
                    state,
                    stuff,
                    &entity,
                    geometry,
                    material,
                    program.uniform_locations(),
                );

                // before draw of material and geometry
                (&mut *material).before_draw(state, &entity);
                (&mut *geometry).before_draw(state, &entity);
                // draws
                self.draw(state, geometry, material);
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
pub struct StandardEntitiesCollector {
    name: String,
    resource_name: String,
}

impl StandardEntitiesCollector {
    pub fn new(name: impl Into<String>, resource_name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            resource_name: resource_name.into(),
        }
    }
}

impl Executor for StandardEntitiesCollector {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(
        &mut self,
        _: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut HashMap<String, Box<dyn Any>>,
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

        resources.insert(
            self.resource_name.clone(),
            Box::new(
                entities
                    .into_iter()
                    .map(|entity| entity.entity)
                    .collect::<Vec<_>>(),
            ),
        );

        Ok(())
    }
}

/// Executor update camera by current frame.
pub struct UpdateCameraFrame(String);

impl UpdateCameraFrame {
    pub fn new(name: impl Into<String>) -> Self {
        UpdateCameraFrame(name.into())
    }
}

impl Executor for UpdateCameraFrame {
    fn name(&self) -> &str {
        &self.0
    }

    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        _: &mut HashMap<String, Box<dyn Any>>,
    ) -> Result<(), Error> {
        stuff.camera_mut().update_frame(state);
        Ok(())
    }
}

/// Executor resets [`WebGl2RenderingContext`] to default state.
pub struct ResetWebGLState(String);

impl ResetWebGLState {
    pub fn new(name: impl Into<String>) -> Self {
        ResetWebGLState(name.into())
    }
}

impl Executor for ResetWebGLState {
    fn name(&self) -> &str {
        &self.0
    }

    fn execute(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
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
