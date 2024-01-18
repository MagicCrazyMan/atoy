pub mod cleanup;
pub mod collector;
pub mod composer;
pub mod drawer;
pub mod picking;
pub mod preparation;

use gl_matrix4rust::vec4::Vec4;
use web_sys::WebGl2RenderingContext;

use crate::{render::Pipeline, scene::Scene};

use self::{
    cleanup::StandardCleanup,
    collector::StandardEntitiesCollector,
    composer::StandardComposer,
    drawer::{
        hdr::StandardHdrDrawer, hdr_multisamples::StandardMultisamplesHdrDrawer,
        simple::StandardSimpleDrawer, simple_multisamples::StandardMultisamplesSimpleDrawer,
        HdrToneMappingType,
    },
    preparation::StandardPreparation,
};

use super::{
    buffer::{BufferDescriptor, BufferSource, BufferUsage, MemoryPolicy},
    error::Error,
    state::FrameState,
    uniform::{UBO_LIGHTS_BYTES_LENGTH, UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH},
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

    multisamples: Option<i32>,
    hdr_supported: Option<bool>,
    enable_hdr: bool,
    hdr_tone_mapping_type: HdrToneMappingType,
    enable_bloom: bool,
    bloom_blur_epoch: usize,
}

impl StandardPipeline {
    pub fn new() -> Self {
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

            multisamples: Some(DEFAULT_MULTISAMPLES),
            hdr_supported: None,
            enable_hdr: DEFAULT_HDR_ENABLED,
            hdr_tone_mapping_type: DEFAULT_HDR_TONE_MAPPING_TYPE,
            enable_bloom: DEFAULT_BLOOM_ENABLED,
            bloom_blur_epoch: DEFAULT_BLOOM_BLUR_EPOCH,
        }
    }

    #[inline]
    pub fn clear_color(&self) -> &Vec4 {
        self.composer.clear_color()
    }

    #[inline]
    pub fn set_clear_color(&mut self, clear_color: Vec4) {
        self.composer.set_clear_color(clear_color);
    }

    pub fn hdr_supported(&mut self, gl: &WebGl2RenderingContext) -> bool {
        if let Some(hdr_supported) = self.hdr_supported {
            return hdr_supported;
        }

        let supported = gl
            .get_extension("EXT_color_buffer_float")
            .map(|extension| extension.is_some())
            .unwrap_or(false);
        self.hdr_supported = Some(supported);
        supported
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
    }

    /// Disables culling by bounding volumes.
    #[inline]
    pub fn disable_culling(&mut self) {
        self.entities_collector.disable_culling();
    }

    /// Returns `true` if entity distance sorting enabled.
    #[inline]
    pub fn distance_sorting_enabled(&self) -> bool {
        self.entities_collector.distance_sorting_enabled()
    }

    /// Enables distance sorting by bounding volumes.
    #[inline]
    pub fn enable_distance_sorting(&mut self) {
        self.entities_collector.enable_distance_sorting()
    }

    /// Disables distance sorting by bounding volumes.
    #[inline]
    pub fn disable_distance_sorting(&mut self) {
        self.entities_collector.disable_distance_sorting()
    }

    #[inline]
    pub fn hdr_enabled(&self) -> bool {
        self.enable_hdr
    }

    #[inline]
    pub fn enable_hdr(&mut self) {
        self.enable_hdr = true;
    }

    #[inline]
    pub fn disable_hdr(&mut self) {
        self.enable_hdr = false;
    }

    #[inline]
    pub fn hdr_tone_mapping_type(&self) -> HdrToneMappingType {
        self.hdr_tone_mapping_type
    }

    #[inline]
    pub fn set_hdr_tone_mapping_type(&mut self, tone_mapping_type: HdrToneMappingType) {
        self.hdr_tone_mapping_type = tone_mapping_type;
    }

    #[inline]
    pub fn bloom_enabled(&self) -> bool {
        self.enable_bloom
    }

    #[inline]
    pub fn enable_bloom(&mut self) {
        self.enable_bloom = true;
    }

    #[inline]
    pub fn disable_bloom(&mut self) {
        self.enable_bloom = false;
    }

    #[inline]
    pub fn bloom_blur_epoch(&self) -> usize {
        self.bloom_blur_epoch
    }

    #[inline]
    pub fn set_bloom_blur_epoch(&mut self, epoch: usize) {
        self.bloom_blur_epoch = epoch;
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
    }
}

impl Pipeline for StandardPipeline {
    type State = FrameState;

    type Error = Error;

    fn execute(&mut self, state: &mut Self::State, scene: &mut Scene) -> Result<(), Self::Error> {
        let hdr = self.hdr_enabled() && self.hdr_supported(state.gl());
        let bloom_blur = self.bloom_enabled();
        let bloom_blur_epoch = self.bloom_blur_epoch();
        let multisamples = self.multisamples();

        self.preparation
            .prepare(state, scene, &mut self.universal_ubo, &mut self.lights_ubo)?;

        let collected_entities = self.entities_collector.collect_entities(state, scene);

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

        Ok(())
    }
}

// pub struct StandardPipeline {
//     pipeline: GraphPipeline<FrameState, Error>,
//     enable_culling_key: ResourceKey<bool>,
//     enable_sorting_key: ResourceKey<bool>,
//     clear_color_key: ResourceKey<Vec4>,
//     multisample_key: ResourceKey<i32>,
//     hdr_key: ResourceKey<bool>,
//     hdr_tone_mapping_type_key: ResourceKey<HdrToneMappingType>,
//     bloom_key: ResourceKey<bool>,
//     bloom_epoch_key: ResourceKey<usize>,
// }

// impl StandardPipeline {
//     /// Returns `true` if entity culling enabled.
//     pub fn culling_enabled(&self) -> bool {
//         self.pipeline
//             .resources()
//             .get(&self.enable_culling_key)
//             .copied()
//             .unwrap_or(DEFAULT_CULLING_ENABLED)
//     }

//     pub fn enable_culling(&mut self) {
//         self.pipeline
//             .resources_mut()
//             .insert(self.enable_culling_key.clone(), true);
//     }

//     pub fn disable_culling(&mut self) {
//         self.pipeline
//             .resources_mut()
//             .insert(self.enable_culling_key.clone(), false);
//     }

//     /// Returns `true` if entity distance sorting enabled.
//     pub fn distance_sorting_enabled(&self) -> bool {
//         self.pipeline
//             .resources()
//             .get(&self.enable_sorting_key)
//             .copied()
//             .unwrap_or(DEFAULT_SORTING_ENABLED)
//     }

//     pub fn enable_distance_sorting(&mut self) {
//         self.pipeline
//             .resources_mut()
//             .insert(self.enable_sorting_key.clone(), true);
//     }

//     pub fn disable_distance_sorting(&mut self) {
//         self.pipeline
//             .resources_mut()
//             .insert(self.enable_sorting_key.clone(), false);
//     }

//     pub fn clear_color(&self) -> Vec4 {
//         self.pipeline
//             .resources()
//             .get(&self.clear_color_key)
//             .cloned()
//             .unwrap_or(DEFAULT_CLEAR_COLOR)
//     }

//     pub fn set_clear_color(&mut self, clear_color: Vec4) {
//         self.pipeline
//             .resources_mut()
//             .insert(self.clear_color_key.clone(), clear_color);
//     }

//     pub fn multisample(&self) -> Option<i32> {
//         match self.pipeline.resources().get(&self.multisample_key) {
//             Some(samples) => {
//                 if *samples == 0 {
//                     None
//                 } else {
//                     Some(*samples)
//                 }
//             }
//             None => Some(DEFAULT_MULTISAMPLE),
//         }
//     }

//     pub fn set_multisample(&mut self, samples: Option<i32>) {
//         match samples {
//             Some(samples) => self
//                 .pipeline
//                 .resources_mut()
//                 .insert(self.multisample_key.clone(), samples),
//             None => {
//                 self.pipeline
//                     .resources_mut()
//                     .remove_unchecked(&self.multisample_key);
//             }
//         };
//     }

//     pub fn hdr_enabled(&self) -> bool {
//         self.pipeline
//             .resources()
//             .get(&self.hdr_key)
//             .cloned()
//             .unwrap_or(DEFAULT_HDR_ENABLED)
//     }

//     pub fn enable_hdr(&mut self) {
//         self.pipeline
//             .resources_mut()
//             .insert(self.hdr_key.clone(), true);
//     }

//     pub fn disable_hdr(&mut self) {
//         self.pipeline
//             .resources_mut()
//             .insert(self.hdr_key.clone(), false);
//     }

//     pub fn hdr_tone_mapping_type(&self) -> HdrToneMappingType {
//         self.pipeline
//             .resources()
//             .get(&self.hdr_tone_mapping_type_key)
//             .cloned()
//             .unwrap_or(DEFAULT_HDR_TONE_MAPPING_TYPE)
//     }

//     pub fn set_hdr_tone_mapping_type(&mut self, hdr_tone_mapping_type: HdrToneMappingType) {
//         self.pipeline.resources_mut().insert(
//             self.hdr_tone_mapping_type_key.clone(),
//             hdr_tone_mapping_type,
//         );
//     }

//     pub fn bloom_enabled(&self) -> bool {
//         self.pipeline
//             .resources()
//             .get(&self.bloom_key)
//             .cloned()
//             .unwrap_or(DEFAULT_BLOOM_ENABLED)
//     }

//     pub fn enable_bloom(&mut self) {
//         self.pipeline
//             .resources_mut()
//             .insert(self.bloom_key.clone(), true);
//     }

//     pub fn disable_bloom(&mut self) {
//         self.pipeline
//             .resources_mut()
//             .insert(self.bloom_key.clone(), false);
//     }

//     pub fn bloom_blur_epoch(&self) -> usize {
//         self.pipeline
//             .resources()
//             .get(&self.bloom_epoch_key)
//             .cloned()
//             .unwrap_or(DEFAULT_BLOOM_BLUR_EPOCH)
//     }

//     pub fn set_bloom_blur_epoch(&mut self, epoch: usize) {
//         self.pipeline
//             .resources_mut()
//             .insert(self.bloom_epoch_key.clone(), epoch);
//     }
// }

// impl StandardPipeline {
//     pub fn new() -> Self {
//         let preparation_key = ItemKey::new_uuid();
//         let collector_key = ItemKey::new_uuid();
//         let drawer_key = ItemKey::new_uuid();
//         let composer_key = ItemKey::new_uuid();

//         let enable_culling_key = ResourceKey::new_persist_uuid();
//         let enable_sorting_key = ResourceKey::new_persist_uuid();
//         let clear_color_key = ResourceKey::new_persist_uuid();
//         let multisample_key = ResourceKey::new_persist_uuid();
//         let bloom_key = ResourceKey::new_persist_uuid();
//         let bloom_epoch_key = ResourceKey::new_persist_uuid();
//         let hdr_key = ResourceKey::new_persist_uuid();
//         let hdr_tone_mapping_type_key = ResourceKey::new_persist_uuid();
//         let collected_entities_key = ResourceKey::new_runtime_uuid();
//         let standard_draw_texture_key = ResourceKey::new_runtime_uuid();

//         let mut pipeline = GraphPipeline::new();
//         pipeline.add_executor(preparation_key.clone(), StandardPreparation::new());
//         pipeline.add_executor(
//             collector_key.clone(),
//             StandardEntitiesCollector::new(
//                 collected_entities_key.clone(),
//                 Some(enable_culling_key.clone()),
//                 Some(enable_sorting_key.clone()),
//             ),
//         );
//         pipeline.add_executor(
//             drawer_key.clone(),
//             StandardDrawer::new(
//                 collected_entities_key,
//                 standard_draw_texture_key.clone(),
//                 Some(multisample_key.clone()),
//                 Some(hdr_key.clone()),
//                 Some(hdr_tone_mapping_type_key.clone()),
//                 Some(bloom_key.clone()),
//                 Some(bloom_epoch_key.clone()),
//             ),
//         );
//         pipeline.add_executor(
//             composer_key.clone(),
//             StandardComposer::new(
//                 vec![standard_draw_texture_key],
//                 Some(clear_color_key.clone()),
//             ),
//         );

//         // safely unwraps
//         pipeline.connect(&collector_key, &preparation_key).unwrap();
//         pipeline.connect(&preparation_key, &drawer_key).unwrap();
//         pipeline.connect(&drawer_key, &composer_key).unwrap();

//         Self {
//             pipeline,
//             enable_culling_key,
//             enable_sorting_key,
//             clear_color_key,
//             multisample_key,
//             hdr_key,
//             hdr_tone_mapping_type_key,
//             bloom_key,
//             bloom_epoch_key,
//         }
//     }
// }

// impl Pipeline for StandardPipeline {
//     type State = FrameState;

//     type Error = Error;

//     fn execute(&mut self, state: &mut Self::State, scene: &mut Scene) -> Result<(), Self::Error> {
//         self.pipeline.execute(state, scene)
//     }
// }
