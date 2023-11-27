use std::{
    collections::{HashMap, VecDeque},
    marker::PhantomData,
    rc::Rc,
};

use gl_matrix4rust::{
    mat4::{AsMat4, Mat4},
    vec3::AsVec3,
    vec4::Vec4,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::console_log;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

use crate::{
    entity::Entity,
    geometry::Geometry,
    material::{self, Material},
    scene::Scene,
};

use self::{
    attribute::{AttributeBinding, AttributeValue},
    buffer::{BufferStore, BufferTarget},
    conversion::{GLfloat, GLint, GLuint, ToGlEnum},
    draw::{CullFace, Draw},
    error::Error,
    pick::EntityPicker,
    pipeline::{
        policy::{GeometryPolicy, MaterialPolicy},
        RenderPipeline, RenderState, RenderStuff,
    },
    program::ProgramStore,
    texture::TextureStore,
    uniform::{UniformBinding, UniformValue},
};

pub mod attribute;
pub mod buffer;
pub mod conversion;
pub mod draw;
pub mod error;
pub mod pick;
pub mod pipeline;
pub mod program;
pub mod texture;
pub mod uniform;

// #[wasm_bindgen(typescript_custom_section)]
// const WEBGL2_RENDER_OPTIONS_TYPE: &'static str = r#"
// export type WebGL2RenderOptions = WebGLContextAttributes;
// "#;

// #[wasm_bindgen]
// extern "C" {
//     #[wasm_bindgen(typescript_type = "WebGL2RenderOptions")]
//     pub type WebGL2RenderOptionsObject;
// }

pub struct WebGL2Render {
    gl: WebGl2RenderingContext,
    entity_picker: EntityPicker,
    program_store: ProgramStore,
    buffer_store: BufferStore,
    texture_store: TextureStore,
}

impl WebGL2Render {
    // /// Constructs a new WebGL2 render.
    // pub fn new(scene: &Scene) -> Result<WebGL2Render, Error> {
    //     Self::new_inner(scene, None)
    // }

    // /// Constructs a new WebGL2 render.
    // pub fn with_options(
    //     scene: &Scene,
    //     options: WebGL2RenderOptionsObject,
    // ) -> Result<WebGL2Render, Error> {
    //     Self::new_inner(scene, Some(options))
    // }

    // fn new_inner(
    //     scene: &Scene,
    //     options: Option<WebGL2RenderOptionsObject>,
    // ) -> Result<WebGL2Render, Error> {
    //     let gl = Self::gl_context(scene.canvas(), options)?;
    //     let mut render = Self {
    //         program_store: ProgramStore::new(gl.clone()),
    //         buffer_store: BufferStore::with_max_memory(gl.clone(), 2 * 1024 * 1024 * 1024),
    //         // buffer_store: BufferStore::with_max_memory(gl.clone(), 2000),
    //         texture_store: TextureStore::new(gl.clone()),
    //         entity_picker: EntityPicker::new(gl.clone()),
    //         gl,
    //         depth_test: true,
    //         cull_face: None,
    //         clear_depth: 0.0,
    //         clear_color: Vec4::new(),
    //     };

    //     render.set_clear_color(Vec4::new());
    //     render.set_cull_face(None);
    //     render.set_depth_test(true);

    //     Ok(render)
    // }

    // /// Gets WebGl2RenderingContext.
    // fn gl_context(
    //     canvas: &HtmlCanvasElement,
    //     options: Option<WebGL2RenderOptionsObject>,
    // ) -> Result<WebGl2RenderingContext, Error> {
    //     let options = match options {
    //         Some(options) => options.obj,
    //         None => JsValue::UNDEFINED,
    //     };

    //     let gl = canvas
    //         .get_context_with_context_options("webgl2", &options)
    //         .ok()
    //         .and_then(|context| context)
    //         .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
    //         .ok_or(Error::WebGl2RenderingContextNotFound)?;

    //     gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

    //     Ok(gl)
    // }

    // fn new_inner(scene: &Scene, options: Option<&JsValue>) -> Result<WebGL2Render, Error> {
    //     let gl = Self::gl_context(scene.canvas(), options)?;
    //     let mut render = Self {
    //         program_store: ProgramStore::new(gl.clone()),
    //         buffer_store: BufferStore::with_max_memory(gl.clone(), 2 * 1024 * 1024 * 1024),
    //         // buffer_store: BufferStore::with_max_memory(gl.clone(), 2000),
    //         texture_store: TextureStore::new(gl.clone()),
    //         entity_picker: EntityPicker::new(gl.clone()),
    //         gl,
    //         depth_test: true,
    //         cull_face: None,
    //         clear_depth: 0.0,
    //         clear_color: Vec4::new(),
    //     };

    //     render.set_clear_color(Vec4::new());
    //     render.set_cull_face(None);
    //     render.set_depth_test(true);

    //     Ok(render)
    // }

    // /// Gets WebGl2RenderingContext.
    // fn gl_context<'a>(
    //     canvas: &'a HtmlCanvasElement,
    //     options: Option<&'a JsValue>,
    // ) -> Result<WebGl2RenderingContext, Error> {
    //     let gl = canvas
    //         .get_context_with_context_options("webgl2", options.unwrap_or(&JsValue::undefined()))
    //         .ok()
    //         .and_then(|context| context)
    //         .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
    //         .ok_or(Error::WenGL2Unsupported)?;

    //     Ok(gl)
    // }
}

impl WebGL2Render {
    pub fn buffer_store(&self) -> &BufferStore {
        &self.buffer_store
    }

    pub fn buffer_store_mut(&mut self) -> &mut BufferStore {
        &mut self.buffer_store
    }
}

struct RenderGroup<'a> {
    program: *const WebGlProgram,
    attribute_locations: *const HashMap<AttributeBinding, GLuint>,
    uniform_locations: *const HashMap<UniformBinding, WebGlUniformLocation>,
    entities: Vec<RenderingEntityState<'a>>,
}

/// A objects collection for rendering an entity.
/// Including a [`WebGl2RenderingContext`], current rendering [`Scene`],
/// current [`Entity`] and its carrying [`Geometry`] and [`Material`].
pub struct RenderingEntityState<'a> {
    gl: *mut WebGl2RenderingContext,
    scene: *mut Scene,
    entity: *mut Entity,
    geometry: *mut dyn Geometry,
    material: *mut dyn Material,
    _p: PhantomData<&'a ()>,
}

impl<'a> RenderingEntityState<'a> {
    /// Gets [`WebGl2RenderingContext`].
    #[inline]
    pub fn gl(&self) -> &WebGl2RenderingContext {
        unsafe { &*self.gl }
    }

    /// Gets mutable [`Scene`].
    #[inline]
    pub fn scene(&self) -> &mut Scene {
        unsafe { &mut *self.scene }
    }

    /// Gets mutable [`Entity`].
    #[inline]
    pub fn entity(&self) -> &mut Entity {
        unsafe { &mut *self.entity }
    }

    /// Gets mutable [`Geometry`] belongs to this entity.
    #[inline]
    pub fn geometry(&self) -> &mut dyn Geometry {
        unsafe { &mut *self.geometry }
    }

    /// Gets mutable [`Material`] belongs to this entity.
    #[inline]
    pub fn material(&self) -> &mut dyn Material {
        unsafe { &mut *self.material }
    }
}

pub enum GeometryHolder<'a> {
    Borrowed(&'a mut dyn Geometry),
    Owned(Box<dyn Geometry>),
}

pub enum MaterialHolder<'a> {
    Borrowed(&'a mut dyn Material),
    Owned(Box<dyn Material>),
}

impl<'a> MaterialHolder<'a> {
    fn as_mut(&mut self) -> &mut dyn Material {
        match self {
            MaterialHolder::Borrowed(m) => *m,
            MaterialHolder::Owned(m) => m.as_mut(),
        }
    }
}

impl WebGL2Render {
    pub fn render<S, P>(
        &mut self,
        pipeline: &mut P,
        stuff: &mut S,
        frame_time: f64,
    ) -> Result<(), Error>
    where
        S: RenderStuff,
        P: RenderPipeline,
    {
        // prepares stage, obtains a render stuff
        pipeline.prepare(stuff)?;

        // constructs render state
        let mut state = RenderState::new(
            Self::gl(stuff.canvas(), stuff.ctx_options())?,
            frame_time,
            stuff,
        );

        // preprocess stages
        pipeline.pre_process(&mut state)?;

        let view_matrix = state.camera().view_matrix();
        let proj_matrix = state.camera().proj_matrix();
        let material_policy = pipeline.material_policy(&state)?;
        // let geometry_policy = pipeline.geometry_policy(&state)?;

        // render each entities group
        // for (
        //     _,
        //     RenderGroup {
        //         program,
        //         attribute_locations,
        //         uniform_locations,
        //         entities,
        //     },
        // ) in group
        // {
        //     let (program, attribute_locations, uniform_locations) =
        //         unsafe { (&*program, &*attribute_locations, &*uniform_locations) };

        //     // binds program
        //     self.gl.use_program(Some(program));

        //     // render each entity
        //     for state in entities {
        //         // pre-render
        //         self.pre_render(&state);
        //         // binds attributes
        //         self.bind_attributes(attribute_locations, &state);
        //         // binds uniforms
        //         self.bind_uniforms(uniform_locations, &state);
        //         // draws
        //         self.draw(&state);
        //         // post-render
        //         self.post_render(&state);
        //     }
        // }

        self.reset();

        Ok(())
    }

    fn gl(
        canvas: &HtmlCanvasElement,
        options: Option<&JsValue>,
    ) -> Result<WebGl2RenderingContext, Error> {
        let gl = canvas
            .get_context_with_context_options("webgl2", options.unwrap_or(&JsValue::undefined()))
            .ok()
            .and_then(|context| context)
            .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(Error::WenGL2Unsupported)?;

        Ok(gl)
    }

    /// Resets WebGl status.
    fn reset(&self) {}

    /// Prepares graphic scene.
    /// Updates entities matrices using current frame status, collects and groups all entities.
    fn collect_entity<S>(
        &mut self,
        state: &mut RenderState<'_>,
        view_matrix: &Mat4,
        proj_matrix: &Mat4,
        material_policy: &mut MaterialPolicy,
        geometry_policy: &mut GeometryPolicy,
    ) -> Result<HashMap<String, RenderGroup>, Error> {
        let mut group: HashMap<String, RenderGroup> = HashMap::new();

        let mut collections = VecDeque::from([(Mat4::new_identity(), state.entity_collection())]);
        while let Some((parent_model_matrix, collection)) = collections.pop_front() {
            // update collection matrices
            collection.update_frame_matrices(&parent_model_matrix);
            let collection_model_matrix = *collection.model_matrix();

            for entity in collection.entities_mut() {
                // update matrices
                if let Err(err) = entity.update_frame_matrices(
                    &collection_model_matrix,
                    &view_matrix,
                    &proj_matrix,
                ) {
                    // should log warning
                    console_log!("{}", err);
                    continue;
                }

                // selects material
                let mut material = match material_policy {
                    MaterialPolicy::FollowEntity => entity
                        .material_mut()
                        .map(|material| MaterialHolder::Borrowed(material)),
                    MaterialPolicy::Overwrite(material) => material
                        .as_mut()
                        .map(|material| MaterialHolder::Borrowed(material.as_mut())),
                    MaterialPolicy::Custom(func) => {
                        func(entity).map(|material| MaterialHolder::Owned(material))
                    }
                };
                // trigger material preparation
                if let Some(material) = material.as_mut() {
                    // material.as_mut().prepare(state, entity);
                };

                // skip if has no material or not ready yet
                if material
                    .as_mut()
                    .map(|material| material.as_mut().ready())
                    .unwrap_or(false)
                {
                    continue;
                }

                // selects geometry
                let mut geometry = match geometry_policy {
                    GeometryPolicy::FollowEntity => entity
                        .geometry_mut()
                        .map(|geom| GeometryHolder::Borrowed(geom)),
                    GeometryPolicy::Overwrite(geometry) => geometry
                        .as_mut()
                        .map(|geom| GeometryHolder::Borrowed(geom.as_mut())),
                    GeometryPolicy::Custom(func) => {
                        func(entity).map(|geom| GeometryHolder::Owned(geom))
                    }
                };
                // prepare material
            }

            // filters any entity that has no geometry or material
            // groups entities by material to prevent unnecessary program switching
            // if let (Some(geometry), Some(material)) = (geometry, material) {
            // let state = RenderingEntityState {
            //     gl: &mut self.gl,
            //     entity,
            //     geometry,
            //     material,
            //     scene,
            //     _p: PhantomData,
            // };

            // // calls prepare callback
            // state.material().prepare(&state);

            // // check whether material is ready or not
            // if material.ready() {
            //     match group.get_mut(material.name()) {
            //         Some(group) => group.entities.push(state),
            //         None => {
            //             // precompile material to program
            //             let item = self.program_store.use_program(material)?;

            //             group.insert(
            //                 material.name().to_string(),
            //                 RenderGroup {
            //                     program: item.program(),
            //                     attribute_locations: item.attribute_locations(),
            //                     uniform_locations: item.uniform_locations(),
            //                     entities: vec![state],
            //                 },
            //             );
            //         }
            //     };
            // }
            // }

            // add children to rollings list
            collections.extend(
                collection
                    .collections_mut()
                    .iter_mut()
                    .map(|collection| (collection_model_matrix, collection)),
            );
        }

        Ok(group)
    }

    /// Calls pre-render callback of the entity.
    fn pre_render(&self, state: &RenderingEntityState) {
        // state.material().pre_render(state);
    }

    /// Calls post-render callback of the entity.
    fn post_render(&self, state: &RenderingEntityState) {
        // state.material().post_render(state);
    }

    /// Binds attributes of the entity.
    fn bind_attributes(
        &mut self,
        attribute_locations: &HashMap<AttributeBinding, GLuint>,
        state: &RenderingEntityState,
    ) {
        let gl = &self.gl;

        for (binding, location) in attribute_locations {
            let value = match binding {
                AttributeBinding::GeometryPosition => state.geometry().vertices(),
                AttributeBinding::GeometryTextureCoordinate => {
                    state.geometry().texture_coordinates()
                }
                AttributeBinding::GeometryNormal => state.geometry().normals(),
                AttributeBinding::FromGeometry(name) => {
                    state.geometry().attribute_value(name, &state)
                }
                AttributeBinding::FromMaterial(name) => {
                    state.material().attribute_value(name, &state)
                }
                AttributeBinding::FromEntity(name) => {
                    state.entity().attribute_values().get(*name).cloned()
                }
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
                    let buffer = match self.buffer_store.use_buffer(descriptor, target) {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log error
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    gl.bind_buffer(target.gl_enum(), Some(&buffer));
                    gl.vertex_attrib_pointer_with_i32(
                        *location,
                        component_size as GLint,
                        data_type.gl_enum(),
                        normalized,
                        bytes_stride,
                        bytes_offset,
                    );
                    gl.enable_vertex_attrib_array(*location);
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
                    let buffer = match self.buffer_store.use_buffer(descriptor, target) {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log error
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    gl.bind_buffer(target.gl_enum(), Some(&buffer));

                    let component_size = component_size as GLint;
                    // binds each instance
                    for i in 0..components_length_per_instance {
                        let offset_location = *location + (i as GLuint);
                        gl.vertex_attrib_pointer_with_i32(
                            offset_location,
                            component_size,
                            data_type.gl_enum(),
                            normalized,
                            data_type.bytes_length()
                                * component_size
                                * components_length_per_instance,
                            i * data_type.bytes_length() * component_size,
                        );
                        gl.enable_vertex_attrib_array(offset_location);
                        gl.vertex_attrib_divisor(offset_location, divisor);
                    }
                }
                AttributeValue::Vertex1f(x) => gl.vertex_attrib1f(*location, x),
                AttributeValue::Vertex2f(x, y) => gl.vertex_attrib2f(*location, x, y),
                AttributeValue::Vertex3f(x, y, z) => gl.vertex_attrib3f(*location, x, y, z),
                AttributeValue::Vertex4f(x, y, z, w) => gl.vertex_attrib4f(*location, x, y, z, w),
                AttributeValue::Vertex1fv(v) => gl.vertex_attrib1fv_with_f32_array(*location, &v),
                AttributeValue::Vertex2fv(v) => gl.vertex_attrib2fv_with_f32_array(*location, &v),
                AttributeValue::Vertex3fv(v) => gl.vertex_attrib3fv_with_f32_array(*location, &v),
                AttributeValue::Vertex4fv(v) => gl.vertex_attrib4fv_with_f32_array(*location, &v),
            };
        }
    }

    /// Binds uniform data of the entity.
    fn bind_uniforms(
        &mut self,
        uniform_locations: &HashMap<UniformBinding, WebGlUniformLocation>,
        state: &RenderingEntityState,
    ) {
        let gl = &self.gl;

        for (binding, location) in uniform_locations {
            let value = match binding {
                UniformBinding::FromGeometry(name) => state.geometry().uniform_value(name, &state),
                UniformBinding::FromMaterial(name) => state.material().uniform_value(name, &state),
                UniformBinding::FromEntity(name) => {
                    state.entity().uniform_values().get(*name).cloned()
                }
                UniformBinding::ModelMatrix
                | UniformBinding::NormalMatrix
                | UniformBinding::ModelViewMatrix
                | UniformBinding::ModelViewProjMatrix
                | UniformBinding::ViewProjMatrix => {
                    let mat = match binding {
                        UniformBinding::ModelMatrix => state.entity().model_matrix().to_gl(),
                        UniformBinding::NormalMatrix => state.entity().normal_matrix().to_gl(),
                        UniformBinding::ModelViewMatrix => {
                            state.entity().model_view_matrix().to_gl()
                        }
                        UniformBinding::ModelViewProjMatrix => {
                            state.entity().model_view_proj_matrix().to_gl()
                        }
                        UniformBinding::ViewProjMatrix => {
                            state.scene().active_camera().view_proj_matrix().to_gl()
                        }
                        _ => unreachable!(),
                    };

                    Some(UniformValue::Matrix4 {
                        data: mat,
                        transpose: false,
                    })
                }
                UniformBinding::ActiveCameraPosition | UniformBinding::ActiveCameraDirection => {
                    let vec = match binding {
                        UniformBinding::ActiveCameraPosition => {
                            state.scene().active_camera().position().to_gl()
                        }
                        UniformBinding::ActiveCameraDirection => {
                            state.scene().active_camera().direction().to_gl()
                        }
                        _ => unreachable!(),
                    };

                    Some(UniformValue::FloatVector3(vec))
                }
            };
            let Some(value) = value else {
                // should log warning
                continue;
            };

            match value {
                UniformValue::UnsignedInteger1(x) => gl.uniform1ui(Some(location), x),
                UniformValue::UnsignedInteger2(x, y) => gl.uniform2ui(Some(location), x, y),
                UniformValue::UnsignedInteger3(x, y, z) => gl.uniform3ui(Some(location), x, y, z),
                UniformValue::UnsignedInteger4(x, y, z, w) => {
                    gl.uniform4ui(Some(location), x, y, z, w)
                }
                UniformValue::FloatVector1(data) => {
                    gl.uniform1fv_with_f32_array(Some(location), &data)
                }
                UniformValue::FloatVector2(data) => {
                    gl.uniform2fv_with_f32_array(Some(location), &data)
                }
                UniformValue::FloatVector3(data) => {
                    gl.uniform3fv_with_f32_array(Some(location), &data)
                }
                UniformValue::FloatVector4(data) => {
                    gl.uniform4fv_with_f32_array(Some(location), &data)
                }
                UniformValue::IntegerVector1(data) => {
                    gl.uniform1iv_with_i32_array(Some(location), &data)
                }
                UniformValue::IntegerVector2(data) => {
                    gl.uniform2iv_with_i32_array(Some(location), &data)
                }
                UniformValue::IntegerVector3(data) => {
                    gl.uniform3iv_with_i32_array(Some(location), &data)
                }
                UniformValue::IntegerVector4(data) => {
                    gl.uniform4iv_with_i32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector1(data) => {
                    gl.uniform1uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector2(data) => {
                    gl.uniform2uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector3(data) => {
                    gl.uniform3uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::UnsignedIntegerVector4(data) => {
                    gl.uniform4uiv_with_u32_array(Some(location), &data)
                }
                UniformValue::Matrix2 { data, transpose } => {
                    gl.uniform_matrix2fv_with_f32_array(Some(location), transpose, &data)
                }
                UniformValue::Matrix3 { data, transpose } => {
                    gl.uniform_matrix3fv_with_f32_array(Some(location), transpose, &data)
                }
                UniformValue::Matrix4 { data, transpose } => {
                    gl.uniform_matrix4fv_with_f32_array(Some(location), transpose, &data)
                }
                UniformValue::Texture {
                    descriptor,
                    params,
                    texture_unit,
                } => {
                    // active texture
                    gl.active_texture(texture_unit.gl_enum());

                    let (target, texture) = match self.texture_store.use_texture(&descriptor) {
                        Ok(texture) => texture,
                        Err(err) => {
                            // should log warning
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    // binds texture
                    gl.bind_texture(target, Some(texture));
                    // setups sampler parameters
                    params
                        .iter()
                        .for_each(|param| param.tex_parameteri(gl, target));
                    // binds to shader
                    gl.uniform1i(Some(location), texture_unit.unit_index());
                }
            };
        }
    }

    fn draw(&mut self, state: &RenderingEntityState) {
        let gl = &self.gl;

        // draws entity
        if let Some(num_instances) = state.material().instanced() {
            // draw instanced
            match state.geometry().draw() {
                Draw::Arrays {
                    mode,
                    first,
                    count: num_vertices,
                } => gl.draw_arrays_instanced(mode.gl_enum(), first, num_vertices, num_instances),
                Draw::Elements {
                    mode,
                    count: num_vertices,
                    element_type,
                    offset,
                    indices,
                } => {
                    let buffer = match self
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

                    gl.bind_buffer(BufferTarget::ElementArrayBuffer.gl_enum(), Some(&buffer));
                    gl.draw_elements_instanced_with_i32(
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
            match state.geometry().draw() {
                Draw::Arrays {
                    mode,
                    first,
                    count: num_vertices,
                } => gl.draw_arrays(mode.gl_enum(), first, num_vertices),
                Draw::Elements {
                    mode,
                    count: num_vertices,
                    element_type,
                    offset,
                    indices,
                } => {
                    let buffer = match self
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

                    gl.bind_buffer(BufferTarget::ElementArrayBuffer.gl_enum(), Some(&buffer));
                    gl.draw_elements_with_i32(
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
