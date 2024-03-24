use web_sys::WebGlTexture;

use crate::{
    pipeline::webgl::{
        collector::CollectedEntities,
        shading::{draw_entities, DrawState},
    },
    renderer::webgl::{
        error::Error,
        framebuffer::{
            AttachmentSource, Framebuffer, FramebufferAttachmentTarget, FramebufferBuilder,
            FramebufferTarget,
        },
        renderbuffer::RenderbufferInternalFormat,
        state::FrameState,
        texture::TextureUncompressedInternalFormat,
    },
};

pub struct StandardSimpleShading {
    framebuffer: Framebuffer,
}

impl StandardSimpleShading {
    pub fn new() -> Self {
        Self {
            framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA8,
                ))
                .set_depth_stencil_attachment(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                ))
                .build(),
        }
    }

    pub fn draw_texture(&self) -> Result<Option<&WebGlTexture>, Error> {
        self.framebuffer
            .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)
    }

    pub unsafe fn draw(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
        lighting: bool,
    ) -> Result<(), Error> {
        self.framebuffer.init(state.gl())?;
        self.framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.framebuffer.clear_buffers()?;
        draw_entities(
            state,
            DrawState::Draw {
                lighting,
                bloom: false,
            },
            collected_entities,
        )?;
        self.framebuffer.unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        Ok(())
    }
}
