use web_sys::WebGlTexture;

use crate::{
    pipeline::webgl::{
        collector::CollectedEntities,
        shading::{draw_entities, DrawState},
    },
    renderer::webgl::{
        error::Error,
        framebuffer::{
            AttachmentSource, BlitFlilter, BlitMask, Framebuffer, FramebufferAttachmentTarget,
            FramebufferBuilder, FramebufferTarget,
        },
        renderbuffer::RenderbufferInternalFormat,
        state::FrameState,
        texture::TextureUncompressedInternalFormat,
    },
};

pub struct StandardMultisamplesSimpleShading {
    multisample_framebuffer: Framebuffer,
    framebuffer: Framebuffer,
}

impl StandardMultisamplesSimpleShading {
    pub fn new() -> Self {
        Self {
            multisample_framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::RGBA8,
                ))
                .with_depth_stencil_attachment(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                ))
                .build(),
            framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA8,
                ))
                .build(),
        }
    }

    pub fn draw_texture(&self) -> Result<Option<&WebGlTexture>, Error> {
        self.framebuffer
            .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)
    }

    pub fn draw(
        &mut self,
        state: &mut FrameState,
        samples: usize,
        collected_entities: &CollectedEntities,
        lighting: bool,
    ) -> Result<(), Error> {
        self.draw_multisamples(state, samples, collected_entities, lighting)?;
        self.blit(state)?;
        Ok(())
    }

    fn draw_multisamples(
        &mut self,
        state: &mut FrameState,
        samples: usize,
        collected_entities: &CollectedEntities,
        lighting: bool,
    ) -> Result<(), Error> {
        self.multisample_framebuffer.init(state.gl())?;
        self.multisample_framebuffer
            .set_renderbuffer_samples(Some(samples));
        self.multisample_framebuffer
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.multisample_framebuffer.clear_buffers()?;
        draw_entities(
            state,
            DrawState::Draw {
                lighting,
                bloom: false,
            },
            collected_entities,
        )?;
        self.multisample_framebuffer
            .unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        Ok(())
    }

    fn blit(&mut self, state: &mut FrameState) -> Result<(), Error> {
        self.framebuffer.init(state.gl())?;
        state.blit_framebuffers(
            &mut self.multisample_framebuffer,
            &mut self.framebuffer,
            BlitMask::COLOR_BUFFER_BIT,
            BlitFlilter::LINEAR,
        )?;

        Ok(())
    }
}
