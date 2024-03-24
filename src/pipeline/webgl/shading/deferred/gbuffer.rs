use web_sys::{WebGlRenderbuffer, WebGlTexture};

use crate::{
    pipeline::webgl::{
        collector::CollectedEntities,
        shading::{draw_opaque_entities, DrawState},
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

pub struct StandardGBufferCollector {
    framebuffer: Framebuffer,
}

impl StandardGBufferCollector {
    pub fn new() -> Self {
        Self {
            framebuffer: FramebufferBuilder::new()
                // positions and specular shininess
                .set_color_attachment0(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA32F,
                ))
                // normals
                .set_color_attachment1(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA32F,
                ))
                // albedo
                .set_color_attachment2(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA32F,
                ))
                .with_depth_stencil_attachment(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                ))
                .build(),
        }
    }

    pub fn deferred_shading_textures(
        &self,
    ) -> Result<
        Option<(
            &WebGlTexture,
            &WebGlTexture,
            &WebGlTexture,
            &WebGlRenderbuffer,
        )>,
        Error,
    > {
        if let (
            Some(positions_and_specular_shininess_texture),
            Some(normals_texture),
            Some(albedo_texture),
            Some(depth_stencil_renderbuffer),
        ) = (
            self.framebuffer
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)?,
            self.framebuffer
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT1)?,
            self.framebuffer
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT2)?,
            self.framebuffer
                .renderbuffer(FramebufferAttachmentTarget::DEPTH_STENCIL_ATTACHMENT)?,
        ) {
            Ok(Some((
                positions_and_specular_shininess_texture,
                normals_texture,
                albedo_texture,
                depth_stencil_renderbuffer,
            )))
        } else {
            Ok(None)
        }
    }

    pub fn collect(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
    ) -> Result<(), Error> {
        self.framebuffer.init(state.gl())?;
        self.framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.framebuffer.clear_buffers()?;
        draw_opaque_entities(state, DrawState::GBuffer, collected_entities)?;
        self.framebuffer.unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;

        Ok(())
    }
}
