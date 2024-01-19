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
    texture::{TextureDataType, TextureFormat, TextureInternalFormat, TextureUnit},
    uniform::{UniformBlockValue, UniformValue},
};

pub struct StandardHdrShading {
    framebuffer: Option<Framebuffer>,
    hdr_framebuffer: Option<Framebuffer>,
    hdr_bloom_framebuffer: Option<Framebuffer>,
    hdr_bloom_blur_even_framebuffer: Option<Framebuffer>,
    hdr_bloom_blur_odd_framebuffer: Option<Framebuffer>,
    hdr_bloom_blend_framebuffer: Option<Framebuffer>,
}

impl StandardHdrShading {
    pub fn new() -> Self {
        Self {
            framebuffer: None,
            hdr_framebuffer: None,
            hdr_bloom_framebuffer: None,
            hdr_bloom_blur_even_framebuffer: None,
            hdr_bloom_blur_odd_framebuffer: None,
            hdr_bloom_blend_framebuffer: None,
        }
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
                )],
                [],
                [],
                None,
            )
        })
    }

    fn hdr_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                FramebufferSizePolicy::FollowDrawingBuffer,
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA32F,
                    TextureFormat::RGBA,
                    TextureDataType::FLOAT,
                )],
                [RenderbufferProvider::new(
                    FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
                    RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                )],
                [],
                None,
            )
        })
    }

    fn hdr_bloom_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_bloom_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                FramebufferSizePolicy::FollowDrawingBuffer,
                [
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT0,
                        TextureInternalFormat::RGBA32F,
                        TextureFormat::RGBA,
                        TextureDataType::FLOAT,
                    ),
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT1,
                        TextureInternalFormat::RGBA32F,
                        TextureFormat::RGBA,
                        TextureDataType::FLOAT,
                    ),
                ],
                [RenderbufferProvider::new(
                    FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
                    RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                )],
                [
                    FramebufferDrawBuffer::COLOR_ATTACHMENT0,
                    FramebufferDrawBuffer::COLOR_ATTACHMENT1,
                ],
                None,
            )
        })
    }

    fn hdr_bloom_blur_even_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_bloom_blur_even_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                FramebufferSizePolicy::FollowDrawingBuffer,
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA32F,
                    TextureFormat::RGBA,
                    TextureDataType::FLOAT,
                )],
                [],
                [],
                None,
            )
        })
    }

    fn hdr_bloom_blur_odd_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_bloom_blur_odd_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                FramebufferSizePolicy::FollowDrawingBuffer,
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA32F,
                    TextureFormat::RGBA,
                    TextureDataType::FLOAT,
                )],
                [],
                [],
                None,
            )
        })
    }

    fn hdr_bloom_blend_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_bloom_blend_framebuffer.get_or_insert_with(|| {
            state.create_framebuffer(
                FramebufferSizePolicy::FollowDrawingBuffer,
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA32F,
                    TextureFormat::RGBA,
                    TextureDataType::FLOAT,
                )],
                [],
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
        bloom: bool,
        bloom_blur_epoch: usize,
        tone_mapping_type: HdrToneMappingType,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: Option<&BufferDescriptor>,
        gaussian_kernel_ubo: &BufferDescriptor,
    ) -> Result<(), Error> {
        if bloom {
            self.draw_hdr_bloom(state, collected_entities, universal_ubo, lights_ubo)?;
            self.blur_bloom(state, bloom_blur_epoch, gaussian_kernel_ubo)?;
            self.blend_bloom(state, bloom_blur_epoch)?;
            self.tone_mapping_bloom(state, tone_mapping_type)?;
        } else {
            self.draw_hdr(state, collected_entities, universal_ubo, lights_ubo)?;
            self.tone_mapping(state, tone_mapping_type)?;
        }
        Ok(())
    }

    unsafe fn draw_hdr(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: Option<&BufferDescriptor>,
    ) -> Result<(), Error> {
        let fbo = self.hdr_framebuffer(state);
        fbo.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        draw_entities(
            state,
            DrawState::Draw {
                universal_ubo,
                lights_ubo,
                bloom: false,
            },
            collected_entities,
        )?;
        fbo.unbind();
        Ok(())
    }

    unsafe fn draw_hdr_bloom(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: Option<&BufferDescriptor>,
    ) -> Result<(), Error> {
        let fbo = self.hdr_bloom_framebuffer(state);
        fbo.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        draw_entities(
            state,
            DrawState::Draw {
                universal_ubo,
                lights_ubo,
                bloom: true,
            },
            collected_entities,
        )?;
        fbo.unbind();
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
                    UniformValue::Float1(exposure),
                )?;
                program
            }
        };
        state.bind_uniform_value_by_variable_name(
            program,
            HDR_TEXTURE_UNIFORM_NAME,
            UniformValue::Integer1(0),
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
            self.hdr_framebuffer.as_ref().unwrap().texture(0).unwrap(),
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
                .texture(0)
                .unwrap(),
            TextureUnit::TEXTURE0,
        )]);
        self.framebuffer(state).unbind();

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
                let (from, from_texture_index, to) = if i % 2 == 0 {
                    if i == 0 {
                        // first epoch, do some initialization
                        state.bind_uniform_block_value_by_block_name(
                            program,
                            UBO_GAUSSIAN_KERNEL_BLOCK_NAME,
                            UniformBlockValue::BufferBase {
                                descriptor: gaussian_kernel_ubo.clone(),
                                binding: UBO_GAUSSIAN_BLUR_BINDING,
                            },
                        )?;

                        (
                            &mut *hdr_bloom_blur_first_framebuffer,
                            1,
                            &mut *hdr_bloom_blur_odd_framebuffer,
                        )
                    } else {
                        (
                            &mut *hdr_bloom_blur_even_framebuffer,
                            0,
                            &mut *hdr_bloom_blur_odd_framebuffer,
                        )
                    }
                } else {
                    (
                        &mut *hdr_bloom_blur_odd_framebuffer,
                        0,
                        &mut *hdr_bloom_blur_even_framebuffer,
                    )
                };
                to.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
                state.do_computation([(
                    from.texture(from_texture_index).unwrap(),
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
                .texture(0)
                .unwrap();
            let hdr_bloom_blur_texture: *const WebGlTexture = if bloom_blur_epoch == 0 {
                self.hdr_bloom_framebuffer
                    .as_ref()
                    .unwrap()
                    .texture(1)
                    .unwrap()
            } else if bloom_blur_epoch % 2 == 0 {
                self.hdr_bloom_blur_even_framebuffer
                    .as_ref()
                    .unwrap()
                    .texture(0)
                    .unwrap()
            } else {
                self.hdr_bloom_blur_odd_framebuffer
                    .as_ref()
                    .unwrap()
                    .texture(0)
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
                UniformValue::Integer1(0),
            )?;
            state.bind_uniform_value_by_variable_name(
                program,
                BLOOM_BLUR_TEXTURE_UNIFORM_NAME,
                UniformValue::Integer1(1),
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
