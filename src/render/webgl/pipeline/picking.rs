use std::borrow::Cow;

use gl_matrix4rust::{GLF32, vec3::Vec3};
use log::warn;
use wasm_bindgen::JsCast;
use web_sys::{js_sys::Uint32Array, HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    entity::Entity,
    material::Transparency,
    render::webgl::{
        error::Error,
        framebuffer::{
            Framebuffer, FramebufferAttachment, FramebufferDrawBuffer, FramebufferTarget,
            RenderbufferProvider, TextureProvider,
        },
        program::{FragmentShaderSource, ProgramSource, VertexShaderSource},
        renderbuffer::RenderbufferInternalFormat,
        state::FrameState,
        texture::{TextureDataType, TextureFormat, TextureInternalFormat},
        uniform::UniformValue,
    },
};

use super::collector::CollectedEntities;

pub struct StandardPicking {
    framebuffer: Option<Framebuffer>,
    pixel: Uint32Array,

    last_gl: Option<WebGl2RenderingContext>,
    last_entities: Vec<*mut Entity>,
}

impl StandardPicking {
    pub fn new() -> Self {
        Self {
            framebuffer: None,
            pixel: Uint32Array::new_with_length(4),

            last_gl: None,
            last_entities: Vec::new(),
        }
    }

    fn framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                [
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT0,
                        TextureInternalFormat::R32UI,
                        TextureFormat::RED_INTEGER,
                        TextureDataType::UNSIGNED_INT,
                        0,
                    ),
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT1,
                        TextureInternalFormat::RGBA32UI,
                        TextureFormat::RGBA_INTEGER,
                        TextureDataType::UNSIGNED_INT,
                        0,
                    ),
                ],
                [RenderbufferProvider::new(
                    FramebufferAttachment::DEPTH_ATTACHMENT,
                    RenderbufferInternalFormat::DEPTH_COMPONENT24,
                )],
                [
                    FramebufferDrawBuffer::COLOR_ATTACHMENT0,
                    FramebufferDrawBuffer::COLOR_ATTACHMENT1,
                ],
                None,
            )
        })
    }

    pub fn render(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
    ) -> Result<(), Error> {
        self.last_gl = None;
        self.last_entities.clear();

        self.framebuffer(&state)
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);

        state
            .gl()
            .clear_bufferuiv_with_u32_array(WebGl2RenderingContext::COLOR, 0, &[0, 0, 0, 0]);
        state
            .gl()
            .clear_bufferuiv_with_u32_array(WebGl2RenderingContext::COLOR, 1, &[0, 0, 0, 0]);
        state
            .gl()
            .clear_bufferfv_with_f32_array(WebGl2RenderingContext::DEPTH, 0, &[1.0]);

        let entities = collected_entities.entities();
        let max_entities_len = (u32::MAX - 1) as usize;
        if entities.len() > max_entities_len {
            warn!(
                target: "Picking",
                "too many entities, some entities maybe ignored."
            );
        }

        // prepare material
        let program = state.program_store_mut().use_program(&PickingMaterial)?;
        let position_location = program
            .get_or_retrieve_attribute_locations(POSITION_ATTRIBUTE_NAME)
            .unwrap();
        let index_location = program.get_or_retrieve_uniform_location(INDEX_UNIFORM_NAME);
        let model_matrix_location =
            program.get_or_retrieve_uniform_location(MODEL_MATRIX_UNIFORM_NAME);

        state.bind_uniform_value_by_variable_name(
            program,
            VIEW_PROJ_MATRIX_UNIFORM_NAME,
            UniformValue::Matrix4 {
                data: state.camera().view_proj_matrix().gl_f32(),
                transpose: false,
            },
        )?;

        // render each entity by picking material
        for (index, entity) in entities.into_iter().enumerate() {
            // skips if overflows
            if index > max_entities_len {
                break;
            }

            let entity = entity.entity();
            let Some(geometry) = entity.geometry() else {
                continue;
            };
            let Some(vertices) = geometry.vertices() else {
                continue;
            };

            // do not pick entity has no material or transparent material or not ready
            if let Some(material) = entity.material() {
                if material.transparency() == Transparency::Transparent {
                    continue;
                } else if !material.ready() {
                    continue;
                }
            } else {
                continue;
            }

            state.gl().uniform_matrix4fv_with_f32_array(
                model_matrix_location.as_ref(),
                false,
                &entity.compose_model_matrix().gl_f32(),
            );
            state
                .gl()
                .uniform1ui(index_location.as_ref(), (index + 1) as u32);
            let bound_attributes = state.bind_attribute_value(position_location, vertices)?;
            state.draw(&geometry.draw())?;
            state.unbind_attributes(bound_attributes);
        }

        self.framebuffer(&state).unbind();
        state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);

        self.last_gl = Some(state.gl().clone());
        self.last_entities.extend(
            collected_entities
                .entities()
                .into_iter()
                .map(|e| e.entity_mut() as *mut Entity),
        );

        Ok(())
    }

    /// Returns picked entity.
    pub unsafe fn pick_entity<'a, 'b>(
        &'a mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<&'b mut Entity>, Error> {
        let Some(fbo) = self.framebuffer.as_mut() else {
            return Ok(None);
        };
        let Some(gl) = self.last_gl.as_ref() else {
            return Ok(None);
        };
        let Some(canvas) = gl
            .canvas()
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
        else {
            return Ok(None);
        };

        fbo.read_pixels(
            window_position_x,
            canvas.height() as i32 - window_position_y,
            1,
            1,
            TextureFormat::RED_INTEGER,
            TextureDataType::UNSIGNED_INT,
            &self.pixel,
            0,
        )?;

        let index = self.pixel.get_index(0) as usize;
        if index > 0 {
            let entity = self
                .last_entities
                .get_mut(index - 1)
                .map(|entity| unsafe { &mut **entity });
            if let Some(entity) = entity {
                return Ok(Some(entity));
            }
        }

        Ok(None)
    }

    /// Returns picked position.
    pub unsafe fn pick_position(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<Vec3>, Error> {
        let Some(fbo) = self.framebuffer.as_mut() else {
            return Ok(None);
        };
        let Some(gl) = self.last_gl.as_ref() else {
            return Ok(None);
        };
        let Some(canvas) = gl
            .canvas()
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
        else {
            return Ok(None);
        };

        fbo.read_pixels_with_read_buffer(
            FramebufferDrawBuffer::COLOR_ATTACHMENT1,
            window_position_x,
            canvas.height() as i32 - window_position_y,
            1,
            1,
            TextureFormat::RGBA_INTEGER,
            TextureDataType::UNSIGNED_INT,
            &self.pixel,
            0,
        )?;

        let position = [
            f32::from_ne_bytes(self.pixel.get_index(0).to_ne_bytes()),
            f32::from_ne_bytes(self.pixel.get_index(1).to_ne_bytes()),
            f32::from_ne_bytes(self.pixel.get_index(2).to_ne_bytes()),
            f32::from_ne_bytes(self.pixel.get_index(3).to_ne_bytes()),
        ]; // converts unsigned int back to float
        if position != [0.0, 0.0, 0.0, 0.0] {
            Ok(Some(Vec3::<f64>::new(
                position[0] as f64,
                position[1] as f64,
                position[2] as f64,
            )))
        } else {
            Ok(None)
        }
    }
}

const POSITION_ATTRIBUTE_NAME: &'static str = "a_Position";
const INDEX_UNIFORM_NAME: &'static str = "u_Index";
const MODEL_MATRIX_UNIFORM_NAME: &'static str = "u_ModelMatrix";
const VIEW_PROJ_MATRIX_UNIFORM_NAME: &'static str = "u_ViewProjMatrix";

struct PickingMaterial;

impl ProgramSource for PickingMaterial {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("PickingMaterial")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/picking.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/picking.frag")))
    }
}
