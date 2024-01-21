use web_sys::WebGlTexture;

use crate::render::webgl::{
    buffer::BufferDescriptor,
    error::Error,
    framebuffer::{
        AttachmentProvider, ClearPolicy, Framebuffer, FramebufferAttachment, FramebufferBuilder,
        FramebufferTarget,
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
            state.create_framebuffer_with_builder(
                FramebufferBuilder::new()
                    .with_color_attachment0(AttachmentProvider::new_texture(
                        TextureInternalFormat::RGBA8,
                        TextureFormat::RGBA,
                        TextureDataType::UNSIGNED_BYTE,
                        ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
                    ))
                    .with_depth_stencil_attachment(AttachmentProvider::new_renderbuffer(
                        RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                        ClearPolicy::DepthStencil(1.0, 0),
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
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: Option<&BufferDescriptor>,
    ) -> Result<(), Error> {
        let framebuffer = self.framebuffer(state);
        framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        framebuffer.clear_buffer_bits()?;
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
