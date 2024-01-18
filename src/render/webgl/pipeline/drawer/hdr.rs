use web_sys::WebGlTexture;

use crate::render::webgl::{
    buffer::BufferDescriptor,
    error::Error,
    framebuffer::{
        Framebuffer, FramebufferAttachment, FramebufferDrawBuffer, FramebufferTarget,
        RenderbufferProvider, TextureProvider,
    },
    pipeline::collector::CollectedEntities,
    renderbuffer::RenderbufferInternalFormat,
    state::FrameState,
    texture::{TextureDataType, TextureFormat, TextureInternalFormat, TextureUnit},
    uniform::{UniformBlockValue, UniformValue, UBO_GAUSSIAN_BLUR_BINDING},
};

use super::{
    draw_entities, gaussian_kernel, BloomBlendMapping, GaussianBlurMapping, HdrExposureToneMapping,
    HdrReinhardToneMapping, HdrToneMappingType, BASE_TEXTURE, BLOOM_BLUR_TEXTURE,
    GAUSSIAN_KERNEL_BLOCK_NAME, HDR_EXPOSURE_UNIFORM_NAME, HDR_TEXTURE_UNIFORM_NAME,
};

pub struct StandardHdrDrawer {
    framebuffer: Option<Framebuffer>,
    hdr_framebuffer: Option<Framebuffer>,
    hdr_bloom_framebuffer: Option<Framebuffer>,
    hdr_bloom_blur_even_framebuffer: Option<Framebuffer>,
    hdr_bloom_blur_odd_framebuffer: Option<Framebuffer>,
    hdr_bloom_blend_framebuffer: Option<Framebuffer>,
}

impl StandardHdrDrawer {
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
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA,
                    TextureFormat::RGBA,
                    TextureDataType::UNSIGNED_BYTE,
                    0,
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
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA32F,
                    TextureFormat::RGBA,
                    TextureDataType::FLOAT,
                    0,
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
                [
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT0,
                        TextureInternalFormat::RGBA32F,
                        TextureFormat::RGBA,
                        TextureDataType::FLOAT,
                        0,
                    ),
                    TextureProvider::new(
                        FramebufferAttachment::COLOR_ATTACHMENT1,
                        TextureInternalFormat::RGBA32F,
                        TextureFormat::RGBA,
                        TextureDataType::FLOAT,
                        0,
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
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA32F,
                    TextureFormat::RGBA,
                    TextureDataType::FLOAT,
                    0,
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
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA32F,
                    TextureFormat::RGBA,
                    TextureDataType::FLOAT,
                    0,
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
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA32F,
                    TextureFormat::RGBA,
                    TextureDataType::FLOAT,
                    0,
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

    pub fn draw(
        &mut self,
        state: &mut FrameState,
        bloom_blur: bool,
        bloom_blur_epoch: usize,
        tone_mapping_type: HdrToneMappingType,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: &BufferDescriptor,
    ) -> Result<(), Error> {
        if bloom_blur {
            self.draw_hdr_bloom(state, collected_entities, universal_ubo, lights_ubo)?;
            self.blur_bloom(state, bloom_blur_epoch)?;
            self.blend_bloom(state, bloom_blur_epoch)?;
            self.tone_mapping_bloom(state, tone_mapping_type)?;
        } else {
            self.draw_hdr(state, collected_entities, universal_ubo, lights_ubo)?;
            self.tone_mapping(state, tone_mapping_type)?;
        }
        Ok(())
    }

    fn draw_hdr(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: &BufferDescriptor,
    ) -> Result<(), Error> {
        let fbo = self.hdr_framebuffer(state);
        fbo.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        draw_entities(state, false, collected_entities, universal_ubo, lights_ubo)?;
        fbo.unbind();
        Ok(())
    }

    fn draw_hdr_bloom(
        &mut self,
        state: &mut FrameState,
        collected_entities: &CollectedEntities,
        universal_ubo: &BufferDescriptor,
        lights_ubo: &BufferDescriptor,
    ) -> Result<(), Error> {
        let fbo = self.hdr_bloom_framebuffer(state);
        fbo.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        draw_entities(state, true, collected_entities, universal_ubo, lights_ubo)?;
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
                .use_program(&HdrReinhardToneMapping)?,
            HdrToneMappingType::Exposure(exposure) => {
                let program = state
                    .program_store_mut()
                    .use_program(&HdrExposureToneMapping)?;
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

        self.framebuffer(state).bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
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
        
        self.framebuffer(state).bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        state.do_computation([(
            self.hdr_bloom_blend_framebuffer.as_ref().unwrap().texture(0).unwrap(),
            TextureUnit::TEXTURE0,
        )]);
        self.framebuffer(state).unbind();

        Ok(())
    }

    fn blur_bloom(
        &mut self,
        state: &mut FrameState,
        bloom_blur_epoch: usize,
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
                .use_program(&GaussianBlurMapping)?;

            for i in 0..bloom_blur_epoch {
                let (from, from_texture_index, to) = if i % 2 == 0 {
                    if i == 0 {
                        // first epoch, do some initialization
                        state.bind_uniform_block_value_by_block_name(
                            program,
                            GAUSSIAN_KERNEL_BLOCK_NAME,
                            UniformBlockValue::BufferBase {
                                descriptor: gaussian_kernel(),
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

            let program = state.program_store_mut().use_program(&BloomBlendMapping)?;
            state.bind_uniform_value_by_variable_name(
                program,
                BASE_TEXTURE,
                UniformValue::Integer1(0),
            )?;
            state.bind_uniform_value_by_variable_name(
                program,
                BLOOM_BLUR_TEXTURE,
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
