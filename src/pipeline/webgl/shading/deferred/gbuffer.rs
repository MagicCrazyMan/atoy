use web_sys::WebGlTexture;

use crate::{
    pipeline::webgl::{
        collector::CollectedEntities,
        shading::{draw_opaque_entities, DrawState},
    },
    renderer::webgl::{
        blit::{Blit, BlitFlilter, BlitMask},
        error::Error,
        framebuffer::{
            AttachmentSource, Framebuffer, FramebufferAttachmentTarget, FramebufferBuilder,
            FramebufferTarget, OperableBuffer,
        },
        renderbuffer::RenderbufferInternalFormat,
        state::FrameState,
        texture::TextureUncompressedInternalFormat,
    },
};

pub struct StandardGBufferCollector {
    fbo_ms: Framebuffer,
    fbo: Framebuffer,
}

impl StandardGBufferCollector {
    pub fn new() -> Self {
        Self {
            fbo_ms: FramebufferBuilder::new()
                // positions and specular shininess
                .set_color_attachment0(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::RGBA32F,
                ))
                // normals
                .set_color_attachment1(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::RGBA32F,
                ))
                // albedo
                .set_color_attachment2(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::RGBA32F,
                ))
                .set_depth_stencil_attachment(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                ))
                .build(),
            fbo: FramebufferBuilder::new()
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
                .set_depth_stencil_attachment(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::DEPTH32F_STENCIL8,
                ))
                .build(),
        }
    }

    pub fn collect(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
        multisamples: Option<usize>,
    ) -> Result<(&WebGlTexture, &WebGlTexture, &WebGlTexture, &WebGlTexture), Error> {
        self.fbo_ms.init(state.gl())?;
        self.fbo.init(state.gl())?;

        self.fbo_ms.set_renderbuffer_samples(multisamples);
        self.fbo_ms
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.fbo_ms.clear_buffers()?;
        draw_opaque_entities(state, DrawState::GBuffer, collected_entities)?;
        self.fbo_ms
            .unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;

        Blit::with_buffers(
            state.gl(),
            &mut self.fbo_ms,
            OperableBuffer::COLOR_ATTACHMENT0,
            &mut self.fbo,
            vec![OperableBuffer::COLOR_ATTACHMENT0],
            BlitMask::COLOR_BUFFER_BIT,
            BlitFlilter::NEAREST,
        )
        .blit()?;
        Blit::with_buffers(
            state.gl(),
            &mut self.fbo_ms,
            OperableBuffer::COLOR_ATTACHMENT1,
            &mut self.fbo,
            vec![OperableBuffer::NONE, OperableBuffer::COLOR_ATTACHMENT1],
            BlitMask::COLOR_BUFFER_BIT,
            BlitFlilter::NEAREST,
        )
        .blit()?;
        Blit::with_buffers(
            state.gl(),
            &mut self.fbo_ms,
            OperableBuffer::COLOR_ATTACHMENT2,
            &mut self.fbo,
            vec![
                OperableBuffer::NONE,
                OperableBuffer::NONE,
                OperableBuffer::COLOR_ATTACHMENT2,
            ],
            BlitMask::COLOR_BUFFER_BIT,
            BlitFlilter::LINEAR,
        )
        .blit()?;
        Blit::with_params(
            state.gl(),
            &mut self.fbo_ms,
            None,
            &mut self.fbo,
            None,
            BlitMask::DEPTH_BUFFER_BIT,
            BlitFlilter::NEAREST,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .blit()?;
        Blit::with_params(
            state.gl(),
            &mut self.fbo_ms,
            None,
            &mut self.fbo,
            None,
            BlitMask::STENCIL_BUFFER_BIT,
            BlitFlilter::NEAREST,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .blit()?;

        Ok((
            self.fbo
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)?
                .unwrap(),
            self.fbo
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT1)?
                .unwrap(),
            self.fbo
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT2)?
                .unwrap(),
            self.fbo
                .texture(FramebufferAttachmentTarget::DEPTH_STENCIL_ATTACHMENT)?
                .unwrap(),
        ))
    }
}
