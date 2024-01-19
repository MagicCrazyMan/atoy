use web_sys::WebGlTexture;

use crate::render::webgl::{
    buffer::BufferDescriptor,
    error::Error,
    framebuffer::{
        Framebuffer, FramebufferAttachment, FramebufferDrawBuffer, FramebufferSizePolicy,
        FramebufferTarget, RenderbufferProvider, TextureProvider,
    },
    pipeline::{
        collector::CollectedEntities,
        shading::{draw_entities, DrawState},
    },
    renderbuffer::RenderbufferInternalFormat,
    state::FrameState,
    texture::{TextureDataType, TextureFormat, TextureInternalFormat},
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
            state.create_framebuffer(
                FramebufferSizePolicy::FollowDrawingBuffer,
                [
                    // positions and specular shininess
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT0,
                        TextureInternalFormat::RGBA32F,
                        TextureFormat::RGBA,
                        TextureDataType::FLOAT,
                    ),
                    // normals
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT1,
                        TextureInternalFormat::RGBA32F,
                        TextureFormat::RGBA,
                        TextureDataType::FLOAT,
                    ),
                    // albedo and transparency
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT2,
                        TextureInternalFormat::RGBA32F,
                        TextureFormat::RGBA,
                        TextureDataType::FLOAT,
                    ),
                    // depths
                    TextureProvider::new(
                        FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
                        TextureInternalFormat::DEPTH32F_STENCIL8,
                        TextureFormat::DEPTH_STENCIL,
                        TextureDataType::FLOAT_32_UNSIGNED_INT_24_8_REV,
                    ),
                ],
                [
                //     RenderbufferProvider::new(
                //     FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
                //     RenderbufferInternalFormat::DEPTH24_STENCIL8,
                // )
                ],
                [
                    FramebufferDrawBuffer::COLOR_ATTACHMENT0,
                    FramebufferDrawBuffer::COLOR_ATTACHMENT1,
                    FramebufferDrawBuffer::COLOR_ATTACHMENT2,
                    // FramebufferDrawBuffer::COLOR_ATTACHMENT3,
                ],
                None,
            )
        })
    }

    pub fn positions_and_specular_shininess_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer.as_ref().and_then(|fbo| fbo.texture(0))
    }

    pub fn normals_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer.as_ref().and_then(|fbo| fbo.texture(1))
    }

    pub fn albedo_and_transparency_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer.as_ref().and_then(|fbo| fbo.texture(2))
    }

    pub fn deferred_shading_textures(&self) -> Option<[&WebGlTexture; 3]> {
        if let (
            Some(positions_and_specular_shininess_texture),
            Some(normals_texture),
            Some(albedo_and_transparency_texture),
        ) = (
            self.framebuffer.as_ref().and_then(|fbo| fbo.texture(0)),
            self.framebuffer.as_ref().and_then(|fbo| fbo.texture(1)),
            self.framebuffer.as_ref().and_then(|fbo| fbo.texture(2)),
        ) {
            Some([
                positions_and_specular_shininess_texture,
                normals_texture,
                albedo_and_transparency_texture,
            ])
        } else {
            None
        }
    }

    pub unsafe fn draw(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
    ) -> Result<(), Error> {
        let framebuffer = self.framebuffer(state);
        framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        draw_entities(
            state,
            DrawState::GBuffer { universal_ubo },
            collected_entities,
        )?;
        framebuffer.unbind();
        Ok(())
    }
}
