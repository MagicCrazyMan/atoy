use std::borrow::Cow;

use web_sys::WebGl2RenderingContext;

use crate::{
    entity::Entity,
    material::StandardMaterial,
    render::webgl::{
        buffer::BufferDescriptor,
        conversion::ToGlEnum,
        error::Error,
        program::{FragmentShaderSource, Program, ProgramSource, VertexShaderSource},
        state::FrameState,
        uniform::{UniformBlockValue, UniformValue},
    },
};

use super::{
    collector::CollectedEntities, UBO_LIGHTS_BINDING, UBO_LIGHTS_BLOCK_NAME,
    UBO_UNIVERSAL_UNIFORMS_BINDING, UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME,
};

pub mod gbuffer;
pub mod hdr;
pub mod hdr_multisamples;
pub mod simple;
pub mod simple_multisamples;
pub mod deferred;

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
    draw_state: DrawState,
    collected_entities: &CollectedEntities,
) -> Result<(), Error> {
    state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
    state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
    state.gl().clear_depth(1.0);
    state
        .gl()
        .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

    draw_opaque_entities(state, &draw_state, collected_entities)?;
    draw_translucent_entities(state, &draw_state, collected_entities)?;

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

fn prepare_program<'a, 'b, 'c>(
    state: &'a mut FrameState,
    draw_state: &DrawState,
    material: &'b dyn StandardMaterial,
) -> Result<&'c mut Program, Error> {
    let defines: Option<&[Cow<'_, str>]> = match draw_state {
        DrawState::Draw {
            lights_ubo, bloom, ..
        } => match (lights_ubo.is_some(), bloom) {
            (true, true) => Some(&[Cow::Borrowed(LIGHTING_DEFINE), Cow::Borrowed(BLOOM_DEFINE)]),
            (true, false) => Some(&[Cow::Borrowed(LIGHTING_DEFINE)]),
            (false, true) => Some(&[Cow::Borrowed(BLOOM_DEFINE)]),
            (false, false) => None,
        },
        DrawState::GBuffer { .. } => Some(&[Cow::Borrowed(GBUFFER_DEFINE)]),
    };

    let program = match defines {
        Some(defines) => state.program_store_mut().use_program_with_defines(
            material.as_program_source(),
            &[],
            defines,
        )?,
        None => state
            .program_store_mut()
            .use_program(material.as_program_source())?,
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
                UniformBlockValue::BufferBase {
                    descriptor: (*universal_ubo).clone(),
                    binding: UBO_UNIVERSAL_UNIFORMS_BINDING,
                },
            )?;

            // binds atoy_Lights
            if let Some(lights_ubo) = lights_ubo {
                state.bind_uniform_block_value_by_block_name(
                    program,
                    UBO_LIGHTS_BLOCK_NAME,
                    UniformBlockValue::BufferBase {
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
                    UniformValue::FloatVector3(BLOOM_THRESHOLD_VALUES),
                )?;
            }
        }
        DrawState::GBuffer { universal_ubo } => {
            // binds atoy_UniversalUniforms
            state.bind_uniform_block_value_by_block_name(
                program,
                UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME,
                UniformBlockValue::BufferBase {
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
    if let Some(material) = entity.material_mut() {
        material.prepare(state);
        if !material.ready() {
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
    state.bind_uniforms(program, &entity, geometry, material)?;
    state.draw(&geometry.draw())?;
    state.unbind_attributes(bound_attributes);

    Ok(())
}

unsafe fn draw_opaque_entities(
    state: &mut FrameState,
    draw_state: &DrawState,
    collected_entities: &CollectedEntities,
) -> Result<(), Error> {
    let entities = collected_entities.entities();
    let opaque_entity_indices = collected_entities.opaque_entity_indices();

    // draws opaque enable DEPTH_TEST and disable BLEND and draws them from nearest to farthest first
    state.gl().depth_mask(true);
    for index in opaque_entity_indices {
        let entity = entities[*index].entity_mut();
        draw_entity(state, draw_state, true, entity)?;
    }

    Ok(())
}

unsafe fn draw_translucent_entities(
    state: &mut FrameState,
    draw_state: &DrawState,
    collected_entities: &CollectedEntities,
) -> Result<(), Error> {
    let entities = collected_entities.entities();
    let translucent_entity_indices = collected_entities.translucent_entity_indices();

    // draws translucents first with DEPTH_TEST unchangeable and enable BLEND and draws them from farthest to nearest
    state.gl().enable(WebGl2RenderingContext::BLEND);
    state.gl().blend_equation(WebGl2RenderingContext::FUNC_ADD);
    state.gl().blend_func(
        WebGl2RenderingContext::SRC_ALPHA,
        WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA,
    );
    state.gl().depth_mask(false);
    for index in translucent_entity_indices.iter().rev() {
        let entity = entities[*index].entity_mut();
        draw_entity(state, draw_state, false, entity)?;
        // transparency entities never cull face
    }

    Ok(())
}

pub(self) struct HdrReinhardToneMappingProgram;

impl ProgramSource for HdrReinhardToneMappingProgram {
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

impl ProgramSource for HdrExposureToneMappingProgram {
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

impl ProgramSource for BloomMapping {
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

impl ProgramSource for GaussianBlurMappingProgram {
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

impl ProgramSource for BloomBlendMappingProgram {
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
