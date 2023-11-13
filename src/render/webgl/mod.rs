use std::{borrow::Cow, collections::VecDeque};

use gl_matrix4rust::vec4::Vec4;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsError, JsValue};
use web_sys::{HtmlCanvasElement, WebGl2RenderingContext};

use crate::{entity::Entity, scene::Scene};

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
}

#[wasm_bindgen]
impl WebGL2Render {
    /// Constructs a new WebGL2 render.
    #[wasm_bindgen(constructor)]
    pub fn new(
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
    fn prepare<'a, 'b>(&'a self, scene: &'b mut Scene) -> Vec<*const Entity> {
        let view = *scene.active_camera().view();
        let proj = *scene.active_camera().proj();

        let mut entities = Vec::new();
        let mut rollings = VecDeque::from([scene.root_entity_mut()]);
        while let Some(entity) = rollings.pop_front() {
            // update composed matrices for all entities
            let composed_model = match entity.parent() {
                Some(parent) => *parent.matrices().model() * *entity.matrices().model(),
                None => *entity.matrices().model(),
            };
            let composed_model_view = view * composed_model;
            let composed_model_view_proj = proj * composed_model_view;
            if let Err(err) = entity.matrices_mut().set_composed_model(composed_model) {
                // if meet error, skip this entity and all its children
                // should log error
                continue;
            }
            entity
                .matrices_mut()
                .set_composed_model_view(composed_model_view);
            entity
                .matrices_mut()
                .set_composed_model_view_proj(composed_model_view_proj);

            // filters any entity that has no geometry or material
            if entity.geometry().is_some() && entity.material().is_some() {
                entities.push(entity as *const Entity);
                // entities.push(RenderEntity {
                //     matrices: entity.matrices(),
                //     geometry,
                //     material,
                // });
            }

            // adds children to rolling list
            rollings.extend(entity.children_mut().iter_mut().map(|child| child.as_mut()));
        }

        entities
    }

    pub fn render(&mut self, scene: &mut Scene) -> Result<(), JsError> {
        let gl = &self.gl;

        // clear scene
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);

        // collects entities and render each
        let camera_direction = *scene.active_camera().direction();
        let camera_position = *scene.active_camera().position();
        let entities = self.prepare(scene);
        for entity in entities {
            let entity = unsafe { &*entity };
            let (matrices, geometry, material) = {
                (
                    entity.matrices(),
                    entity.geometry().unwrap(),
                    entity.material().unwrap(),
                )
            };

            let (program, attributes_locations, uniform_locations) =
                match self.program_store.program_or_compile(material) {
                    Ok(material) => material,
                    Err(err) => {
                        // should log here
                        continue;
                    }
                };

            // binds program
            gl.use_program(Some(program));
            // binds attribute values
            for (binding, location) in attributes_locations {
                let value = match binding {
                    AttributeBinding::GeometryPosition => geometry.vertices(),
                    AttributeBinding::GeometryTextureCoordinate => geometry.texture_coordinates(),
                    AttributeBinding::GeometryNormal => geometry.normals(),
                    AttributeBinding::FromGeometry(name) => geometry
                        .attribute_values()
                        .get(name.as_str())
                        .map(|value| Cow::Borrowed(value)),
                    AttributeBinding::FromMaterial(name) => material
                        .attribute_values()
                        .get(name.as_str())
                        .map(|value| Cow::Borrowed(value)),
                    AttributeBinding::FromEntity(name) => entity
                        .attribute_values()
                        .get(name.as_str())
                        .map(|value| Cow::Borrowed(value)),
                };
                let Some(value) = value else {
                    // should log warning
                    continue;
                };

                match value.as_ref() {
                    AttributeValue::ArrayBuffer {
                        descriptor,
                        target,
                        size,
                        data_type,
                        normalized,
                        stride,
                        offset,
                    } => {
                        let mut descriptor = descriptor.borrow_mut();
                        let buffer =
                            match self.buffer_store.buffer_or_create(&mut descriptor, target) {
                                Ok(buffer) => buffer,
                                Err(err) => {
                                    // should log error
                                    continue;
                                }
                            };

                        gl.bind_buffer(target.to_gl_enum(), Some(buffer));
                        gl.vertex_attrib_pointer_with_i32(
                            *location,
                            *size,
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

            // binds uniform values
            for (binding, location) in uniform_locations {
                let value = match binding {
                    UniformBinding::ModelMatrix => Some(Cow::Owned(UniformValue::Matrix4 {
                        data: Cow::Borrowed(matrices.composed_model().raw()),
                        transpose: false,
                        src_offset: 0,
                        src_length: 0,
                    })),
                    UniformBinding::NormalMatrix => Some(Cow::Owned(UniformValue::Matrix4 {
                        data: Cow::Borrowed(matrices.composed_normal().raw()),
                        transpose: false,
                        src_offset: 0,
                        src_length: 0,
                    })),
                    UniformBinding::ModelViewMatrix => Some(Cow::Owned(UniformValue::Matrix4 {
                        data: Cow::Borrowed(matrices.composed_model_view().raw()),
                        transpose: false,
                        src_offset: 0,
                        src_length: 0,
                    })),
                    UniformBinding::ModelViewProjMatrix => {
                        Some(Cow::Owned(UniformValue::Matrix4 {
                            data: Cow::Borrowed(matrices.composed_model_view_proj().raw()),
                            transpose: false,
                            src_offset: 0,
                            src_length: 0,
                        }))
                    }
                    UniformBinding::ActiveCameraPosition => {
                        Some(Cow::Owned(UniformValue::FloatVector3 {
                            data: Cow::Borrowed(camera_position.raw()),
                            src_offset: 0,
                            src_length: 0,
                        }))
                    }
                    UniformBinding::ActiveCameraDirection => {
                        Some(Cow::Owned(UniformValue::FloatVector3 {
                            data: Cow::Borrowed(camera_direction.raw()),
                            src_offset: 0,
                            src_length: 0,
                        }))
                    }
                    UniformBinding::FromGeometry(name) => geometry
                        .uniform_values()
                        .get(name.as_str())
                        .map(|value| Cow::Borrowed(value)),
                    UniformBinding::FromMaterial(name) => material
                        .uniform_values()
                        .get(name.as_str())
                        .map(|value| Cow::Borrowed(value)),
                    UniformBinding::FromEntity(name) => entity
                        .uniform_values()
                        .get(name.as_str())
                        .map(|value| Cow::Borrowed(value)),
                };
                let Some(value) = value else {
                    // should log warning
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
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
                            &data,
                            *src_offset,
                            *src_length,
                        ),
                }
            }

            // draw!
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

            // unbinds program after drawing
            gl.use_program(None);
        }

        Ok(())
    }
}
