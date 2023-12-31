pub mod collector;
// pub mod composer;
pub mod drawer;
// pub mod gaussian_blur;
// pub mod outlining;
pub mod picking;

use crate::{
    render::pp::{GraphPipeline, ItemKey, Pipeline, ResourceKey, State},
    scene::Scene,
};

use self::{collector::StandardEntitiesCollector, drawer::StandardDrawer};

use super::error::Error;

pub struct StandardPipeline {
    pipeline: GraphPipeline<Error>
}

impl StandardPipeline {
    pub fn new() -> Self {
        let collector = ItemKey::new_uuid();
        // let picking = ItemKey::new_uuid();
        // let outlining = ItemKey::new_uuid();
        // let gaussian_blur = ItemKey::new_uuid();
        let drawer = ItemKey::new_uuid();
        // let composer = ItemKey::new_uuid();
    
        let collected_entities = ResourceKey::new_runtime_uuid();
        // let picked_entity = ResourceKey::new_runtime_uuid();
        // let picked_position = ResourceKey::new_runtime_uuid();
        // let outline_texture = ResourceKey::new_runtime_uuid();
        let standard_draw_texture = ResourceKey::new_runtime_uuid();
        // let gaussian_blur_texture = ResourceKey::new_runtime_uuid();
    
        let mut pipeline = GraphPipeline::new();
        pipeline.add_executor(
            collector.clone(),
            StandardEntitiesCollector::new(collected_entities.clone()),
        );
        // pipeline.add_executor(
        //     picking.clone(),
        //     Picking::new(
        //         collected_entities.clone(),
        //         in_window_position,
        //         picked_entity.clone(),
        //         picked_position.clone(),
        //     ),
        // );
        // pipeline.add_executor(
        //     outlining.clone(),
        //     Outlining::new(picked_entity, outline_texture.clone()),
        // );
        // pipeline.add_executor(
        //     gaussian_blur.clone(),
        //     GaussianBlur::new(outline_texture, gaussian_blur_texture.clone()),
        // );
        pipeline.add_executor(
            drawer.clone(),
            StandardDrawer::new(collected_entities, standard_draw_texture.clone()),
        );
        // pipeline.add_executor(
        //     composer.clone(),
        //     StandardComposer::new(
        //         vec![standard_draw_texture, gaussian_blur_texture],
        //         in_clear_color,
        //     ),
        // );
    
        // safely unwraps
        // pipeline.connect(&collector, &picking).unwrap();
        // pipeline.connect(&picking, &outlining).unwrap();
        // pipeline.connect(&outlining, &gaussian_blur).unwrap();
        // pipeline.connect(&gaussian_blur, &composer).unwrap();
        pipeline.connect(&collector, &drawer).unwrap();
        // pipeline.connect(&drawer, &composer).unwrap();
    
        Self {
            pipeline
        }
    }
}

impl Pipeline for StandardPipeline {
    type Error = Error;

    fn execute(&mut self, state: &mut State, scene: &mut Scene) -> Result<(), Self::Error> {
        self.pipeline.execute(state, scene)
    }
}
