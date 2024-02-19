use std::borrow::Cow;

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
};

use super::{
    collector::CollectedEntities, UBO_LIGHTS_BINDING, UBO_LIGHTS_BLOCK_NAME,
    UBO_UNIVERSAL_UNIFORMS_BINDING, UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME,
};

// pub mod deferred;
pub mod forward;
// pub mod picking;

const BLOOM_DEFINE: &'static str = "BLOOM";
const LIGHTING_DEFINE: &'static str = "LIGHTING";
const GBUFFER_DEFINE: &'static str = "GBUFFER";

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
    for collected_entity in collected_entities.opaque_entities() {
        draw_entity(state, draw_state, true, collected_entity.entity_mut())?;
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
    for collected_entity in collected_entities.translucent_entities().iter().rev() {
        // transparency entities never cull face
        draw_entity(state, draw_state, false, collected_entity.entity_mut())?;
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
    struct StandardMaterialShaderProvider<'a> {
        material: &'a dyn StandardMaterial,
        vertex_defines: Vec<Define<'a>>,
        fragment_defines: Vec<Define<'a>>,
    }

    impl<'a> StandardMaterialShaderProvider<'a> {
        fn new(material: &dyn StandardMaterial) -> Self {
            let mut standard_defines = Vec::new();
            if material.use_position_eye_space() {
                standard_defines.push(Define::WithoutValue(Cow::Borrowed(
                    "USE_POSITION_EYE_SPACE",
                )));
            }
            if material.use_normal() {
                standard_defines.push(Define::WithoutValue(Cow::Borrowed("USE_NORMAL")));
            }
            if material.use_texture_coordinate() {
                standard_defines.push(Define::WithoutValue(Cow::Borrowed(
                    "USE_TEXTURE_COORDINATE",
                )));
            }
            if material.use_tbn() {
                standard_defines.push(Define::WithoutValue(Cow::Borrowed("USE_TBN")));
            }
            if material.use_tbn_invert() {
                standard_defines.push(Define::WithoutValue(Cow::Borrowed("USE_TBN_INVERT")));
            }
            if material.use_calculated_bitangent() {
                standard_defines.push(Define::WithoutValue(Cow::Borrowed(
                    "USE_CALCULATED_BITANGENT",
                )));
            }

            Self {
                material,
                vertex_defines: standard_defines
                    .clone()
                    .into_iter()
                    .chain(material.vertex_defines().to_vec().into_iter())
                    .collect(),
                fragment_defines: standard_defines
                    .into_iter()
                    .chain(material.fragment_defines().to_vec().into_iter())
                    .collect(),
            }
        }
    }

    impl<'a> ShaderProvider for StandardMaterialShaderProvider<'a> {
        fn name(&self) -> Cow<'_, str> {
            const DEFINE_NAME_VALUE_SEPARATOR: &'static str = "!!";
            const DEFINE_SEPARATOR: &'static str = "##";
            const VERTEX_FRAGMENT_SEPARATOR: &'static str = "@@";

            let vertex = self
                .vertex_defines()
                .iter()
                .map(|define| match define {
                    Define::WithValue(name, value) => {
                        Cow::Owned(format!("{}{}{}", name, DEFINE_NAME_VALUE_SEPARATOR, value))
                    }
                    Define::WithoutValue(name) => Cow::Borrowed(name.as_ref()),
                })
                .collect::<Vec<_>>()
                .join(DEFINE_SEPARATOR);
            let fragment = self
                .fragment_defines()
                .iter()
                .map(|define| match define {
                    Define::WithValue(name, value) => {
                        Cow::Owned(format!("{}{}{}", name, DEFINE_NAME_VALUE_SEPARATOR, value))
                    }
                    Define::WithoutValue(name) => Cow::Borrowed(name.as_ref()),
                })
                .collect::<Vec<_>>()
                .join(DEFINE_SEPARATOR);

            if vertex.len() + fragment.len() == 0 {
                self.name()
            } else {
                Cow::Owned(format!(
                    "{}{}{}{}{}",
                    self.name().as_ref(),
                    VERTEX_FRAGMENT_SEPARATOR,
                    vertex,
                    VERTEX_FRAGMENT_SEPARATOR,
                    fragment
                ))
            }
        }

        fn vertex_source(&self) -> Cow<'_, str> {
            Cow::Borrowed(include_str!("../shaders/standard.vert"))
        }

        fn fragment_source(&self) -> Cow<'_, str> {
            Cow::Borrowed(include_str!("../shaders/standard.frag"))
        }

        fn vertex_defines(&self) -> &[Define<'_>] {
            &self.vertex_defines
        }

        fn fragment_defines(&self) -> &[Define<'_>] {
            self.fragment_defines.as_ref()
        }

        fn snippet(&self, name: &str) -> Option<Cow<'_, str>> {
            self.material.snippet(name)
        }
    }

    let defines: Option<&[Define]> = match draw_state {
        DrawState::Draw {
            lights_ubo, bloom, ..
        } => match (lights_ubo.is_some(), bloom) {
            (true, true) => Some(&[
                Define::WithoutValue(Cow::Borrowed(LIGHTING_DEFINE)),
                Define::WithoutValue(Cow::Borrowed(BLOOM_DEFINE)),
            ]),
            (true, false) => Some(&[Define::WithoutValue(Cow::Borrowed(LIGHTING_DEFINE))]),
            (false, true) => Some(&[Define::WithoutValue(Cow::Borrowed(BLOOM_DEFINE))]),
            (false, false) => None,
        },
        DrawState::GBuffer { .. } => Some(&[Define::WithoutValue(Cow::Borrowed(GBUFFER_DEFINE))]),
    };

    let program = match defines {
        Some(defines) => state.program_store_mut().use_program_with_defines(
            material.as_program_provider(),
            &[],
            defines,
        )?,
        None => state
            .program_store_mut()
            .use_program(material.as_program_provider())?,
    };

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
        DrawState::GBuffer { universal_ubo } => {
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
    entity: &mut Entity,
) -> Result<(), Error> {
    // checks material availability
    if entity.material().is_some() {
        // prepares material if not ready yet
        if !entity.material().unwrap().ready() {
            let callback = entity.material_callback();
            entity.material_mut().unwrap().prepare(state, callback);
            return Ok(());
        }
    } else {
        return Ok(());
    }
    // checks geometry availability
    if entity.geometry().is_none() {
        return Ok(());
    }

    let geometry = entity.geometry().unwrap();
    let material = entity.material().unwrap();

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

    // binds program
    let program = prepare_program(state, draw_state, material)?;

    let bound_attributes = state.bind_attributes(program, &entity, geometry, material)?;
    let bound_uniforms = state.bind_uniforms(program, &entity, geometry, material)?;
    state.draw(&geometry.draw())?;
    state.unbind_attributes(bound_attributes);
    state.unbind_uniforms(bound_uniforms)?;

    Ok(())
}

pub(self) struct HdrReinhardToneMappingProgram;

impl ShaderProvider for HdrReinhardToneMappingProgram {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("HdrReinhardToneMappingProgram")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!(
            "../shaders/hdr_reinhard_tone_mapping.frag"
        )))
    }
}

pub(self) struct HdrExposureToneMappingProgram;

impl ShaderProvider for HdrExposureToneMappingProgram {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("HdrExposureToneMappingProgram")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!(
            "../shaders/hdr_exposure_tone_mapping.frag"
        )))
    }
}

pub(self) struct BloomMapping;

impl ShaderProvider for BloomMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("BloomMapping")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/bloom_mapping.frag")))
    }
}

struct GaussianBlurMappingProgram;

impl ShaderProvider for GaussianBlurMappingProgram {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("GaussianBlurMappingProgram")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/gaussian_blur.frag")))
    }
}

struct BloomBlendMappingProgram;

impl ShaderProvider for BloomBlendMappingProgram {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("BloomBlendMappingProgram")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/bloom_blend.frag")))
    }
}
