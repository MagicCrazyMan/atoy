use web_sys::{WebGlRenderbuffer, WebGlTexture};

use crate::{
    pipeline::webgl::{
        collector::CollectedEntities,
        shading::{draw_opaque_entities, DrawState},
    },
    renderer::webgl::{
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

pub struct StandardGBufferCollector {
    framebuffer: Option<Framebuffer>,
}

impl StandardGBufferCollector {
    pub fn new() -> Self {
        Self { framebuffer: None }
    }

    fn framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(
                FramebufferBuilder::new()
                    // positions and specular shininess
                    .set_color_attachment0(AttachmentProvider::new_texture(
                        TextureColorFormat::RGBA32F,
                    ))
                    // normals
                    .set_color_attachment1(AttachmentProvider::new_texture(
                        TextureColorFormat::RGBA32F,
                    ))
                    // albedo
                    .set_color_attachment2(AttachmentProvider::new_texture(
                        TextureColorFormat::RGBA32F,
                    ))
                    .with_depth_stencil_attachment(AttachmentProvider::new_renderbuffer(
                        RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                    )),
            )
        })
    }

    pub fn positions_and_specular_shininess_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer
            .as_ref()
            .and_then(|fbo| fbo.texture(FramebufferAttachment::COLOR_ATTACHMENT0))
    }

    pub fn normals_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer
            .as_ref()
            .and_then(|fbo| fbo.texture(FramebufferAttachment::COLOR_ATTACHMENT1))
    }

    pub fn albedo_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer
            .as_ref()
            .and_then(|fbo| fbo.texture(FramebufferAttachment::COLOR_ATTACHMENT2))
    }

    pub fn depth_stencil_renderbuffer(&self) -> Option<&WebGlTexture> {
        self.framebuffer
            .as_ref()
            .and_then(|fbo| fbo.texture(FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT))
    }

    pub fn deferred_shading_textures(
        &self,
    ) -> Option<(
        &WebGlTexture,
        &WebGlTexture,
        &WebGlTexture,
        &WebGlRenderbuffer,
    )> {
        if let (
            Some(positions_and_specular_shininess_texture),
            Some(normals_texture),
            Some(albedo_texture),
            Some(depth_stencil_renderbuffer),
        ) = (
            self.framebuffer
                .as_ref()
                .and_then(|fbo| fbo.texture(FramebufferAttachment::COLOR_ATTACHMENT0)),
            self.framebuffer
                .as_ref()
                .and_then(|fbo| fbo.texture(FramebufferAttachment::COLOR_ATTACHMENT1)),
            self.framebuffer
                .as_ref()
                .and_then(|fbo| fbo.texture(FramebufferAttachment::COLOR_ATTACHMENT2)),
            self.framebuffer
                .as_ref()
                .and_then(|fbo| fbo.renderbuffer(FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT)),
        ) {
            Some((
                positions_and_specular_shininess_texture,
                normals_texture,
                albedo_texture,
                depth_stencil_renderbuffer,
            ))
        } else {
            None
        }
    }

    pub unsafe fn collect(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
    ) -> Result<(), Error> {
        let framebuffer = self.framebuffer(state);
        framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        framebuffer.clear_buffers()?;
        draw_opaque_entities(state, DrawState::GBuffer, collected_entities)?;
        framebuffer.unbind();

        Ok(())
    }
}
