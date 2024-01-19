use crate::render::webgl::{
    framebuffer::{
        Framebuffer, FramebufferAttachment, FramebufferDrawBuffer, FramebufferSizePolicy,
        RenderbufferProvider, TextureProvider,
    },
    pipeline::collector::CollectedEntities,
    renderbuffer::RenderbufferInternalFormat,
    state::FrameState,
    texture::{TextureDataType, TextureFormat, TextureInternalFormat},
};

pub struct GBufferCollector {
    framebuffer: Option<Framebuffer>,
}

impl GBufferCollector {
    fn framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                FramebufferSizePolicy::FollowDrawingBuffer,
                [
                    // positions
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
                    // albedo and specular shininess
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT2,
                        TextureInternalFormat::RGBA32F,
                        TextureFormat::RGBA,
                        TextureDataType::FLOAT,
                    ),
                    // depths
                    TextureProvider::new(
                        FramebufferAttachment::DEPTH_ATTACHMENT,
                        TextureInternalFormat::DEPTH_COMPONENT,
                        TextureFormat::DEPTH_COMPONENT,
                        TextureDataType::UNSIGNED_INT,
                    ),
                ],
                [RenderbufferProvider::new(
                    FramebufferAttachment::STENCIL_ATTACHMENT,
                    RenderbufferInternalFormat::STENCIL_INDEX8,
                )],
                [
                    FramebufferDrawBuffer::COLOR_ATTACHMENT0,
                    FramebufferDrawBuffer::COLOR_ATTACHMENT1,
                    FramebufferDrawBuffer::COLOR_ATTACHMENT2,
                ],
                None,
            )
        })
    }

    pub fn collect(&mut self, state: &mut FrameState, collected_entities: &CollectedEntities) {}
}
