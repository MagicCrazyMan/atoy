use std::collections::VecDeque;

use gl_matrix4rust::vec4::Vec4;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsError, JsValue};
use wasm_bindgen_test::console_log;
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{entity::Entity, ncor::Ncor, scene::Scene, window};

use self::{
    buffer::BufferStore,
    draw::Draw,
    program::{AttributeBinding, AttributeValue, ProgramStore, UniformBinding, UniformValue},
};

pub mod buffer;
pub mod draw;
pub mod program;

#[wasm_bindgen(typescript_custom_section)]
const WEBGL2_RENDER_OPTIONS_TYPE: &'static str = r#"
export type WebGL2RenderOptions = WebGLContextAttributes;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "WebGL2RenderOptions")]
    pub type WebGL2RenderOptionsObject;
}

#[wasm_bindgen]
pub struct WebGL2Render {
    gl: WebGl2RenderingContext,
    program_store: ProgramStore,
    buffer_store: BufferStore,
    depth_test: bool,
    cull_face_mode: Option<u32>,
    clear_color: Vec4,
    first: bool,
}

#[wasm_bindgen]
impl WebGL2Render {
    /// Constructs a new WebGL2 render.
    #[wasm_bindgen(constructor)]
    pub fn new_constructor(
        scene: &Scene,
        options: Option<WebGL2RenderOptionsObject>,
    ) -> Result<WebGL2Render, JsError> {
        Self::with_options(scene, options)
    }
}

impl WebGL2Render {
    /// Constructs a new WebGL2 render.
    pub fn new(scene: &Scene) -> Result<WebGL2Render, JsError> {
        let gl = Self::gl_context(scene.canvas(), None)?;
        Ok(Self {
            program_store: ProgramStore::new(gl.clone()),
            buffer_store: BufferStore::new(gl.clone()),
            gl,
            depth_test: true,
            cull_face_mode: None,
            clear_color: Vec4::new(),
            first: true,
        })
    }

    /// Constructs a new WebGL2 render.
    pub fn with_options(
        scene: &Scene,
        options: Option<WebGL2RenderOptionsObject>,
    ) -> Result<WebGL2Render, JsError> {
        let gl = Self::gl_context(scene.canvas(), options)?;
        Ok(Self {
            program_store: ProgramStore::new(gl.clone()),
            buffer_store: BufferStore::new(gl.clone()),
            gl,
            depth_test: true,
            cull_face_mode: None,
            clear_color: Vec4::new(),
            first: true,
        })
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

        console_log!("{} {}", canvas.width(), canvas.height());
        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        console_log!(
            "{} {}",
            gl.drawing_buffer_width(),
            gl.drawing_buffer_height()
        );

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

    pub fn cull_face(&self) -> Option<u32> {
        self.cull_face_mode
    }

    pub fn set_cull_face(&mut self, cull_face_mode: Option<u32>) {
        self.cull_face_mode = cull_face_mode;
        match self.cull_face_mode {
            Some(cull_face_mode) => {
                self.gl.enable(WebGl2RenderingContext::CULL_FACE);
                self.gl.cull_face(cull_face_mode)
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

impl WebGL2Render {
    fn prepare(&self, scene: &Scene) -> Vec<*const Entity> {
        let view = *scene.active_camera().view();
        let proj = *scene.active_camera().proj();

        let mut entities = Vec::new();
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
            if entity.geometry().is_some() && entity.material().is_some() {
                entities.push(entity as *const Entity);
            }

            // adds children to rolling list
            rollings.extend(entity.children().iter().map(|child| child.as_ref()));
        }

        entities
    }

    pub fn render(&mut self, scene: &Scene) -> Result<(), JsError> {
        let mut bind_prom = 0.0;
        let mut unbind_prom = 0.0;
        let mut attr = 0.0;
        let mut prep = 0.0;
        let mut unif = 0.0;
        let mut rend = 0.0;

        let gl = &self.gl;

        // clear scene
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);

        // collects entities and render each
        let camera_direction = *scene.active_camera().direction();
        let camera_position = *scene.active_camera().position();
        let start = window().performance().unwrap().now();
        let entities = self.prepare(scene);
        let end = window().performance().unwrap().now();
        prep += end - start;
        for entity in entities {
            let entity = unsafe { &*entity };
            let (geometry, material) = { (entity.geometry().unwrap(), entity.material().unwrap()) };

            let (program, attributes_locations, uniform_locations) =
                match self.program_store.program_or_compile(material) {
                    Ok(material) => material,
                    Err(err) => {
                        // should log here
                        console_log!("2");
                        continue;
                    }
                };

            // binds program
            let start = window().performance().unwrap().now();
            gl.use_program(Some(program));
            let end = window().performance().unwrap().now();
            bind_prom += end - start;

            // binds attribute values
            let start = window().performance().unwrap().now();
            for (binding, location) in attributes_locations {
                let value = match binding {
                    AttributeBinding::GeometryPosition => geometry.vertices(),
                    AttributeBinding::GeometryTextureCoordinate => geometry.texture_coordinates(),
                    AttributeBinding::GeometryNormal => geometry.normals(),
                    AttributeBinding::FromGeometry(name) => geometry.attribute_value(name.as_str()),
                    AttributeBinding::FromMaterial(name) => material.attribute_value(name.as_str()),
                    AttributeBinding::FromEntity(name) => entity.attribute_value(name.as_str()),
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
                        size,
                        data_type,
                        normalized,
                        stride,
                        offset,
                    } => {
                        let buffer = match self
                            .buffer_store
                            .buffer_or_create(descriptor.as_ref(), target)
                        {
                            Ok(buffer) => buffer,
                            Err(err) => {
                                // should log error
                                console_log!("4");
                                continue;
                            }
                        };

                        gl.bind_buffer(target.to_gl_enum(), Some(buffer));
                        gl.vertex_attrib_pointer_with_i32(
                            *location,
                            size.to_i32(),
                            data_type.to_gl_enum(),
                            *normalized,
                            *stride,
                            *offset,
                        );
                        gl.enable_vertex_attrib_array(*location);
                        gl.bind_buffer(target.to_gl_enum(), None);
                    }
                    AttributeValue::Vertex1f(x) => gl.vertex_attrib1f(*location, *x),
                    AttributeValue::Vertex2f(x, y) => gl.vertex_attrib2f(*location, *x, *y),
                    AttributeValue::Vertex3f(x, y, z) => gl.vertex_attrib3f(*location, *x, *y, *z),
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
            let end = window().performance().unwrap().now();
            attr += end - start;

            // binds uniform values
            let start = window().performance().unwrap().now();
            for (binding, location) in uniform_locations {
                let value = match binding {
                    UniformBinding::ModelMatrix => Some(Ncor::Owned(UniformValue::Matrix4 {
                        data: Box::new(*entity.composed_model_matrix().borrow()),
                        transpose: false,
                        src_offset: 0,
                        src_length: 0,
                    })),
                    UniformBinding::NormalMatrix => Some(Ncor::Owned(UniformValue::Matrix4 {
                        data: Box::new(*entity.composed_normal_matrix().borrow()),
                        transpose: false,
                        src_offset: 0,
                        src_length: 0,
                    })),
                    UniformBinding::ModelViewMatrix => Some(Ncor::Owned(UniformValue::Matrix4 {
                        data: Box::new(*entity.composed_model_view_matrix().borrow()),
                        transpose: false,
                        src_offset: 0,
                        src_length: 0,
                    })),
                    UniformBinding::ModelViewProjMatrix => {
                        Some(Ncor::Owned(UniformValue::Matrix4 {
                            data: Box::new(*entity.composed_model_view_proj_matrix().borrow()),
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
                    UniformBinding::FromGeometry(name) => geometry.uniform_value(name.as_str()),
                    UniformBinding::FromMaterial(name) => material.uniform_value(name.as_str()),
                    UniformBinding::FromEntity(name) => entity.uniform_value(name.as_str()),
                };
                let Some(value) = value else {
                    // should log warning
                    console_log!("5");
                    continue;
                };

                match value.as_ref() {
                    UniformValue::UnsignedInteger1(x) => gl.uniform1ui(Some(location), *x),
                    UniformValue::UnsignedInteger2(x, y) => gl.uniform2ui(Some(location), *x, *y),
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
                    } => self
                        .gl
                        .uniform1fv_with_f32_array_and_src_offset_and_src_length(
                            Some(location),
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                    UniformValue::FloatVector2 {
                        data,
                        src_offset,
                        src_length,
                    } => self
                        .gl
                        .uniform2fv_with_f32_array_and_src_offset_and_src_length(
                            Some(location),
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                    UniformValue::FloatVector3 {
                        data,
                        src_offset,
                        src_length,
                    } => self
                        .gl
                        .uniform3fv_with_f32_array_and_src_offset_and_src_length(
                            Some(location),
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                    UniformValue::FloatVector4 {
                        data,
                        src_offset,
                        src_length,
                    } => self
                        .gl
                        .uniform4fv_with_f32_array_and_src_offset_and_src_length(
                            Some(location),
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                    UniformValue::IntegerVector1 {
                        data,
                        src_offset,
                        src_length,
                    } => self
                        .gl
                        .uniform1iv_with_i32_array_and_src_offset_and_src_length(
                            Some(location),
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                    UniformValue::IntegerVector2 {
                        data,
                        src_offset,
                        src_length,
                    } => self
                        .gl
                        .uniform2iv_with_i32_array_and_src_offset_and_src_length(
                            Some(location),
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                    UniformValue::IntegerVector3 {
                        data,
                        src_offset,
                        src_length,
                    } => self
                        .gl
                        .uniform3iv_with_i32_array_and_src_offset_and_src_length(
                            Some(location),
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                    UniformValue::IntegerVector4 {
                        data,
                        src_offset,
                        src_length,
                    } => self
                        .gl
                        .uniform4iv_with_i32_array_and_src_offset_and_src_length(
                            Some(location),
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                    UniformValue::UnsignedIntegerVector1 {
                        data,
                        src_offset,
                        src_length,
                    } => self
                        .gl
                        .uniform1uiv_with_u32_array_and_src_offset_and_src_length(
                            Some(location),
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                    UniformValue::UnsignedIntegerVector2 {
                        data,
                        src_offset,
                        src_length,
                    } => self
                        .gl
                        .uniform2uiv_with_u32_array_and_src_offset_and_src_length(
                            Some(location),
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                    UniformValue::UnsignedIntegerVector3 {
                        data,
                        src_offset,
                        src_length,
                    } => self
                        .gl
                        .uniform3uiv_with_u32_array_and_src_offset_and_src_length(
                            Some(location),
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                    UniformValue::UnsignedIntegerVector4 {
                        data,
                        src_offset,
                        src_length,
                    } => self
                        .gl
                        .uniform4uiv_with_u32_array_and_src_offset_and_src_length(
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
                    } => self
                        .gl
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
                    } => self
                        .gl
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
                    } => self
                        .gl
                        .uniform_matrix4fv_with_f32_array_and_src_offset_and_src_length(
                            Some(location),
                            *transpose,
                            data.as_ref().as_ref(),
                            *src_offset,
                            *src_length,
                        ),
                }
            }
            let end = window().performance().unwrap().now();
            unif += end - start;

            // draw!
            let start = window().performance().unwrap().now();
            match geometry.draw() {
                Draw::Arrays { mode, first, count } => {
                    gl.draw_arrays(mode.to_gl_enum(), first, count)
                }
                Draw::Elements {
                    mode,
                    count,
                    element_type,
                    offset,
                } => gl.draw_elements_with_i32(
                    mode.to_gl_enum(),
                    count,
                    element_type.to_gl_enum(),
                    offset,
                ),
            }
            let end = window().performance().unwrap().now();
            rend += end - start;

            self.first = false;

            // unbinds program after drawing
            let start = window().performance().unwrap().now();
            gl.use_program(None);
            let end = window().performance().unwrap().now();
            unbind_prom += end - start;
        }

        console_log!(
            "bind {:.2}ms, prep {:.2}ms, attr {:.2}ms, unif {:.2}ms, rend {:.2}ms, unbind {:.2}ms, total {:.2}ms",
            bind_prom,
            prep,
            attr,
            unif,
            rend,
            unbind_prom,
            bind_prom + prep + attr + unif + rend + unbind_prom
        );

        Ok(())
    }
}
