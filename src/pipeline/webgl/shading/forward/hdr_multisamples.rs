use web_sys::WebGlTexture;

use crate::{
    pipeline::webgl::{
        collector::CollectedEntities,
        shading::{
            draw_entities, BloomBlendMapping, DrawState, GaussianBlurMapping,
            HdrExposureToneMapping, HdrReinhardToneMapping, BASE_TEXTURE_UNIFORM_BINDING,
            BLOOM_BLUR_TEXTURE_UNIFORM_BINDING, HDR_EXPOSURE_UNIFORM_BINDING,
            HDR_TEXTURE_UNIFORM_BINDING,
        },
        HdrToneMappingType, UBO_GAUSSIAN_BLUR_UNIFORM_BLOCK_MOUNT_POINT,
        UBO_GAUSSIAN_KERNEL_BLOCK_BINDING,
    },
    renderer::webgl::{
        buffer::Buffer,
        error::Error,
        framebuffer::{
            AttachmentSource, BlitFlilter, BlitMask, Framebuffer, FramebufferAttachmentTarget,
            FramebufferBuilder, FramebufferTarget, OperableBuffer,
        },
        program::Program,
        renderbuffer::RenderbufferInternalFormat,
        state::FrameState,
        texture::{TextureUncompressedInternalFormat, TextureUnit},
        uniform::{UniformBlockValue, UniformValue},
    },
    value::Readonly,
};

pub struct StandardMultisamplesHdrShading {
    hdr_multisamples_framebuffer: Framebuffer,
    hdr_framebuffer: Framebuffer,
    hdr_multisamples_bloom_framebuffer: Framebuffer,
    hdr_bloom_framebuffer: Framebuffer,
    hdr_bloom_blur_even_framebuffer: Framebuffer,
    hdr_bloom_blur_odd_framebuffer: Framebuffer,
    hdr_bloom_blend_framebuffer: Framebuffer,
    framebuffer: Framebuffer,
}

impl StandardMultisamplesHdrShading {
    pub fn new() -> Self {
        Self {
            hdr_multisamples_framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::RGBA32F,
                ))
                .with_depth_stencil_attachment(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                ))
                .build(),
            hdr_framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA32F,
                ))
                .build(),
            hdr_multisamples_bloom_framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::RGBA32F,
                ))
                .set_color_attachment1(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::RGBA32F,
                ))
                .with_depth_stencil_attachment(AttachmentSource::new_renderbuffer(
                    RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                ))
                .build(),
            hdr_bloom_framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA32F,
                ))
                .set_color_attachment1(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA32F,
                ))
                .build(),
            hdr_bloom_blur_even_framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA32F,
                ))
                .build(),
            hdr_bloom_blur_odd_framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA32F,
                ))
                .build(),
            hdr_bloom_blend_framebuffer: FramebufferBuilder::new()
                .set_color_attachment0(AttachmentSource::new_texture(
                    TextureUncompressedInternalFormat::RGBA32F,
                ))
                .build(),
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
        samples: usize,
        bloom: bool,
        bloom_blur_epoch: usize,
        tone_mapping_type: HdrToneMappingType,
        collected_entities: &CollectedEntities,
        lighting: bool,
        gaussian_kernel_ubo: &Buffer,
    ) -> Result<(), Error> {
        if bloom {
            self.draw_hdr_multisamples_bloom(state, samples, collected_entities, lighting)?;
            self.blit_bloom(state)?;
            self.blur_bloom(state, bloom_blur_epoch, gaussian_kernel_ubo)?;
            self.blend_bloom(state, bloom_blur_epoch)?;
            self.tone_mapping_bloom(state, tone_mapping_type)?;
        } else {
            self.draw_hdr_multisamples(state, samples, collected_entities, lighting)?;
            self.blit(state)?;
            self.tone_mapping(state, tone_mapping_type)?;
        }
        Ok(())
    }

    fn draw_hdr_multisamples(
        &mut self,
        state: &mut FrameState,
        samples: usize,
        collected_entities: &CollectedEntities,
        lighting: bool,
    ) -> Result<(), Error> {
        self.hdr_multisamples_framebuffer.set_renderbuffer_samples(Some(samples));
        self.hdr_multisamples_framebuffer.init(state.gl())?;
        self.hdr_multisamples_framebuffer
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.hdr_multisamples_framebuffer.clear_buffers()?;
        draw_entities(
            state,
            DrawState::Draw {
                lighting,
                bloom: false,
            },
            collected_entities,
        )?;
        self.hdr_multisamples_framebuffer
            .unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        Ok(())
    }

    fn draw_hdr_multisamples_bloom(
        &mut self,
        state: &mut FrameState,
        samples: usize,
        collected_entities: &CollectedEntities,
        lighting: bool,
    ) -> Result<(), Error> {
        self.hdr_multisamples_bloom_framebuffer.set_renderbuffer_samples(Some(samples));
        self.hdr_multisamples_bloom_framebuffer.init(state.gl())?;
        self.hdr_multisamples_bloom_framebuffer
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        self.hdr_multisamples_bloom_framebuffer.clear_buffers()?;
        draw_entities(
            state,
            DrawState::Draw {
                lighting,
                bloom: true,
            },
            collected_entities,
        )?;
        self.hdr_multisamples_bloom_framebuffer
            .unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        Ok(())
    }

    fn prepare_tone_mapping(
        &mut self,
        state: &mut FrameState,
        tone_mapping_type: HdrToneMappingType,
    ) -> Result<Program, Error> {
        let program = match tone_mapping_type {
            HdrToneMappingType::Reinhard => {
                let program = state
                    .program_store_mut()
                    .get_or_compile_program(&HdrReinhardToneMapping)?;
                program.use_program()?;

                program
            }
            HdrToneMappingType::Exposure(exposure) => {
                let program = state
                    .program_store_mut()
                    .get_or_compile_program(&HdrExposureToneMapping)?;
                program.use_program()?;
                program.bind_uniform_value_by_binding(
                    &HDR_EXPOSURE_UNIFORM_BINDING,
                    &UniformValue::Float1(exposure),
                    None,
                )?;

                program
            }
        };
        program.bind_uniform_value_by_binding(
            &HDR_TEXTURE_UNIFORM_BINDING,
            &UniformValue::Integer1(0),
            None,
        )?;

        Ok(program)
    }

    fn tone_mapping(
        &mut self,
        state: &mut FrameState,
        tone_mapping_type: HdrToneMappingType,
    ) -> Result<(), Error> {
        let program = self.prepare_tone_mapping(state, tone_mapping_type)?;

        self.framebuffer.init(state.gl())?;
        self.framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        state.do_computation([(
            self.hdr_framebuffer
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)?
                .unwrap(),
            TextureUnit::TEXTURE0,
        )])?;
        self.framebuffer.unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;

        program.unuse_program()?;

        Ok(())
    }

    fn tone_mapping_bloom(
        &mut self,
        state: &mut FrameState,
        tone_mapping_type: HdrToneMappingType,
    ) -> Result<(), Error> {
        let program = self.prepare_tone_mapping(state, tone_mapping_type)?;

        self.framebuffer.init(state.gl())?;
        self.framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        state.do_computation([(
            self.hdr_bloom_blend_framebuffer
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)?
                .unwrap(),
            TextureUnit::TEXTURE0,
        )])?;
        self.framebuffer
            .unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;

        program.unuse_program()?;

        Ok(())
    }

    fn blit(&mut self, state: &mut FrameState) -> Result<(), Error> {
        self.hdr_framebuffer.init(state.gl())?;
        state.blit_framebuffers(
            &mut self.hdr_multisamples_framebuffer,
            &mut self.hdr_framebuffer,
            BlitMask::COLOR_BUFFER_BIT,
            BlitFlilter::LINEAR,
        )?;
        Ok(())
    }

    fn blit_bloom(&mut self, state: &mut FrameState) -> Result<(), Error> {
        self.hdr_bloom_framebuffer.init(state.gl())?;
        state.blit_framebuffers_with_buffers(
            &mut self.hdr_multisamples_bloom_framebuffer,
            OperableBuffer::COLOR_ATTACHMENT0,
            &mut self.hdr_bloom_framebuffer,
            [OperableBuffer::COLOR_ATTACHMENT0, OperableBuffer::NONE],
            BlitMask::COLOR_BUFFER_BIT,
            BlitFlilter::LINEAR,
        )?;
        state.blit_framebuffers_with_buffers(
            &mut self.hdr_multisamples_bloom_framebuffer,
            OperableBuffer::COLOR_ATTACHMENT1,
            &mut self.hdr_bloom_framebuffer,
            [OperableBuffer::NONE, OperableBuffer::COLOR_ATTACHMENT1],
            BlitMask::COLOR_BUFFER_BIT,
            BlitFlilter::LINEAR,
        )?;

        Ok(())
    }

    fn blur_bloom(
        &mut self,
        state: &mut FrameState,
        bloom_blur_epoch: usize,
        gaussian_kernel_ubo: &Buffer,
    ) -> Result<(), Error> {
        self.hdr_bloom_framebuffer.init(state.gl())?;
        self.hdr_bloom_blur_even_framebuffer.init(state.gl())?;
        self.hdr_bloom_blur_odd_framebuffer.init(state.gl())?;

        let program = state
            .program_store_mut()
            .get_or_compile_program(&GaussianBlurMapping)?;
        program.use_program()?;
        for i in 0..bloom_blur_epoch {
            let (from, from_attachment, to) = if i % 2 == 0 {
                if i == 0 {
                    // first epoch, do some initialization
                    program.bind_uniform_block_value_by_binding(
                        &UBO_GAUSSIAN_KERNEL_BLOCK_BINDING,
                        &UniformBlockValue::BufferBase {
                            buffer: Readonly::Borrowed(gaussian_kernel_ubo),
                            mount_point: UBO_GAUSSIAN_BLUR_UNIFORM_BLOCK_MOUNT_POINT,
                        },
                        None,
                    )?;

                    (
                        &mut self.hdr_bloom_framebuffer,
                        FramebufferAttachmentTarget::COLOR_ATTACHMENT1,
                        &mut self.hdr_bloom_blur_odd_framebuffer,
                    )
                } else {
                    (
                        &mut self.hdr_bloom_blur_even_framebuffer,
                        FramebufferAttachmentTarget::COLOR_ATTACHMENT0,
                        &mut self.hdr_bloom_blur_odd_framebuffer,
                    )
                }
            } else {
                (
                    &mut self.hdr_bloom_blur_odd_framebuffer,
                    FramebufferAttachmentTarget::COLOR_ATTACHMENT0,
                    &mut self.hdr_bloom_blur_even_framebuffer,
                )
            };
            to.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
            state.do_computation([(
                from.texture(from_attachment)?.unwrap(),
                TextureUnit::TEXTURE0,
            )])?;
            to.unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        }
        program.unuse_program()?;

        Ok(())
    }

    fn blend_bloom(
        &mut self,
        state: &mut FrameState,
        bloom_blur_epoch: usize,
    ) -> Result<(), Error> {
        let hdr_base_texture = self
            .hdr_bloom_framebuffer
            .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)?
            .unwrap();
        let hdr_bloom_blur_texture = if bloom_blur_epoch == 0 {
            self.hdr_bloom_framebuffer
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT1)?
                .unwrap()
        } else if bloom_blur_epoch % 2 == 0 {
            self.hdr_bloom_blur_even_framebuffer
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)?
                .unwrap()
        } else {
            self.hdr_bloom_blur_odd_framebuffer
                .texture(FramebufferAttachmentTarget::COLOR_ATTACHMENT0)?
                .unwrap()
        };

        self.hdr_bloom_blend_framebuffer.init(state.gl())?;

        let program = state
            .program_store_mut()
            .get_or_compile_program(&BloomBlendMapping)?;
        program.use_program()?;
        program.bind_uniform_value_by_binding(
            &BASE_TEXTURE_UNIFORM_BINDING,
            &UniformValue::Integer1(0),
            None,
        )?;
        program.bind_uniform_value_by_binding(
            &BLOOM_BLUR_TEXTURE_UNIFORM_BINDING,
            &UniformValue::Integer1(1),
            None,
        )?;

        self.hdr_bloom_blend_framebuffer
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        state.do_computation([
            (hdr_base_texture, TextureUnit::TEXTURE0),
            (hdr_bloom_blur_texture, TextureUnit::TEXTURE1),
        ])?;
        self.hdr_bloom_blend_framebuffer
            .unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;

        program.unuse_program()?;

        Ok(())
    }
}
