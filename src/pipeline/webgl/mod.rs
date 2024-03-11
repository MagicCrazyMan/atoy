pub mod cleanup;
pub mod collector;
pub mod composer;
pub mod preparation;
pub mod shading;

use gl_matrix4rust::{vec3::Vec3, vec4::Vec4};
use serde::{Deserialize, Serialize};

use crate::{
    clock::WebClock,
    entity::Entity,
    renderer::webgl::{
        buffer::{self, Buffer, BufferSource, BufferSourceData, BufferUsage, MemoryPolicy, Preallocation},
        error::Error,
        state::FrameState,
    },
    scene::{Scene, MAX_AREA_LIGHTS, MAX_DIRECTIONAL_LIGHTS, MAX_POINT_LIGHTS, MAX_SPOT_LIGHTS},
    share::Share,
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

use super::Pipeline;

/// Uniform Buffer Object `atoy_UniversalUniforms`.
pub const UBO_UNIVERSAL_UNIFORMS_BLOCK_NAME: &'static str = "atoy_UniversalUniforms";
/// Uniform Buffer Object `atoy_Lights`.
pub const UBO_LIGHTS_BLOCK_NAME: &'static str = "atoy_Lights";
/// Uniform Buffer Object `atoy_GaussianKernel`.
pub const UBO_GAUSSIAN_KERNEL_BLOCK_NAME: &'static str = "atoy_GaussianKernel";

/// Uniform Buffer Object mount point for `atoy_UniversalUniformsVert` and `atoy_UniversalUniformsFrag`.
pub const UBO_UNIVERSAL_UNIFORMS_BINDING_MOUNT_POINT: u32 = 0;
/// Uniform Buffer Object mount point for `atoy_Lights`.
pub const UBO_LIGHTS_BINDING_MOUNT_POINT: u32 = 1;
/// Uniform Buffer Object mount point for gaussian blur.
pub const UBO_GAUSSIAN_BLUR_BINDING_MOUNT_POINT: u32 = 2;

/// Uniform Buffer Object bytes length for `u_RenderTime`.
pub const UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTE_LENGTH: usize = 16;
/// Uniform Buffer Object bytes length for `u_CameraPosition`.
pub const UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTE_LENGTH: usize = 16;
/// Uniform Buffer Object bytes length for `u_ViewMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTE_LENGTH: usize = 64;
/// Uniform Buffer Object bytes length for `u_ProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTE_LENGTH: usize = 64;
/// Uniform Buffer Object bytes length for `u_ViewProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTE_LENGTH: usize = 64;

/// Uniform Buffer Object bytes length for `atoy_UniversalUniformsVert` and `atoy_UniversalUniformsFrag`.
pub const UBO_UNIVERSAL_UNIFORMS_BYTE_LENGTH: usize = UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTE_LENGTH
    + UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTE_LENGTH
    + UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTE_LENGTH
    + UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTE_LENGTH
    + UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTE_LENGTH;

/// Uniform Buffer Object bytes offset for `u_RenderTime`.
pub const UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTE_OFFSET: usize = 0;
/// Uniform Buffer Object bytes offset for `u_CameraPosition`.
pub const UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTE_OFFSET: usize = 16;
/// Uniform Buffer Object bytes offset for `u_ViewMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTE_OFFSET: usize = 32;
/// Uniform Buffer Object bytes offset for `u_ProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTE_OFFSET: usize = 96;
/// Uniform Buffer Object bytes offset for `u_ViewProjMatrix`.
pub const UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTE_OFFSET: usize = 160;

/// Uniform Buffer Object bytes length for `u_Attenuations`.
pub const UBO_LIGHTS_ATTENUATIONS_BYTE_LENGTH: usize = 16;
/// Uniform Buffer Object bytes length for a `u_AmbientLight` item.
pub const UBO_LIGHTS_AMBIENT_LIGHT_BYTE_LENGTH: usize = 16;
/// Uniform Buffer Object bytes length for a `u_DirectionalLights` item.
pub const UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTE_LENGTH: usize = 64;
/// Uniform Buffer Object bytes length for a `u_PointLights` item.
pub const UBO_LIGHTS_POINT_LIGHT_BYTE_LENGTH: usize = 64;
/// Uniform Buffer Object bytes length for a `u_SpotLights` item.
pub const UBO_LIGHTS_SPOT_LIGHT_BYTE_LENGTH: usize = 80;
/// Uniform Buffer Object bytes length for a `u_AreaLights` item.
pub const UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH: usize = 112;

/// Uniform Buffer Object bytes length for `atoy_Lights`.
pub const UBO_LIGHTS_BYTE_LENGTH: usize = UBO_LIGHTS_ATTENUATIONS_BYTE_LENGTH
    + UBO_LIGHTS_AMBIENT_LIGHT_BYTE_LENGTH
    + UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTE_LENGTH * MAX_DIRECTIONAL_LIGHTS
    + UBO_LIGHTS_POINT_LIGHT_BYTE_LENGTH * MAX_POINT_LIGHTS
    + UBO_LIGHTS_SPOT_LIGHT_BYTE_LENGTH * MAX_SPOT_LIGHTS
    + UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH * MAX_AREA_LIGHTS;

/// Uniform Buffer Object bytes offset for `u_Attenuations`.
pub const UBO_LIGHTS_ATTENUATIONS_BYTE_OFFSET: usize = 0;
/// Uniform Buffer Object bytes offset for `u_AmbientLight`.
pub const UBO_LIGHTS_AMBIENT_LIGHT_BYTE_OFFSET: usize = 16;
/// Uniform Buffer Object bytes offset for `u_DirectionalLights`.
pub const UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTE_OFFSET: usize = 32;
/// Uniform Buffer Object bytes offset for `u_PointLights`.
pub const UBO_LIGHTS_POINT_LIGHTS_BYTE_OFFSET: usize = UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTE_OFFSET
    + MAX_DIRECTIONAL_LIGHTS * UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTE_LENGTH;
/// Uniform Buffer Object bytes offset for `u_SpotLights`.
pub const UBO_LIGHTS_SPOT_LIGHTS_BYTE_OFFSET: usize =
    UBO_LIGHTS_POINT_LIGHTS_BYTE_OFFSET + MAX_POINT_LIGHTS * UBO_LIGHTS_POINT_LIGHT_BYTE_LENGTH;
/// Uniform Buffer Object bytes offset for `u_AreaLights`.
pub const UBO_LIGHTS_AREA_LIGHTS_BYTE_OFFSET: usize =
    UBO_LIGHTS_SPOT_LIGHTS_BYTE_OFFSET + MAX_SPOT_LIGHTS * UBO_LIGHTS_SPOT_LIGHT_BYTE_LENGTH;

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
pub const UBO_GAUSSIAN_KERNEL_BYTES: &[u8; 324 * 4] =
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
pub const DEFAULT_MULTISAMPLES_ENABLED: bool = true;
pub const DEFAULT_MULTISAMPLES_COUNT: i32 = 4;
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
    picking: StandardPicking,

    gbuffer: StandardGBufferCollector,
    deferred_shading: StandardDeferredShading,
    deferred_translucent_shading: StandardDeferredTransparentShading,

    universal_ubo: Buffer,
    lights_ubo: Buffer,
    gaussian_kernel_ubo: Buffer,

    lighting: bool,
    multisamples: bool,
    multisamples_count: i32,
    hdr: bool,
    hdr_tone_mapping_type: HdrToneMappingType,
    bloom: bool,
    bloom_blur_epoch: usize,
}

#[derive(Debug)]
struct GaussianKernelBufferSource;

impl BufferSource for GaussianKernelBufferSource {
    fn data(&self) -> BufferSourceData<'_> {
        BufferSourceData::BytesBorrowed(UBO_GAUSSIAN_KERNEL_BYTES)
    }
    
    fn src_element_offset(&self) -> Option<usize> {
        None
    }
    
    fn src_element_length(&self) -> Option<usize> {
        None
    }
}

impl StandardPipeline {
    pub fn new() -> Self {
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

            universal_ubo: buffer::Builder::new(BufferUsage::DYNAMIC_DRAW)
                .buffer_data(Preallocation::new(UBO_UNIVERSAL_UNIFORMS_BYTE_LENGTH))
                .set_memory_policy(MemoryPolicy::Unfree)
                .build(),
            lights_ubo: buffer::Builder::new(BufferUsage::DYNAMIC_DRAW)
                .buffer_data(Preallocation::new(UBO_LIGHTS_BYTE_LENGTH))
                .set_memory_policy(MemoryPolicy::Unfree)
                .build(),
            gaussian_kernel_ubo: buffer::Builder::new(BufferUsage::STATIC_DRAW)
                .buffer_data(UBO_GAUSSIAN_KERNEL_BYTES)
                .set_memory_policy(MemoryPolicy::restorable(GaussianKernelBufferSource))
                .build(),

            lighting: DEFAULT_LIGHTING_ENABLED,
            multisamples: DEFAULT_MULTISAMPLES_ENABLED,
            multisamples_count: DEFAULT_MULTISAMPLES_COUNT,
            hdr: DEFAULT_HDR_ENABLED,
            hdr_tone_mapping_type: DEFAULT_HDR_TONE_MAPPING_TYPE,
            bloom: DEFAULT_BLOOM_ENABLED,
            bloom_blur_epoch: DEFAULT_BLOOM_BLUR_EPOCH,
        }
    }

    pub fn set_dirty(&mut self) {}

    pub fn pipeline_shading(&self) -> StandardPipelineShading {
        self.pipeline_shading
    }

    pub fn set_pipeline_shading(&mut self, pipeline_shading: StandardPipelineShading) {
        self.pipeline_shading = pipeline_shading;
    }

    pub fn clear_color(&self) -> &Vec4<f32> {
        self.composer.clear_color()
    }

    pub fn set_clear_color(&mut self, clear_color: Vec4<f32>) {
        self.composer.set_clear_color(clear_color);
        self.set_dirty();
    }

    pub fn gamma_correction_enabled(&self) -> bool {
        self.composer.gamma_correction_enabled()
    }

    pub fn enable_gamma_correction(&mut self) {
        self.composer.enable_gamma_correction();
    }

    pub fn disable_gamma_correction(&mut self) {
        self.composer.disable_gamma_correction();
    }

    pub fn gamma(&self) -> f32 {
        self.composer.gamma()
    }

    pub fn set_gamma(&mut self, gamma: f32) {
        self.composer.set_gamma(gamma);
    }

    /// Returns `true` if entity culling enabled.

    pub fn culling_enabled(&self) -> bool {
        self.entities_collector.culling_enabled()
    }

    /// Enables culling by bounding volumes.

    pub fn enable_culling(&mut self) {
        self.entities_collector.enable_culling();
        self.set_dirty();
    }

    /// Disables culling by bounding volumes.

    pub fn disable_culling(&mut self) {
        self.entities_collector.disable_culling();
        self.set_dirty();
    }

    /// Returns `true` if entity distance sorting enabled.

    pub fn distance_sorting_enabled(&self) -> bool {
        self.entities_collector.distance_sorting_enabled()
    }

    /// Enables distance sorting by bounding volumes.

    pub fn enable_distance_sorting(&mut self) {
        self.entities_collector.enable_distance_sorting();
        self.set_dirty();
    }

    /// Disables distance sorting by bounding volumes.

    pub fn disable_distance_sorting(&mut self) {
        self.entities_collector.disable_distance_sorting();
        self.set_dirty();
    }

    /// Returns `true` if enable lighting.
    /// Diffuse color of material used directly if lighting is disabled.

    pub fn lighting_enabled(&self) -> bool {
        self.lighting
    }

    /// Enables lighting.

    pub fn enable_lighting(&mut self) {
        self.lighting = true;
        self.set_dirty();
    }

    /// Disables lighting.

    pub fn disable_lighting(&mut self) {
        self.lighting = false;
        self.set_dirty();
    }

    pub fn hdr_enabled(&self) -> bool {
        self.hdr
    }

    pub fn enable_hdr(&mut self) {
        self.hdr = true;
        self.set_dirty();
    }

    pub fn disable_hdr(&mut self) {
        self.hdr = false;
        self.set_dirty();
    }

    pub fn hdr_tone_mapping_type(&self) -> HdrToneMappingType {
        self.hdr_tone_mapping_type
    }

    pub fn set_hdr_tone_mapping_type(&mut self, tone_mapping_type: HdrToneMappingType) {
        self.hdr_tone_mapping_type = tone_mapping_type;
        self.set_dirty();
    }

    pub fn bloom_enabled(&self) -> bool {
        self.bloom
    }

    pub fn enable_bloom(&mut self) {
        self.bloom = true;
        self.set_dirty();
    }

    pub fn disable_bloom(&mut self) {
        self.bloom = false;
        self.set_dirty();
    }

    pub fn bloom_blur_epoch(&self) -> usize {
        self.bloom_blur_epoch
    }

    pub fn set_bloom_blur_epoch(&mut self, epoch: usize) {
        self.bloom_blur_epoch = epoch;
        self.set_dirty();
    }

    pub fn multisamples_enabled(&self) -> bool {
        self.multisamples
    }

    pub fn enable_multisamples(&mut self) {
        self.multisamples = true;
        self.set_dirty();
    }

    pub fn disable_multisamples(&mut self) {
        self.multisamples = false;
        self.set_dirty();
    }

    pub fn multisamples_count(&self) -> i32 {
        self.multisamples_count
    }

    pub fn set_multisamples_count(&mut self, count: i32) {
        self.multisamples_count = count;
        self.set_dirty();
    }

    /// Returns picked entity index.
    /// Executes [`StandardPipeline::picking`] before calling this method, or the result maybe incorrect.
    pub unsafe fn pick_entity(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<Share<dyn Entity>>, Error> {
        self.picking.pick_entity(
            window_position_x,
            window_position_y,
            &self.entities_collector.last_collected_entities(),
        )
    }

    /// Returns picked position.
    /// Executes [`StandardPipeline::picking`] before calling this method, or the result maybe incorrect.
    pub unsafe fn pick_position(
        &mut self,
        window_position_x: i32,
        window_position_y: i32,
    ) -> Result<Option<Vec3>, Error> {
        self.picking.pick_position(
            window_position_x,
            window_position_y,
            &self.entities_collector.last_collected_entities(),
        )
    }
}

impl StandardPipeline {
    fn forward_shading(
        &mut self,
        state: &mut FrameState,
        scene: &mut Scene<WebClock>,
    ) -> Result<(), Error> {
        let lighting = self.lighting_enabled();
        let hdr_supported = state.capabilities().color_buffer_float_supported();
        let hdr = hdr_supported && self.hdr_enabled();
        let bloom = self.bloom_enabled();
        let bloom_blur_epoch = self.bloom_blur_epoch();
        let multisamples = self.multisamples_enabled() && self.multisamples_count() != 0;

        unsafe {
            let collected_entities = self.entities_collector.collect_entities(state, scene);
            let compose_textures = match (hdr, multisamples) {
                (true, false) => {
                    self.hdr_shading.draw(
                        state,
                        bloom,
                        bloom_blur_epoch,
                        self.hdr_tone_mapping_type,
                        &collected_entities,
                        lighting,
                        &self.gaussian_kernel_ubo,
                    )?;
                    self.hdr_shading.draw_texture().unwrap()
                }
                (true, true) => {
                    self.multisamples_hdr_shading.draw(
                        state,
                        self.multisamples_count,
                        bloom,
                        bloom_blur_epoch,
                        self.hdr_tone_mapping_type,
                        &collected_entities,
                        lighting,
                        &self.gaussian_kernel_ubo,
                    )?;
                    self.multisamples_hdr_shading.draw_texture().unwrap()
                }
                (false, false) => {
                    self.simple_shading
                        .draw(state, &collected_entities, lighting)?;
                    self.simple_shading.draw_texture().unwrap()
                }
                (false, true) => {
                    self.multisamples_simple_shading.draw(
                        state,
                        self.multisamples_count,
                        &collected_entities,
                        lighting,
                    )?;
                    self.multisamples_simple_shading.draw_texture().unwrap()
                }
            };
            self.composer.draw(state, [compose_textures])?;
        };

        Ok(())
    }

    fn deferred_shading(
        &mut self,
        state: &mut FrameState,
        scene: &mut Scene<WebClock>,
    ) -> Result<(), Error> {
        let lighting = self.lighting_enabled();

        let collected_entities = self.entities_collector.collect_entities(state, scene);

        unsafe {
            // deferred shading on opaque entities
            self.gbuffer.collect(state, &collected_entities)?;
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
                lighting,
            )?;

            // then forward shading on translucent entities
            self.deferred_translucent_shading.draw(
                state,
                &depth_stencil,
                &collected_entities,
                lighting,
            )?;

            let opaque_textures = self.deferred_shading.draw_texture().unwrap();
            let translucent_texture = self.deferred_translucent_shading.draw_texture().unwrap();
            self.composer
                .draw(state, [opaque_textures, translucent_texture])?;
        }

        Ok(())
    }

    fn picking(
        &mut self,
        state: &mut FrameState,
        scene: &mut Scene<WebClock>,
    ) -> Result<(), Error> {
        let collected_entities = self.entities_collector.collect_entities(state, scene);

        unsafe {
            self.picking.draw(state, &collected_entities)?;
        }

        Ok(())
    }
}

impl Pipeline for StandardPipeline {
    type State = FrameState;

    type Clock = WebClock;

    type Error = Error;

    fn execute(
        &mut self,
        state: &mut Self::State,
        scene: &mut Scene<Self::Clock>,
    ) -> Result<(), Self::Error> {
        match self.pipeline_shading {
            StandardPipelineShading::Picking => {
                self.picking(state, scene)?;
            }
            _ => {
                self.preparation.prepare(
                    state,
                    scene,
                    &mut self.universal_ubo,
                    &mut self.lights_ubo,
                )?;

                {
                    match self.pipeline_shading {
                        StandardPipelineShading::ForwardShading => {
                            self.forward_shading(state, scene)?;
                        }
                        StandardPipelineShading::DeferredShading => {
                            if state.capabilities().color_buffer_float_supported() {
                                self.deferred_shading(state, scene)?;
                            } else {
                                // fallback to forward shading if color buffer float not supported
                                self.forward_shading(state, scene)?;
                                self.pipeline_shading = StandardPipelineShading::ForwardShading;
                            }
                        }
                        StandardPipelineShading::Picking => unreachable!(),
                    };
                };

                self.cleanup
                    .cleanup(&self.universal_ubo, &self.lights_ubo)?;
            }
        };

        // flushes all commands
        state.gl().flush();

        Ok(())
    }
}
