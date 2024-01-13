pub mod collector;
pub mod composer;
pub mod drawer;
// pub mod gaussian_blur;
// pub mod outlining;
pub mod picking;
pub mod preparation;

use gl_matrix4rust::vec4::Vec4;

use crate::{
    render::{GraphPipeline, ItemKey, Pipeline, ResourceKey},
    scene::Scene,
};

use self::{
    collector::{StandardEntitiesCollector, DEFAULT_CULLING_ENABLED, DEFAULT_SORTING_ENABLED},
    composer::{StandardComposer, DEFAULT_CLEAR_COLOR},
    drawer::{
        HdrToneMappingType, StandardDrawer, DEFAULT_BLOOM_BLUR_EPOCH, DEFAULT_BLOOM_ENABLED,
        DEFAULT_HDR_ENABLED, DEFAULT_HDR_TONE_MAPPING_TYPE, DEFAULT_MULTISAMPLE,
    },
    preparation::StandardPreparation,
};

use super::{error::Error, state::FrameState};

pub struct StandardPipeline {
    pipeline: GraphPipeline<FrameState, Error>,
    enable_culling_key: ResourceKey<bool>,
    enable_sorting_key: ResourceKey<bool>,
    clear_color_key: ResourceKey<Vec4>,
    multisample_key: ResourceKey<i32>,
    hdr_key: ResourceKey<bool>,
    hdr_tone_mapping_type_key: ResourceKey<HdrToneMappingType>,
    bloom_key: ResourceKey<bool>,
    bloom_epoch_key: ResourceKey<usize>,
}

impl StandardPipeline {
    /// Returns `true` if entity culling enabled.
    pub fn culling_enabled(&self) -> bool {
        self.pipeline
            .resources()
            .get(&self.enable_culling_key)
            .copied()
            .unwrap_or(DEFAULT_CULLING_ENABLED)
    }

    pub fn enable_culling(&mut self) {
        self.pipeline
            .resources_mut()
            .insert(self.enable_culling_key.clone(), true);
    }

    pub fn disable_culling(&mut self) {
        self.pipeline
            .resources_mut()
            .insert(self.enable_culling_key.clone(), false);
    }

    /// Returns `true` if entity distance sorting enabled.
    pub fn distance_sorting_enabled(&self) -> bool {
        self.pipeline
            .resources()
            .get(&self.enable_sorting_key)
            .copied()
            .unwrap_or(DEFAULT_SORTING_ENABLED)
    }

    pub fn enable_distance_sorting(&mut self) {
        self.pipeline
            .resources_mut()
            .insert(self.enable_sorting_key.clone(), true);
    }

    pub fn disable_distance_sorting(&mut self) {
        self.pipeline
            .resources_mut()
            .insert(self.enable_sorting_key.clone(), false);
    }

    pub fn clear_color(&self) -> Vec4 {
        self.pipeline
            .resources()
            .get(&self.clear_color_key)
            .cloned()
            .unwrap_or(DEFAULT_CLEAR_COLOR)
    }

    pub fn set_clear_color(&mut self, clear_color: Vec4) {
        self.pipeline
            .resources_mut()
            .insert(self.clear_color_key.clone(), clear_color);
    }

    pub fn multisample(&self) -> Option<i32> {
        match self.pipeline.resources().get(&self.multisample_key) {
            Some(samples) => {
                if *samples == 0 {
                    None
                } else {
                    Some(*samples)
                }
            }
            None => Some(DEFAULT_MULTISAMPLE),
        }
    }

    pub fn set_multisample(&mut self, samples: Option<i32>) {
        match samples {
            Some(samples) => self
                .pipeline
                .resources_mut()
                .insert(self.multisample_key.clone(), samples),
            None => {
                self.pipeline
                    .resources_mut()
                    .remove_unchecked(&self.multisample_key);
            }
        };
    }

    pub fn hdr_enabled(&self) -> bool {
        self.pipeline
            .resources()
            .get(&self.hdr_key)
            .cloned()
            .unwrap_or(DEFAULT_HDR_ENABLED)
    }

    pub fn enable_hdr(&mut self) {
        self.pipeline
            .resources_mut()
            .insert(self.hdr_key.clone(), true);
    }

    pub fn disable_hdr(&mut self) {
        self.pipeline
            .resources_mut()
            .insert(self.hdr_key.clone(), false);
    }

    pub fn hdr_tone_mapping_type(&self) -> HdrToneMappingType {
        self.pipeline
            .resources()
            .get(&self.hdr_tone_mapping_type_key)
            .cloned()
            .unwrap_or(DEFAULT_HDR_TONE_MAPPING_TYPE)
    }

    pub fn set_hdr_tone_mapping_type(&mut self, hdr_tone_mapping_type: HdrToneMappingType) {
        self.pipeline.resources_mut().insert(
            self.hdr_tone_mapping_type_key.clone(),
            hdr_tone_mapping_type,
        );
    }

    pub fn bloom_enabled(&self) -> bool {
        self.pipeline
            .resources()
            .get(&self.bloom_key)
            .cloned()
            .unwrap_or(DEFAULT_BLOOM_ENABLED)
    }

    pub fn enable_bloom(&mut self) {
        self.pipeline
            .resources_mut()
            .insert(self.bloom_key.clone(), true);
    }

    pub fn disable_bloom(&mut self) {
        self.pipeline
            .resources_mut()
            .insert(self.bloom_key.clone(), false);
    }

    pub fn bloom_blur_epoch(&self) -> usize {
        self.pipeline
            .resources()
            .get(&self.bloom_epoch_key)
            .cloned()
            .unwrap_or(DEFAULT_BLOOM_BLUR_EPOCH)
    }

    pub fn set_bloom_blur_epoch(&mut self, epoch: usize) {
        self.pipeline
            .resources_mut()
            .insert(self.bloom_epoch_key.clone(), epoch);
    }
}

impl StandardPipeline {
    pub fn new() -> Self {
        let preparation_key = ItemKey::new_uuid();
        let collector_key = ItemKey::new_uuid();
        let drawer_key = ItemKey::new_uuid();
        let composer_key = ItemKey::new_uuid();

        let enable_culling_key = ResourceKey::new_persist_uuid();
        let enable_sorting_key = ResourceKey::new_persist_uuid();
        let clear_color_key = ResourceKey::new_persist_uuid();
        let multisample_key = ResourceKey::new_persist_uuid();
        let bloom_key = ResourceKey::new_persist_uuid();
        let bloom_epoch_key = ResourceKey::new_persist_uuid();
        let hdr_key = ResourceKey::new_persist_uuid();
        let hdr_tone_mapping_type_key = ResourceKey::new_persist_uuid();
        let collected_entities_key = ResourceKey::new_runtime_uuid();
        let standard_draw_texture_key = ResourceKey::new_runtime_uuid();

        let mut pipeline = GraphPipeline::new();
        pipeline.add_executor(preparation_key.clone(), StandardPreparation::new());
        pipeline.add_executor(
            collector_key.clone(),
            StandardEntitiesCollector::new(
                collected_entities_key.clone(),
                Some(enable_culling_key.clone()),
                Some(enable_sorting_key.clone()),
            ),
        );
        pipeline.add_executor(
            drawer_key.clone(),
            StandardDrawer::new(
                collected_entities_key,
                standard_draw_texture_key.clone(),
                Some(multisample_key.clone()),
                Some(hdr_key.clone()),
                Some(hdr_tone_mapping_type_key.clone()),
                Some(bloom_key.clone()),
                Some(bloom_epoch_key.clone()),
            ),
        );
        pipeline.add_executor(
            composer_key.clone(),
            StandardComposer::new(
                vec![standard_draw_texture_key],
                Some(clear_color_key.clone()),
            ),
        );

        // safely unwraps
        pipeline.connect(&collector_key, &preparation_key).unwrap();
        pipeline.connect(&preparation_key, &drawer_key).unwrap();
        pipeline.connect(&drawer_key, &composer_key).unwrap();

        Self {
            pipeline,
            enable_culling_key,
            enable_sorting_key,
            clear_color_key,
            multisample_key,
            hdr_key,
            hdr_tone_mapping_type_key,
            bloom_key,
            bloom_epoch_key,
        }
    }
}

impl Pipeline for StandardPipeline {
    type State = FrameState;

    type Error = Error;

    fn execute(&mut self, state: &mut Self::State, scene: &mut Scene) -> Result<(), Self::Error> {
        self.pipeline.execute(state, scene)
    }
}
