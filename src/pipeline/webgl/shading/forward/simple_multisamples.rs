use web_sys::WebGlTexture;

use crate::{
    pipeline::webgl::{
        collector::CollectedEntities,
        shading::{draw_entities, DrawState},
    },
    renderer::webgl::{
        error::Error,
        framebuffer::{
            AttachmentProvider, BlitFlilter, BlitMask, Framebuffer, FramebufferAttachment,
            FramebufferBuilder, FramebufferTarget,
        },
        renderbuffer::RenderbufferInternalFormat,
        state::FrameState,
        texture::TextureUncompressedInternalFormat,
    },
};

pub struct StandardMultisamplesSimpleShading {
    multisample_framebuffer: Option<Framebuffer>,
    framebuffer: Option<Framebuffer>,
}

impl StandardMultisamplesSimpleShading {
    pub fn new() -> Self {
        Self {
            multisample_framebuffer: None,
            framebuffer: None,
        }
    }

    fn framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(FramebufferBuilder::new().set_color_attachment0(
                AttachmentProvider::new_texture(TextureUncompressedInternalFormat::RGBA8),
            ))
        })
    }

    fn multisample_framebuffer(&mut self, state: &FrameState, samples: i32) -> &mut Framebuffer {
        let fbo = self.multisample_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(
                FramebufferBuilder::new()
                    .set_color_attachment0(AttachmentProvider::new_renderbuffer(
                        RenderbufferInternalFormat::RGBA8,
                    ))
                    .with_depth_stencil_attachment(AttachmentProvider::new_renderbuffer(
                        RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                    ))
                    .set_renderbuffer_samples(samples),
            )
        });
        fbo.set_renderbuffer_samples(Some(samples)).unwrap();
        fbo
    }

    pub fn draw_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer
            .as_ref()
            .and_then(|f| f.texture(FramebufferAttachment::COLOR_ATTACHMENT0))
    }

    pub unsafe fn draw(
        &mut self,
        state: &mut FrameState,
        samples: i32,
        collected_entities: &CollectedEntities,
        lighting: bool,
    ) -> Result<(), Error> {
        self.draw_multisamples(state, samples, collected_entities, lighting)?;
        self.blit(state)?;
        Ok(())
    }

    unsafe fn draw_multisamples(
        &mut self,
        state: &mut FrameState,
        samples: i32,
        collected_entities: &CollectedEntities,
        lighting: bool,
    ) -> Result<(), Error> {
        let multisample_framebuffer = self.multisample_framebuffer(state, samples);
        multisample_framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        multisample_framebuffer.clear_buffers()?;
        draw_entities(
            state,
            DrawState::Draw {
                lighting,
                bloom: false,
            },
            collected_entities,
        )?;
        multisample_framebuffer.unbind();
        Ok(())
    }

    fn blit(&mut self, state: &mut FrameState) -> Result<(), Error> {
        unsafe {
            let framebuffer: *mut Framebuffer = self.framebuffer(state);
            let multisample_framebuffer: *mut Framebuffer =
                self.multisample_framebuffer.as_mut().unwrap();
            state.blit_framebuffers(
                &mut *multisample_framebuffer,
                &mut *framebuffer,
                BlitMask::COLOR_BUFFER_BIT,
                BlitFlilter::LINEAR,
            )?;
        }
        Ok(())
    }
}
