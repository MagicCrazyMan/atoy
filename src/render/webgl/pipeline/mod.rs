pub mod cleanup;
pub mod collector;
pub mod composer;
pub mod preparation;
pub mod shading;

use gl_matrix4rust::{vec3::Vec3, vec4::Vec4};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entity::Entity,
    light::{
        area_light::MAX_AREA_LIGHTS, directional_light::MAX_DIRECTIONAL_LIGHTS,
        point_light::MAX_POINT_LIGHTS, spot_light::MAX_SPOT_LIGHTS,
    },
    render::Pipeline,
    scene::Scene,
};

use self::{
    cleanup::StandardCleanup,
    collector::StandardEntitiesCollector,
    composer::StandardComposer,
    preparation::StandardPreparation,
    shading::{
        deferred::{
            gbuffer::StandardGBufferCollector, simple::StandardDeferredTransparentShading,
            StandardDeferredShading,
        },
        forward::{
            hdr::StandardHdrShading, hdr_multisamples::StandardMultisamplesHdrShading,
            simple::StandardSimpleShading, simple_multisamples::StandardMultisamplesSimpleShading,
        },
        picking::StandardPicking,
    },
};

use super::{
    buffer::{BufferDescriptor, BufferSource, BufferUsage, MemoryPolicy},
    error::Error,
    state::FrameState,
};

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

/// Uniform Buffer Object bytes length for `u_RenderTime`.
pub const UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_LENGTH: u32 = 16;
/// Uniform Buffer Object bytes length for `u_CameraPosition`.
pub const UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_LENGTH: u32 = 16;
/// Uniform Buffer Object bytes length for `u_ViewMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_ProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_ViewProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_LENGTH: u32 = 64;

/// Uniform Buffer Object bytes length for `atoy_UniversalUniformsVert` and `atoy_UniversalUniformsFrag`.
pub const UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH: u32 = UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_LENGTH
    + UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_LENGTH
    + UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_LENGTH
    + UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_LENGTH
    + UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_LENGTH;

/// Uniform Buffer Object bytes offset for `u_RenderTime`.
pub const UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET: u32 = 0;
/// Uniform Buffer Object bytes offset for `u_CameraPosition`.
pub const UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET: u32 = 16;
/// Uniform Buffer Object bytes offset for `u_ViewMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET: u32 = 32;
/// Uniform Buffer Object bytes offset for `u_ProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET: u32 = 96;
/// Uniform Buffer Object bytes offset for `u_ViewProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET: u32 = 160;

/// Uniform Buffer Object bytes length for `u_Attenuations`.
pub const UBO_LIGHTS_ATTENUATIONS_BYTES_LENGTH: u32 = 16;
/// Uniform Buffer Object bytes length for a `u_AmbientLight` item.
pub const UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH: u32 = 16;
/// Uniform Buffer Object bytes length for a `u_DirectionalLights` item.
pub const UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for a `u_PointLights` item.
pub const UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for a `u_SpotLights` item.
pub const UBO_LIGHTS_SPOT_LIGHT_BYTES_LENGTH: u32 = 80;
/// Uniform Buffer Object bytes length for a `u_AreaLights` item.
pub const UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH: u32 = 112;

/// Uniform Buffer Object bytes length for `atoy_Lights`.
pub const UBO_LIGHTS_BYTES_LENGTH: u32 = UBO_LIGHTS_ATTENUATIONS_BYTES_LENGTH
    + UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH
    + UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH * MAX_DIRECTIONAL_LIGHTS as u32
    + UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH * MAX_POINT_LIGHTS as u32
    + UBO_LIGHTS_SPOT_LIGHT_BYTES_LENGTH * MAX_SPOT_LIGHTS as u32
    + UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH * MAX_AREA_LIGHTS as u32;

/// Uniform Buffer Object bytes offset for `u_Attenuations`.
pub const UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET: u32 = 0;
/// Uniform Buffer Object bytes offset for `u_AmbientLight`.
pub const UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET: u32 = 16;
/// Uniform Buffer Object bytes offset for `u_DirectionalLights`.
pub const UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET: u32 = 32;
/// Uniform Buffer Object bytes offset for `u_PointLights`.
pub const UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET: u32 = UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET
    + MAX_DIRECTIONAL_LIGHTS as u32 * UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH;
/// Uniform Buffer Object bytes offset for `u_SpotLights`.
pub const UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET: u32 = UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET
    + MAX_POINT_LIGHTS as u32 * UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH;
/// Uniform Buffer Object bytes offset for `u_AreaLights`.
pub const UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET: u32 = UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET
    + MAX_SPOT_LIGHTS as u32 * UBO_LIGHTS_SPOT_LIGHT_BYTES_LENGTH;

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
pub const UBO_GAUSSIAN_KERNEL_U8: &[u8; 324 * 4] =
    unsafe { std::mem::transmute::<&[f32; 324], &[u8; 324 * 4]>(&UBO_GAUSSIAN_KERNEL) };

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum HdrToneMappingType {
    Reinhard,
    Exposure(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum StandardPipelineShading {
    ForwardShading,
    DeferredShading,
    Picking,
}

pub const DEFAULT_SHADING: StandardPipelineShading = StandardPipelineShading::DeferredShading;
pub const DEFAULT_LIGHTING_ENABLED: bool = true;
pub const DEFAULT_MULTISAMPLES: i32 = 4;
pub const DEFAULT_HDR_ENABLED: bool = true;
pub const DEFAULT_HDR_TONE_MAPPING_TYPE: HdrToneMappingType = HdrToneMappingType::Reinhard;
pub const DEFAULT_BLOOM_ENABLED: bool = false;
pub const DEFAULT_BLOOM_BLUR_EPOCH: usize = 5;

pub struct StandardPipeline {
    pipeline_shading: StandardPipelineShading,

    preparation: StandardPreparation,
    entities_collector: StandardEntitiesCollector,
    simple_shading: StandardSimpleShading,
    multisamples_simple_shading: StandardMultisamplesSimpleShading,
    hdr_shading: StandardHdrShading,
    multisamples_hdr_shading: StandardMultisamplesHdrShading,
    composer: StandardComposer,
    cleanup: StandardCleanup,

    gbuffer: StandardGBufferCollector,
    deferred_shading: StandardDeferredShading,
    deferred_translucent_shading: StandardDeferredTransparentShading,

    picking: StandardPicking,

    universal_ubo: BufferDescriptor,
    lights_ubo: BufferDescriptor,
    gaussian_kernel_ubo: BufferDescriptor,

    hdr_supported: bool,

    enable_lighting: bool,
    multisamples: Option<i32>,
    enable_hdr: bool,
    hdr_tone_mapping_type: HdrToneMappingType,
    enable_bloom: bool,
    bloom_blur_epoch: usize,
}

impl StandardPipeline {
    pub fn new(hdr_supported: bool) -> Self {
        Self {
            pipeline_shading: DEFAULT_SHADING,

            preparation: StandardPreparation::new(),
            entities_collector: StandardEntitiesCollector::new(),
            simple_shading: StandardSimpleShading::new(),
            multisamples_simple_shading: StandardMultisamplesSimpleShading::new(),
            multisamples_hdr_shading: StandardMultisamplesHdrShading::new(),
            hdr_shading: StandardHdrShading::new(),
            composer: StandardComposer::new(),
            cleanup: StandardCleanup::new(),
            picking: StandardPicking::new(),

            gbuffer: StandardGBufferCollector::new(),
            deferred_shading: StandardDeferredShading::new(),
            deferred_translucent_shading: StandardDeferredTransparentShading::new(),

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
            gaussian_kernel_ubo: BufferDescriptor::with_memory_policy(
                BufferSource::from_binary(
                    &UBO_GAUSSIAN_KERNEL_U8,
                    0,
                    UBO_GAUSSIAN_KERNEL_U8.len() as u32,
                ),
                BufferUsage::StaticDraw,
                MemoryPolicy::restorable(|| {
                    BufferSource::from_binary(
                        &UBO_GAUSSIAN_KERNEL_U8,
                        0,
                        UBO_GAUSSIAN_KERNEL_U8.len() as u32,
                    )
                }),
            ),

            hdr_supported,

            enable_lighting: DEFAULT_LIGHTING_ENABLED,
            multisamples: Some(DEFAULT_MULTISAMPLES),
            enable_hdr: DEFAULT_HDR_ENABLED,
            hdr_tone_mapping_type: DEFAULT_HDR_TONE_MAPPING_TYPE,
            enable_bloom: DEFAULT_BLOOM_ENABLED,
            bloom_blur_epoch: DEFAULT_BLOOM_BLUR_EPOCH,
        }
    }

    #[inline]
    pub fn set_dirty(&mut self) {}

    #[inline]
    pub fn pipeline_shading(&self) -> StandardPipelineShading {
        self.pipeline_shading
    }

    #[inline]
    pub fn set_pipeline_shading(&mut self, pipeline_shading: StandardPipelineShading) {
        self.pipeline_shading = pipeline_shading;
    }

    #[inline]
    pub fn clear_color(&self) -> &Vec4<f32> {
        self.composer.clear_color()
    }

    #[inline]
    pub fn set_clear_color(&mut self, clear_color: Vec4<f32>) {
        self.composer.set_clear_color(clear_color);
        self.set_dirty();
    }

    #[inline]
    pub fn gamma_correction_enabled(&self) -> bool {
        self.composer.gamma_correction_enabled()
    }

    #[inline]
    pub fn enable_gamma_correction(&mut self) {
        self.composer.enable_gamma_correction();
    }

    #[inline]
    pub fn disable_gamma_correction(&mut self) {
        self.composer.disable_gamma_correction();
    }

    #[inline]
    pub fn gamma(&self) -> f32 {
        self.composer.gamma()
    }

    #[inline]
    pub fn set_gamma(&mut self, gamma: f32) {
        self.composer.set_gamma(gamma);
    }

    /// Returns `true` if entity culling enabled.
    #[inline]
    pub fn culling_enabled(&self) -> bool {
        self.entities_collector.culling_enabled()
    }

    /// Enables culling by bounding volumes.
    #[inline]
    pub fn enable_culling(&mut self) {
        self.entities_collector.enable_culling();
        self.set_dirty();
    }

    /// Disables culling by bounding volumes.
    #[inline]
    pub fn disable_culling(&mut self) {
        self.entities_collector.disable_culling();
        self.set_dirty();
    }

    /// Returns `true` if entity distance sorting enabled.
    #[inline]
    pub fn distance_sorting_enabled(&self) -> bool {
        self.entities_collector.distance_sorting_enabled()
    }

    /// Enables distance sorting by bounding volumes.
    #[inline]
    pub fn enable_distance_sorting(&mut self) {
        self.entities_collector.enable_distance_sorting();
        self.set_dirty();
    }

    /// Disables distance sorting by bounding volumes.
    #[inline]
    pub fn disable_distance_sorting(&mut self) {
        self.entities_collector.disable_distance_sorting();
        self.set_dirty();
    }

    /// Returns `true` if enable lighting.
    /// Diffuse color of material used directly if lighting is disabled.
    #[inline]
    pub fn lighting_enabled(&self) -> bool {
        self.enable_lighting
    }

    /// Enables lighting.
    #[inline]
    pub fn enable_lighting(&mut self) {
        self.enable_lighting = true;
        self.set_dirty();
    }

    /// Disables lighting.
    #[inline]
    pub fn disable_lighting(&mut self) {
        self.enable_lighting = false;
        self.set_dirty();
    }

    #[inline]
    pub fn hdr_enabled(&self) -> bool {
        self.enable_hdr
    }

    #[inline]
    pub fn enable_hdr(&mut self) {
        self.enable_hdr = true;
        self.set_dirty();
    }

    #[inline]
    pub fn disable_hdr(&mut self) {
        self.enable_hdr = false;
        self.set_dirty();
    }

    #[inline]
    pub fn hdr_tone_mapping_type(&self) -> HdrToneMappingType {
        self.hdr_tone_mapping_type
    }

    #[inline]
    pub fn set_hdr_tone_mapping_type(&mut self, tone_mapping_type: HdrToneMappingType) {
        self.hdr_tone_mapping_type = tone_mapping_type;
        self.set_dirty();
    }

    #[inline]
    pub fn bloom_enabled(&self) -> bool {
        self.enable_bloom
    }

    #[inline]
    pub fn enable_bloom(&mut self) {
        self.enable_bloom = true;
        self.set_dirty();
    }

    #[inline]
    pub fn disable_bloom(&mut self) {
        self.enable_bloom = false;
        self.set_dirty();
    }

    #[inline]
    pub fn bloom_blur_epoch(&self) -> usize {
        self.bloom_blur_epoch
    }

    #[inline]
    pub fn set_bloom_blur_epoch(&mut self, epoch: usize) {
        self.bloom_blur_epoch = epoch;
        self.set_dirty();
    }

    #[inline]
    pub fn multisamples(&self) -> Option<i32> {
        match self.multisamples {
            Some(samples) => {
                if samples == 0 {
                    None
                } else {
                    Some(samples)
                }
            }
            None => None,
        }
    }

    pub fn set_multisamples(&mut self, samples: Option<i32>) {
        match samples {
            Some(samples) => {
                if samples == 0 {
                    self.multisamples = None;
                } else {
                    self.multisamples = Some(samples);
                }
            }
            None => {
                self.multisamples = None;
            }
        };
        self.set_dirty();
    }

    /// Returns picked entity index.
    /// Executes [`StandardPipeline::picking`] before calling this method, or the result maybe incorrect.
    pub unsafe fn pick_entity(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<&mut Entity>, Error> {
        let Some(last_collected_entities) = self.entities_collector.last_collected_entities()
        else {
            return Ok(None);
        };

        self.picking.pick_entity(
            window_position_x,
            window_position_y,
            &last_collected_entities,
        )
    }

    /// Returns picked entity id.
    /// Executes [`StandardPipeline::picking`] before calling this method, or the result maybe incorrect.
    pub unsafe fn pick_entity_id(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<Uuid>, Error> {
        let Some(entity) = self.pick_entity(window_position_x, window_position_y)? else {
            return Ok(None);
        };
        Ok(Some(*entity.id()))
    }

    /// Returns picked position.
    /// Executes [`StandardPipeline::picking`] before calling this method, or the result maybe incorrect.
    pub unsafe fn pick_position(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<Vec3>, Error> {
        let Some(last_collected_entities) = self.entities_collector.last_collected_entities()
        else {
            return Ok(None);
        };

        self.picking.pick_position(
            window_position_x,
            window_position_y,
            &last_collected_entities,
        )
    }
}

impl StandardPipeline {
    fn forward_shading(&mut self, state: &mut FrameState, scene: &mut Scene) -> Result<(), Error> {
        let lights_ubo = if self.lighting_enabled() {
            Some(&self.lights_ubo)
        } else {
            None
        };
        let hdr = self.hdr_enabled() && self.hdr_supported;
        let bloom = self.bloom_enabled();
        let bloom_blur_epoch = self.bloom_blur_epoch();
        let multisamples = self.multisamples();

        unsafe {
            let collected_entities = self.entities_collector.collect_entities(state, scene);
            let compose_textures = match (hdr, multisamples) {
                (true, None) => {
                    self.hdr_shading.draw(
                        state,
                        bloom,
                        bloom_blur_epoch,
                        self.hdr_tone_mapping_type,
                        &collected_entities,
                        &self.universal_ubo,
                        lights_ubo,
                        &self.gaussian_kernel_ubo,
                    )?;
                    self.hdr_shading.draw_texture().unwrap()
                }
                (true, Some(samples)) => {
                    self.multisamples_hdr_shading.draw(
                        state,
                        samples,
                        bloom,
                        bloom_blur_epoch,
                        self.hdr_tone_mapping_type,
                        &collected_entities,
                        &self.universal_ubo,
                        lights_ubo,
                        &self.gaussian_kernel_ubo,
                    )?;
                    self.multisamples_hdr_shading.draw_texture().unwrap()
                }
                (false, None) => {
                    self.simple_shading.draw(
                        state,
                        &collected_entities,
                        &self.universal_ubo,
                        lights_ubo,
                    )?;
                    self.simple_shading.draw_texture().unwrap()
                }
                (false, Some(samples)) => {
                    self.multisamples_simple_shading.draw(
                        state,
                        samples,
                        &collected_entities,
                        &self.universal_ubo,
                        lights_ubo,
                    )?;
                    self.multisamples_simple_shading.draw_texture().unwrap()
                }
            };
            self.composer.compose(state, [compose_textures])?;
            self.cleanup.cleanup(state);
        };

        Ok(())
    }

    fn deferred_shading(&mut self, state: &mut FrameState, scene: &mut Scene) -> Result<(), Error> {
        let lights_ubo = if self.lighting_enabled() {
            Some(&self.lights_ubo)
        } else {
            None
        };

        let collected_entities = self.entities_collector.collect_entities(state, scene);

        unsafe {
            // deferred shading on opaque entities
            self.gbuffer
                .draw(state, &collected_entities, &self.universal_ubo)?;
            let (
                positions_and_specular_shininess_texture,
                normals_texture,
                albedo_texture,
                depth_stencil,
            ) = self.gbuffer.deferred_shading_textures().unwrap();
            self.deferred_shading.draw(
                state,
                positions_and_specular_shininess_texture,
                normals_texture,
                albedo_texture,
                &self.universal_ubo,
                lights_ubo,
            )?;

            // then forward shading on translucent entities
            self.deferred_translucent_shading.draw(
                state,
                &depth_stencil,
                &collected_entities,
                &self.universal_ubo,
                lights_ubo,
            )?;

            let opaque_textures = self.deferred_shading.draw_texture().unwrap();
            let translucent_texture = self.deferred_translucent_shading.draw_texture().unwrap();
            self.composer
                .compose(state, [opaque_textures, translucent_texture])?;
        }

        Ok(())
    }

    fn picking(&mut self, state: &mut FrameState, scene: &mut Scene) -> Result<(), Error> {
        let collected_entities = self.entities_collector.collect_entities(state, scene);

        unsafe {
            self.picking.draw(state, &collected_entities)?;
        }

        Ok(())
    }
}

impl Pipeline for StandardPipeline {
    type State = FrameState;

    type Error = Error;

    fn execute(&mut self, state: &mut Self::State, scene: &mut Scene) -> Result<(), Self::Error> {
        match self.pipeline_shading {
            StandardPipelineShading::Picking => self.picking(state, scene),
            _ => {
                // prepares
                {
                    let lights_ubo = if self.lighting_enabled() {
                        Some(&mut self.lights_ubo)
                    } else {
                        None
                    };
                    self.preparation
                        .prepare(state, scene, &mut self.universal_ubo, lights_ubo)?;
                }

                // shading
                {
                    match self.pipeline_shading {
                        StandardPipelineShading::ForwardShading => {
                            self.forward_shading(state, scene)
                        }
                        StandardPipelineShading::DeferredShading => {
                            self.deferred_shading(state, scene)
                        }
                        StandardPipelineShading::Picking => unreachable!(),
                    }
                }
            }
        }
    }
}
