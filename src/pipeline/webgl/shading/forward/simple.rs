use web_sys::WebGlTexture;

use crate::{
    pipeline::webgl::{
        collector::CollectedEntities,
        shading::{draw_entities, DrawState},
    },
    renderer::webgl::{
        buffer::Buffer,
        error::Error,
        framebuffer::{
            AttachmentProvider, Framebuffer, FramebufferAttachment, FramebufferBuilder,
            FramebufferTarget,
        },
        renderbuffer::RenderbufferInternalFormat,
        state::FrameState,
        texture::TextureColorFormat,
    },
};

pub struct StandardSimpleShading {
    framebuffer: Option<Framebuffer>,
}

impl StandardSimpleShading {
    pub fn new() -> Self {
        Self { framebuffer: None }
    }

    fn framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(
                FramebufferBuilder::new()
                    .set_color_attachment0(AttachmentProvider::new_texture(
                        TextureColorFormat::RGBA8,
                    ))
                    .with_depth_stencil_attachment(AttachmentProvider::new_renderbuffer(
                        RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                    )),
            )
        })
    }

    pub fn draw_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer
            .as_ref()
            .and_then(|f| f.texture(FramebufferAttachment::COLOR_ATTACHMENT0))
    }

    pub unsafe fn draw(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
        universal_ubo: &Buffer,
        lights_ubo: Option<&Buffer>,
    ) -> Result<(), Error> {
        let framebuffer = self.framebuffer(state);
        framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        framebuffer.clear_buffers()?;
        draw_entities(
            state,
            &DrawState::Draw {
                universal_ubo,
                lights_ubo,
                bloom: false,
            },
            collected_entities,
        )?;
        framebuffer.unbind();
        Ok(())
    }
}
