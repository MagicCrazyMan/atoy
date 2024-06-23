use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

use uuid::Uuid;

use super::{
    carrier::{Carrier, Listener},
    clock::Clock,
    ecs::archetype::Archetype,
    engine::{RenderContext, RenderEngine},
    resource::Resources,
    runner::{Job, Runner},
    scene::Scene,
    Rrc,
};

pub struct Initialize {
    pub context: AppContext,
}

pub struct PreRender {
    pub context: AppContext,
}

pub struct PostRender {
    pub context: AppContext,
}

pub struct Tick {
    pub context: AppContext,
    pub start_time: f64,
    pub previous_time: f64,
    pub current_time: f64,
    pub delta_time: f64,
}

pub struct AddEntity {
    pub context: AppContext,
    pub entity_id: Uuid,
}

pub struct RemoveEntity {
    pub context: AppContext,
    pub entity_id: Uuid,
}

pub struct UpdateComponent {
    pub context: AppContext,
    pub entity_id: Uuid,
}

pub struct AddComponent {
    pub context: AppContext,
    pub entity_id: Uuid,
    pub old_archetype: Archetype,
    pub new_archetype: Archetype,
}

pub struct RemoveComponent {
    pub context: AppContext,
    pub entity_id: Uuid,
    pub old_archetype: Archetype,
    pub new_archetype: Archetype,
}

pub struct ReplaceComponent {
    pub context: AppContext,
    pub entity_id: Uuid,
}

pub struct AppConfig {
    initialize: Carrier<Initialize>,
    pre_render: Carrier<PreRender>,
    post_render: Carrier<PostRender>,
    add_entity: Carrier<AddEntity>,
    remove_entity: Carrier<RemoveEntity>,
    update_component: Carrier<UpdateComponent>,
    add_component: Carrier<AddComponent>,
    remove_component: Carrier<RemoveComponent>,
    replace_component: Carrier<ReplaceComponent>,
    tick: Carrier<Tick>,

    resources: Resources,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            initialize: Carrier::new(),
            pre_render: Carrier::new(),
            post_render: Carrier::new(),
            tick: Carrier::new(),
            add_entity: Carrier::new(),
            remove_entity: Carrier::new(),
            update_component: Carrier::new(),
            add_component: Carrier::new(),
            remove_component: Carrier::new(),
            replace_component: Carrier::new(),

            resources: Resources::new(),
        }
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }

    pub fn on_initialize(&self) -> &Carrier<Initialize> {
        &self.initialize
    }

    pub fn on_pre_render(&self) -> &Carrier<PreRender> {
        &self.pre_render
    }

    pub fn on_post_render(&self) -> &Carrier<PostRender> {
        &self.post_render
    }

    pub fn on_tick(&self) -> &Carrier<Tick> {
        &self.tick
    }

    pub fn on_add_entity(&self) -> &Carrier<AddEntity> {
        &self.add_entity
    }

    pub fn on_remove_entity(&self) -> &Carrier<RemoveEntity> {
        &self.remove_entity
    }

    pub fn on_update_component(&self) -> &Carrier<UpdateComponent> {
        &self.update_component
    }

    pub fn on_add_component(&self) -> &Carrier<AddComponent> {
        &self.add_component
    }

    pub fn on_remove_component(&self) -> &Carrier<RemoveComponent> {
        &self.remove_component
    }

    pub fn on_replace_component(&self) -> &Carrier<ReplaceComponent> {
        &self.replace_component
    }
}

#[derive(Clone)]
pub struct AppContext {
    pub scene: Rrc<Scene>,
    pub clock: Rrc<dyn Clock>,
    pub engine: Rrc<dyn RenderEngine>,
    pub resources: Rrc<Resources>,
}

pub struct App {
    scene: Rrc<Scene>,
    clock: Rrc<dyn Clock>,
    engine: Rrc<dyn RenderEngine>,
    resources: Rrc<Resources>,
    runner: Box<dyn Runner>,

    pre_render: Carrier<PreRender>,
    post_render: Carrier<PostRender>,
    tick: Carrier<Tick>,
    add_entity: Carrier<AddEntity>,
    remove_entity: Carrier<RemoveEntity>,
    update_component: Carrier<UpdateComponent>,
    add_component: Carrier<AddComponent>,
    remove_component: Carrier<RemoveComponent>,
    replace_component: Carrier<ReplaceComponent>,
}

impl App {
    pub fn new<CLK, RE, R>(
        app_config: AppConfig,
        scene: Scene,
        clock: CLK,
        engine: RE,
        runner: R,
    ) -> Self
    where
        CLK: Clock + 'static,
        RE: RenderEngine + 'static,
        R: Runner + 'static,
    {
        let app: App = Self {
            scene: Rc::new(RefCell::new(scene)),
            clock: Rc::new(RefCell::new(clock)),
            engine: Rc::new(RefCell::new(engine)),
            runner: Box::new(runner),

            resources: Rc::new(RefCell::new(app_config.resources)),

            pre_render: app_config.pre_render,
            post_render: app_config.post_render,
            tick: app_config.tick,
            add_entity: app_config.add_entity,
            remove_entity: app_config.remove_entity,
            update_component: app_config.update_component,
            add_component: app_config.add_component,
            remove_component: app_config.remove_component,
            replace_component: app_config.replace_component,
        };

        app_config.initialize.send(&mut Initialize {
            context: app.context(),
        });

        app.clock
            .borrow()
            .on_tick()
            .register(ClockListener::new(&app));
        let entity_manager = Ref::map(app.scene.borrow(), |scene| scene.entity_manager());
        // entity_manager
        //     .on_add_entity()
        //     .register(AddEntityListener::new(&app));
        // entity_manager
        //     .on_remove_entity()
        //     .register(RemoveEntityListener::new(&app));
        // entity_manager
        //     .on_update_component()
        //     .register(UpdateComponentListener::new(&app));
        // entity_manager
        //     .on_add_component()
        //     .register(AddComponentListener::new(&app));
        // entity_manager
        //     .on_remove_component()
        //     .register(RemoveComponentListener::new(&app));
        // entity_manager
        //     .on_replace_component()
        //     .register(ReplaceComponentListener::new(&app));
        drop(entity_manager);

        // app.tick.register(TestSystem);
        // app.pre_render.register(TestSystem);

        app
    }

    pub fn scene(&self) -> &Rrc<Scene> {
        &self.scene
    }

    pub fn clock(&self) -> &Rrc<dyn Clock> {
        &self.clock
    }

    pub fn engine(&self) -> &Rrc<dyn RenderEngine> {
        &self.engine
    }

    pub fn resources(&self) -> &Rrc<Resources> {
        &self.resources
    }

    pub fn context(&self) -> AppContext {
        AppContext {
            scene: Rc::clone(&self.scene),
            clock: Rc::clone(&self.clock),
            engine: Rc::clone(&self.engine),
            resources: Rc::clone(&self.resources),
        }
    }

    pub fn on_pre_render(&self) -> &Carrier<PreRender> {
        &self.pre_render
    }

    pub fn on_post_render(&self) -> &Carrier<PostRender> {
        &self.post_render
    }

    pub fn on_tick(&self) -> &Carrier<Tick> {
        &self.tick
    }

    pub fn on_add_entity(&self) -> &Carrier<AddEntity> {
        &self.add_entity
    }

    pub fn on_remove_entity(&self) -> &Carrier<RemoveEntity> {
        &self.remove_entity
    }

    pub fn on_update_component(&self) -> &Carrier<UpdateComponent> {
        &self.update_component
    }

    pub fn on_add_component(&self) -> &Carrier<AddComponent> {
        &self.add_component
    }

    pub fn on_remove_component(&self) -> &Carrier<RemoveComponent> {
        &self.remove_component
    }

    pub fn on_replace_component(&self) -> &Carrier<ReplaceComponent> {
        &self.replace_component
    }
}

impl App {
    pub fn start(&mut self) {
        self.runner.start(Box::new(AppJob::new(self)));
    }

    pub fn stop(&mut self) {
        self.runner.stop();
    }
}

struct AppJob {
    context: AppContext,
    pre_render: Carrier<PreRender>,
    post_render: Carrier<PostRender>,
}

impl AppJob {
    fn new(app: &App) -> Self {
        Self {
            context: app.context(),
            pre_render: app.pre_render.clone(),
            post_render: app.post_render.clone(),
        }
    }
}

impl Job for AppJob {
    fn execute(&mut self) {
        self.pre_render.send(&mut PreRender {
            context: self.context.clone(),
        });

        self.context.engine.borrow_mut().render(RenderContext {
            scene: Rc::clone(&self.context.scene),
            clock: Rc::clone(&self.context.clock),
            resources: Rc::clone(&self.context.resources),
        });

        self.post_render.send(&mut PostRender {
            context: self.context.clone(),
        });
    }
}

struct ClockListener {
    context: AppContext,
    tick: Carrier<Tick>,
}

impl ClockListener {
    fn new(app: &App) -> Self {
        Self {
            context: app.context(),
            tick: app.tick.clone(),
        }
    }
}

impl Listener<super::clock::Tick> for ClockListener {
    fn execute(&mut self, tick: &mut super::clock::Tick) {
        self.tick.send(&mut Tick {
            context: self.context.clone(),
            start_time: tick.start_time,
            previous_time: tick.previous_time,
            current_time: tick.current_time,
            delta_time: tick.delta_time,
        });
    }
}

struct AddEntityListener {
    context: AppContext,
    add_entity: Carrier<AddEntity>,
}

impl AddEntityListener {
    fn new(app: &App) -> Self {
        Self {
            context: app.context(),
            add_entity: app.add_entity.clone(),
        }
    }
}

impl Listener<super::ecs::manager::AddEntity> for AddEntityListener {
    fn execute(&mut self, payload: &mut super::ecs::manager::AddEntity) {
        self.add_entity.send(&mut AddEntity {
            context: self.context.clone(),
            entity_id: payload.entity_id,
        })
    }
}

struct RemoveEntityListener {
    context: AppContext,
    remove_entity: Carrier<RemoveEntity>,
}

impl RemoveEntityListener {
    fn new(app: &App) -> Self {
        Self {
            context: app.context(),
            remove_entity: app.remove_entity.clone(),
        }
    }
}

impl Listener<super::ecs::manager::RemoveEntity> for RemoveEntityListener {
    fn execute(&mut self, payload: &mut super::ecs::manager::RemoveEntity) {
        self.remove_entity.send(&mut RemoveEntity {
            context: self.context.clone(),
            entity_id: payload.entity_id,
        })
    }
}

struct UpdateComponentListener {
    context: AppContext,
    update_component: Carrier<UpdateComponent>,
}

impl UpdateComponentListener {
    fn new(app: &App) -> Self {
        Self {
            context: app.context(),
            update_component: app.update_component.clone(),
        }
    }
}

impl Listener<super::ecs::manager::UpdateComponent> for UpdateComponentListener {
    fn execute(&mut self, payload: &mut super::ecs::manager::UpdateComponent) {
        self.update_component.send(&mut UpdateComponent {
            context: self.context.clone(),
            entity_id: payload.entity_id,
        })
    }
}

struct AddComponentListener {
    context: AppContext,
    add_component: Carrier<AddComponent>,
}

impl AddComponentListener {
    fn new(app: &App) -> Self {
        Self {
            context: app.context(),
            add_component: app.add_component.clone(),
        }
    }
}

impl Listener<super::ecs::manager::AddComponent> for AddComponentListener {
    fn execute(&mut self, payload: &mut super::ecs::manager::AddComponent) {
        self.add_component.send(&mut AddComponent {
            context: self.context.clone(),
            entity_id: payload.entity_id,
            old_archetype: payload.old_archetype.clone(),
            new_archetype: payload.new_archetype.clone(),
        })
    }
}

struct RemoveComponentListener {
    context: AppContext,
    remove_component: Carrier<RemoveComponent>,
}

impl RemoveComponentListener {
    fn new(app: &App) -> Self {
        Self {
            context: app.context(),
            remove_component: app.remove_component.clone(),
        }
    }
}

impl Listener<super::ecs::manager::RemoveComponent> for RemoveComponentListener {
    fn execute(&mut self, payload: &mut super::ecs::manager::RemoveComponent) {
        self.remove_component.send(&mut RemoveComponent {
            context: self.context.clone(),
            entity_id: payload.entity_id,
            old_archetype: payload.old_archetype.clone(),
            new_archetype: payload.new_archetype.clone(),
        })
    }
}

struct ReplaceComponentListener {
    context: AppContext,
    replace_component: Carrier<ReplaceComponent>,
}

impl ReplaceComponentListener {
    fn new(app: &App) -> Self {
        Self {
            context: app.context(),
            replace_component: app.replace_component.clone(),
        }
    }
}

impl Listener<super::ecs::manager::ReplaceComponent> for ReplaceComponentListener {
    fn execute(&mut self, payload: &mut super::ecs::manager::ReplaceComponent) {
        self.replace_component.send(&mut ReplaceComponent {
            context: self.context.clone(),
            entity_id: payload.entity_id,
        })
    }
}

// struct TestSystem;

// #[derive(AsAny, Component)]
// struct A;

// #[derive(AsAny, Component)]
// struct B;

// #[derive(AsAny, Component)]
// struct C;

// impl System<tick> for TestSystem {
//     type Query = (A, B);

//     fn execute(
//         &mut self,
//         tick {
//             tick,
//             scene,
//             clock,
//             engine,
//             resources,
//         }: &mut tick,
//     ) {
//         let mut binding = scene.borrow_mut();
//         let entity = binding
//             .entity_manager_mut()
//             .entities_of_archetype::<Self::Query>();
//     }
// }

// impl System<PreRender> for TestSystem {
//     type Query = (A, B, C);

//     fn execute(&mut self, message: &mut PreRender) {
//         todo!()
//     }
// }
