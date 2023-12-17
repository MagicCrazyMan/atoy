pub mod collector;
pub mod composer;
pub mod drawer;
// pub mod outlining;
pub mod picking;

use crate::render::pp::{ItemKey, Pipeline, ResourceKey};

use self::{
    collector::StandardEntitiesCollector, composer::StandardComposer, drawer::StandardDrawer,
};

use super::error::Error;

pub fn create_standard_pipeline(window_position: ResourceKey<(i32, i32)>) -> Pipeline<Error> {
    let collector = ItemKey::from_uuid();
    // let picking = ItemKey::from_uuid();
    // let outlining = ItemKey::from_uuid();
    let drawer = ItemKey::from_uuid();
    let composer = ItemKey::from_uuid();

    let collected_entities = ResourceKey::runtime_uuid();
    let standard_draw_texture = ResourceKey::runtime_uuid();
    // let picked_entity = ResourceKey::runtime_uuid();
    // let picked_position = ResourceKey::runtime_uuid();

    let mut pipeline = Pipeline::new();
    pipeline.add_executor(
        collector.clone(),
        StandardEntitiesCollector::new(collected_entities.clone()),
    );
    // pipeline.add_executor(
    //     picking.clone(),
    //     Picking::new(
    //         window_position,
    //         collected_entities.clone(),
    //         picked_entity.clone(),
    //         picked_position.clone(),
    //     ),
    // );
    // pipeline.add_executor(outlining.clone(), Outlining::new(picked_entity));
    pipeline.add_executor(
        drawer.clone(),
        StandardDrawer::new(collected_entities, standard_draw_texture.clone()),
    );
    pipeline.add_executor(
        composer.clone(),
        StandardComposer::new(vec![standard_draw_texture]),
    );

    // safely unwraps
    // pipeline.connect(&collector, &picking).unwrap();
    pipeline.connect(&collector, &drawer).unwrap();
    pipeline.connect(&drawer, &composer).unwrap();
    // pipeline.connect(&picking, &outlining).unwrap();
    // pipeline.connect(&outlining, &drawer).unwrap();

    pipeline
}
