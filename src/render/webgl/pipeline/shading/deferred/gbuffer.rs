use web_sys::WebGlTexture;

use crate::render::webgl::{
    buffer::BufferDescriptor,
    error::Error,
    framebuffer::{
        ClearPolicy, Framebuffer, FramebufferAttachment, FramebufferDrawBuffer,
        FramebufferSizePolicy, FramebufferTarget, RenderbufferProvider, TextureProvider,
    },
    pipeline::{
        collector::CollectedEntities,
        shading::{draw_opaque_entities, DrawState},
    },
    renderbuffer::RenderbufferInternalFormat,
    state::FrameState,
    texture::{TextureDataType, TextureFormat, TextureInternalFormat},
};

pub struct StandardGBufferCollector {
    framebuffer: Option<Framebuffer>,

    last_collected_entities_id: Option<usize>,
}

impl StandardGBufferCollector {
    pub fn new() -> Self {
        Self {
            framebuffer: None,
            last_collected_entities_id: None,
        }
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
                        ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
                    ),
                    // normals
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT1,
                        TextureInternalFormat::RGBA32F,
                        TextureFormat::RGBA,
                        TextureDataType::FLOAT,
                        ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
                    ),
                    // albedo and transparency
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT2,
                        TextureInternalFormat::RGBA32F,
                        TextureFormat::RGBA,
                        TextureDataType::FLOAT,
                        ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
                    ),
                ],
                [RenderbufferProvider::new(
                    FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
                    RenderbufferInternalFormat::DEPTH24_STENCIL8,
                    ClearPolicy::DepthStencil(1.0, 0),
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

    pub fn positions_and_specular_shininess_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer.as_ref().and_then(|fbo| fbo.texture(0))
    }

    pub fn normals_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer.as_ref().and_then(|fbo| fbo.texture(1))
    }

    pub fn albedo_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer.as_ref().and_then(|fbo| fbo.texture(2))
    }

    pub fn deferred_shading_textures(&self) -> Option<[&WebGlTexture; 3]> {
        if let (
            Some(positions_and_specular_shininess_texture),
            Some(normals_texture),
            Some(albedo_texture),
        ) = (
            self.framebuffer.as_ref().and_then(|fbo| fbo.texture(0)),
            self.framebuffer.as_ref().and_then(|fbo| fbo.texture(1)),
            self.framebuffer.as_ref().and_then(|fbo| fbo.texture(2)),
        ) {
            Some([
                positions_and_specular_shininess_texture,
                normals_texture,
                albedo_texture,
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
        // only redraw gbuffer when collected entities changed
        // if self
        //     .last_collected_entities_id
        //     .map(|id| collected_entities.id() == id)
        //     .unwrap_or(false)
        // {
        //     return Ok(());
        // }

        let framebuffer = self.framebuffer(state);
        framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        framebuffer.clear_buffers();

        draw_opaque_entities(
            state,
            &DrawState::GBuffer { universal_ubo },
            collected_entities,
        )?;
        framebuffer.unbind();

        self.last_collected_entities_id = Some(collected_entities.id());

        Ok(())
    }
}
