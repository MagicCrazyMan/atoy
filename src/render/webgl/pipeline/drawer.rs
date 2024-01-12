use std::{any::Any, borrow::Cow, cell::OnceCell, ptr::NonNull};

use serde::{Deserialize, Serialize};
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    entity::Entity,
    geometry::Geometry,
    material::{StandardMaterial, Transparency},
    render::{
        webgl::{
            attribute::AttributeBinding,
            buffer::{BufferDescriptor, BufferSource, BufferUsage, MemoryPolicy},
            conversion::ToGlEnum,
            draw::CullFace,
            error::Error,
            framebuffer::{
                Framebuffer, FramebufferAttachment, FramebufferTarget, RenderbufferProvider,
                TextureProvider,
            },
            program::{ProgramSource, ShaderSource},
            renderbuffer::RenderbufferInternalFormat,
            state::FrameState,
            texture::{TextureDataType, TextureFormat, TextureInternalFormat},
            uniform::{
                UniformBinding, UniformBlockBinding, UniformStructuralBinding,
                UBO_GAUSSIAN_BLUR_BINDING, UBO_LIGHTS_BINDING, UBO_UNIVERSAL_UNIFORMS_BINDING,
            },
        },
        Executor, ResourceKey, Resources,
    },
    scene::Scene,
};

static UNIVERSAL_UNIFORM_BLOCK_NAME: &'static str = "atoy_UniversalUniforms";
static LIGHTS_BLOCK_NAME: &'static str = "atoy_Lights";

const SAMPLER_UNIFORM: UniformBinding = UniformBinding::Manual(Cow::Borrowed("u_Sampler"));
const SAMPLER_BLOOM_BLUR_UNIFORM: UniformBinding =
    UniformBinding::Manual(Cow::Borrowed("u_SamplerBloomBlur"));
const EXPOSURE_UNIFORM: UniformBinding = UniformBinding::Manual(Cow::Borrowed("u_Exposure"));
const BLOOM_THRESHOLD_UNIFORM: UniformBinding =
    UniformBinding::Manual(Cow::Borrowed("u_BloomThreshold"));
const GAUSSIAN_KERNEL_UNIFORM_BLOCK: UniformBlockBinding =
    UniformBlockBinding::Manual(Cow::Borrowed("Kernel"));

pub static DEFAULT_MULTISAMPLE: i32 = 4;
pub static DEFAULT_BLOOM_ENABLED: bool = true;
pub static DEFAULT_HDR_ENABLED: bool = true;
pub static DEFAULT_HDR_TONE_MAPPING_TYPE: HdrToneMappingType = HdrToneMappingType::Reinhard;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum HdrToneMappingType {
    Reinhard,
    Exposure(f32),
}

/// Standard drawer, draws all entities with its own material and geometry.
///
/// # Get Resources & Data Type
/// - `entities`: [`Vec<NonNull<Entity>>`], a list contains entities to draw.
///
/// # Provides Resources & Data Type
/// - `texture`: [`ResourceKey<WebGlTexture>`], a resource key telling where to get the draw texture.
pub struct StandardDrawer {
    normal_framebuffer: Option<Framebuffer>,
    normal_multisample_framebuffer: Option<Framebuffer>,
    hdr_framebuffer: Option<Framebuffer>,
    hdr_multisample_framebuffer: Option<Framebuffer>,
    hdr_bloom_blur_even_framebuffer: Option<Framebuffer>,
    hdr_bloom_blur_odd_framebuffer: Option<Framebuffer>,
    hdr_bloom_blend_framebuffer: Option<Framebuffer>,

    hdr_supported: Option<bool>,

    entities_key: ResourceKey<Vec<NonNull<Entity>>>,
    texture_key: ResourceKey<WebGlTexture>,
    multisample_key: Option<ResourceKey<i32>>,
    bloom_key: Option<ResourceKey<bool>>,
    hdr_key: Option<ResourceKey<bool>>,
    hdr_tone_mapping_type_key: Option<ResourceKey<HdrToneMappingType>>,
}

impl StandardDrawer {
    pub fn new(
        entities_key: ResourceKey<Vec<NonNull<Entity>>>,
        texture_key: ResourceKey<WebGlTexture>,
        multisample_key: Option<ResourceKey<i32>>,
        bloom_key: Option<ResourceKey<bool>>,
        hdr_key: Option<ResourceKey<bool>>,
        hdr_tone_mapping_type_key: Option<ResourceKey<HdrToneMappingType>>,
    ) -> Self {
        Self {
            normal_framebuffer: None,
            normal_multisample_framebuffer: None,
            hdr_framebuffer: None,
            hdr_multisample_framebuffer: None,
            hdr_bloom_blur_even_framebuffer: None,
            hdr_bloom_blur_odd_framebuffer: None,
            hdr_bloom_blend_framebuffer: None,

            hdr_supported: None,

            entities_key,
            texture_key,
            multisample_key,
            bloom_key,
            hdr_key,
            hdr_tone_mapping_type_key,
        }
    }

    fn bloom_enabled(&mut self, resources: &Resources) -> bool {
        self.bloom_key
            .as_ref()
            .and_then(|key| resources.get(key))
            .copied()
            .unwrap_or(DEFAULT_BLOOM_ENABLED)
    }

    fn bloom_blur_iterations(&self) -> usize {
        20
    }

    fn hdr_supported(&mut self, state: &FrameState) -> bool {
        if let Some(hdr_supported) = self.hdr_supported {
            return hdr_supported;
        }

        let supported = state
            .gl()
            .get_extension("EXT_color_buffer_float")
            .map(|extension| extension.is_some())
            .unwrap_or(false);
        self.hdr_supported = Some(supported);
        supported
    }

    fn hdr_enabled(&mut self, state: &FrameState, resources: &Resources) -> bool {
        if !self.hdr_supported(state) {
            return false;
        }

        self.hdr_key
            .as_ref()
            .and_then(|key| resources.get(key))
            .copied()
            .unwrap_or(DEFAULT_HDR_ENABLED)
    }

    fn hdr_tone_mapping_type(&self, resources: &Resources) -> HdrToneMappingType {
        self.hdr_tone_mapping_type_key
            .as_ref()
            .and_then(|key| resources.get(key))
            .copied()
            .unwrap_or(DEFAULT_HDR_TONE_MAPPING_TYPE)
    }

    fn multisample(&self, resources: &Resources) -> Option<i32> {
        self.multisample_key
            .as_ref()
            .map(|key| resources.get(key).cloned().unwrap_or(DEFAULT_MULTISAMPLE))
            .and_then(|samples| if samples == 0 { None } else { Some(samples) })
    }

    #[inline]
    fn normal_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.normal_framebuffer.get_or_insert_with(|| {
            log::info!("1");
            state.create_framebuffer(
                [TextureProvider::new(
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA,
                    TextureFormat::RGBA,
                    TextureDataType::UNSIGNED_BYTE,
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

    #[inline]
    fn normal_multisample_framebuffer(
        &mut self,
        state: &FrameState,
        samples: i32,
    ) -> &mut Framebuffer {
        if self
            .normal_multisample_framebuffer
            .as_ref()
            .and_then(|framebuffer| framebuffer.renderbuffer_samples())
            .map(|s| s == samples)
            .unwrap_or(false)
        {
            self.normal_multisample_framebuffer.as_mut().unwrap()
        } else {
            log::info!("3");
            self.normal_multisample_framebuffer
                .insert(state.create_framebuffer(
                    [],
                    [
                        RenderbufferProvider::new(
                            FramebufferAttachment::COLOR_ATTACHMENT0,
                            RenderbufferInternalFormat::RGBA8,
                        ),
                        RenderbufferProvider::new(
                            FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
                            RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                        ),
                    ],
                    [],
                    Some(samples),
                ))
        }
    }

    #[inline]
    fn hdr_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_framebuffer.get_or_insert_with(|| {
            log::info!("2");
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

    #[inline]
    fn hdr_multisample_framebuffer(
        &mut self,
        state: &FrameState,
        samples: i32,
    ) -> &mut Framebuffer {
        if self
            .hdr_multisample_framebuffer
            .as_ref()
            .and_then(|framebuffer| framebuffer.renderbuffer_samples())
            .map(|s| s == samples)
            .unwrap_or(false)
        {
            self.hdr_multisample_framebuffer.as_mut().unwrap()
        } else {
            log::info!("4");
            self.hdr_multisample_framebuffer
                .insert(state.create_framebuffer(
                    [],
                    [
                        RenderbufferProvider::new(
                            FramebufferAttachment::COLOR_ATTACHMENT0,
                            RenderbufferInternalFormat::RGBA32F,
                        ),
                        RenderbufferProvider::new(
                            FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
                            RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                        ),
                    ],
                    [],
                    Some(samples),
                ))
        }
    }

    fn hdr_bloom_blur_even_framebuffer(&mut self, state: &FrameState) -> &mut Framebuffer {
        self.hdr_bloom_blur_even_framebuffer.get_or_insert_with(|| {
            log::info!("5");
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
            log::info!("6");
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
            log::info!("7");
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

    fn prepare_entities<'a, 'b>(
        &'a self,
        state: &mut FrameState,
        resources: &mut Resources,
    ) -> Result<
        Option<(
            Vec<(&'b Entity, &'b dyn Geometry, &'b dyn StandardMaterial)>,
            Vec<(&'b Entity, &'b dyn Geometry, &'b dyn StandardMaterial)>,
        )>,
        Error,
    > {
        let Some(entities) = resources.get_mut(&self.entities_key) else {
            return Ok(None);
        };

        let mut opaques = Vec::new();
        let mut translucents = Vec::new();
        entities.iter_mut().for_each(|entity| unsafe {
            // prepares material and geometry
            if let Some(material) = entity.as_mut().material_mut() {
                material.prepare(state, entity.as_ref());
            };

            let entity = entity.as_ref();
            if let (Some(geometry), Some(material)) = (entity.geometry(), entity.material()) {
                // filters unready material
                if !material.ready() {
                    return;
                }

                // filters transparent material
                if material.transparency() == Transparency::Transparent {
                    return;
                }

                if material.transparency() == Transparency::Opaque {
                    opaques.push((entity, geometry, material));
                } else {
                    translucents.push((entity, geometry, material));
                }
            }
        });

        Ok(Some((opaques, translucents)))
    }

    fn draw_entity(
        &self,
        state: &mut FrameState,
        entity: &Entity,
        geometry: &dyn Geometry,
        material: &dyn StandardMaterial,
        cull_face: Option<CullFace>,
    ) -> Result<(), Error> {
        if let Some(cull_face) = cull_face {
            state.gl().enable(WebGl2RenderingContext::CULL_FACE);
            state.gl().cull_face(cull_face.gl_enum());
        } else {
            state.gl().disable(WebGl2RenderingContext::CULL_FACE);
        }

        let program = state.program_store_mut().use_program(&material.source())?;
        let bound_attributes = state.bind_attributes(&entity, geometry, material)?;
        let bound_uniforms = state.bind_uniforms(&entity, geometry, material)?;
        state.gl().uniform_block_binding(
            program.gl_program(),
            state
                .gl()
                .get_uniform_block_index(program.gl_program(), UNIVERSAL_UNIFORM_BLOCK_NAME),
            UBO_UNIVERSAL_UNIFORMS_BINDING,
        );
        state.gl().uniform_block_binding(
            program.gl_program(),
            state
                .gl()
                .get_uniform_block_index(program.gl_program(), LIGHTS_BLOCK_NAME),
            UBO_LIGHTS_BINDING,
        );
        state.draw(&geometry.draw())?;
        state.unbind_attributes(bound_attributes);
        state.unbind_uniforms(bound_uniforms);

        Ok(())
    }

    fn draw_entities(
        &self,
        state: &mut FrameState,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
    ) -> Result<(), Error> {
        state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl().clear_depth(1.0);
        state.gl().clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );

        // draws opaque enable DEPTH_TEST and disable BLEND and draws them from nearest to farthest first
        state.gl().depth_mask(true);
        for (entity, geometry, material) in opaques {
            self.draw_entity(state, entity, geometry, material, geometry.cull_face())?;
        }

        // then draws translucents first with DEPTH_TEST unchangeable and enable BLEND and draws them from farthest to nearest
        state.gl().enable(WebGl2RenderingContext::BLEND);
        state.gl().blend_equation(WebGl2RenderingContext::FUNC_ADD);
        state.gl().blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        state.gl().depth_mask(false);
        for (entity, geometry, material) in translucents.into_iter().rev() {
            self.draw_entity(state, entity, geometry, material, None)?; // transparency entities never cull face
        }

        // reset to default
        state.gl().depth_mask(true);
        state.gl().disable(WebGl2RenderingContext::BLEND);
        state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl().disable(WebGl2RenderingContext::CULL_FACE);
        state.gl().cull_face(WebGl2RenderingContext::BACK);
        state
            .gl()
            .blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ZERO);

        Ok(())
    }

    fn draw_normal(
        &mut self,
        state: &mut FrameState,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
    ) -> Result<(), Error> {
        self.normal_framebuffer(state)
            .bind(FramebufferTarget::FRAMEBUFFER)?;
        self.draw_entities(state, opaques, translucents)?;
        self.normal_framebuffer(state).unbind();

        Ok(())
    }

    fn draw_normal_multisample(
        &mut self,
        state: &mut FrameState,
        samples: i32,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
    ) -> Result<(), Error> {
        self.normal_multisample_framebuffer(state, samples)
            .bind(FramebufferTarget::FRAMEBUFFER)?;
        self.draw_entities(state, opaques, translucents)?;
        self.normal_multisample_framebuffer(state, samples).unbind();

        Ok(())
    }

    fn blit_normal_multisample(&mut self, state: &FrameState, samples: i32) -> Result<(), Error> {
        unsafe {
            let normal_framebuffer: *mut Framebuffer = self.normal_framebuffer(state);
            let normal_multisample_framebuffer: *mut Framebuffer =
                self.normal_multisample_framebuffer(state, samples);

            (*normal_framebuffer).bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
            (*normal_multisample_framebuffer).bind(FramebufferTarget::READ_FRAMEBUFFER)?;
            state.gl().blit_framebuffer(
                0,
                0,
                (*normal_multisample_framebuffer).width(),
                (*normal_multisample_framebuffer).height(),
                0,
                0,
                (*normal_framebuffer).width(),
                (*normal_framebuffer).height(),
                WebGl2RenderingContext::COLOR_BUFFER_BIT,
                WebGl2RenderingContext::LINEAR,
            );
            (*normal_framebuffer).unbind();
            (*normal_multisample_framebuffer).unbind();
        }

        Ok(())
    }

    fn draw_hdr(
        &mut self,
        state: &mut FrameState,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
    ) -> Result<(), Error> {
        self.hdr_framebuffer(state)
            .bind(FramebufferTarget::FRAMEBUFFER)?;
        self.draw_entities(state, opaques, translucents)?;
        self.hdr_framebuffer(state).unbind();

        Ok(())
    }

    fn draw_hdr_multisample(
        &mut self,
        state: &mut FrameState,
        samples: i32,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
    ) -> Result<(), Error> {
        self.hdr_multisample_framebuffer(state, samples)
            .bind(FramebufferTarget::FRAMEBUFFER)?;
        self.draw_entities(state, opaques, translucents)?;
        self.hdr_multisample_framebuffer(state, samples).unbind();

        Ok(())
    }

    fn blit_hdr_multisample(&mut self, state: &FrameState, samples: i32) -> Result<(), Error> {
        unsafe {
            let hdr_framebuffer: *mut Framebuffer = self.hdr_framebuffer(state);
            let hdr_multisample_framebuffer: *mut Framebuffer =
                self.hdr_multisample_framebuffer(state, samples);

            (*hdr_framebuffer).bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
            (*hdr_multisample_framebuffer).bind(FramebufferTarget::READ_FRAMEBUFFER)?;
            state.gl().blit_framebuffer(
                0,
                0,
                (*hdr_multisample_framebuffer).width(),
                (*hdr_multisample_framebuffer).height(),
                0,
                0,
                (*hdr_framebuffer).width(),
                (*hdr_framebuffer).height(),
                WebGl2RenderingContext::COLOR_BUFFER_BIT,
                WebGl2RenderingContext::LINEAR,
            );
            (*hdr_framebuffer).unbind();
            (*hdr_multisample_framebuffer).unbind();
        }

        Ok(())
    }

    fn hdr_reinhard_tone_mapping(
        &mut self,
        state: &mut FrameState,
        texture: &WebGlTexture,
    ) -> Result<(), Error> {
        let normal_framebuffer = self.normal_framebuffer(state);
        let tone_mapping_program = state
            .program_store_mut()
            .use_program(&HdrReinhardToneMapping)?;

        normal_framebuffer.bind(FramebufferTarget::FRAMEBUFFER)?;
        state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        state.gl().uniform1i(
            tone_mapping_program
                .uniform_locations()
                .get(&SAMPLER_UNIFORM),
            0,
        );
        state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);
        state
            .gl()
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
        state.gl().tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        state.gl().tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        state.gl().tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
            0,
        );
        state
            .gl()
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);
        state
            .gl()
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        normal_framebuffer.unbind();

        Ok(())
    }

    fn hdr_exposure_tone_mapping(
        &mut self,
        state: &mut FrameState,
        texture: &WebGlTexture,
        exposure: f32,
    ) -> Result<(), Error> {
        let normal_framebuffer = self.normal_framebuffer(state);
        let tone_mapping_program = state
            .program_store_mut()
            .use_program(&HdrExposureToneMapping)?;

        normal_framebuffer.bind(FramebufferTarget::FRAMEBUFFER)?;
        state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        state.gl().uniform1i(
            tone_mapping_program
                .uniform_locations()
                .get(&SAMPLER_UNIFORM),
            0,
        );
        state.gl().uniform1f(
            tone_mapping_program
                .uniform_locations()
                .get(&EXPOSURE_UNIFORM),
            exposure,
        );
        state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);
        state
            .gl()
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
        state.gl().tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MIN_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        state.gl().tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_MAG_FILTER,
            WebGl2RenderingContext::NEAREST as i32,
        );
        state.gl().tex_parameteri(
            WebGl2RenderingContext::TEXTURE_2D,
            WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
            0,
        );
        state
            .gl()
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);
        state
            .gl()
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        normal_framebuffer.unbind();

        Ok(())
    }

    fn hdr_tone_mapping(
        &mut self,
        state: &mut FrameState,
        resources: &Resources,
    ) -> Result<(), Error> {
        let texture: *const WebGlTexture = match self.bloom_enabled(resources) {
            true => self.hdr_bloom_blend_framebuffer(state).texture(0).unwrap(),
            false => self.hdr_framebuffer(state).texture(0).unwrap(),
        };

        match self.hdr_tone_mapping_type(resources) {
            HdrToneMappingType::Reinhard => {
                self.hdr_reinhard_tone_mapping(state, unsafe { &*texture })?
            }
            HdrToneMappingType::Exposure(exposure) => {
                self.hdr_exposure_tone_mapping(state, unsafe { &*texture }, exposure)?
            }
        };
        Ok(())
    }

    fn hdr_bloom_mapping(&mut self, state: &mut FrameState) -> Result<(), Error> {
        unsafe {
            let hdr_framebuffer: *mut Framebuffer = self.hdr_framebuffer(state);
            let hdr_bloom_blur_even_framebuffer: *mut Framebuffer =
                self.hdr_bloom_blur_even_framebuffer(state);
            let program = state.program_store_mut().use_program(&BloomMapping)?;

            (*hdr_bloom_blur_even_framebuffer).bind(FramebufferTarget::FRAMEBUFFER)?;
            state.gl().uniform3f(
                program.uniform_locations().get(&BLOOM_THRESHOLD_UNIFORM),
                0.2126,
                0.7152,
                0.0722,
            );
            state
                .gl()
                .uniform1i(program.uniform_locations().get(&SAMPLER_UNIFORM), 0);
            state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);
            state.gl().bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                (*hdr_framebuffer).texture(0),
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                0,
            );
            state
                .gl()
                .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);
            state
                .gl()
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            (*hdr_bloom_blur_even_framebuffer).unbind();
        }

        Ok(())
    }

    fn hdr_bloom_blur(&mut self, state: &mut FrameState) -> Result<(), Error> {
        unsafe {
            let hdr_bloom_blur_even_framebuffer: *mut Framebuffer =
                self.hdr_bloom_blur_even_framebuffer(state);
            let hdr_bloom_blur_odd_framebuffer: *mut Framebuffer =
                self.hdr_bloom_blur_odd_framebuffer(state);
            let program = state
                .program_store_mut()
                .use_program(&GaussianBlurMapping)?;

            state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);
            state
                .buffer_store_mut()
                .bind_uniform_buffer_object(&gaussian_kernel(), UBO_GAUSSIAN_BLUR_BINDING)?;
            state.gl().uniform_block_binding(
                program.gl_program(),
                program
                    .uniform_block_indices()
                    .get(&GAUSSIAN_KERNEL_UNIFORM_BLOCK)
                    .cloned()
                    .unwrap(),
                UBO_GAUSSIAN_BLUR_BINDING,
            );
            state
                .gl()
                .uniform1i(program.uniform_locations().get(&SAMPLER_UNIFORM), 0);

            for i in 0..self.bloom_blur_iterations() {
                let (from, to) = if i % 2 == 0 {
                    (
                        &mut *hdr_bloom_blur_even_framebuffer,
                        &mut *hdr_bloom_blur_odd_framebuffer,
                    )
                } else {
                    (
                        &mut *hdr_bloom_blur_odd_framebuffer,
                        &mut *hdr_bloom_blur_even_framebuffer,
                    )
                };

                to.bind(FramebufferTarget::FRAMEBUFFER)?;
                // state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
                // state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
                state
                    .gl()
                    .bind_texture(WebGl2RenderingContext::TEXTURE_2D, from.texture(0));
                state.gl().tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                    WebGl2RenderingContext::NEAREST as i32,
                );
                state.gl().tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                    WebGl2RenderingContext::NEAREST as i32,
                );
                state.gl().tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    WebGl2RenderingContext::TEXTURE_WRAP_S,
                    WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
                );
                state.gl().tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    WebGl2RenderingContext::TEXTURE_WRAP_T,
                    WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
                );
                state.gl().tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                    0,
                );
                state
                    .gl()
                    .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);
                to.unbind();
            }

            state
                .gl()
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        }
        Ok(())
    }

    fn hdr_bloom_blend(&mut self, state: &mut FrameState) -> Result<(), Error> {
        unsafe {
            let hdr_framebuffer: *mut Framebuffer = self.hdr_framebuffer(state);
            let hdr_bloom_blur_framebuffer: *mut Framebuffer =
                if self.bloom_blur_iterations() % 2 == 0 {
                    self.hdr_bloom_blur_even_framebuffer(state)
                } else {
                    self.hdr_bloom_blur_odd_framebuffer(state)
                };
            let hdr_bloom_blend_framebuffer: *mut Framebuffer =
                self.hdr_bloom_blend_framebuffer(state);
            let program = state.program_store_mut().use_program(&BloomBlendMapping)?;

            (*hdr_bloom_blend_framebuffer).bind(FramebufferTarget::FRAMEBUFFER)?;
            state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
            state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
            state
                .gl()
                .uniform1i(program.uniform_locations().get(&SAMPLER_UNIFORM), 0);
            state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);
            state.gl().bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                (*hdr_framebuffer).texture(0),
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                0,
            );

            state.gl().uniform1i(
                program.uniform_locations().get(&SAMPLER_BLOOM_BLUR_UNIFORM),
                1,
            );
            state.gl().active_texture(WebGl2RenderingContext::TEXTURE1);
            state.gl().bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                (*hdr_bloom_blur_framebuffer).texture(0),
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                0,
            );

            state
                .gl()
                .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);

            state
                .gl()
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);
            state
                .gl()
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            (*hdr_bloom_blend_framebuffer).unbind();
        }

        Ok(())
    }
}

impl Executor for StandardDrawer {
    type State = FrameState;

    type Error = Error;

    fn before(
        &mut self,
        _: &mut Self::State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<bool, Self::Error> {
        Ok(resources.contains_resource(&self.entities_key))
    }

    fn after(
        &mut self,
        _: &mut Self::State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        if let Some(texture) = self
            .normal_framebuffer
            .as_ref()
            .and_then(|fbo| fbo.texture(0))
        {
            resources.insert(self.texture_key.clone(), texture.clone());
        }

        Ok(())
    }

    fn execute(
        &mut self,
        state: &mut Self::State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        let Some((opaques, translucents)) = self.prepare_entities(state, resources)? else {
            return Ok(());
        };

        match (
            self.multisample(resources),
            self.hdr_enabled(&state, resources),
        ) {
            (None, true) => {
                self.draw_hdr(state, opaques, translucents)?;
                if self.bloom_enabled(resources) {
                    self.hdr_bloom_mapping(state)?;
                    self.hdr_bloom_blur(state)?;
                    self.hdr_bloom_blend(state)?;
                }
                self.hdr_tone_mapping(state, &resources)?;
            }
            (None, false) => {
                self.draw_normal(state, opaques, translucents)?;
            }
            (Some(samples), true) => {
                self.draw_hdr_multisample(state, samples, opaques, translucents)?;
                self.blit_hdr_multisample(state, samples)?;
                if self.bloom_enabled(resources) {
                    self.hdr_bloom_mapping(state)?;
                    self.hdr_bloom_blur(state)?;
                    self.hdr_bloom_blend(state)?;
                }
                self.hdr_tone_mapping(state, &resources)?;
            }
            (Some(samples), false) => {
                self.draw_normal_multisample(state, samples, opaques, translucents)?;
                self.blit_normal_multisample(state, samples)?;
            }
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct HdrReinhardToneMapping;

impl ProgramSource for HdrReinhardToneMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("HdrReinhardToneMapping")
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::VertexRaw(Cow::Borrowed(include_str!("./shaders/computation.vert"))),
            ShaderSource::FragmentRaw(Cow::Borrowed(include_str!(
                "./shaders/hdr_reinhard_tone_mapping.frag"
            ))),
        ]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        vec![]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![SAMPLER_UNIFORM]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }
}

struct HdrExposureToneMapping;

impl ProgramSource for HdrExposureToneMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("HdrExposureToneMapping")
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::VertexRaw(Cow::Borrowed(include_str!("./shaders/computation.vert"))),
            ShaderSource::FragmentRaw(Cow::Borrowed(include_str!(
                "./shaders/hdr_exposure_tone_mapping.frag"
            ))),
        ]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        vec![]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![SAMPLER_UNIFORM, EXPOSURE_UNIFORM]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }
}

struct BloomMapping;

impl ProgramSource for BloomMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("BloomMapping")
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::VertexRaw(Cow::Borrowed(include_str!("./shaders/computation.vert"))),
            ShaderSource::FragmentRaw(Cow::Borrowed(include_str!("./shaders/bloom_mapping.frag"))),
        ]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        vec![]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![SAMPLER_UNIFORM, BLOOM_THRESHOLD_UNIFORM]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }
}

#[rustfmt::skip]
static GAUSSIAN_KERNEL: [f32; 324] = [
    0.0002629586560000000, 0.0, 0.0, 0.0,
    0.0008765396640000000, 0.0, 0.0, 0.0,
    0.0019722158656000000, 0.0, 0.0, 0.0,
    0.0031555460336000003, 0.0, 0.0, 0.0,
    0.0036814698320000003, 0.0, 0.0, 0.0,
    0.0031555460336000003, 0.0, 0.0, 0.0,
    0.0019722158656000000, 0.0, 0.0, 0.0,
    0.0008765396640000000, 0.0, 0.0, 0.0,
    0.0002629586560000000, 0.0, 0.0, 0.0,
    0.0008765396640000000, 0.0, 0.0, 0.0,
    0.0029218349159999997, 0.0, 0.0, 0.0,
    0.0065741339663999990, 0.0, 0.0, 0.0,
    0.0105186165084000000, 0.0, 0.0, 0.0,
    0.0122717174580000000, 0.0, 0.0, 0.0,
    0.0105186165084000000, 0.0, 0.0, 0.0,
    0.0065741339663999990, 0.0, 0.0, 0.0,
    0.0029218349159999997, 0.0, 0.0, 0.0,
    0.0008765396640000000, 0.0, 0.0, 0.0,
    0.0019722158656000000, 0.0, 0.0, 0.0,
    0.0065741339663999990, 0.0, 0.0, 0.0,
    0.0147918135865600000, 0.0, 0.0, 0.0,
    0.0236669066033600000, 0.0, 0.0, 0.0,
    0.0276113869832000000, 0.0, 0.0, 0.0,
    0.0236669066033600000, 0.0, 0.0, 0.0,
    0.0147918135865600000, 0.0, 0.0, 0.0,
    0.0065741339663999990, 0.0, 0.0, 0.0,
    0.0019722158656000000, 0.0, 0.0, 0.0,
    0.0031555460336000003, 0.0, 0.0, 0.0,
    0.0105186165084000000, 0.0, 0.0, 0.0,
    0.0236669066033600000, 0.0, 0.0, 0.0,
    0.0378670583491600000, 0.0, 0.0, 0.0,
    0.0441782282542000000, 0.0, 0.0, 0.0,
    0.0378670583491600000, 0.0, 0.0, 0.0,
    0.0236669066033600000, 0.0, 0.0, 0.0,
    0.0105186165084000000, 0.0, 0.0, 0.0,
    0.0031555460336000003, 0.0, 0.0, 0.0,
    0.0036814698320000003, 0.0, 0.0, 0.0,
    0.0122717174580000000, 0.0, 0.0, 0.0,
    0.0276113869832000000, 0.0, 0.0, 0.0,
    0.0441782282542000000, 0.0, 0.0, 0.0,
    0.0515412587290000060, 0.0, 0.0, 0.0,
    0.0441782282542000000, 0.0, 0.0, 0.0,
    0.0276113869832000000, 0.0, 0.0, 0.0,
    0.0122717174580000000, 0.0, 0.0, 0.0,
    0.0036814698320000003, 0.0, 0.0, 0.0,
    0.0031555460336000003, 0.0, 0.0, 0.0,
    0.0105186165084000000, 0.0, 0.0, 0.0,
    0.0236669066033600000, 0.0, 0.0, 0.0,
    0.0378670583491600000, 0.0, 0.0, 0.0,
    0.0441782282542000000, 0.0, 0.0, 0.0,
    0.0378670583491600000, 0.0, 0.0, 0.0,
    0.0236669066033600000, 0.0, 0.0, 0.0,
    0.0105186165084000000, 0.0, 0.0, 0.0,
    0.0031555460336000003, 0.0, 0.0, 0.0,
    0.0019722158656000000, 0.0, 0.0, 0.0,
    0.0065741339663999990, 0.0, 0.0, 0.0,
    0.0147918135865600000, 0.0, 0.0, 0.0,
    0.0236669066033600000, 0.0, 0.0, 0.0,
    0.0276113869832000000, 0.0, 0.0, 0.0,
    0.0236669066033600000, 0.0, 0.0, 0.0,
    0.0147918135865600000, 0.0, 0.0, 0.0,
    0.0065741339663999990, 0.0, 0.0, 0.0,
    0.0019722158656000000, 0.0, 0.0, 0.0,
    0.0008765396640000000, 0.0, 0.0, 0.0,
    0.0029218349159999997, 0.0, 0.0, 0.0,
    0.0065741339663999990, 0.0, 0.0, 0.0,
    0.0105186165084000000, 0.0, 0.0, 0.0,
    0.0122717174580000000, 0.0, 0.0, 0.0,
    0.0105186165084000000, 0.0, 0.0, 0.0,
    0.0065741339663999990, 0.0, 0.0, 0.0,
    0.0029218349159999997, 0.0, 0.0, 0.0,
    0.0008765396640000000, 0.0, 0.0, 0.0,
    0.0002629586560000000, 0.0, 0.0, 0.0,
    0.0008765396640000000, 0.0, 0.0, 0.0,
    0.0019722158656000000, 0.0, 0.0, 0.0,
    0.0031555460336000003, 0.0, 0.0, 0.0,
    0.0036814698320000003, 0.0, 0.0, 0.0,
    0.0031555460336000003, 0.0, 0.0, 0.0,
    0.0019722158656000000, 0.0, 0.0, 0.0,
    0.0008765396640000000, 0.0, 0.0, 0.0,
    0.0002629586560000000, 0.0, 0.0, 0.0,
];

static GAUSSIAN_KERNEL_BINARY: [u8; 324 * 4] =
    unsafe { std::mem::transmute_copy::<[f32; 324], [u8; 324 * 4]>(&GAUSSIAN_KERNEL) };

static mut GAUSSIAN_KERNEL_BUFFER_DESCRIPTOR: OnceCell<BufferDescriptor> = OnceCell::new();

fn gaussian_kernel() -> BufferDescriptor {
    unsafe {
        GAUSSIAN_KERNEL_BUFFER_DESCRIPTOR
            .get_or_init(|| {
                BufferDescriptor::with_memory_policy(
                    BufferSource::from_binary(
                        &GAUSSIAN_KERNEL_BINARY,
                        0,
                        GAUSSIAN_KERNEL_BINARY.len() as u32,
                    ),
                    BufferUsage::StaticDraw,
                    MemoryPolicy::restorable(|| {
                        BufferSource::from_binary(
                            &GAUSSIAN_KERNEL_BINARY,
                            0,
                            GAUSSIAN_KERNEL_BINARY.len() as u32,
                        )
                    }),
                )
            })
            .clone()
    }
}

struct GaussianBlurMapping;

impl ProgramSource for GaussianBlurMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("GaussianBlurMapping")
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::VertexRaw(Cow::Borrowed(include_str!("./shaders/computation.vert"))),
            ShaderSource::FragmentRaw(Cow::Borrowed(include_str!("./shaders/gaussian_blur.frag"))),
        ]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        vec![]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![SAMPLER_UNIFORM]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![GAUSSIAN_KERNEL_UNIFORM_BLOCK]
    }
}

struct BloomBlendMapping;

impl ProgramSource for BloomBlendMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("BloomBlendMapping")
    }

    fn sources(&self) -> Vec<ShaderSource> {
        vec![
            ShaderSource::VertexRaw(Cow::Borrowed(include_str!("./shaders/computation.vert"))),
            ShaderSource::FragmentRaw(Cow::Borrowed(include_str!("./shaders/bloom_blend.frag"))),
        ]
    }

    fn attribute_bindings(&self) -> Vec<AttributeBinding> {
        vec![]
    }

    fn uniform_bindings(&self) -> Vec<UniformBinding> {
        vec![SAMPLER_UNIFORM, SAMPLER_BLOOM_BLUR_UNIFORM]
    }

    fn uniform_structural_bindings(&self) -> Vec<UniformStructuralBinding> {
        vec![]
    }

    fn uniform_block_bindings(&self) -> Vec<UniformBlockBinding> {
        vec![]
    }
}
