use web_sys::WebGlTexture;

use crate::{
    pipeline::webgl::{
        collector::CollectedEntities,
        shading::{draw_translucent_entities, DrawState},
    },
    renderer::webgl::{
        error::Error,
        framebuffer::{
            AttachmentSource, ClearPolicy, Framebuffer, FramebufferAttachmentTarget,
            FramebufferBuilder, FramebufferTarget,
        },
        state::FrameState,
        texture::TextureUncompressedInternalFormat,
    },
};

pub struct StandardDeferredTransparentShading {
    framebuffer: Framebuffer,
}

impl StandardDeferredTransparentShading {
    pub fn new() -> Self {
        Self {
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
        depth_stencil: &WebGlTexture,
        collected_entities: &CollectedEntities,
        lighting: bool,
    ) -> Result<(), Error> {
        self.framebuffer.init(state.gl())?;
        self.framebuffer.set_attachment(
            FramebufferAttachmentTarget::DEPTH_STENCIL_ATTACHMENT,
            Some(AttachmentSource::from_texture(
                depth_stencil.clone(),
                0,
                ClearPolicy::DepthStencil(1.0, 0),
            )),
        )?;
        self.framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        // do not clear depth buffer!!!
        self.framebuffer
            .clear_buffer(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)?;
        draw_translucent_entities(
            state,
            DrawState::Draw {
                lighting,
                bloom: false,
            },
            collected_entities,
        )?;
        self.framebuffer
            .unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        Ok(())
    }
}
