use std::collections::{HashMap, VecDeque};

use gl_matrix4rust::{
    mat4::{AsMat4, Mat4},
    vec3::AsVec3,
    vec4::Vec4,
};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use wasm_bindgen_test::console_log;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation};

use crate::{entity::Entity, geometry::Geometry, material::WebGLMaterial, scene::Scene};

use self::{
    buffer::{BufferStore, BufferTarget},
    draw::Draw,
    error::Error,
    program::{AttributeBinding, AttributeValue, ProgramStore, UniformBinding, UniformValue},
    texture::TextureStore,
};

pub mod buffer;
pub mod draw;
pub mod error;
pub mod program;
pub mod texture;

#[wasm_bindgen(typescript_custom_section)]
const WEBGL2_RENDER_OPTIONS_TYPE: &'static str = r#"
export type WebGL2RenderOptions = WebGLContextAttributes;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "WebGL2RenderOptions")]
    pub type WebGL2RenderOptionsObject;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullFace {
    Front,
    Back,
    Both,
}

impl CullFace {
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            CullFace::Front => WebGl2RenderingContext::FRONT,
            CullFace::Back => WebGl2RenderingContext::BACK,
            CullFace::Both => WebGl2RenderingContext::FRONT_AND_BACK,
        }
    }
}

#[wasm_bindgen]
pub struct WebGL2Render {
    gl: WebGl2RenderingContext,
    program_store: ProgramStore,
    buffer_store: BufferStore,
    texture_store: TextureStore,
    depth_test: bool,
    cull_face_mode: Option<CullFace>,
    clear_color: Vec4,
}

#[wasm_bindgen]
impl WebGL2Render {
    /// Constructs a new WebGL2 render.
    #[wasm_bindgen(constructor)]
    pub fn new_constructor(
        scene: &Scene,
        options: Option<WebGL2RenderOptionsObject>,
    ) -> Result<WebGL2Render, Error> {
        Self::new_inner(scene, options)
    }
}

impl WebGL2Render {
    /// Constructs a new WebGL2 render.
    pub fn new(scene: &Scene) -> Result<WebGL2Render, Error> {
        Self::new_inner(scene, None)
    }

    /// Constructs a new WebGL2 render.
    pub fn with_options(
        scene: &Scene,
        options: WebGL2RenderOptionsObject,
    ) -> Result<WebGL2Render, Error> {
        Self::new_inner(scene, Some(options))
    }

    fn new_inner(
        scene: &Scene,
        options: Option<WebGL2RenderOptionsObject>,
    ) -> Result<WebGL2Render, Error> {
        let gl = Self::gl_context(scene.canvas(), options)?;
        let mut render = Self {
            program_store: ProgramStore::new(gl.clone()),
            buffer_store: BufferStore::new(gl.clone()),
            texture_store: TextureStore::new(gl.clone()),
            gl,
            depth_test: true,
            cull_face_mode: None,
            clear_color: Vec4::new(),
        };

        render.set_clear_color(Vec4::new());
        render.set_cull_face(None);
        render.set_depth_test(true);

        Ok(render)
    }

    /// Gets WebGl2RenderingContext.
    fn gl_context(
        canvas: &HtmlCanvasElement,
        options: Option<WebGL2RenderOptionsObject>,
    ) -> Result<WebGl2RenderingContext, Error> {
        let options = match options {
            Some(options) => options.obj,
            None => JsValue::UNDEFINED,
        };

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .ok()
            .and_then(|context| context)
            .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(Error::WebGl2RenderingContextNotFound)?;

        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);

        Ok(gl)
    }
}

impl WebGL2Render {
    pub fn depth_test(&self) -> bool {
        self.depth_test
    }

    pub fn set_depth_test(&mut self, depth_test: bool) {
        self.depth_test = depth_test;
        if self.depth_test {
            self.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
        } else {
            self.gl.disable(WebGl2RenderingContext::DEPTH_TEST);
        }
    }

    pub fn cull_face(&self) -> Option<CullFace> {
        self.cull_face_mode
    }

    pub fn set_cull_face(&mut self, cull_face_mode: Option<CullFace>) {
        self.cull_face_mode = cull_face_mode;
        match self.cull_face_mode {
            Some(cull_face_mode) => {
                self.gl.enable(WebGl2RenderingContext::CULL_FACE);
                self.gl.cull_face(cull_face_mode.to_gl_enum())
            }
            None => self.gl.disable(WebGl2RenderingContext::CULL_FACE),
        }
    }

    pub fn clear_color(&self) -> Vec4 {
        self.clear_color
    }

    pub fn set_clear_color(&mut self, clear_color: Vec4) {
        self.clear_color = clear_color;
        self.gl.clear_color(
            self.clear_color.0[0] as f32,
            self.clear_color.0[1] as f32,
            self.clear_color.0[2] as f32,
            self.clear_color.0[3] as f32,
        );
    }
}

struct RenderGroup {
    program: *const WebGlProgram,
    attribute_locations: *const HashMap<AttributeBinding, u32>,
    uniform_locations: *const HashMap<UniformBinding, WebGlUniformLocation>,
    entities: Vec<RenderItem>,
}

struct RenderItem {
    entity: *mut Entity,
    geometry: *mut dyn Geometry,
    material: *mut dyn WebGLMaterial,
}

impl WebGL2Render {
    pub fn render(&mut self, scene: &mut Scene) -> Result<(), Error> {
        // update WebGL viewport
        self.gl.viewport(
            0,
            0,
            scene.canvas().width() as i32,
            scene.canvas().height() as i32,
        );

        // clear scene
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        self.gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);

        // extracts to pointers
        let scene_ptr: *mut Scene = scene;

        // collects entities and render console_error_panic_hook
        let entities_group = self.prepare(scene_ptr)?;

        // render each entities group
        for (
            _,
            RenderGroup {
                program,
                attribute_locations,
                uniform_locations,
                entities,
            },
        ) in entities_group
        {
            let (program, attribute_locations, uniform_locations) =
                unsafe { (&*program, &*attribute_locations, &*uniform_locations) };

            // binds program
            self.gl.use_program(Some(program));

            // render each entity
            for entity_item in entities {
                let RenderItem {
                    entity,
                    geometry,
                    material,
                } = entity_item;
                let (entity, geometry, material) =
                    unsafe { (&mut *entity, &mut *geometry, &mut *material) };

                // pre-render
                self.pre_render(scene, entity, geometry, material);
                // binds attributes
                self.bind_attributes(attribute_locations, entity, geometry, material);
                // binds uniforms
                self.bind_uniforms(scene, uniform_locations, entity, geometry, material);
                // draws
                self.draw(geometry, material);
                // post-render
                self.post_render(scene, entity, geometry, material);
            }

            // unbinds for good practices
            self.gl.use_program(None);
            self.gl
                .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
            self.gl
                .bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);
        }

        Ok(())
    }

    fn prepare(&mut self, scene_ptr: *mut Scene) -> Result<HashMap<String, RenderGroup>, Error> {
        let scene = unsafe { &mut *scene_ptr };

        let view_matrix = scene.active_camera().view_matrix();
        let proj_matrix = scene.active_camera().proj_matrix();

        let mut group: HashMap<String, RenderGroup> = HashMap::new();

        let mut rollings: VecDeque<*mut Entity> =
            VecDeque::from([scene.root_entity_mut() as *mut Entity]);
        while let Some(entity) = rollings.pop_front() {
            let entity: &mut Entity = unsafe { &mut *entity };

            // update entity matrices in current frame
            let parent_model_matrix = entity
                .parent()
                .map(|parent| parent.model_matrix() as *const Mat4);

            if let Err(err) =
                entity.update_frame_matrices(parent_model_matrix, &view_matrix, &proj_matrix)
            {
                // should log warning
                console_log!("{}", err);
                continue;
            }

            // filters any entity that has no geometry or material
            // groups entities by material to prevent unnecessary program switching
            if let (Some(geometry), Some(material)) = (entity.geometry_raw(), entity.material_raw())
            {
                let (geometry, material) = unsafe { (&mut *geometry, &mut *material) };

                material.prepare(scene, entity, geometry);

                // check whether material is ready or not
                if material.ready() {
                    let render_item = RenderItem {
                        entity,
                        geometry,
                        material,
                    };
                    match group.get_mut(material.name()) {
                        Some(group) => group.entities.push(render_item),
                        None => {
                            // precompile material to program
                            let item = self.program_store.program_or_compile(material)?;

                            group.insert(
                                material.name().to_string(),
                                RenderGroup {
                                    program: item.program(),
                                    attribute_locations: item.attribute_locations(),
                                    uniform_locations: item.uniform_locations(),
                                    entities: vec![render_item],
                                },
                            );
                        }
                    };
                }
            }

            // add children to rollings list
            rollings.extend(
                entity
                    .children_mut()
                    .iter_mut()
                    .map(|child| child.as_mut() as *mut Entity),
            );
        }

        Ok(group)
    }

    fn pre_render(
        &self,
        scene: &mut Scene,
        entity: &mut Entity,
        geometry: &mut dyn Geometry,
        material: &mut dyn WebGLMaterial,
    ) {
        material.pre_render(scene, entity, geometry);
    }

    fn post_render(
        &self,
        scene: &mut Scene,
        entity: &mut Entity,
        geometry: &mut dyn Geometry,
        material: &mut dyn WebGLMaterial,
    ) {
        material.post_render(scene, entity, geometry);
    }

    fn bind_attributes(
        &mut self,
        attribute_locations: &HashMap<AttributeBinding, u32>,
        entity: &Entity,
        geometry: &dyn Geometry,
        material: &dyn WebGLMaterial,
    ) {
        let gl = &self.gl;

        for (binding, location) in attribute_locations {
            let value = match binding {
                AttributeBinding::GeometryPosition => geometry.vertices(),
                AttributeBinding::GeometryTextureCoordinate => geometry.texture_coordinates(),
                AttributeBinding::GeometryNormal => geometry.normals(),
                AttributeBinding::FromGeometry(name) => geometry.attribute_value(name),
                AttributeBinding::FromMaterial(name) => material.attribute_value(name),
                AttributeBinding::FromEntity(name) => entity.attribute_value(name),
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
                    let buffer = match self.buffer_store.buffer_or_create(descriptor, target) {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log error
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    gl.bind_buffer(target.to_gl_enum(), Some(buffer));
                    gl.vertex_attrib_pointer_with_i32(
                        *location,
                        component_size.to_i32(),
                        data_type.to_gl_enum(),
                        normalized,
                        bytes_stride,
                        bytes_offset,
                    );
                    gl.enable_vertex_attrib_array(*location);
                    gl.bind_buffer(target.to_gl_enum(), None);
                }
                AttributeValue::InstancedBuffer {
                    descriptor,
                    target,
                    component_size,
                    data_type,
                    normalized,
                    components_length_per_instance,
                    divisor,
                } => {
                    let buffer = match self.buffer_store.buffer_or_create(descriptor, target) {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log error
                            console_log!("{}", err);
                            continue;
                        }
                    };

                    gl.bind_buffer(target.to_gl_enum(), Some(buffer));
                    // binds each instance
                    for i in 0..components_length_per_instance {
                        let offset_location = *location + i;
                        gl.vertex_attrib_pointer_with_i32(
                            offset_location,
                            component_size.to_i32(),
                            data_type.to_gl_enum(),
                            normalized,
                            data_type.bytes_length()
                                * component_size.to_i32()
                                * (components_length_per_instance as i32),
                            (i as i32) * data_type.bytes_length() * component_size.to_i32(),
                        );
                        gl.enable_vertex_attrib_array(offset_location);
                        gl.vertex_attrib_divisor(offset_location, divisor);
                    }
                    gl.bind_buffer(target.to_gl_enum(), None);
                }
                AttributeValue::Vertex1f(x) => gl.vertex_attrib1f(*location, x),
                AttributeValue::Vertex2f(x, y) => gl.vertex_attrib2f(*location, x, y),
                AttributeValue::Vertex3f(x, y, z) => gl.vertex_attrib3f(*location, x, y, z),
                AttributeValue::Vertex4f(x, y, z, w) => gl.vertex_attrib4f(*location, x, y, z, w),
                AttributeValue::Vertex1fv(v) => gl.vertex_attrib1fv_with_f32_array(*location, &v),
                AttributeValue::Vertex2fv(v) => gl.vertex_attrib2fv_with_f32_array(*location, &v),
                AttributeValue::Vertex3fv(v) => gl.vertex_attrib3fv_with_f32_array(*location, &v),
                AttributeValue::Vertex4fv(v) => gl.vertex_attrib4fv_with_f32_array(*location, &v),
            }
        }
    }

    fn bind_uniforms(
        &mut self,
        scene: &Scene,
        uniform_locations: &HashMap<UniformBinding, WebGlUniformLocation>,
        entity: &Entity,
        geometry: &dyn Geometry,
        material: &dyn WebGLMaterial,
    ) {
        let gl = &self.gl;

        let mut tmp_mat4;
        let mut tmp_vec3;

        for (binding, location) in uniform_locations {
            let value = match binding {
                UniformBinding::FromGeometry(name) => geometry.uniform_value(name),
                UniformBinding::FromMaterial(name) => material.uniform_value(name),
                UniformBinding::FromEntity(name) => entity.uniform_value(name),
                UniformBinding::ParentModelMatrix
                | UniformBinding::ModelMatrix
                | UniformBinding::NormalMatrix
                | UniformBinding::ModelViewMatrix
                | UniformBinding::ModelViewProjMatrix
                | UniformBinding::ViewProjMatrix => {
                    tmp_mat4 = match binding {
                        UniformBinding::ParentModelMatrix => match entity.parent() {
                            Some(parent) => parent.local_matrix().to_gl(),
                            None => Mat4::<f32>::new_identity().to_gl(), // use identity if not exists
                        },
                        UniformBinding::ModelMatrix => entity.local_matrix().to_gl(),
                        UniformBinding::NormalMatrix => entity.normal_matrix().to_gl(),
                        UniformBinding::ModelViewMatrix => entity.model_view_matrix().to_gl(),
                        UniformBinding::ModelViewProjMatrix => {
                            entity.model_view_proj_matrix().to_gl()
                        }
                        UniformBinding::ViewProjMatrix => {
                            scene.active_camera().view_proj_matrix().to_gl()
                        }
                        _ => unreachable!(),
                    };

                    Some(UniformValue::Matrix4 {
                        data: &tmp_mat4,
                        transpose: false,
                        src_offset: 0,
                        src_length: 0,
                    })
                }
                UniformBinding::ActiveCameraPosition | UniformBinding::ActiveCameraDirection => {
                    tmp_vec3 = match binding {
                        UniformBinding::ActiveCameraPosition => {
                            scene.active_camera().position().to_gl()
                        }
                        UniformBinding::ActiveCameraDirection => {
                            scene.active_camera().direction().to_gl()
                        }
                        _ => unreachable!(),
                    };

                    Some(UniformValue::FloatVector3 {
                        data: &tmp_vec3,
                        src_offset: 0,
                        src_length: 0,
                    })
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
                UniformValue::FloatVector1 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform1fv_with_f32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::FloatVector2 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform2fv_with_f32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::FloatVector3 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform3fv_with_f32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::FloatVector4 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform4fv_with_f32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::IntegerVector1 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform1iv_with_i32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::IntegerVector2 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform2iv_with_i32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::IntegerVector3 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform3iv_with_i32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::IntegerVector4 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform4iv_with_i32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::UnsignedIntegerVector1 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform1uiv_with_u32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::UnsignedIntegerVector2 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform2uiv_with_u32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::UnsignedIntegerVector3 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform3uiv_with_u32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::UnsignedIntegerVector4 {
                    data,
                    src_offset,
                    src_length,
                } => gl.uniform4uiv_with_u32_array_and_src_offset_and_src_length(
                    Some(location),
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::Matrix2 {
                    data,
                    transpose,
                    src_offset,
                    src_length,
                } => gl.uniform_matrix2fv_with_f32_array_and_src_offset_and_src_length(
                    Some(location),
                    transpose,
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::Matrix3 {
                    data,
                    transpose,
                    src_offset,
                    src_length,
                } => gl.uniform_matrix3fv_with_f32_array_and_src_offset_and_src_length(
                    Some(location),
                    transpose,
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::Matrix4 {
                    data,
                    transpose,
                    src_offset,
                    src_length,
                } => gl.uniform_matrix4fv_with_f32_array_and_src_offset_and_src_length(
                    Some(location),
                    transpose,
                    data.as_ref().as_ref(),
                    src_offset,
                    src_length,
                ),
                UniformValue::Texture {
                    descriptor,
                    params,
                    active_unit,
                } => {
                    // active texture
                    gl.active_texture(WebGl2RenderingContext::TEXTURE0 + active_unit);

                    let (target, texture) = match self.texture_store.texture_or_create(descriptor) {
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
                    gl.uniform1i(Some(location), active_unit as i32);
                    gl.bind_texture(target, None);
                }
            }
        }
    }

    fn draw(&mut self, geometry: &dyn Geometry, material: &dyn WebGLMaterial) {
        let gl = &self.gl;

        // draws entity
        if let Some(num_instances) = material.instanced() {
            // dr: f64aw instanced
            match geometry.draw() {
                Draw::Arrays {
                    mode,
                    first,
                    count: num_vertices,
                } => {
                    gl.draw_arrays_instanced(mode.to_gl_enum(), first, num_vertices, num_instances)
                }
                Draw::Elements {
                    mode,
                    count: num_vertices,
                    element_type,
                    offset,
                    indices,
                } => {
                    let buffer = match self
                        .buffer_store
                        .buffer_or_create(indices, BufferTarget::ElementArrayBuffer)
                    {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log warning
                            console_log!("{}", err);
                            return;
                        }
                    };
                    gl.bind_buffer(BufferTarget::ElementArrayBuffer.to_gl_enum(), Some(buffer));

                    gl.draw_elements_instanced_with_i32(
                        mode.to_gl_enum(),
                        num_vertices,
                        element_type.to_gl_enum(),
                        offset,
                        num_instances,
                    );
                }
            }
        } else {
            // draw normally!
            match geometry.draw() {
                Draw::Arrays {
                    mode,
                    first,
                    count: num_vertices,
                } => gl.draw_arrays(mode.to_gl_enum(), first, num_vertices),
                Draw::Elements {
                    mode,
                    count: num_vertices,
                    element_type,
                    offset,
                    indices,
                } => {
                    let buffer = match self
                        .buffer_store
                        .buffer_or_create(indices, BufferTarget::ElementArrayBuffer)
                    {
                        Ok(buffer) => buffer,
                        Err(err) => {
                            // should log warning
                            console_log!("{}", err);
                            return;
                        }
                    };
                    gl.bind_buffer(BufferTarget::ElementArrayBuffer.to_gl_enum(), Some(buffer));

                    gl.draw_elements_with_i32(
                        mode.to_gl_enum(),
                        num_vertices,
                        element_type.to_gl_enum(),
                        offset,
                    );
                }
            }
        }
    }
}
