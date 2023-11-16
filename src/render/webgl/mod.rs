use std::collections::{HashMap, VecDeque};

use gl_matrix4rust::{mat4::Mat4, vec4::Vec4};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsError, JsValue};
use wasm_bindgen_test::console_log;
use web_sys::{
    js_sys::Date, HtmlCanvasElement, WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation,
};

use crate::{document, entity::Entity, ncor::Ncor, scene::Scene};

use self::{
    buffer::{BufferStore, BufferTarget},
    draw::Draw,
    program::{AttributeBinding, AttributeValue, ProgramStore, UniformBinding, UniformValue},
    texture::TextureStore,
};

pub mod buffer;
pub mod draw;
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
    ) -> Result<WebGL2Render, JsError> {
        Self::new_inner(scene, options)
    }
}

impl WebGL2Render {
    /// Constructs a new WebGL2 render.
    pub fn new(scene: &Scene) -> Result<WebGL2Render, JsError> {
        Self::new_inner(scene, None)
    }

    /// Constructs a new WebGL2 render.
    pub fn with_options(
        scene: &Scene,
        options: WebGL2RenderOptionsObject,
    ) -> Result<WebGL2Render, JsError> {
        Self::new_inner(scene, Some(options))
    }

    fn new_inner(
        scene: &Scene,
        options: Option<WebGL2RenderOptionsObject>,
    ) -> Result<WebGL2Render, JsError> {
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
    ) -> Result<WebGl2RenderingContext, JsError> {
        let options = match options {
            Some(options) => options.obj,
            None => JsValue::UNDEFINED,
        };

        let gl = canvas
            .get_context_with_context_options("webgl2", &options)
            .ok()
            .and_then(|context| context)
            .and_then(|context| context.dyn_into::<WebGl2RenderingContext>().ok())
            .ok_or(JsError::new("failed to get WebGL2 context"))?;

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

    pub fn clear_color(&self) -> Vec4<f32> {
        self.clear_color
    }

    pub fn set_clear_color(&mut self, clear_color: Vec4) {
        self.clear_color = clear_color;
        self.gl.clear_color(
            self.clear_color.0[0],
            self.clear_color.0[1],
            self.clear_color.0[2],
            self.clear_color.0[3],
        );
    }
}

struct RenderGroup {
    program: *const WebGlProgram,
    attribute_locations: *const HashMap<AttributeBinding, u32>,
    uniform_locations: *const HashMap<UniformBinding, WebGlUniformLocation>,
    entities: Vec<*const Entity>,
}

impl WebGL2Render {
    fn prepare(&mut self, scene: &Scene) -> Result<HashMap<String, RenderGroup>, String> {
        let view = *scene.active_camera().view_matrix();
        let proj = *scene.active_camera().proj_matrix();

        let mut group: HashMap<String, RenderGroup> = HashMap::new();
        let mut rollings = VecDeque::from([scene.root_entity()]);
        while let Some(entity) = rollings.pop_front() {
            // update composed matrices for all entities
            let composed_model = match entity.parent() {
                Some(parent) => *parent.model_matrix() * *entity.model_matrix(),
                None => *entity.model_matrix(),
            };
            let composed_normal = match composed_model.invert() {
                Ok(inverted) => inverted.transpose(),
                Err(err) => {
                    //should err
                    console_log!("{}", err);
                    continue;
                }
            };
            let composed_model_view = view * composed_model;
            let composed_model_view_proj = proj * composed_model_view;
            entity.composed_model_matrix().replace(composed_model);
            entity.composed_normal_matrix().replace(composed_normal);
            entity
                .composed_model_view_matrix()
                .replace(composed_model_view);
            entity
                .composed_model_view_proj_matrix()
                .replace(composed_model_view_proj);

            // filters any entity that has no geometry or material
            // groups entities by material to prevent unnecessary program switching
            if entity.geometry().is_some() && entity.material().is_some() {
                let geometry = entity.geometry().unwrap().borrow();
                let mut material = entity.material().unwrap().borrow_mut();
                let material = material.as_mut();

                material.pre_render(scene, entity, geometry.as_ref());

                if material.ready() {
                    match group.get_mut(material.name()) {
                        Some(group) => group.entities.push(entity),
                        None => {
                            // precompile material to program
                            let (program, attribute_locations, uniform_locations) =
                                self.program_store.program_or_compile(material)?;

                            group.insert(
                                material.name().to_string(),
                                RenderGroup {
                                    program,
                                    attribute_locations,
                                    uniform_locations,
                                    entities: vec![entity],
                                },
                            );
                        }
                    };
                }
            }

            // add children to rollings list
            rollings.extend(entity.children().iter().map(|child| child.as_ref()));
        }

        Ok(group)
    }

    pub fn render(&mut self, scene: &Scene) {
        let mut bind_prom = 0.0;
        let mut unbind_prom = 0.0;
        let mut attr = 0.0;
        let mut prep = 0.0;
        let mut unif = 0.0;
        let mut rend = 0.0;

        let total_start = Date::now();

        // collects entities and render console_error_panic_hook
        let start = Date::now();
        let entities_group = match self.prepare(scene) {
            Ok(group) => group,
            Err(err) => {
                // should log error
                console_log!("{}", err);
                return;
            }
        };
        let end = Date::now();
        prep += end - start;

        let gl = &self.gl;

        // clear scene
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);

        // extract camera direction and position
        let camera_direction = *scene.active_camera().direction();
        let camera_position = *scene.active_camera().position();
        let camera_view_proj_matrix = *scene.active_camera().view_proj_matrix();

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
            let start = Date::now();
            gl.use_program(Some(program));
            let end = Date::now();
            bind_prom += end - start;

            // render each entity
            for entity in entities {
                let entity = unsafe { &*entity };
                let geometry = entity.geometry().unwrap();
                let material = entity.material().unwrap();

                {
                    let geometry = geometry.borrow();
                    let material = material.borrow();

                    // binds attribute values
                    {
                        let start = Date::now();
                        for (binding, location) in attribute_locations {
                            let value = match binding {
                                AttributeBinding::GeometryPosition => geometry.vertices(),
                                AttributeBinding::GeometryTextureCoordinate => {
                                    geometry.texture_coordinates()
                                }
                                AttributeBinding::GeometryNormal => geometry.normals(),
                                AttributeBinding::FromGeometry(name) => {
                                    geometry.attribute_value(name.as_str())
                                }
                                AttributeBinding::FromMaterial(name) => {
                                    material.attribute_value(name.as_str())
                                }
                                AttributeBinding::FromEntity(name) => {
                                    entity.attribute_value(name.as_str())
                                }
                            };

                            let Some(value) = value else {
                                // should log warning
                                console_log!("3");
                                continue;
                            };

                            match value.as_ref() {
                                AttributeValue::Buffer {
                                    descriptor,
                                    target,
                                    component_size,
                                    data_type,
                                    normalized,
                                    bytes_stride,
                                    bytes_offset,
                                } => {
                                    let buffer = match self
                                        .buffer_store
                                        .buffer_or_create(descriptor.as_ref(), *target)
                                    {
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
                                        *normalized,
                                        *bytes_stride,
                                        *bytes_offset,
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
                                    let buffer = match self
                                        .buffer_store
                                        .buffer_or_create(descriptor.as_ref(), *target)
                                    {
                                        Ok(buffer) => buffer,
                                        Err(err) => {
                                            // should log error
                                            console_log!("{}", err);
                                            continue;
                                        }
                                    };

                                    gl.bind_buffer(target.to_gl_enum(), Some(buffer));
                                    // binds each instance
                                    for i in 0..*components_length_per_instance {
                                        let offset_location = *location + i;
                                        gl.vertex_attrib_pointer_with_i32(
                                            offset_location,
                                            component_size.to_i32(),
                                            data_type.to_gl_enum(),
                                            *normalized,
                                            data_type.bytes_length()
                                                * component_size.to_i32()
                                                * (*components_length_per_instance as i32),
                                            (i as i32)
                                                * data_type.bytes_length()
                                                * component_size.to_i32(),
                                        );
                                        gl.enable_vertex_attrib_array(offset_location);
                                        gl.vertex_attrib_divisor(offset_location, *divisor);
                                    }
                                    gl.bind_buffer(target.to_gl_enum(), None);
                                }
                                AttributeValue::Vertex1f(x) => gl.vertex_attrib1f(*location, *x),
                                AttributeValue::Vertex2f(x, y) => {
                                    gl.vertex_attrib2f(*location, *x, *y)
                                }
                                AttributeValue::Vertex3f(x, y, z) => {
                                    gl.vertex_attrib3f(*location, *x, *y, *z)
                                }
                                AttributeValue::Vertex4f(x, y, z, w) => {
                                    gl.vertex_attrib4f(*location, *x, *y, *z, *w)
                                }
                                AttributeValue::Vertex1fv(v) => {
                                    gl.vertex_attrib1fv_with_f32_array(*location, v)
                                }
                                AttributeValue::Vertex2fv(v) => {
                                    gl.vertex_attrib2fv_with_f32_array(*location, v)
                                }
                                AttributeValue::Vertex3fv(v) => {
                                    gl.vertex_attrib3fv_with_f32_array(*location, v)
                                }
                                AttributeValue::Vertex4fv(v) => {
                                    gl.vertex_attrib4fv_with_f32_array(*location, v)
                                }
                            }
                        }
                        let end = Date::now();
                        attr += end - start;
                    }

                    // binds uniform values
                    {
                        let start = Date::now();
                        for (binding, location) in uniform_locations {
                            let value = match binding {
                                UniformBinding::ParentModelMatrix => {
                                    let parent_model_matrix = match entity.parent() {
                                        Some(parent) => *parent.model_matrix(),
                                        // use identity if not exists
                                        None => Mat4::new_identity(),
                                    };
                                    Some(Ncor::Owned(UniformValue::Matrix4 {
                                        data: Box::new(parent_model_matrix),
                                        transpose: false,
                                        src_offset: 0,
                                        src_length: 0,
                                    }))
                                }
                                UniformBinding::ModelMatrix => {
                                    Some(Ncor::Owned(UniformValue::Matrix4 {
                                        data: Box::new(*entity.composed_model_matrix().borrow()),
                                        transpose: false,
                                        src_offset: 0,
                                        src_length: 0,
                                    }))
                                }
                                UniformBinding::NormalMatrix => {
                                    Some(Ncor::Owned(UniformValue::Matrix4 {
                                        data: Box::new(*entity.composed_normal_matrix().borrow()),
                                        transpose: false,
                                        src_offset: 0,
                                        src_length: 0,
                                    }))
                                }
                                UniformBinding::ModelViewMatrix => {
                                    Some(Ncor::Owned(UniformValue::Matrix4 {
                                        data: Box::new(
                                            *entity.composed_model_view_matrix().borrow(),
                                        ),
                                        transpose: false,
                                        src_offset: 0,
                                        src_length: 0,
                                    }))
                                }
                                UniformBinding::ModelViewProjMatrix => {
                                    Some(Ncor::Owned(UniformValue::Matrix4 {
                                        data: Box::new(
                                            *entity.composed_model_view_proj_matrix().borrow(),
                                        ),
                                        transpose: false,
                                        src_offset: 0,
                                        src_length: 0,
                                    }))
                                }
                                UniformBinding::ViewProjMatrix => {
                                    Some(Ncor::Owned(UniformValue::Matrix4 {
                                        data: Box::new(camera_view_proj_matrix),
                                        transpose: false,
                                        src_offset: 0,
                                        src_length: 0,
                                    }))
                                }
                                UniformBinding::ActiveCameraPosition => {
                                    Some(Ncor::Owned(UniformValue::FloatVector3 {
                                        data: Box::new(camera_position),
                                        src_offset: 0,
                                        src_length: 0,
                                    }))
                                }
                                UniformBinding::ActiveCameraDirection => {
                                    Some(Ncor::Owned(UniformValue::FloatVector3 {
                                        data: Box::new(camera_direction),
                                        src_offset: 0,
                                        src_length: 0,
                                    }))
                                }
                                UniformBinding::FromGeometry(name) => {
                                    geometry.uniform_value(name.as_str())
                                }
                                UniformBinding::FromMaterial(name) => {
                                    material.uniform_value(name.as_str())
                                }
                                UniformBinding::FromEntity(name) => {
                                    entity.uniform_value(name.as_str())
                                }
                            };
                            let Some(value) = value else {
                                // should log warning
                                console_log!("uniform {} not found", binding.as_str());
                                continue;
                            };

                            match value.as_ref() {
                                UniformValue::UnsignedInteger1(x) => {
                                    gl.uniform1ui(Some(location), *x)
                                }
                                UniformValue::UnsignedInteger2(x, y) => {
                                    gl.uniform2ui(Some(location), *x, *y)
                                }
                                UniformValue::UnsignedInteger3(x, y, z) => {
                                    gl.uniform3ui(Some(location), *x, *y, *z)
                                }
                                UniformValue::UnsignedInteger4(x, y, z, w) => {
                                    gl.uniform4ui(Some(location), *x, *y, *z, *w)
                                }
                                UniformValue::FloatVector1 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform1fv_with_f32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::FloatVector2 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform2fv_with_f32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::FloatVector3 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform3fv_with_f32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::FloatVector4 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform4fv_with_f32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::IntegerVector1 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform1iv_with_i32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::IntegerVector2 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform2iv_with_i32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::IntegerVector3 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform3iv_with_i32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::IntegerVector4 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform4iv_with_i32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::UnsignedIntegerVector1 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform1uiv_with_u32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::UnsignedIntegerVector2 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform2uiv_with_u32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::UnsignedIntegerVector3 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform3uiv_with_u32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::UnsignedIntegerVector4 {
                                    data,
                                    src_offset,
                                    src_length,
                                } => gl.uniform4uiv_with_u32_array_and_src_offset_and_src_length(
                                    Some(location),
                                    data.as_ref().as_ref(),
                                    *src_offset,
                                    *src_length,
                                ),
                                UniformValue::Matrix2 {
                                    data,
                                    transpose,
                                    src_offset,
                                    src_length,
                                } => gl
                                    .uniform_matrix2fv_with_f32_array_and_src_offset_and_src_length(
                                        Some(location),
                                        *transpose,
                                        data.as_ref().as_ref(),
                                        *src_offset,
                                        *src_length,
                                    ),
                                UniformValue::Matrix3 {
                                    data,
                                    transpose,
                                    src_offset,
                                    src_length,
                                } => gl
                                    .uniform_matrix3fv_with_f32_array_and_src_offset_and_src_length(
                                        Some(location),
                                        *transpose,
                                        data.as_ref().as_ref(),
                                        *src_offset,
                                        *src_length,
                                    ),
                                UniformValue::Matrix4 {
                                    data,
                                    transpose,
                                    src_offset,
                                    src_length,
                                } => gl
                                    .uniform_matrix4fv_with_f32_array_and_src_offset_and_src_length(
                                        Some(location),
                                        *transpose,
                                        data.as_ref().as_ref(),
                                        *src_offset,
                                        *src_length,
                                    ),
                                UniformValue::Texture {
                                    descriptor,
                                    params,
                                    active_unit,
                                } => {
                                    // active texture
                                    gl.active_texture(
                                        WebGl2RenderingContext::TEXTURE0 + *active_unit,
                                    );

                                    let (target, texture) = match self
                                        .texture_store
                                        .texture_or_create(descriptor.as_ref())
                                    {
                                        Ok(texture) => texture,
                                        Err(err) => {
                                            // should log error
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
                                    gl.uniform1i(Some(location), *active_unit as i32);
                                    // gl.bind_texture(target, None);
                                }
                            }
                        }
                        let end = Date::now();
                        unif += end - start;
                    }

                    // draws entity
                    let start = Date::now();
                    if let Some(num_instances) = material.instanced() {
                        // draw instanced
                        match geometry.draw() {
                            Draw::Arrays {
                                mode,
                                first,
                                num_vertices,
                            } => gl.draw_arrays_instanced(
                                mode.to_gl_enum(),
                                first,
                                num_vertices,
                                num_instances,
                            ),
                            Draw::Elements {
                                mode,
                                count,
                                element_type,
                                offset,
                                indices,
                            } => {
                                let buffer = match self.buffer_store.buffer_or_create(
                                    indices.as_ref(),
                                    BufferTarget::ElementArrayBuffer,
                                ) {
                                    Ok(buffer) => buffer,
                                    Err(err) => {
                                        // should log error
                                        console_log!("{}", err);
                                        continue;
                                    }
                                };
                                gl.bind_buffer(
                                    BufferTarget::ElementArrayBuffer.to_gl_enum(),
                                    Some(buffer),
                                );

                                gl.draw_elements_instanced_with_i32(
                                    mode.to_gl_enum(),
                                    count,
                                    element_type.to_gl_enum(),
                                    offset,
                                    num_instances,
                                )
                            }
                        }
                    } else {
                        // draw normally!
                        match geometry.draw() {
                            Draw::Arrays {
                                mode,
                                first,
                                num_vertices,
                            } => gl.draw_arrays(mode.to_gl_enum(), first, num_vertices),
                            Draw::Elements {
                                mode,
                                count,
                                element_type,
                                offset,
                                indices,
                            } => {
                                let buffer = match self.buffer_store.buffer_or_create(
                                    indices.as_ref(),
                                    BufferTarget::ElementArrayBuffer,
                                ) {
                                    Ok(buffer) => buffer,
                                    Err(err) => {
                                        // should log error
                                        console_log!("{}", err);
                                        continue;
                                    }
                                };
                                gl.bind_buffer(
                                    BufferTarget::ElementArrayBuffer.to_gl_enum(),
                                    Some(buffer),
                                );

                                gl.draw_elements_with_i32(
                                    mode.to_gl_enum(),
                                    count,
                                    element_type.to_gl_enum(),
                                    offset,
                                )
                            }
                        }
                    }
                    let end = Date::now();
                    rend += end - start;
                }

                {
                    material
                        .borrow_mut()
                        .post_render(scene, entity, geometry.borrow().as_ref());
                }
            }
        }

        // unbinds data after drawing
        let start = Date::now();
        gl.use_program(None);
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);
        let end = Date::now();
        unbind_prom += end - start;

        let total_end = Date::now();

        document()
            .get_element_by_id("preparation")
            .unwrap()
            .set_inner_html(&prep.to_string());
        document()
            .get_element_by_id("bind_program")
            .unwrap()
            .set_inner_html(&bind_prom.to_string());
        document()
            .get_element_by_id("attributes")
            .unwrap()
            .set_inner_html(&attr.to_string());
        document()
            .get_element_by_id("uniforms")
            .unwrap()
            .set_inner_html(&unif.to_string());
        document()
            .get_element_by_id("render")
            .unwrap()
            .set_inner_html(&rend.to_string());
        document()
            .get_element_by_id("unbind_program")
            .unwrap()
            .set_inner_html(&unbind_prom.to_string());
        document()
            .get_element_by_id("total")
            .unwrap()
            .set_inner_html(&(total_end - total_start).to_string());
    }
}
