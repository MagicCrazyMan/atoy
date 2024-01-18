use std::borrow::Cow;

use serde::{Deserialize, Serialize};
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

pub mod hdr;
pub mod hdr_multisamples;
pub mod simple;
pub mod simple_multisamples;

const BLOOM_GLSL_DEFINE: &'static str = "BLOOM";
const LIGHTING_GLSL_DEFINE: &'static str = "LIGHTING";

const BLOOM_THRESHOLD_UNIFORM_NAME: &'static str = "u_BloomThreshold";
const BLOOM_THRESHOLD_VALUES: [f32; 3] = [0.2126, 0.7152, 0.0722];
const BASE_TEXTURE_UNIFORM_NAME: &'static str = "u_BaseTexture";
const BLOOM_BLUR_TEXTURE_UNIFORM_NAME: &'static str = "u_BloomBlurTexture";
const HDR_TEXTURE_UNIFORM_NAME: &'static str = "u_HdrTexture";
const HDR_EXPOSURE_UNIFORM_NAME: &'static str = "u_HdrExposure";

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum HdrToneMappingType {
    Reinhard,
    Exposure(f32),
}

pub(self) fn draw_entities(
    state: &mut FrameState,
    lighting: bool,
    bloom_blur: bool,
    collected_entities: &CollectedEntities,
    universal_ubo: &BufferDescriptor,
    lights_ubo: &BufferDescriptor,
) -> Result<(), Error> {
    state.gl().enable(WebGl2RenderingContext::DEPTH_TEST);
    state.gl().clear_color(0.0, 0.0, 0.0, 0.0);
    state.gl().clear_depth(1.0);
    state
        .gl()
        .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT);

    draw_opaque_entities(
        state,
        lighting,
        bloom_blur,
        collected_entities,
        universal_ubo,
        lights_ubo,
    )?;
    draw_translucent_entities(
        state,
        lighting,
        bloom_blur,
        collected_entities,
        universal_ubo,
        lights_ubo,
    )?;

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
    lighting: bool,
    bloom_blur: bool,
    material: &'b dyn StandardMaterial,
    universal_ubo: &BufferDescriptor,
    lights_ubo: &BufferDescriptor,
) -> Result<&'c mut Program, Error> {
    let defines = match (lighting, bloom_blur) {
        (true, true) => Some(vec![
            Cow::Borrowed(LIGHTING_GLSL_DEFINE),
            Cow::Borrowed(BLOOM_GLSL_DEFINE),
        ]),
        (true, false) => Some(vec![Cow::Borrowed(LIGHTING_GLSL_DEFINE)]),
        (false, true) => Some(vec![Cow::Borrowed(BLOOM_GLSL_DEFINE)]),
        (false, false) => None,
    };

    let program = match defines {
        Some(defines) => state.program_store_mut().use_program_with_defines(
            material.as_program_source(),
            vec![],
            defines,
        )?,
        None => state
            .program_store_mut()
            .use_program(material.as_program_source())?,
    };

    // binds atoy_UniversalUniforms
    state.bind_uniform_block_value_by_block_name(
        program,
        UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME,
        UniformBlockValue::BufferBase {
            descriptor: universal_ubo.clone(),
            binding: UBO_UNIVERSAL_UNIFORMS_BINDING,
        },
    )?;

    // binds atoy_Lights
    if lighting {
        state.bind_uniform_block_value_by_block_name(
            program,
            UBO_LIGHTS_BLOCK_NAME,
            UniformBlockValue::BufferBase {
                descriptor: lights_ubo.clone(),
                binding: UBO_LIGHTS_BINDING,
            },
        )?;
    }

    // binds bloom blur threshold
    if bloom_blur {
        state.bind_uniform_value_by_variable_name(
            program,
            BLOOM_THRESHOLD_UNIFORM_NAME,
            UniformValue::FloatVector3(BLOOM_THRESHOLD_VALUES),
        )?;
    }

    Ok(program)
}

fn draw_entity(
    state: &mut FrameState,
    lighting: bool,
    bloom_blur: bool,
    should_cull_face: bool,
    entity: &mut Entity,
    universal_ubo: &BufferDescriptor,
    lights_ubo: &BufferDescriptor,
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
    let program = prepare_program(
        state,
        lighting,
        bloom_blur,
        material,
        universal_ubo,
        lights_ubo,
    )?;

    let bound_attributes = state.bind_attributes(program, &entity, geometry, material)?;
    state.bind_uniforms(program, &entity, geometry, material)?;
    state.draw(&geometry.draw())?;
    state.unbind_attributes(bound_attributes);

    Ok(())
}

fn draw_opaque_entities(
    state: &mut FrameState,
    lighting: bool,
    bloom_blur: bool,
    collected_entities: &CollectedEntities,
    universal_ubo: &BufferDescriptor,
    lights_ubo: &BufferDescriptor,
) -> Result<(), Error> {
    let entities = collected_entities.entities();
    let opaque_entity_indices = collected_entities.opaque_entity_indices();

    // draws opaque enable DEPTH_TEST and disable BLEND and draws them from nearest to farthest first
    state.gl().depth_mask(true);
    for index in opaque_entity_indices {
        let entity = entities[*index].entity_mut();
        draw_entity(
            state,
            lighting,
            bloom_blur,
            true,
            entity,
            universal_ubo,
            lights_ubo,
        )?;
    }

    Ok(())
}

fn draw_translucent_entities(
    state: &mut FrameState,
    lighting: bool,
    bloom_blur: bool,
    collected_entities: &CollectedEntities,
    universal_ubo: &BufferDescriptor,
    lights_ubo: &BufferDescriptor,
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
        draw_entity(
            state,
            lighting,
            bloom_blur,
            false,
            entity,
            universal_ubo,
            lights_ubo,
        )?;
        // transparency entities never cull face
    }

    Ok(())
}

pub(self) struct HdrReinhardToneMapping;

impl ProgramSource for HdrReinhardToneMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("HdrReinhardToneMapping")
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

pub(self) struct HdrExposureToneMapping;

impl ProgramSource for HdrExposureToneMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("HdrExposureToneMapping")
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

struct GaussianBlurMapping;

impl ProgramSource for GaussianBlurMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("GaussianBlurMapping")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/gaussian_blur.frag")))
    }
}

struct BloomBlendMapping;

impl ProgramSource for BloomBlendMapping {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("BloomBlendMapping")
    }

    fn vertex_source(&self) -> VertexShaderSource {
        VertexShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/computation.vert")))
    }

    fn fragment_source(&self) -> FragmentShaderSource {
        FragmentShaderSource::Raw(Cow::Borrowed(include_str!("../shaders/bloom_blend.frag")))
    }
}
