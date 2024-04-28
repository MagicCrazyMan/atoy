use std::{
    borrow::Cow,
    cell::RefCell,
    rc::{Rc, Weak},
};

use gl_matrix4rust::vec3::Vec3;
use log::warn;
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{js_sys::Uint32Array, HtmlCanvasElement, WebGl2RenderingContext};

use crate::{
    core::webgl::client_wait::ClientWaitAsync,
    entity::Entity,
    material::Transparency,
    pipeline::webgl::{
        collector::CollectedEntities, UBO_UNIVERSAL_UNIFORMS_BLOCK_BINDING,
        UBO_UNIVERSAL_UNIFORM_BLOCK_MOUNT_POINT,
    },
    renderer::webgl::{
        draw::Draw,
        error::Error,
        framebuffer::{
            AttachmentSource, ClearPolicy, Framebuffer, FramebufferBuilder, FramebufferTarget,
            OperableBuffer,
        },
        program::{Define, ProgramSource},
        renderbuffer::RenderbufferInternalFormat,
        state::FrameState,
        texture::{
            TextureUncompressedInternalFormat, TextureUncompressedPixelDataType,
            TextureUncompressedPixelFormat,
        },
        uniform::{UniformBinding, UniformValue},
    },
};

pub struct StandardPicking {
    framebuffer: Framebuffer,
    pixel: Uint32Array,

    gl: Option<WebGl2RenderingContext>,
}

impl StandardPicking {
    pub fn new() -> Self {
        Self {
            framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_texture_with_clear_policy(
                    TextureUncompressedInternalFormat::R32UI,
                    ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
                ))
                .set_color_attachment1(AttachmentSource::new_texture_with_clear_policy(
                    TextureUncompressedInternalFormat::RGBA32UI,
                    ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
                ))
                .set_depth_attachment(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::DEPTH_COMPONENT24,
                ))
                .build(),
            pixel: Uint32Array::new_with_length(4),

            gl: None,
        }
    }

    pub fn draw(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
    ) -> Result<(), Error> {
        // skips if collected entities is empty
        if collected_entities.entities().len() == 0 {
            return Ok(());
        }

        state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
        self.framebuffer.init(state.gl())?;
        self.framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.framebuffer.clear_buffers()?;

        let entities = collected_entities.entities();
        let max_entities_len = (u32::MAX - 1) as usize;
        if entities.len() > max_entities_len {
            warn!(
                target: "Picking",
                "too many entities, some entities maybe ignored."
            );
        }

        let program = state
            .program_store_mut()
            .get_or_compile_program(&PickingShaderProvider)?;
        program.use_program()?;
        program.mount_uniform_block_by_binding(
            &UBO_UNIVERSAL_UNIFORMS_BLOCK_BINDING,
            UBO_UNIVERSAL_UNIFORM_BLOCK_MOUNT_POINT,
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
            };

            program.bind_uniform_value_by_binding(
                &INDEX_UNIFORM_BINDING,
                &UniformValue::UnsignedInteger1((index + 1) as u32),
                None,
            )?;
            program.bind_uniforms(Some(&state), Some(&*entity), Some(geometry), None)?;
            program.bind_attributes(Some(&state), Some(&*entity), Some(geometry), None)?;
            Draw::from_geometry(geometry).draw(state.gl(), Some(state.buffer_store()))?;
            program.unbind_attributes()?;
        }

        self.framebuffer
            .unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        program.unuse_program()?;
        state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);

        self.gl = Some(state.gl().clone());

        Ok(())
    }

    /// Returns picked entity.
    pub fn pick_entity(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
        collected_entities: &CollectedEntities,
    ) -> Result<Option<Rc<RefCell<dyn Entity>>>, Error> {
        if collected_entities.entities().len() == 0 {
            return Ok(None);
        }
        let Some(gl) = self.gl.as_ref() else {
            return Ok(None);
        };
        let Some(canvas) = gl
            .canvas()
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
        else {
            return Ok(None);
        };

        self.framebuffer.bind(FramebufferTarget::READ_FRAMEBUFFER)?;
        self.framebuffer.read_pixels(
            window_position_x,
            canvas.height() as i32 - window_position_y,
            1,
            1,
            TextureUncompressedPixelFormat::RED_INTEGER,
            TextureUncompressedPixelDataType::UNSIGNED_INT,
            &self.pixel,
            0,
        )?;
        self.framebuffer
            .unbind(FramebufferTarget::READ_FRAMEBUFFER)?;

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

    /// Returns picked entity.
    pub async fn pick_entity_async(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
        last_entities: Vec<Weak<RefCell<dyn Entity>>>,
    ) -> Result<Option<Uuid>, Error> {
        if last_entities.len() == 0 {
            return Ok(None);
        }
        let Some(gl) = self.gl.as_ref() else {
            return Ok(None);
        };
        let Some(canvas) = gl
            .canvas()
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
        else {
            return Ok(None);
        };

        let client_await = ClientWaitAsync::new(gl.clone(), 0, 10, None);
        client_await.wait().await.unwrap();
        self.framebuffer.bind(FramebufferTarget::READ_FRAMEBUFFER)?;
        self.framebuffer.read_pixels(
            window_position_x,
            canvas.height() as i32 - window_position_y,
            1,
            1,
            TextureUncompressedPixelFormat::RED_INTEGER,
            TextureUncompressedPixelDataType::UNSIGNED_INT,
            &self.pixel,
            0,
        )?;
        self.framebuffer
            .unbind(FramebufferTarget::READ_FRAMEBUFFER)?;

        let index = self.pixel.get_index(0) as usize;
        if index >= 1 {
            Ok(last_entities
                .get(index - 1)
                .and_then(|entity| entity.upgrade())
                .map(|entity| entity.borrow().id().clone()))
        } else {
            Ok(None)
        }
    }

    /// Returns picked position.
    pub fn pick_position(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
        collected_entities: &CollectedEntities,
    ) -> Result<Option<Vec3>, Error> {
        if collected_entities.entities().len() == 0 {
            return Ok(None);
        }
        let Some(gl) = self.gl.as_ref() else {
            return Ok(None);
        };
        let Some(canvas) = gl
            .canvas()
            .and_then(|canvas| canvas.dyn_into::<HtmlCanvasElement>().ok())
        else {
            return Ok(None);
        };

        self.framebuffer.bind(FramebufferTarget::READ_FRAMEBUFFER)?;
        self.framebuffer
            .set_read_buffer(OperableBuffer::COLOR_ATTACHMENT1)?;
        self.framebuffer.read_pixels(
            window_position_x,
            canvas.height() as i32 - window_position_y,
            1,
            1,
            TextureUncompressedPixelFormat::RGBA_INTEGER,
            TextureUncompressedPixelDataType::UNSIGNED_INT,
            &self.pixel,
            0,
        )?;
        self.framebuffer
            .unbind(FramebufferTarget::READ_FRAMEBUFFER)?;

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

const INDEX_UNIFORM_NAME: &'static str = "u_Index";
const INDEX_UNIFORM_BINDING: UniformBinding =
    UniformBinding::Custom(Cow::Borrowed(INDEX_UNIFORM_NAME));

struct PickingShaderProvider;

impl ProgramSource for PickingShaderProvider {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("Picking")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../../shaders/picking.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../../shaders/picking.frag"))
    }

    fn universal_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}
