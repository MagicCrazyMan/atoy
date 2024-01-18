use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use web_sys::WebGl2RenderingContext;

use crate::{
    entity::Entity,
    light::{
        area_light::MAX_AREA_LIGHTS, directional_light::MAX_DIRECTIONAL_LIGHTS,
        point_light::MAX_POINT_LIGHTS, spot_light::MAX_SPOT_LIGHTS,
    },
    render::webgl::{
        buffer::BufferDescriptor,
        conversion::ToGlEnum,
        error::Error,
        program::{FragmentShaderSource, ProgramSource, VertexShaderSource},
        state::FrameState,
        uniform::{UniformBlockValue, UniformValue},
    },
};

use super::collector::CollectedEntities;

pub mod hdr;
pub mod hdr_multisamples;
pub mod simple;
pub mod simple_multisamples;

/// Uniform Buffer Object `atoy_UniversalUniforms`.
pub const UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME: &'static str = "atoy_UniversalUniforms";
/// Uniform Buffer Object `atoy_Lights`.
pub const UBO_LIGHTS_BLOCK_NAME: &'static str = "atoy_Lights";
/// Uniform Buffer Object `atoy_GaussianKernel`.
pub const UBO_GAUSSIAN_KERNEL_BLOCK_NAME: &'static str = "atoy_GaussianKernel";

/// Uniform Buffer Object mount point for `atoy_UniversalUniformsVert` and `atoy_UniversalUniformsFrag`.
pub const UBO_UNIVERSAL_UNIFORMS_BINDING: u32 = 0;
/// Uniform Buffer Object mount point for `atoy_Lights`.
pub const UBO_LIGHTS_BINDING: u32 = 1;
/// Uniform Buffer Object mount point for gaussian blur.
pub const UBO_GAUSSIAN_BLUR_BINDING: u32 = 2;

/// Uniform Buffer Object bytes length for `atoy_UniversalUniformsVert` and `atoy_UniversalUniformsFrag`.
pub const UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH: u32 = 16 + 16 + 64 + 64 + 64;
/// Uniform Buffer Object bytes length for `u_RenderTime`.
pub const UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_LENGTH: u32 = 4;
/// Uniform Buffer Object bytes length for `u_EnableLighting`.
pub const UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_LENGTH: u32 = 4;
/// Uniform Buffer Object bytes length for `u_CameraPosition`.
pub const UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_LENGTH: u32 = 12;
/// Uniform Buffer Object bytes length for `u_ViewMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_ProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_ViewProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_LENGTH: u32 = 64;

/// Uniform Buffer Object bytes offset for `u_RenderTime`.
pub const UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET: u32 = 0;
/// Uniform Buffer Object bytes offset for `u_EnableLighting`.
pub const UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_OFFSET: u32 = 4;
/// Uniform Buffer Object bytes offset for `u_CameraPosition`.
pub const UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET: u32 = 16;
/// Uniform Buffer Object bytes offset for `u_ViewMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET: u32 = 32;
/// Uniform Buffer Object bytes offset for `u_ProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET: u32 = 96;
/// Uniform Buffer Object bytes offset for `u_ViewProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET: u32 = 160;

/// Uniform Buffer Object bytes length for `atoy_Lights`.
pub const UBO_LIGHTS_BYTES_LENGTH: u32 = 16
    + 16
    + 64 * MAX_DIRECTIONAL_LIGHTS as u32
    + 64 * MAX_POINT_LIGHTS as u32
    + 80 * MAX_SPOT_LIGHTS as u32
    + 112 * MAX_AREA_LIGHTS as u32;
/// Uniform Buffer Object bytes length for `u_Attenuations`.
pub const UBO_LIGHTS_ATTENUATIONS_BYTES_LENGTH: u32 = 12;
/// Uniform Buffer Object bytes length for `u_AmbientLight`.
pub const UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH: u32 = 16;
/// Uniform Buffer Object bytes length for `u_DirectionalLights`.
pub const UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_PointLights`.
pub const UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_SpotLights`.
pub const UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH: u32 = 80;
/// Uniform Buffer Object bytes length for `u_AreaLights`.
pub const UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH: u32 = 112;

/// Uniform Buffer Object bytes offset for `u_Attenuations`.
pub const UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET: u32 = 0;
/// Uniform Buffer Object bytes offset for `u_AmbientLight`.
pub const UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET: u32 = 16;
/// Uniform Buffer Object bytes offset for `u_DirectionalLights`.
pub const UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET: u32 = 32;
/// Uniform Buffer Object bytes offset for `u_PointLights`.
pub const UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET: u32 = 800;
/// Uniform Buffer Object bytes offset for `u_SpotLights`.
pub const UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET: u32 = 1568;
/// Uniform Buffer Object bytes offset for `u_AreaLights`.
pub const UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET: u32 = 2528;

/// Uniform Buffer Object data in f32 for `atoy_GaussianKernel`.
#[rustfmt::skip]
pub const UBO_GAUSSIAN_KERNEL: [f32; 324] = [
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
/// Uniform Buffer Object data in u8 for `atoy_GaussianKernel`.
pub const UBO_GAUSSIAN_KERNEL_U8: [u8; 324 * 4] =
    unsafe { std::mem::transmute_copy::<[f32; 324], [u8; 324 * 4]>(&UBO_GAUSSIAN_KERNEL) };

const BLOOM_GLSL_DEFINE: &'static str = "BLOOM";

const BLOOM_THRESHOLD_UNIFORM_NAME: &'static str = "u_BloomThreshold";
const BLOOM_THRESHOLD_VALUES: [f32; 3] = [0.2126, 0.7152, 0.0722];
const BASE_TEXTURE: &'static str = "u_BaseTexture";
const BLOOM_BLUR_TEXTURE: &'static str = "u_BloomBlurTexture";
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
        bloom_blur,
        collected_entities,
        universal_ubo,
        lights_ubo,
    )?;
    draw_translucent_entities(
        state,
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

fn draw_entity(
    state: &mut FrameState,
    enable_bloom: bool,
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
    let program = if enable_bloom {
        let program = state.program_store_mut().use_program_with_defines(
            material.as_program_source(),
            vec![],
            vec![Cow::Borrowed(BLOOM_GLSL_DEFINE)],
        )?;
        state.bind_uniform_value_by_variable_name(
            program,
            BLOOM_THRESHOLD_UNIFORM_NAME,
            UniformValue::FloatVector3(BLOOM_THRESHOLD_VALUES),
        )?;
        program
    } else {
        state
            .program_store_mut()
            .use_program(material.as_program_source())?
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
    state.bind_uniform_block_value_by_block_name(
        program,
        UBO_LIGHTS_BLOCK_NAME,
        UniformBlockValue::BufferBase {
            descriptor: lights_ubo.clone(),
            binding: UBO_LIGHTS_BINDING,
        },
    )?;

    let bound_attributes = state.bind_attributes(program, &entity, geometry, material)?;
    state.bind_uniforms(program, &entity, geometry, material)?;
    state.draw(&geometry.draw())?;
    state.unbind_attributes(bound_attributes);

    Ok(())
}

fn draw_opaque_entities(
    state: &mut FrameState,
    enable_bloom: bool,
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
        draw_entity(state, enable_bloom, true, entity, universal_ubo, lights_ubo)?;
    }

    Ok(())
}

fn draw_translucent_entities(
    state: &mut FrameState,
    enable_bloom: bool,
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
            enable_bloom,
            false,
            entity,
            universal_ubo,
            lights_ubo,
        )?; // transparency entities never cull face
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