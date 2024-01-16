use std::{any::Any, borrow::Cow, cell::OnceCell, iter::FromIterator, ptr::NonNull};

use gl_matrix4rust::GLF32;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::{
    js_sys::{Array, ArrayBuffer, Float32Array},
    WebGl2RenderingContext, WebGlTexture,
};

use crate::{
    entity::Entity,
    geometry::Geometry,
    material::{StandardMaterial, Transparency},
    render::{
        webgl::{
            buffer::{BufferDescriptor, BufferSource, BufferUsage, MemoryPolicy},
            conversion::ToGlEnum,
            draw::CullFace,
            error::Error,
            framebuffer::{
                Framebuffer, FramebufferAttachment, FramebufferDrawBuffer, FramebufferTarget,
                RenderbufferProvider, TextureProvider,
            },
            program::{FragmentShaderSource, ProgramSource, VertexShaderSource},
            renderbuffer::RenderbufferInternalFormat,
            state::FrameState,
            texture::{TextureDataType, TextureFormat, TextureInternalFormat},
            uniform::{
                UniformBlockValue, UniformValue, UBO_GAUSSIAN_BLUR_BINDING,
                UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH, UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET,
                UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH, UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET,
                UBO_LIGHTS_ATTENUATIONS_BYTES_LENGTH, UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET,
                UBO_LIGHTS_BINDING, UBO_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET, UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET, UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET, UBO_UNIVERSAL_UNIFORMS_BINDING,
                UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET,
                UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_OFFSET,
                UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET,
                UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET,
                UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET,
                UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET,
            },
        },
        Executor, ResourceKey, Resources,
    },
    scene::Scene,
};

static ENABLE_BLOOM_DEFINE: &'static str = "BLOOM";

static UNIVERSAL_UNIFORMS_BLOCK: &'static str = "atoy_UniversalUniforms";
static LIGHTS_BLOCK: &'static str = "atoy_Lights";
static GAUSSIAN_KERNEL_BLOCK: &'static str = "atoy_GaussianKernel";

static BLOOM_THRESHOLD: &'static str = "u_BloomThreshold";
static BASE_TEXTURE: &'static str = "u_BaseTexture";
static BLOOM_BLUR_TEXTURE: &'static str = "u_BloomBlurTexture";
static HDR_TEXTURE: &'static str = "u_HdrTexture";
static HDR_EXPOSURE: &'static str = "u_HdrExposure";

pub static DEFAULT_MULTISAMPLE: i32 = 4;
pub static DEFAULT_BLOOM_ENABLED: bool = true;
pub static DEFAULT_BLOOM_BLUR_EPOCH: usize = 10;
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
    universal_ubo: BufferDescriptor,
    lights_ubo: BufferDescriptor,

    entities_key: ResourceKey<Vec<NonNull<Entity>>>,
    texture_key: ResourceKey<WebGlTexture>,
    multisample_key: Option<ResourceKey<i32>>,
    bloom_key: Option<ResourceKey<bool>>,
    bloom_epoch_key: Option<ResourceKey<usize>>,
    hdr_key: Option<ResourceKey<bool>>,
    hdr_tone_mapping_type_key: Option<ResourceKey<HdrToneMappingType>>,

    previous_bloom_enabled: Option<bool>,
    blit_color_attachment_1: Array,
    blit_color_attachment_reset: Array,
}

impl StandardDrawer {
    pub fn new(
        entities_key: ResourceKey<Vec<NonNull<Entity>>>,
        texture_key: ResourceKey<WebGlTexture>,
        multisample_key: Option<ResourceKey<i32>>,
        hdr_key: Option<ResourceKey<bool>>,
        hdr_tone_mapping_type_key: Option<ResourceKey<HdrToneMappingType>>,
        bloom_key: Option<ResourceKey<bool>>,
        bloom_epoch_key: Option<ResourceKey<usize>>,
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
            universal_ubo: BufferDescriptor::with_memory_policy(
                BufferSource::preallocate(UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH as i32),
                BufferUsage::DynamicDraw,
                MemoryPolicy::Unfree,
            ),
            lights_ubo: BufferDescriptor::with_memory_policy(
                BufferSource::preallocate(UBO_LIGHTS_BYTES_LENGTH as i32),
                BufferUsage::DynamicDraw,
                MemoryPolicy::Unfree,
            ),

            entities_key,
            texture_key,
            multisample_key,
            bloom_key,
            bloom_epoch_key,
            hdr_key,
            hdr_tone_mapping_type_key,

            previous_bloom_enabled: None,
            blit_color_attachment_1: Array::from_iter([
                JsValue::from_f64(WebGl2RenderingContext::NONE as f64),
                JsValue::from_f64(WebGl2RenderingContext::COLOR_ATTACHMENT1 as f64),
            ]),
            blit_color_attachment_reset: Array::from_iter([
                JsValue::from_f64(WebGl2RenderingContext::COLOR_ATTACHMENT0 as f64),
                JsValue::from_f64(WebGl2RenderingContext::COLOR_ATTACHMENT1 as f64),
            ]),
        }
    }

    fn update_universal_ubo(
        &mut self,
        state: &mut FrameState,
        scene: &mut Scene,
    ) -> Result<(), Error> {
        let data = ArrayBuffer::new(UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH);

        // u_RenderTime
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_LENGTH / 4,
        )
        .set_index(0, state.timestamp() as f32);

        // u_EnableLighting
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_LENGTH / 4,
        )
        .set_index(0, if scene.lighting_enabled() { 1.0 } else { 0.0 });

        // u_CameraPosition
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_LENGTH / 4,
        )
        .copy_from(&state.camera().position().gl_f32());

        // u_ViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_LENGTH / 4,
        )
        .copy_from(&state.camera().view_matrix().gl_f32());

        // u_ProjMatrix
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_LENGTH / 4,
        )
        .copy_from(&state.camera().proj_matrix().gl_f32());

        // u_ProjViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_LENGTH / 4,
        )
        .copy_from(&state.camera().view_proj_matrix().gl_f32());

        self.universal_ubo
            .buffer_sub_data(BufferSource::from_array_buffer(data), 0);
        Ok(())
    }

    fn update_lights_ubo(&mut self, scene: &mut Scene) -> Result<(), Error> {
        let data = ArrayBuffer::new(UBO_LIGHTS_BYTES_LENGTH);

        // u_Attenuations
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET,
            UBO_LIGHTS_ATTENUATIONS_BYTES_LENGTH / 4,
        )
        .copy_from(&scene.light_attenuations().gl_f32());

        // u_AmbientLight
        if let Some(light) = scene.ambient_light() {
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET,
                UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_DirectionalLights
        for (index, light) in scene.directional_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET
                    + index * UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_PointLights
        for (index, light) in scene.point_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET + index * UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_SpotLights
        for (index, light) in scene.spot_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET + index * UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_AreaLights
        for (index, light) in scene.area_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET + index * UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        self.lights_ubo
            .buffer_sub_data(BufferSource::from_array_buffer(data), 0);
        Ok(())
    }

    fn bloom_enabled(&mut self, resources: &Resources) -> bool {
        let enabled = self
            .bloom_key
            .as_ref()
            .and_then(|key| resources.get(key))
            .copied()
            .unwrap_or(DEFAULT_BLOOM_ENABLED);

        if Some(enabled) != self.previous_bloom_enabled {
            self.hdr_framebuffer.take();
            self.hdr_multisample_framebuffer.take();
            self.previous_bloom_enabled = Some(enabled);
        }

        enabled
    }

    fn bloom_blur_epoch(&self, resources: &Resources) -> usize {
        self.bloom_epoch_key
            .as_ref()
            .and_then(|key| resources.get(key))
            .cloned()
            .unwrap_or(DEFAULT_BLOOM_BLUR_EPOCH)
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
    fn hdr_framebuffer(&mut self, state: &FrameState, bloom: bool) -> &mut Framebuffer {
        self.hdr_framebuffer.get_or_insert_with(|| {
            log::info!("2");
            if bloom {
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
            } else {
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
            }
        })
    }

    #[inline]
    fn hdr_multisample_framebuffer(
        &mut self,
        state: &FrameState,
        samples: i32,
        bloom: bool,
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
            if bloom {
                self.hdr_multisample_framebuffer
                    .insert(state.create_framebuffer(
                        [],
                        [
                            RenderbufferProvider::new(
                                FramebufferAttachment::COLOR_ATTACHMENT0,
                                RenderbufferInternalFormat::RGBA32F,
                            ),
                            RenderbufferProvider::new(
                                FramebufferAttachment::COLOR_ATTACHMENT1,
                                RenderbufferInternalFormat::RGBA32F,
                            ),
                            RenderbufferProvider::new(
                                FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
                                RenderbufferInternalFormat::DEPTH32F_STENCIL8,
                            ),
                        ],
                        [
                            FramebufferDrawBuffer::COLOR_ATTACHMENT0,
                            FramebufferDrawBuffer::COLOR_ATTACHMENT1,
                        ],
                        Some(samples),
                    ))
            } else {
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
        bloom: bool,
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

        let program = if bloom {
            let program = state.program_store_mut().use_program_with_defines(
                material.as_program_source(),
                vec![],
                vec![Cow::Borrowed(ENABLE_BLOOM_DEFINE)],
            )?;
            state.bind_uniform_value_by_variable_name(
                program,
                BLOOM_THRESHOLD,
                UniformValue::Float3(0.2126, 0.7152, 0.0722),
            )?;
            program
        } else {
            state
                .program_store_mut()
                .use_program(material.as_program_source())?
        };

        state.bind_uniform_block_value_by_block_name(
            program,
            UNIVERSAL_UNIFORMS_BLOCK,
            UniformBlockValue::BufferBase {
                descriptor: self.universal_ubo.clone(),
                binding: UBO_UNIVERSAL_UNIFORMS_BINDING,
            },
        )?;
        state.bind_uniform_block_value_by_block_name(
            program,
            LIGHTS_BLOCK,
            UniformBlockValue::BufferBase {
                descriptor: self.lights_ubo.clone(),
                binding: UBO_LIGHTS_BINDING,
            },
        )?;

        let bound_attributes = state.bind_attributes(program, &entity, geometry, material)?;
        let bound_uniforms = state.bind_uniforms(program, &entity, geometry, material)?;
        state.draw(&geometry.draw())?;
        state.unbind_attributes(bound_attributes);
        state.unbind_uniforms(bound_uniforms);

        Ok(())
    }

    fn draw_entities(
        &self,
        state: &mut FrameState,
        bloom: bool,
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
            self.draw_entity(
                state,
                bloom,
                entity,
                geometry,
                material,
                geometry.cull_face(),
            )?;
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
            self.draw_entity(state, bloom, entity, geometry, material, None)?; // transparency entities never cull face
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
        self.draw_entities(state, false, opaques, translucents)?;
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
        self.draw_entities(state, false, opaques, translucents)?;
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
        bloom: bool,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
    ) -> Result<(), Error> {
        self.hdr_framebuffer(state, bloom)
            .bind(FramebufferTarget::FRAMEBUFFER)?;
        self.draw_entities(state, bloom, opaques, translucents)?;
        self.hdr_framebuffer(state, bloom).unbind();

        Ok(())
    }

    fn draw_hdr_multisample(
        &mut self,
        state: &mut FrameState,
        samples: i32,
        bloom: bool,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn StandardMaterial)>,
    ) -> Result<(), Error> {
        self.hdr_multisample_framebuffer(state, samples, bloom)
            .bind(FramebufferTarget::FRAMEBUFFER)?;
        self.draw_entities(state, bloom, opaques, translucents)?;
        self.hdr_multisample_framebuffer(state, samples, bloom)
            .unbind();

        Ok(())
    }

    fn blit_hdr_multisample(
        &mut self,
        state: &FrameState,
        samples: i32,
        bloom: bool,
    ) -> Result<(), Error> {
        unsafe {
            let hdr_framebuffer: *mut Framebuffer = self.hdr_framebuffer(state, bloom);
            let hdr_multisample_framebuffer: *mut Framebuffer =
                self.hdr_multisample_framebuffer(state, samples, bloom);

            (*hdr_framebuffer).bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
            (*hdr_multisample_framebuffer).bind(FramebufferTarget::READ_FRAMEBUFFER)?;
            state
                .gl()
                .read_buffer(WebGl2RenderingContext::COLOR_ATTACHMENT0);
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

            if bloom {
                state.gl().draw_buffers(&self.blit_color_attachment_1);
                state
                    .gl()
                    .read_buffer(WebGl2RenderingContext::COLOR_ATTACHMENT1);
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
                state.gl().draw_buffers(&self.blit_color_attachment_reset);
            }

            (*hdr_framebuffer).unbind();
            (*hdr_multisample_framebuffer).unbind();
            state.gl().read_buffer(WebGl2RenderingContext::BACK);
        }

        Ok(())
    }

    fn hdr_reinhard_tone_mapping(
        &mut self,
        state: &mut FrameState,
        texture: &WebGlTexture,
    ) -> Result<(), Error> {
        let normal_framebuffer = self.normal_framebuffer(state);
        let program = state
            .program_store_mut()
            .use_program(&HdrReinhardToneMapping)?;

        normal_framebuffer.bind(FramebufferTarget::FRAMEBUFFER)?;
        state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        state.bind_uniform_value_by_variable_name(
            program,
            HDR_TEXTURE,
            UniformValue::Integer1(0),
        )?;
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
        let program = state
            .program_store_mut()
            .use_program(&HdrExposureToneMapping)?;

        normal_framebuffer.bind(FramebufferTarget::FRAMEBUFFER)?;
        state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        state.bind_uniform_value_by_variable_name(
            program,
            HDR_TEXTURE,
            UniformValue::Integer1(0),
        )?;
        state.bind_uniform_value_by_variable_name(
            program,
            HDR_EXPOSURE,
            UniformValue::Float1(exposure),
        )?;
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
        bloom: bool,
        hdr_tone_mapping_type: &HdrToneMappingType,
    ) -> Result<(), Error> {
        let texture: *const WebGlTexture = match bloom {
            true => self.hdr_bloom_blend_framebuffer(state).texture(0).unwrap(),
            false => self.hdr_framebuffer(state, bloom).texture(0).unwrap(),
        };

        match hdr_tone_mapping_type {
            HdrToneMappingType::Reinhard => {
                self.hdr_reinhard_tone_mapping(state, unsafe { &*texture })?
            }
            HdrToneMappingType::Exposure(exposure) => {
                self.hdr_exposure_tone_mapping(state, unsafe { &*texture }, *exposure)?
            }
        };
        Ok(())
    }

    fn hdr_bloom_blur(&mut self, state: &mut FrameState, bloom_epoch: usize) -> Result<(), Error> {
        unsafe {
            let hdr_bloom_blur_first_framebuffer: *mut Framebuffer =
                self.hdr_framebuffer(state, true);
            let hdr_bloom_blur_even_framebuffer: *mut Framebuffer =
                self.hdr_bloom_blur_even_framebuffer(state);
            let hdr_bloom_blur_odd_framebuffer: *mut Framebuffer =
                self.hdr_bloom_blur_odd_framebuffer(state);

            let program = state
                .program_store_mut()
                .use_program(&GaussianBlurMapping)?;

            for i in 0..bloom_epoch {
                let (from, from_texture_index, to) = if i % 2 == 0 {
                    if i == 0 {
                        // first epoch, do some initialization
                        state.bind_uniform_block_value_by_block_name(
                            program,
                            GAUSSIAN_KERNEL_BLOCK,
                            UniformBlockValue::BufferBase {
                                descriptor: gaussian_kernel(),
                                binding: UBO_GAUSSIAN_BLUR_BINDING,
                            },
                        )?;
                        state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);

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

                to.bind(FramebufferTarget::FRAMEBUFFER)?;
                state.gl().bind_texture(
                    WebGl2RenderingContext::TEXTURE_2D,
                    from.texture(from_texture_index),
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

    fn hdr_bloom_blend(&mut self, state: &mut FrameState, bloom_epoch: usize) -> Result<(), Error> {
        unsafe {
            let hdr_framebuffer: *mut Framebuffer = self.hdr_framebuffer(state, true);
            let (hdr_bloom_blur_framebuffer, hdr_bloom_blur_framebuffer_texture_index): (
                *mut Framebuffer,
                usize,
            ) = if bloom_epoch == 0 {
                (self.hdr_framebuffer(state, true), 1)
            } else if bloom_epoch % 2 == 0 {
                (self.hdr_bloom_blur_even_framebuffer(state), 0)
            } else {
                (self.hdr_bloom_blur_odd_framebuffer(state), 0)
            };
            let hdr_bloom_blend_framebuffer: *mut Framebuffer =
                self.hdr_bloom_blend_framebuffer(state);
            let program = state.program_store_mut().use_program(&BloomBlendMapping)?;

            (*hdr_bloom_blend_framebuffer).bind(FramebufferTarget::FRAMEBUFFER)?;
            state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
            state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
            state.bind_uniform_value_by_variable_name(
                program,
                BASE_TEXTURE,
                UniformValue::Integer1(0),
            )?;
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

            state.bind_uniform_value_by_variable_name(
                program,
                BLOOM_BLUR_TEXTURE,
                UniformValue::Integer1(1),
            )?;
            state.gl().active_texture(WebGl2RenderingContext::TEXTURE1);
            state.gl().bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                (*hdr_bloom_blur_framebuffer).texture(hdr_bloom_blur_framebuffer_texture_index),
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
        scene: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        self.update_universal_ubo(state, scene)?;
        self.update_lights_ubo(scene)?;

        let Some((opaques, translucents)) = self.prepare_entities(state, resources)? else {
            return Ok(());
        };

        match (
            self.multisample(resources),
            self.hdr_enabled(&state, resources),
        ) {
            (None, true) => {
                let bloom = self.bloom_enabled(resources);
                let bloom_epoch = self.bloom_blur_epoch(resources);
                let hdr_tone_mapping_type = self.hdr_tone_mapping_type(resources);

                self.draw_hdr(state, bloom, opaques, translucents)?;
                if bloom {
                    self.hdr_bloom_blur(state, bloom_epoch)?;
                    self.hdr_bloom_blend(state, bloom_epoch)?;
                }
                self.hdr_tone_mapping(state, bloom, &hdr_tone_mapping_type)?;
            }
            (None, false) => {
                self.draw_normal(state, opaques, translucents)?;
            }
            (Some(samples), true) => {
                let bloom = self.bloom_enabled(resources);
                let bloom_epoch = self.bloom_blur_epoch(resources);
                let hdr_tone_mapping_type = self.hdr_tone_mapping_type(resources);

                self.draw_hdr_multisample(state, samples, bloom, opaques, translucents)?;
                self.blit_hdr_multisample(state, samples, bloom)?;
                if bloom {
                    self.hdr_bloom_blur(state, bloom_epoch)?;
                    self.hdr_bloom_blend(state, bloom_epoch)?;
                }
                self.hdr_tone_mapping(state, bloom, &hdr_tone_mapping_type)?;
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

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!(
            "./shaders/hdr_reinhard_tone_mapping.frag"
        )))
    }
}

struct HdrExposureToneMapping;

impl ProgramSource for HdrExposureToneMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("HdrExposureToneMapping")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!(
            "./shaders/hdr_exposure_tone_mapping.frag"
        )))
    }
}

struct BloomMapping;

impl ProgramSource for BloomMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("BloomMapping")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/bloom_mapping.frag")))
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

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/gaussian_blur.frag")))
    }
}

struct BloomBlendMapping;

impl ProgramSource for BloomBlendMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("BloomBlendMapping")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!("./shaders/bloom_blend.frag")))
    }
}
