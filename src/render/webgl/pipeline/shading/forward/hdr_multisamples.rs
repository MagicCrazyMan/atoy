use web_sys::WebGlTexture;

use crate::render::webgl::{
    buffer::BufferDescriptor,
    error::Error,
    framebuffer::{
        AttachmentProvider, BlitFlilter, BlitMask, Framebuffer, FramebufferAttachment,
        FramebufferBuilder, FramebufferTarget, OperatableBuffer,
    },
    pipeline::{
        collector::CollectedEntities,
        shading::{
            draw_entities, BloomBlendMappingProgram, DrawState, GaussianBlurMappingProgram,
            HdrExposureToneMappingProgram, HdrReinhardToneMappingProgram,
            BASE_TEXTURE_UNIFORM_NAME, BLOOM_BLUR_TEXTURE_UNIFORM_NAME, HDR_EXPOSURE_UNIFORM_NAME,
            HDR_TEXTURE_UNIFORM_NAME,
        },
        HdrToneMappingType, UBO_GAUSSIAN_BLUR_BINDING, UBO_GAUSSIAN_KERNEL_BLOCK_NAME,
    },
    renderbuffer::RenderbufferInternalFormat,
    state::FrameState,
    texture::{TextureUncompressedInternalFormat, TextureUnit},
    uniform::{UniformBlockValue, UniformValue},
};

pub struct StandardMultisamplesHdrShading {
    hdr_multisamples_framebuffer: Option<Framebuffer>,
    hdr_framebuffer: Option<Framebuffer>,
    hdr_multisamples_bloom_framebuffer: Option<Framebuffer>,
    hdr_bloom_framebuffer: Option<Framebuffer>,
    hdr_bloom_blur_even_framebuffer: Option<Framebuffer>,
    hdr_bloom_blur_odd_framebuffer: Option<Framebuffer>,
    hdr_bloom_blend_framebuffer: Option<Framebuffer>,
    framebuffer: Option<Framebuffer>,
}

impl StandardMultisamplesHdrShading {
    pub fn new() -> Self {
        Self {
            hdr_multisamples_framebuffer: None,
            hdr_framebuffer: None,
            hdr_multisamples_bloom_framebuffer: None,
            hdr_bloom_framebuffer: None,
            hdr_bloom_blur_even_framebuffer: None,
            hdr_bloom_blur_odd_framebuffer: None,
            hdr_bloom_blend_framebuffer: None,
            framebuffer: None,
        }
    }

    fn framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(FramebufferBuilder::new().with_color_attachment0(
                AttachmentProvider::new_texture(TextureUncompressedInternalFormat::RGBA8),
            ))
        })
    }

    fn hdr_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(FramebufferBuilder::new().with_color_attachment0(
                AttachmentProvider::new_texture(TextureUncompressedInternalFormat::RGBA32F),
            ))
        })
    }

    fn hdr_bloom_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_bloom_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(
                FramebufferBuilder::new()
                    .with_color_attachment0(AttachmentProvider::new_texture(
                        TextureUncompressedInternalFormat::RGBA32F,
                    ))
                    .with_color_attachment1(AttachmentProvider::new_texture(
                        TextureUncompressedInternalFormat::RGBA32F,
                    )),
            )
        })
    }

    fn hdr_multisamples_framebuffer(
        &mut self,
        state: &FrameState,
        samples: i32,
    ) -> &mut Framebuffer {
        let fbo = self.hdr_multisamples_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(
                FramebufferBuilder::new()
                    .with_color_attachment0(AttachmentProvider::new_renderbuffer(
                        RenderbufferInternalFormat::RGBA32F,
                    ))
                    .with_depth_stencil_attachment(AttachmentProvider::new_renderbuffer(
                        RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                    ))
                    .with_samples(samples),
            )
        });
        fbo.set_renderbuffer_samples(Some(samples)).unwrap();
        fbo
    }

    fn hdr_multisamples_bloom_framebuffer(
        &mut self,
        state: &FrameState,
        samples: i32,
    ) -> &mut Framebuffer {
        let fbo = self
            .hdr_multisamples_bloom_framebuffer
            .get_or_insert_with(|| {
                state.create_framebuffer_with_builder(
                    FramebufferBuilder::new()
                        .with_color_attachment0(AttachmentProvider::new_renderbuffer(
                            RenderbufferInternalFormat::RGBA32F,
                        ))
                        .with_color_attachment1(AttachmentProvider::new_renderbuffer(
                            RenderbufferInternalFormat::RGBA32F,
                        ))
                        .with_depth_stencil_attachment(AttachmentProvider::new_renderbuffer(
                            RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                        ))
                        .with_samples(samples),
                )
            });
        fbo.set_renderbuffer_samples(Some(samples)).unwrap();
        fbo
    }

    fn hdr_bloom_blur_even_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_bloom_blur_even_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(FramebufferBuilder::new().with_color_attachment0(
                AttachmentProvider::new_texture(TextureUncompressedInternalFormat::RGBA32F),
            ))
        })
    }

    fn hdr_bloom_blur_odd_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_bloom_blur_odd_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(FramebufferBuilder::new().with_color_attachment0(
                AttachmentProvider::new_texture(TextureUncompressedInternalFormat::RGBA32F),
            ))
        })
    }

    fn hdr_bloom_blend_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_bloom_blend_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer_with_builder(FramebufferBuilder::new().with_color_attachment0(
                AttachmentProvider::new_texture(TextureUncompressedInternalFormat::RGBA32F),
            ))
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
        samples: i32,
        bloom: bool,
        bloom_blur_epoch: usize,
        tone_mapping_type: HdrToneMappingType,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: Option<&BufferDescriptor>,
        gaussian_kernel_ubo: &BufferDescriptor,
    ) -> Result<(), Error> {
        if bloom {
            self.draw_hdr_multisamples_bloom(
                state,
                samples,
                collected_entities,
                universal_ubo,
                lights_ubo,
            )?;
            self.blit_bloom(state)?;
            self.blur_bloom(state, bloom_blur_epoch, gaussian_kernel_ubo)?;
            self.blend_bloom(state, bloom_blur_epoch)?;
            self.tone_mapping_bloom(state, tone_mapping_type)?;
        } else {
            self.draw_hdr_multisamples(
                state,
                samples,
                collected_entities,
                universal_ubo,
                lights_ubo,
            )?;
            self.blit(state)?;
            self.tone_mapping(state, tone_mapping_type)?;
        }
        Ok(())
    }

    unsafe fn draw_hdr_multisamples(
        &mut self,
        state: &mut FrameState,
        samples: i32,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: Option<&BufferDescriptor>,
    ) -> Result<(), Error> {
        let hdr_multisamples_framebuffer = self.hdr_multisamples_framebuffer(state, samples);
        hdr_multisamples_framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        hdr_multisamples_framebuffer.clear_buffers()?;
        draw_entities(
            state,
            &DrawState::Draw {
                universal_ubo,
                lights_ubo,
                bloom: false,
            },
            collected_entities,
        )?;
        hdr_multisamples_framebuffer.unbind();
        Ok(())
    }

    unsafe fn draw_hdr_multisamples_bloom(
        &mut self,
        state: &mut FrameState,
        samples: i32,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: Option<&BufferDescriptor>,
    ) -> Result<(), Error> {
        let hdr_multisamples_bloom_framebuffer =
            self.hdr_multisamples_bloom_framebuffer(state, samples);
        hdr_multisamples_bloom_framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        hdr_multisamples_bloom_framebuffer.clear_buffers()?;
        draw_entities(
            state,
            &DrawState::Draw {
                universal_ubo,
                lights_ubo,
                bloom: true,
            },
            collected_entities,
        )?;
        hdr_multisamples_bloom_framebuffer.unbind();
        Ok(())
    }

    fn prepare_tone_mapping(
        &mut self,
        state: &mut FrameState,
        tone_mapping_type: HdrToneMappingType,
    ) -> Result<(), Error> {
        let program = match tone_mapping_type {
            HdrToneMappingType::Reinhard => state
                .program_store_mut()
                .use_program(&HdrReinhardToneMappingProgram)?,
            HdrToneMappingType::Exposure(exposure) => {
                let program = state
                    .program_store_mut()
                    .use_program(&HdrExposureToneMappingProgram)?;
                state.bind_uniform_value_by_variable_name(
                    program,
                    HDR_EXPOSURE_UNIFORM_NAME,
                    &UniformValue::Float1(exposure),
                )?;
                program
            }
        };
        state.bind_uniform_value_by_variable_name(
            program,
            HDR_TEXTURE_UNIFORM_NAME,
            &UniformValue::Integer1(0),
        )?;

        Ok(())
    }

    fn tone_mapping(
        &mut self,
        state: &mut FrameState,
        tone_mapping_type: HdrToneMappingType,
    ) -> Result<(), Error> {
        self.prepare_tone_mapping(state, tone_mapping_type)?;

        self.framebuffer(state)
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        state.do_computation([(
            self.hdr_framebuffer
                .as_ref()
                .unwrap()
                .texture(FramebufferAttachment::COLOR_ATTACHMENT0)
                .unwrap(),
            TextureUnit::TEXTURE0,
        )]);
        self.framebuffer(state).unbind();

        Ok(())
    }

    fn tone_mapping_bloom(
        &mut self,
        state: &mut FrameState,
        tone_mapping_type: HdrToneMappingType,
    ) -> Result<(), Error> {
        self.prepare_tone_mapping(state, tone_mapping_type)?;

        self.framebuffer(state)
            .bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        state.do_computation([(
            self.hdr_bloom_blend_framebuffer
                .as_ref()
                .unwrap()
                .texture(FramebufferAttachment::COLOR_ATTACHMENT0)
                .unwrap(),
            TextureUnit::TEXTURE0,
        )]);
        self.framebuffer(state).unbind();

        Ok(())
    }

    fn blit(&mut self, state: &mut FrameState) -> Result<(), Error> {
        unsafe {
            let hdr_framebuffer: *mut Framebuffer = self.hdr_framebuffer(state);
            let hdr_multisamples_framebuffer: *mut Framebuffer =
                self.hdr_multisamples_framebuffer.as_mut().unwrap();
            state.blit_framebuffers(
                &mut *hdr_multisamples_framebuffer,
                &mut *hdr_framebuffer,
                BlitMask::COLOR_BUFFER_BIT,
                BlitFlilter::LINEAR,
            )?;
        }
        Ok(())
    }

    fn blit_bloom(&mut self, state: &mut FrameState) -> Result<(), Error> {
        unsafe {
            let hdr_bloom_framebuffer: *mut Framebuffer = self.hdr_bloom_framebuffer(state);
            let hdr_multisamples_bloom_framebuffer: *mut Framebuffer =
                self.hdr_multisamples_bloom_framebuffer.as_mut().unwrap();
            state.blit_framebuffers_with_buffers(
                &mut *hdr_multisamples_bloom_framebuffer,
                OperatableBuffer::COLOR_ATTACHMENT0,
                &mut *hdr_bloom_framebuffer,
                [OperatableBuffer::COLOR_ATTACHMENT0, OperatableBuffer::NONE],
                BlitMask::COLOR_BUFFER_BIT,
                BlitFlilter::LINEAR,
            )?;
            state.blit_framebuffers_with_buffers(
                &mut *hdr_multisamples_bloom_framebuffer,
                OperatableBuffer::COLOR_ATTACHMENT1,
                &mut *hdr_bloom_framebuffer,
                [OperatableBuffer::NONE, OperatableBuffer::COLOR_ATTACHMENT1],
                BlitMask::COLOR_BUFFER_BIT,
                BlitFlilter::LINEAR,
            )?;
        }
        Ok(())
    }

    fn blur_bloom(
        &mut self,
        state: &mut FrameState,
        bloom_blur_epoch: usize,
        gaussian_kernel_ubo: &BufferDescriptor,
    ) -> Result<(), Error> {
        unsafe {
            let hdr_bloom_blur_first_framebuffer: *mut Framebuffer =
                self.hdr_bloom_framebuffer.as_mut().unwrap();
            let hdr_bloom_blur_even_framebuffer: *mut Framebuffer =
                self.hdr_bloom_blur_even_framebuffer(state);
            let hdr_bloom_blur_odd_framebuffer: *mut Framebuffer =
                self.hdr_bloom_blur_odd_framebuffer(state);

            let program = state
                .program_store_mut()
                .use_program(&GaussianBlurMappingProgram)?;

            for i in 0..bloom_blur_epoch {
                let (from, from_attachment, to) = if i % 2 == 0 {
                    if i == 0 {
                        // first epoch, do some initialization
                        state.bind_uniform_block_value_by_block_name(
                            program,
                            UBO_GAUSSIAN_KERNEL_BLOCK_NAME,
                            &UniformBlockValue::BufferBase {
                                descriptor: gaussian_kernel_ubo.clone(),
                                binding: UBO_GAUSSIAN_BLUR_BINDING,
                            },
                        )?;

                        (
                            &mut *hdr_bloom_blur_first_framebuffer,
                            FramebufferAttachment::COLOR_ATTACHMENT1,
                            &mut *hdr_bloom_blur_odd_framebuffer,
                        )
                    } else {
                        (
                            &mut *hdr_bloom_blur_even_framebuffer,
                            FramebufferAttachment::COLOR_ATTACHMENT0,
                            &mut *hdr_bloom_blur_odd_framebuffer,
                        )
                    }
                } else {
                    (
                        &mut *hdr_bloom_blur_odd_framebuffer,
                        FramebufferAttachment::COLOR_ATTACHMENT0,
                        &mut *hdr_bloom_blur_even_framebuffer,
                    )
                };
                to.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
                state.do_computation([(
                    from.texture(from_attachment).unwrap(),
                    TextureUnit::TEXTURE0,
                )]);
                to.unbind();
            }
        }
        Ok(())
    }

    fn blend_bloom(
        &mut self,
        state: &mut FrameState,
        bloom_blur_epoch: usize,
    ) -> Result<(), Error> {
        unsafe {
            let hdr_base_texture: *const WebGlTexture = self
                .hdr_bloom_framebuffer
                .as_ref()
                .unwrap()
                .texture(FramebufferAttachment::COLOR_ATTACHMENT0)
                .unwrap();
            let hdr_bloom_blur_texture: *const WebGlTexture = if bloom_blur_epoch == 0 {
                self.hdr_bloom_framebuffer
                    .as_ref()
                    .unwrap()
                    .texture(FramebufferAttachment::COLOR_ATTACHMENT1)
                    .unwrap()
            } else if bloom_blur_epoch % 2 == 0 {
                self.hdr_bloom_blur_even_framebuffer
                    .as_ref()
                    .unwrap()
                    .texture(FramebufferAttachment::COLOR_ATTACHMENT0)
                    .unwrap()
            } else {
                self.hdr_bloom_blur_odd_framebuffer
                    .as_ref()
                    .unwrap()
                    .texture(FramebufferAttachment::COLOR_ATTACHMENT0)
                    .unwrap()
            };
            let hdr_bloom_blend_framebuffer: *mut Framebuffer =
                self.hdr_bloom_blend_framebuffer(state);

            let program = state
                .program_store_mut()
                .use_program(&BloomBlendMappingProgram)?;
            state.bind_uniform_value_by_variable_name(
                program,
                BASE_TEXTURE_UNIFORM_NAME,
                &UniformValue::Integer1(0),
            )?;
            state.bind_uniform_value_by_variable_name(
                program,
                BLOOM_BLUR_TEXTURE_UNIFORM_NAME,
                &UniformValue::Integer1(1),
            )?;

            (*hdr_bloom_blend_framebuffer).bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
            state.do_computation([
                (&*hdr_base_texture, TextureUnit::TEXTURE0),
                (&*hdr_bloom_blur_texture, TextureUnit::TEXTURE1),
            ]);
            (*hdr_bloom_blend_framebuffer).unbind();
        }

        Ok(())
    }
}
