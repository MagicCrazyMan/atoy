use web_sys::WebGlTexture;

use crate::render::webgl::{
    buffer::BufferDescriptor,
    error::Error,
    framebuffer::{
        BlitFlilter, BlitMask, Framebuffer, FramebufferAttachment, FramebufferSizePolicy,
        FramebufferTarget, RenderbufferProvider, TextureProvider,
    },
    pipeline::collector::CollectedEntities,
    renderbuffer::RenderbufferInternalFormat,
    state::FrameState,
    texture::{TextureDataType, TextureFormat, TextureInternalFormat},
};

use super::draw_entities;

pub struct StandardMultisamplesSimpleDrawer {
    multisample_framebuffer: Option<Framebuffer>,
    framebuffer: Option<Framebuffer>,
}

impl StandardMultisamplesSimpleDrawer {
    pub fn new() -> Self {
        Self {
            multisample_framebuffer: None,
            framebuffer: None,
        }
    }

    fn framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                FramebufferSizePolicy::FollowDrawingBuffer,
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA,
                    TextureFormat::RGBA,
                    TextureDataType::UNSIGNED_BYTE,
                )],
                [],
                [],
                None,
            )
        })
    }

    fn multisample_framebuffer(&mut self, state: &FrameState, samples: i32) -> &mut Framebuffer {
        let fbo = self.multisample_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                FramebufferSizePolicy::FollowDrawingBuffer,
                [],
                [
                    RenderbufferProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT0,
                        RenderbufferInternalFormat::RGBA8,
                    ),
                    RenderbufferProvider::new(
                        FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
                        RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                    ),
                ],
                [],
                Some(samples),
            )
        });
        fbo.set_renderbuffer_samples(Some(samples));
        fbo
    }

    pub fn draw_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer.as_ref().and_then(|f| f.texture(0))
    }

    pub unsafe fn draw(
        &mut self,
        state: &mut FrameState,
        lighting: bool,
        samples: i32,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: &BufferDescriptor,
    ) -> Result<(), Error> {
        self.draw_multisamples(
            state,
            lighting,
            samples,
            collected_entities,
            universal_ubo,
            lights_ubo,
        )?;
        self.blit(state)?;
        Ok(())
    }

    unsafe fn draw_multisamples(
        &mut self,
        state: &mut FrameState,
        lighting: bool,
        samples: i32,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: &BufferDescriptor,
    ) -> Result<(), Error> {
        let multisample_framebuffer = self.multisample_framebuffer(state, samples);
        multisample_framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        draw_entities(
            state,
            lighting,
            false,
            collected_entities,
            universal_ubo,
            lights_ubo,
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
