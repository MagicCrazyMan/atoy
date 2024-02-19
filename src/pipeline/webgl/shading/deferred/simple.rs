use web_sys::{WebGlRenderbuffer, WebGlTexture};

use crate::{
    pipeline::webgl::{
        collector::CollectedEntities,
        shading::{draw_translucent_entities, DrawState},
    },
    renderer::webgl::{
        buffer::BufferDescriptor,
        error::Error,
        framebuffer::{
            AttachmentProvider, Framebuffer, FramebufferAttachment, FramebufferBuilder,
            FramebufferTarget,
        },
        state::FrameState,
        texture::TextureColorFormat,
    },
};

pub struct StandardDeferredTransparentShading {
    framebuffer: Option<Framebuffer>,
}

impl StandardDeferredTransparentShading {
    pub fn new() -> Self {
        Self { framebuffer: None }
    }

    fn framebuffer(
        &mut self,
        state: &FrameState,
        depth_stencil: &WebGlRenderbuffer,
    ) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(
                FramebufferBuilder::new()
                    .set_color_attachment0(AttachmentProvider::new_texture(
                        TextureColorFormat::RGBA8,
                    ))
                    .with_depth_stencil_attachment(AttachmentProvider::from_renderbuffer(
                        depth_stencil.clone(),
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
        depth_stencil: &WebGlRenderbuffer,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: Option<&BufferDescriptor>,
    ) -> Result<(), Error> {
        let framebuffer = self.framebuffer(state, depth_stencil);
        framebuffer.set_attachment(
            FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
            Some(AttachmentProvider::from_renderbuffer(depth_stencil.clone())),
        )?;
        framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        // do not clear depth buffer!!!
        framebuffer.clear_buffers_of_attachments([FramebufferAttachment::COLOR_ATTACHMENT0])?;
        draw_translucent_entities(
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
