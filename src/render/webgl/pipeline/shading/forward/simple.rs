use web_sys::WebGlTexture;

use crate::render::webgl::{
    buffer::BufferDescriptor,
    error::Error,
    framebuffer::{
        ClearPolicy, Framebuffer, FramebufferAttachment, FramebufferSizePolicy, FramebufferTarget,
        RenderbufferProvider, TextureProvider,
    },
    pipeline::{
        collector::CollectedEntities,
        shading::{draw_entities, DrawState},
    },
    renderbuffer::RenderbufferInternalFormat,
    state::FrameState,
    texture::{TextureDataType, TextureFormat, TextureInternalFormat},
};

pub struct StandardSimpleShading {
    framebuffer: Option<Framebuffer>,
}

impl StandardSimpleShading {
    pub fn new() -> Self {
        Self { framebuffer: None }
    }

    fn framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                FramebufferSizePolicy::FollowDrawingBuffer,
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA8,
                    TextureFormat::RGBA,
                    TextureDataType::UNSIGNED_BYTE,
                    ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
                )],
                [RenderbufferProvider::new(
                    FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
                    RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                    ClearPolicy::DepthStencil(1.0, 0),
                )],
                [],
                None,
            )
        })
    }

    pub fn draw_texture(&self) -> Option<&WebGlTexture> {
        self.framebuffer.as_ref().and_then(|f| f.texture(0))
    }

    pub unsafe fn draw(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: Option<&BufferDescriptor>,
    ) -> Result<(), Error> {
        let framebuffer = self.framebuffer(state);
        framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        framebuffer.clear_buffers();
        draw_entities(
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
