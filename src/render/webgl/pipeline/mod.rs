pub mod cleanup;
pub mod collector;
pub mod composer;
pub mod drawer;
pub mod picking;
pub mod preparation;

use gl_matrix4rust::vec4::Vec4;

use crate::{render::Pipeline, scene::Scene};

use self::{
    cleanup::StandardCleanup,
    collector::StandardEntitiesCollector,
    composer::StandardComposer,
    drawer::{
        hdr::StandardHdrDrawer, hdr_multisamples::StandardMultisamplesHdrDrawer,
        simple::StandardSimpleDrawer, simple_multisamples::StandardMultisamplesSimpleDrawer,
        HdrToneMappingType, UBO_GAUSSIAN_KERNEL_U8, UBO_LIGHTS_BYTES_LENGTH,
        UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH,
    },
    preparation::StandardPreparation,
};

use super::{
    buffer::{BufferDescriptor, BufferSource, BufferUsage, MemoryPolicy},
    error::Error,
    state::FrameState,
};

pub const DEFAULT_MULTISAMPLES: i32 = 4;
pub const DEFAULT_HDR_ENABLED: bool = true;
pub const DEFAULT_HDR_TONE_MAPPING_TYPE: HdrToneMappingType = HdrToneMappingType::Reinhard;
pub const DEFAULT_BLOOM_ENABLED: bool = true;
pub const DEFAULT_BLOOM_BLUR_EPOCH: usize = 10;

pub struct StandardPipeline {
    preparation: StandardPreparation,
    entities_collector: StandardEntitiesCollector,
    simple_drawer: StandardSimpleDrawer,
    multisamples_simple_drawer: StandardMultisamplesSimpleDrawer,
    multisamples_hdr_drawer: StandardMultisamplesHdrDrawer,
    hdr_drawer: StandardHdrDrawer,
    composer: StandardComposer,
    cleanup: StandardCleanup,

    universal_ubo: BufferDescriptor,
    lights_ubo: BufferDescriptor,
    gaussian_kernel_ubo: BufferDescriptor,

    hdr_supported: bool,

    multisamples: Option<i32>,
    enable_hdr: bool,
    hdr_tone_mapping_type: HdrToneMappingType,
    enable_bloom: bool,
    bloom_blur_epoch: usize,

    dirty: bool,
    last_render_collected_id: Option<usize>,
}

impl StandardPipeline {
    pub fn new(hdr_supported: bool) -> Self {
        Self {
            preparation: StandardPreparation::new(),
            entities_collector: StandardEntitiesCollector::new(),
            simple_drawer: StandardSimpleDrawer::new(),
            multisamples_simple_drawer: StandardMultisamplesSimpleDrawer::new(),
            multisamples_hdr_drawer: StandardMultisamplesHdrDrawer::new(),
            hdr_drawer: StandardHdrDrawer::new(),
            composer: StandardComposer::new(),
            cleanup: StandardCleanup::new(),

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

            multisamples: Some(DEFAULT_MULTISAMPLES),
            enable_hdr: DEFAULT_HDR_ENABLED,
            hdr_tone_mapping_type: DEFAULT_HDR_TONE_MAPPING_TYPE,
            enable_bloom: DEFAULT_BLOOM_ENABLED,
            bloom_blur_epoch: DEFAULT_BLOOM_BLUR_EPOCH,

            dirty: true,
            last_render_collected_id: None,
        }
    }

    #[inline]
    fn set_dirty(&mut self) {
        self.dirty = true;
    }

    #[inline]
    pub fn clear_color(&self) -> &Vec4 {
        self.composer.clear_color()
    }

    #[inline]
    pub fn set_clear_color(&mut self, clear_color: Vec4) {
        self.composer.set_clear_color(clear_color);
        self.set_dirty();
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
}

impl Pipeline for StandardPipeline {
    type State = FrameState;

    type Error = Error;

    fn execute(&mut self, state: &mut Self::State, scene: &mut Scene) -> Result<(), Self::Error> {
        let hdr = self.hdr_enabled() && self.hdr_supported;
        let bloom_blur = self.bloom_enabled();
        let bloom_blur_epoch = self.bloom_blur_epoch();
        let multisamples = self.multisamples();

        let collected_entities = self.entities_collector.collect_entities(state, scene);

        // skips render if collect_entities unchanged and pipeline is not dirty
        let dirty = self.dirty
            || self
                .last_render_collected_id
                .map(|id| id != collected_entities.id())
                .unwrap_or(true);
        if !dirty {
            return Ok(());
        }

        self.preparation
            .prepare(state, scene, &mut self.universal_ubo, &mut self.lights_ubo)?;

        let compose_textures = match (hdr, multisamples) {
            (true, None) => {
                self.hdr_drawer.draw(
                    state,
                    bloom_blur,
                    bloom_blur_epoch,
                    self.hdr_tone_mapping_type,
                    &collected_entities,
                    &self.universal_ubo,
                    &self.lights_ubo,
                    &self.gaussian_kernel_ubo,
                )?;
                [self.hdr_drawer.draw_texture().unwrap()]
            }
            (true, Some(samples)) => {
                self.multisamples_hdr_drawer.draw(
                    state,
                    samples,
                    bloom_blur,
                    bloom_blur_epoch,
                    self.hdr_tone_mapping_type,
                    &collected_entities,
                    &self.universal_ubo,
                    &self.lights_ubo,
                    &self.gaussian_kernel_ubo,
                )?;
                [self.multisamples_hdr_drawer.draw_texture().unwrap()]
            }
            (false, None) => {
                self.simple_drawer.draw(
                    state,
                    &collected_entities,
                    &self.universal_ubo,
                    &self.lights_ubo,
                )?;
                [self.multisamples_simple_drawer.draw_texture().unwrap()]
            }
            (false, Some(samples)) => {
                self.multisamples_simple_drawer.draw(
                    state,
                    samples,
                    &collected_entities,
                    &self.universal_ubo,
                    &self.lights_ubo,
                )?;
                [self.multisamples_simple_drawer.draw_texture().unwrap()]
            }
        };
        self.composer.compose(state, compose_textures)?;
        self.cleanup.cleanup(state);

        self.last_render_collected_id = Some(collected_entities.id());
        self.dirty = false;

        Ok(())
    }
}
