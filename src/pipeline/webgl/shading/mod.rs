use std::{borrow::Cow, sync::OnceLock};

use web_sys::WebGl2RenderingContext;

use crate::{
    entity::Entity,
    material::webgl::StandardMaterial,
    renderer::webgl::{
        buffer::BufferDescriptor,
        conversion::ToGlEnum,
        error::Error,
        program::{Define, Program, ShaderProvider},
        state::FrameState,
        uniform::{UniformBlockValue, UniformValue},
    },
    scene::{
        AREA_LIGHTS_COUNT_DEFINE, DIRECTIONAL_LIGHTS_COUNT_DEFINE, MAX_AREA_LIGHTS_STRING,
        MAX_DIRECTIONAL_LIGHTS_STRING, MAX_POINT_LIGHTS_STRING, MAX_SPOT_LIGHTS_STRING,
        POINT_LIGHTS_COUNT_DEFINE, SPOT_LIGHTS_COUNT_DEFINE,
    },
    share::Share,
};

use super::{
    collector::CollectedEntities, UBO_LIGHTS_BINDING, UBO_LIGHTS_BLOCK_NAME,
    UBO_UNIVERSAL_UNIFORMS_BINDING, UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME,
};

pub mod deferred;
pub mod forward;
pub mod picking;

const BLOOM_THRESHOLD_UNIFORM_NAME: &'static str = "u_BloomThreshold";
const BLOOM_THRESHOLD_VALUES: [f32; 3] = [0.2126, 0.7152, 0.0722];
const BASE_TEXTURE_UNIFORM_NAME: &'static str = "u_BaseTexture";
const BLOOM_BLUR_TEXTURE_UNIFORM_NAME: &'static str = "u_BloomBlurTexture";
const HDR_TEXTURE_UNIFORM_NAME: &'static str = "u_HdrTexture";
const HDR_EXPOSURE_UNIFORM_NAME: &'static str = "u_HdrExposure";

pub(self) enum DrawState<'a> {
    Draw {
        universal_ubo: &'a BufferDescriptor,
        lights_ubo: Option<&'a BufferDescriptor>,
        bloom: bool,
    },
    GBuffer {
        universal_ubo: &'a BufferDescriptor,
        lights_ubo: Option<&'a BufferDescriptor>,
    },
}

pub(self) unsafe fn draw_entities(
    state: &mut FrameState,
    draw_state: &DrawState,

    collected_entities: &CollectedEntities,
) -> Result<(), Error> {
    draw_opaque_entities(state, &draw_state, collected_entities)?;
    draw_translucent_entities(state, &draw_state, collected_entities)?;
    Ok(())
}

unsafe fn draw_opaque_entities(
    state: &mut FrameState,
    draw_state: &DrawState,

    collected_entities: &CollectedEntities,
) -> Result<(), Error> {
    state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
    state.gl().depth_mask(true);

    // draws opaque enable DEPTH_TEST and disable BLEND and draws them from nearest to farthest first
    for entity in collected_entities.opaque_entities() {
        let Some(entity) = entity.upgrade() else {
            continue;
        };
        draw_entity(state, draw_state, true, entity)?;
    }

    state.gl().disable(WebGl2RenderingContext::CULL_FACE);
    state.gl().cull_face(WebGl2RenderingContext::BACK);
    state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);

    Ok(())
}

unsafe fn draw_translucent_entities(
    state: &mut FrameState,
    draw_state: &DrawState,

    collected_entities: &CollectedEntities,
) -> Result<(), Error> {
    state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
    state.gl().depth_mask(false);
    state.gl().enable(WebGl2RenderingContext::BLEND);
    state.gl().blend_equation(WebGl2RenderingContext::FUNC_ADD);
    state.gl().blend_func(
        WebGl2RenderingContext::ONE,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );

    // draws translucents first with DEPTH_TEST unchangeable and enable BLEND and draws them from farthest to nearest
    for entity in collected_entities.translucent_entities().iter().rev() {
        // transparency entities never cull face
        let Some(entity) = entity.upgrade() else {
            continue;
        };
        draw_entity(state, draw_state, false, entity)?;
    }

    state.gl().depth_mask(true);
    state.gl().disable(WebGl2RenderingContext::DEPTH_TEST);
    state.gl().disable(WebGl2RenderingContext::CULL_FACE);
    state.gl().cull_face(WebGl2RenderingContext::BACK);
    state.gl().disable(WebGl2RenderingContext::BLEND);
    state
        .gl()
        .blend_func(WebGl2RenderingContext::ONE, WebGl2RenderingContext::ZERO);

    Ok(())
}

fn prepare_program<'a, 'b, 'c>(
    state: &'a mut FrameState,
    draw_state: &DrawState,
    material: &'b dyn StandardMaterial,
) -> Result<&'c mut Program, Error> {
    let provider = StandardMaterialShaderProvider::new(material, draw_state);
    let program = state.program_store_mut().use_program(&provider)?;

    match draw_state {
        DrawState::Draw {
            universal_ubo,
            lights_ubo,
            bloom,
        } => {
            // binds atoy_UniversalUniforms
            state.bind_uniform_block_value_by_block_name(
                program,
                UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME,
                &UniformBlockValue::BufferBase {
                    descriptor: (*universal_ubo).clone(),
                    binding: UBO_UNIVERSAL_UNIFORMS_BINDING,
                },
            )?;

            // binds atoy_Lights
            if let Some(lights_ubo) = lights_ubo {
                state.bind_uniform_block_value_by_block_name(
                    program,
                    UBO_LIGHTS_BLOCK_NAME,
                    &UniformBlockValue::BufferBase {
                        descriptor: (*lights_ubo).clone(),
                        binding: UBO_LIGHTS_BINDING,
                    },
                )?;
            }

            // binds bloom blur threshold
            if *bloom {
                state.bind_uniform_value_by_variable_name(
                    program,
                    BLOOM_THRESHOLD_UNIFORM_NAME,
                    &UniformValue::FloatVector3(BLOOM_THRESHOLD_VALUES),
                )?;
            }
        }
        DrawState::GBuffer { universal_ubo, .. } => {
            // binds atoy_UniversalUniforms
            state.bind_uniform_block_value_by_block_name(
                program,
                UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME,
                &UniformBlockValue::BufferBase {
                    descriptor: (*universal_ubo).clone(),
                    binding: UBO_UNIVERSAL_UNIFORMS_BINDING,
                },
            )?;
        }
    }

    Ok(program)
}

fn draw_entity(
    state: &mut FrameState,
    draw_state: &DrawState,

    should_cull_face: bool,
    entity: Share<dyn Entity>,
) -> Result<(), Error> {
    let entity = entity.borrow();
    let geometry = entity.geometry().unwrap();

    // culls face
    if should_cull_face {
        if let Some(cull_face) = geometry.cull_face() {
            state.gl().enable(WebGl2RenderingContext::CULL_FACE);
            state.gl().cull_face(cull_face.gl_enum());
        } else {
            state.gl().disable(WebGl2RenderingContext::CULL_FACE);
        }
    } else {
        state.gl().disable(WebGl2RenderingContext::CULL_FACE);
    }

    let program = prepare_program(state, draw_state, entity.material().unwrap())?;
    let bound_attributes = state.bind_attributes(program, &entity)?;
    let bound_uniforms = state.bind_uniforms(program, &entity)?;
    state.draw(&geometry.draw())?;
    state.unbind_attributes(bound_attributes);
    state.unbind_uniforms(bound_uniforms)?;

    Ok(())
}

struct StandardMaterialShaderProvider<'a> {
    material: &'a dyn StandardMaterial,
    draw_state: &'a DrawState<'a>,
}

impl<'a> StandardMaterialShaderProvider<'a> {
    fn new(material: &'a dyn StandardMaterial, draw_state: &'a DrawState) -> Self {
        Self {
            material,
            draw_state,
        }
    }
}

impl<'a> ShaderProvider for StandardMaterialShaderProvider<'a> {
    fn name(&self) -> Cow<'_, str> {
        const DEFINE_NAME_VALUE_SEPARATOR: &'static str = "!!";
        const DEFINE_SEPARATOR: &'static str = "##";
        const DEFINES_SEPARATOR: &'static str = "@@";

        trait JoinDefines {
            fn join_defines(&self) -> String;
        }

        impl<'a> JoinDefines for &'a [Define<'a>] {
            fn join_defines(&self) -> String {
                self.iter()
                    .map(|define| match define {
                        Define::WithValue(name, value) => {
                            Cow::Owned(format!("{}{}{}", name, DEFINE_NAME_VALUE_SEPARATOR, value))
                        }
                        Define::WithoutValue(name) => Cow::Borrowed(name.as_ref()),
                    })
                    .collect::<Vec<_>>()
                    .join(DEFINE_SEPARATOR)
            }
        }

        let type_name = match self.draw_state {
            DrawState::Draw { .. } => "draw",
            DrawState::GBuffer { .. } => "gbuffer",
        };
        let defines = self.universal_defines().join_defines();
        let vertex_defines = self.vertex_defines().join_defines();
        let fragment_defines = self.fragment_defines().join_defines();

        if defines.len() + vertex_defines.len() + fragment_defines.len() == 0 {
            self.material.name()
        } else {
            Cow::Owned(format!(
                "{}{}{}{}{}{}{}{}{}",
                self.material.name().as_ref(),
                DEFINES_SEPARATOR,
                type_name,
                DEFINES_SEPARATOR,
                defines,
                DEFINES_SEPARATOR,
                vertex_defines,
                DEFINES_SEPARATOR,
                fragment_defines
            ))
        }
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/standard.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        match self.draw_state {
            DrawState::Draw { .. } => Cow::Borrowed(include_str!("../shaders/draw.frag")),
            DrawState::GBuffer { .. } => Cow::Borrowed(include_str!("../shaders/gbuffer.frag")),
        }
    }

    fn universal_defines(&self) -> &[Define<'_>] {
        let defines = unsafe {
            static mut DEFINES: OnceLock<[Define<'static>; 12]> = OnceLock::new();
            DEFINES.get_or_init(|| {
                [
                    Define::WithoutValue(Cow::Borrowed("")),
                    Define::WithoutValue(Cow::Borrowed("")),
                    Define::WithoutValue(Cow::Borrowed("")),
                    Define::WithoutValue(Cow::Borrowed("")),
                    Define::WithoutValue(Cow::Borrowed("")),
                    Define::WithoutValue(Cow::Borrowed("")),
                    Define::WithoutValue(Cow::Borrowed("")),
                    Define::WithoutValue(Cow::Borrowed("")),
                    Define::WithoutValue(Cow::Borrowed("")),
                    Define::WithoutValue(Cow::Borrowed("")),
                    Define::WithoutValue(Cow::Borrowed("")),
                    Define::WithoutValue(Cow::Borrowed("")),
                ]
            });
            DEFINES.get_mut().unwrap()
        };

        let mut count = 0;
        if self.material.use_position_eye_space() {
            defines[count] = Define::WithoutValue(Cow::Borrowed("USE_POSITION_EYE_SPACE"));
            count += 1;
        }
        if self.material.use_normal() {
            defines[count] = Define::WithoutValue(Cow::Borrowed("USE_NORMAL"));
            count += 1;
        }
        if self.material.use_texture_coordinate() {
            defines[count] = Define::WithoutValue(Cow::Borrowed("USE_TEXTURE_COORDINATE"));
            count += 1;
        }
        if self.material.use_tbn() {
            defines[count] = Define::WithoutValue(Cow::Borrowed("USE_TBN"));
            count += 1;
        }
        if self.material.use_tbn_invert() {
            defines[count] = Define::WithoutValue(Cow::Borrowed("USE_TBN_INVERT"));
            count += 1;
        }
        if self.material.use_calculated_bitangent() {
            defines[count] = Define::WithoutValue(Cow::Borrowed("USE_CALCULATED_BITANGENT"));
            count += 1;
        }
        match self.draw_state {
            DrawState::Draw { lights_ubo, .. } | DrawState::GBuffer { lights_ubo, .. } => {
                if lights_ubo.is_some() {
                    defines[count] = Define::WithoutValue(Cow::Borrowed("USE_LIGHTING"));
                    count += 1;
                    defines[count] = Define::WithValue(
                        Cow::Borrowed(DIRECTIONAL_LIGHTS_COUNT_DEFINE),
                        Cow::Borrowed(MAX_DIRECTIONAL_LIGHTS_STRING),
                    );
                    count += 1;
                    defines[count] = Define::WithValue(
                        Cow::Borrowed(POINT_LIGHTS_COUNT_DEFINE),
                        Cow::Borrowed(MAX_POINT_LIGHTS_STRING),
                    );
                    count += 1;
                    defines[count] = Define::WithValue(
                        Cow::Borrowed(SPOT_LIGHTS_COUNT_DEFINE),
                        Cow::Borrowed(MAX_SPOT_LIGHTS_STRING),
                    );
                    count += 1;
                    defines[count] = Define::WithValue(
                        Cow::Borrowed(AREA_LIGHTS_COUNT_DEFINE),
                        Cow::Borrowed(MAX_AREA_LIGHTS_STRING),
                    );
                    count += 1;
                    // enable normal automatically if lighting enabled
                    if defines[0..count]
                        .iter()
                        .all(|define| define.name() != "USE_NORMAL")
                    {
                        defines[count] = Define::WithoutValue(Cow::Borrowed("USE_NORMAL"));
                        count += 1;
                    }
                }
            }
        }
        match self.draw_state {
            DrawState::Draw { bloom, .. } => {
                if *bloom {
                    defines[count] = Define::WithoutValue(Cow::Borrowed("USE_BLOOM"));
                    count += 1;
                }
            }
            _ => {}
        };

        &defines[..count]
    }

    fn vertex_defines(&self) -> &[Define<'_>] {
        &self.material.vertex_defines()
    }

    fn fragment_defines(&self) -> &[Define<'_>] {
        &self.material.fragment_defines()
    }

    fn snippet(&self, name: &str) -> Option<Cow<'_, str>> {
        match name {
            "FragmentProcess" => Some(self.material.fragment_process()),
            _ => self.material.snippet(name),
        }
    }
}

pub(self) struct HdrReinhardToneMapping;

impl ShaderProvider for HdrReinhardToneMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("HdrReinhardToneMapping")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/hdr_reinhard_tone_mapping.frag"))
    }

    fn universal_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn vertex_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn fragment_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}

pub(self) struct HdrExposureToneMapping;

impl ShaderProvider for HdrExposureToneMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("HdrExposureToneMapping")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/hdr_exposure_tone_mapping.frag"))
    }

    fn universal_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn vertex_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn fragment_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}

pub(self) struct BloomMapping;

impl ShaderProvider for BloomMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("BloomMapping")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/bloom_mapping.frag"))
    }

    fn universal_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn vertex_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn fragment_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}

struct GaussianBlurMapping;

impl ShaderProvider for GaussianBlurMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("GaussianBlurMapping")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/gaussian_blur.frag"))
    }

    fn universal_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn vertex_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn fragment_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}

struct BloomBlendMapping;

impl ShaderProvider for BloomBlendMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("BloomBlendMapping")
    }

    fn vertex_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/computation.vert"))
    }

    fn fragment_source(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("../shaders/bloom_blend.frag"))
    }

    fn universal_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn vertex_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn fragment_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }
}
