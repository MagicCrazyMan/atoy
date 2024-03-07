use std::borrow::Cow;

use gl_matrix4rust::{vec3::Vec3, GLF32};
use log::warn;
use wasm_bindgen::JsCast;
use web_sys::{js_sys::Uint32Array, HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    entity::Entity,
    material::Transparency,
    pipeline::webgl::collector::CollectedEntities,
    renderer::webgl::{
        error::Error,
        framebuffer::{
            AttachmentProvider, ClearPolicy, Framebuffer, FramebufferBuilder, FramebufferTarget,
            OperableBuffer,
        },
        program::{Define, ShaderProvider},
        renderbuffer::RenderbufferInternalFormat,
        state::FrameState,
        texture::{TextureColorFormat, TextureDataType, TextureFormat},
        uniform::UniformValue,
    },
    share::Share,
};

pub struct StandardPicking {
    framebuffer: Option<Framebuffer>,
    pixel: Uint32Array,

    gl: Option<WebGl2RenderingContext>,
}

impl StandardPicking {
    pub fn new() -> Self {
        Self {
            framebuffer: None,
            pixel: Uint32Array::new_with_length(4),

            gl: None,
        }
    }

    fn framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(
                FramebufferBuilder::new()
                    .set_color_attachment0(AttachmentProvider::new_texture_with_clear_policy(
                        TextureColorFormat::R32UI,
                        ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
                    ))
                    .set_color_attachment1(AttachmentProvider::new_texture_with_clear_policy(
                        TextureColorFormat::RGBA32UI,
                        ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
                    ))
                    .with_depth_attachment(AttachmentProvider::new_renderbuffer(
                        RenderbufferInternalFormat::DEPTH_COMPONENT24,
                    )),
            )
        })
    }

    pub unsafe fn draw(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
    ) -> Result<(), Error> {
        // skips if collected entities is empty
        if collected_entities.entities().len() == 0 {
            return Ok(());
        }

        state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
        self.framebuffer(&state)
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.framebuffer(&state).clear_buffers()?;

        let entities = collected_entities.entities();
        let max_entities_len = (u32::MAX - 1) as usize;
        if entities.len() > max_entities_len {
            warn!(
                target: "Picking",
                "too many entities, some entities maybe ignored."
            );
        }

        // prepare material
        let program = state
            .program_store_mut()
            .use_program(&PickingShaderProvider)?;

        state.bind_uniform_value_by_variable_name(
            program,
            VIEW_PROJ_MATRIX_UNIFORM_NAME,
            &UniformValue::Matrix4 {
                data: state.camera().view_proj_matrix().gl_f32(),
                transpose: false,
            },
        )?;

        // render each entity by picking material
        for (index, entity) in entities.into_iter().enumerate() {
            let Some(entity) = entity.upgrade() else {
                continue;
            };
            let entity = entity.borrow();

            // skips if overflows
            if index > max_entities_len {
                break;
            }

            let Some(geometry) = entity.geometry() else {
                continue;
            };

            // do not pick entity has no material or has transparent material or not ready
            if let Some(material) = entity.material() {
                if material.transparency() == Transparency::Transparent {
                    continue;
                }
                if !material.ready() {
                    continue;
                }
            } else {
                continue;
            }

            state.bind_uniform_value_by_variable_name(
                program,
                MODEL_MATRIX_UNIFORM_NAME,
                &UniformValue::Matrix4 {
                    data: entity.compose_model_matrix().gl_f32(),
                    transpose: false,
                },
            )?;
            state.bind_uniform_value_by_variable_name(
                program,
                INDEX_UNIFORM_NAME,
                &UniformValue::UnsignedInteger1((index + 1) as u32),
            )?;

            let bound_attributes = state.bind_attribute_value_by_variable_name(
                program,
                POSITION_ATTRIBUTE_NAME,
                &geometry.positions(),
            )?;
            state.draw(&geometry.draw())?;
            state.unbind_attributes(bound_attributes);
        }

        self.framebuffer(&state).unbind();
        state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);

        self.gl = Some(state.gl().clone());

        Ok(())
    }

    /// Returns picked entity.
    pub unsafe fn pick_entity(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
        collected_entities: &CollectedEntities,
    ) -> Result<Option<Share<dyn Entity>>, Error> {
        if collected_entities.entities().len() == 0 {
            return Ok(None);
        }
        let Some(fbo) = self.framebuffer.as_mut() else {
            return Ok(None);
        };
        let Some(gl) = self.gl.as_ref() else {
            return Ok(None);
        };
        let Some(canvas) = gl
            .canvas()
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
        else {
            return Ok(None);
        };

        fbo.bind(FramebufferTarget::READ_FRAMEBUFFER)?;
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
        fbo.unbind();

        let index = self.pixel.get_index(0) as usize;
        if index >= 1 {
            Ok(collected_entities
                .entities()
                .get(index - 1)
                .and_then(|entity| entity.upgrade()))
        } else {
            Ok(None)
        }
    }

    /// Returns picked position.
    pub unsafe fn pick_position(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
        collected_entities: &CollectedEntities,
    ) -> Result<Option<Vec3>, Error> {
        if collected_entities.entities().len() == 0 {
            return Ok(None);
        }
        let Some(fbo) = self.framebuffer.as_mut() else {
            return Ok(None);
        };
        let Some(gl) = self.gl.as_ref() else {
            return Ok(None);
        };
        let Some(canvas) = gl
            .canvas()
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
        else {
            return Ok(None);
        };

        fbo.bind(FramebufferTarget::READ_FRAMEBUFFER)?;
        fbo.set_read_buffer(OperableBuffer::COLOR_ATTACHMENT1)?;
        fbo.read_pixels(
            window_position_x,
            canvas.height() as i32 - window_position_y,
            1,
            1,
            TextureFormat::RGBA_INTEGER,
            TextureDataType::UNSIGNED_INT,
            &self.pixel,
            0,
        )?;
        fbo.unbind();

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

struct PickingShaderProvider;

impl ShaderProvider for PickingShaderProvider {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("Picking")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../../shaders/picking.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../../shaders/picking.frag"))
    }

    fn universal_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn vertex_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn fragment_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}
