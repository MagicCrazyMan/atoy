use std::{any::Any, borrow::Cow, ptr::NonNull};

use serde::{Deserialize, Serialize};
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::{
    entity::Entity,
    geometry::Geometry,
    material::{Material, Transparency},
    render::{
        pp::{Executor, ResourceKey, Resources, State},
        webgl::{
            attribute::{bind_attributes, unbind_attributes, AttributeBinding},
            conversion::ToGlEnum,
            draw::draw,
            error::Error,
            framebuffer::{
                Framebuffer, FramebufferAttachment, FramebufferTarget, RenderbufferProvider,
                TextureProvider,
            },
            program::{ProgramSource, ShaderSource},
            renderbuffer::RenderbufferInternalFormat,
            texture::{TextureDataType, TextureFormat, TextureInternalFormat},
            uniform::{
                bind_uniforms, unbind_uniforms, UniformBinding, UniformBlockBinding,
                UniformStructuralBinding,
            },
        },
    },
    scene::Scene,
};

const SAMPLER_UNIFORM: UniformBinding = UniformBinding::FromMaterial("u_Sampler");
const EXPOSURE_UNIFORM: UniformBinding = UniformBinding::FromMaterial("u_Exposure");

pub const DEFAULT_MULTISAMPLE: i32 = 4;
pub const DEFAULT_HDR_ENABLED: bool = true;
pub const DEFAULT_HDR_TONE_MAPPING_TYPE: HdrToneMappingType = HdrToneMappingType::Reinhard;

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
    normal_framebuffer: Framebuffer,
    hdr_framebuffer: Framebuffer,
    normal_multisample_framebuffer: Option<Framebuffer>,
    hdr_multisample_framebuffer: Option<Framebuffer>,

    hdr_supported: Option<bool>,

    entities_key: ResourceKey<Vec<NonNull<Entity>>>,
    texture_key: ResourceKey<WebGlTexture>,
    multisample_key: Option<ResourceKey<i32>>,
    hdr_key: Option<ResourceKey<bool>>,
    hdr_tone_mapping_type_key: Option<ResourceKey<HdrToneMappingType>>,
}

impl StandardDrawer {
    pub fn new(
        entities_key: ResourceKey<Vec<NonNull<Entity>>>,
        texture_key: ResourceKey<WebGlTexture>,
        multisample_key: Option<ResourceKey<i32>>,
        hdr_key: Option<ResourceKey<bool>>,
        hdr_tone_mapping_type_key: Option<ResourceKey<HdrToneMappingType>>,
    ) -> Self {
        Self {
            entities_key,
            normal_framebuffer: Framebuffer::new(
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
            ),
            hdr_framebuffer: Framebuffer::new(
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
            ),

            hdr_supported: None,

            normal_multisample_framebuffer: None,
            hdr_multisample_framebuffer: None,
            texture_key,
            multisample_key,
            hdr_key,
            hdr_tone_mapping_type_key,
        }
    }

    fn hdr_supported(&mut self, state: &State) -> bool {
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

    fn hdr_enabled(&mut self, state: &State, resources: &Resources) -> bool {
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

    fn normal_multisample_framebuffer(&mut self, samples: i32) -> &mut Framebuffer {
        if self
            .normal_multisample_framebuffer
            .as_ref()
            .and_then(|framebuffer| framebuffer.renderbuffer_samples())
            .map(|s| s == samples)
            .unwrap_or(false)
        {
            self.normal_multisample_framebuffer.as_mut().unwrap()
        } else {
            self.normal_multisample_framebuffer = Some(Framebuffer::new(
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
            ));
            self.normal_multisample_framebuffer.as_mut().unwrap()
        }
    }

    fn hdr_multisample_framebuffer(&mut self, samples: i32) -> &mut Framebuffer {
        if self
            .hdr_multisample_framebuffer
            .as_ref()
            .and_then(|framebuffer| framebuffer.renderbuffer_samples())
            .map(|s| s == samples)
            .unwrap_or(false)
        {
            self.hdr_multisample_framebuffer.as_mut().unwrap()
        } else {
            self.hdr_multisample_framebuffer = Some(Framebuffer::new(
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
            ));
            self.hdr_multisample_framebuffer.as_mut().unwrap()
        }
    }

    fn prepare_entities<'a, 'b>(
        &'a self,
        state: &mut State,
        resources: &mut Resources,
    ) -> Result<
        Option<(
            Vec<(&'b Entity, &'b dyn Geometry, &'b dyn Material)>,
            Vec<(&'b Entity, &'b dyn Geometry, &'b dyn Material)>,
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
        state: &mut State,
        entity: &Entity,
        geometry: &dyn Geometry,
        material: &dyn Material,
    ) -> Result<(), Error> {
        let program_item = state.program_store_mut().use_program(material)?;

        if let Some(cull_face) = geometry.cull_face() {
            state.gl().enable(WebGl2RenderingContext::CULL_FACE);
            state.gl().cull_face(cull_face.gl_enum());
        }

        let bound_attributes = bind_attributes(state, &entity, geometry, material, &program_item);
        let bound_uniforms = bind_uniforms(state, &entity, geometry, material, &program_item);
        draw(state, geometry, material);
        unbind_attributes(state, bound_attributes);
        unbind_uniforms(state, bound_uniforms);

        state.gl().disable(WebGl2RenderingContext::CULL_FACE);
        state.gl().cull_face(WebGl2RenderingContext::BACK);

        Ok(())
    }

    fn draw_entities(
        &self,
        state: &mut State,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn Material)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn Material)>,
    ) -> Result<(), Error> {
        // draws opaque enable DEPTH_TEST and disable BLEND and draws them from nearest to farthest first
        state.gl().depth_mask(true);
        for (entity, geometry, material) in opaques {
            self.draw_entity(state, entity, geometry, material)?;
        }

        // then draws translucents first with DEPTH_TEST unchangeable and enable BLEND and draws theme from farthest to nearest
        state.gl().enable(WebGl2RenderingContext::BLEND);
        state.gl().blend_equation(WebGl2RenderingContext::FUNC_ADD);
        state.gl().blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
        );
        state.gl().depth_mask(false);
        for (entity, geometry, material) in translucents.into_iter().rev() {
            self.draw_entity(state, entity, geometry, material)?;
        }

        state.gl().depth_mask(true);
        state.gl().disable(WebGl2RenderingContext::BLEND);

        Ok(())
    }

    fn draw_normal(
        &mut self,
        state: &mut State,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn Material)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn Material)>,
    ) -> Result<(), Error> {
        self.normal_framebuffer
            .bind(state.gl(), FramebufferTarget::FRAMEBUFFER)?;
        state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl().clear_depth(1.0);
        state.gl().clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );
        self.draw_entities(state, opaques, translucents)?;
        self.normal_framebuffer.unbind(state.gl());

        Ok(())
    }

    fn draw_normal_multisample(
        &mut self,
        state: &mut State,
        samples: i32,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn Material)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn Material)>,
    ) -> Result<(), Error> {
        let normal_multisample_framebuffer = self.normal_multisample_framebuffer(samples);
        normal_multisample_framebuffer.bind(state.gl(), FramebufferTarget::FRAMEBUFFER)?;
        state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl().clear_depth(1.0);
        state.gl().clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );
        self.draw_entities(state, opaques, translucents)?;
        self.normal_multisample_framebuffer
            .as_mut()
            .unwrap()
            .unbind(state.gl());

        Ok(())
    }

    fn blit_normal_multisample(&mut self, state: &mut State) -> Result<(), Error> {
        let normal_framebuffer = &mut self.normal_framebuffer;
        let normal_multisample_framebuffer = self.normal_multisample_framebuffer.as_mut().unwrap();

        normal_framebuffer.bind(state.gl(), FramebufferTarget::DRAW_FRAMEBUFFER)?;
        normal_multisample_framebuffer.bind(state.gl(), FramebufferTarget::READ_FRAMEBUFFER)?;
        state.gl().blit_framebuffer(
            0,
            0,
            normal_multisample_framebuffer.width(),
            normal_multisample_framebuffer.height(),
            0,
            0,
            normal_framebuffer.width(),
            normal_framebuffer.height(),
            WebGl2RenderingContext::COLOR_BUFFER_BIT,
            WebGl2RenderingContext::LINEAR,
        );
        normal_framebuffer.unbind(state.gl());
        normal_multisample_framebuffer.unbind(state.gl());

        Ok(())
    }

    fn draw_hdr(
        &mut self,
        state: &mut State,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn Material)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn Material)>,
    ) -> Result<(), Error> {
        self.hdr_framebuffer
            .bind(state.gl(), FramebufferTarget::FRAMEBUFFER)?;
        state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl().clear_depth(1.0);
        state.gl().clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );
        self.draw_entities(state, opaques, translucents)?;
        self.hdr_framebuffer.unbind(state.gl());
        state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);

        Ok(())
    }

    fn draw_hdr_multisample(
        &mut self,
        state: &mut State,
        samples: i32,
        opaques: Vec<(&Entity, &dyn Geometry, &dyn Material)>,
        translucents: Vec<(&Entity, &dyn Geometry, &dyn Material)>,
    ) -> Result<(), Error> {
        let hdr_multisample_framebuffer = self.hdr_multisample_framebuffer(samples);
        hdr_multisample_framebuffer.bind(state.gl(), FramebufferTarget::FRAMEBUFFER)?;
        state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl().clear_depth(1.0);
        state.gl().clear(
            WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
        );
        self.draw_entities(state, opaques, translucents)?;
        self.hdr_multisample_framebuffer
            .as_mut()
            .unwrap()
            .unbind(state.gl());
        state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);

        Ok(())
    }

    fn blit_hdr_multisample(&mut self, state: &mut State) -> Result<(), Error> {
        let hdr_framebuffer = &mut self.hdr_framebuffer;
        let hdr_multisample_framebuffer = self.hdr_multisample_framebuffer.as_mut().unwrap();

        hdr_framebuffer.bind(state.gl(), FramebufferTarget::DRAW_FRAMEBUFFER)?;
        hdr_multisample_framebuffer.bind(state.gl(), FramebufferTarget::READ_FRAMEBUFFER)?;
        state.gl().blit_framebuffer(
            0,
            0,
            hdr_multisample_framebuffer.width(),
            hdr_multisample_framebuffer.height(),
            0,
            0,
            hdr_framebuffer.width(),
            hdr_framebuffer.height(),
            WebGl2RenderingContext::COLOR_BUFFER_BIT,
            WebGl2RenderingContext::LINEAR,
        );
        hdr_framebuffer.unbind(state.gl());
        hdr_multisample_framebuffer.unbind(state.gl());

        Ok(())
    }

    fn hdr_reinhard_tone_mapping(&mut self, state: &mut State) -> Result<(), Error> {
        let tone_mapping_program_item = state
            .program_store_mut()
            .use_program(&HdrReinhardToneMapping)?;

        self.normal_framebuffer
            .bind(state.gl(), FramebufferTarget::FRAMEBUFFER)?;
        state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        state.gl().uniform1i(
            tone_mapping_program_item
                .uniform_locations()
                .get(&SAMPLER_UNIFORM),
            0,
        );
        state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);
        state.gl().bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            self.hdr_framebuffer
                .textures()
                .and_then(|textures| textures.get(0))
                .map(|(texture, _)| texture),
        );
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
        self.normal_framebuffer.unbind(state.gl());

        Ok(())
    }

    fn hdr_exposure_tone_mapping(&mut self, state: &mut State, exposure: f32) -> Result<(), Error> {
        let tone_mapping_program_item = state
            .program_store_mut()
            .use_program(&HdrExposureToneMapping)?;

        self.normal_framebuffer
            .bind(state.gl(), FramebufferTarget::FRAMEBUFFER)?;
        state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);
        state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
        state.gl().clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
        state.gl().uniform1i(
            tone_mapping_program_item
                .uniform_locations()
                .get(&SAMPLER_UNIFORM),
            0,
        );
        state.gl().uniform1f(
            tone_mapping_program_item
                .uniform_locations()
                .get(&EXPOSURE_UNIFORM),
            exposure,
        );
        state.gl().active_texture(WebGl2RenderingContext::TEXTURE0);
        state.gl().bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            self.hdr_framebuffer
                .textures()
                .and_then(|textures| textures.get(0))
                .map(|(texture, _)| texture),
        );
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
        self.normal_framebuffer.unbind(state.gl());

        Ok(())
    }

    fn hdr_tone_mapping(&mut self, state: &mut State, resources: &Resources) -> Result<(), Error> {
        match self.hdr_tone_mapping_type(resources) {
            HdrToneMappingType::Reinhard => self.hdr_reinhard_tone_mapping(state)?,
            HdrToneMappingType::Exposure(exposure) => {
                self.hdr_exposure_tone_mapping(state, exposure)?
            }
        };
        Ok(())
    }
}

impl Executor for StandardDrawer {
    type Error = Error;

    fn before(
        &mut self,
        _: &mut State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<bool, Self::Error> {
        Ok(resources.contains_resource(&self.entities_key))
    }

    fn after(
        &mut self,
        state: &mut State,
        _: &mut Scene,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        let texture = self
            .normal_framebuffer
            .textures()
            .and_then(|textures| textures.get(0))
            .map(|(texture, _)| texture.clone())
            .unwrap();
        resources.insert(self.texture_key.clone(), texture);

        Ok(())
    }

    fn execute(
        &mut self,
        state: &mut State,
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
                self.hdr_tone_mapping(state, &resources)?;
            }
            (None, false) => {
                self.draw_normal(state, opaques, translucents)?;
            }
            (Some(samples), true) => {
                self.draw_hdr_multisample(state, samples, opaques, translucents)?;
                self.blit_hdr_multisample(state)?;
                self.hdr_tone_mapping(state, &resources)?;
            }
            (Some(samples), false) => {
                self.draw_normal_multisample(state, samples, opaques, translucents)?;
                self.blit_normal_multisample(state)?;
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
