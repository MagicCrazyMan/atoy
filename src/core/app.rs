use std::{cell::RefCell, rc::Rc};

use super::{
    carrier::{Carrier, Listener},
    clock::{Clock, Tick},
    ecs::{
        system::System, AddComponent, AddEntity, RemoveComponent, RemoveEntity, ReplaceComponent,
        UpdateComponent,
    },
    engine::{RenderContext, RenderEngine},
    resource::Resources,
    runner::{Job, Runner},
    scene::Scene,
    Rrc,
};

pub struct Initialize {
    pub scene: Rrc<Scene>,
    pub clock: Rrc<dyn Clock>,
    pub engine: Rrc<dyn RenderEngine>,
    pub resources: Rrc<Resources>,
}

pub struct PreRender {
    pub scene: Rrc<Scene>,
    pub clock: Rrc<dyn Clock>,
    pub engine: Rrc<dyn RenderEngine>,
    pub resources: Rrc<Resources>,
}

pub struct PostRender {
    pub scene: Rrc<Scene>,
    pub clock: Rrc<dyn Clock>,
    pub engine: Rrc<dyn RenderEngine>,
    pub resources: Rrc<Resources>,
}

pub struct Tictac {
    pub tick: super::clock::Tick,
    pub scene: Rrc<Scene>,
    pub clock: Rrc<dyn Clock>,
    pub engine: Rrc<dyn RenderEngine>,
    pub resources: Rrc<Resources>,
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
    tictac: Carrier<Tictac>,

    resources: Resources,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            initialize: Carrier::new(),
            pre_render: Carrier::new(),
            post_render: Carrier::new(),
            tictac: Carrier::new(),
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

    pub fn on_tictac(&self) -> &Carrier<Tictac> {
        &self.tictac
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

pub struct App {
    scene: Rrc<Scene>,
    clock: Rrc<dyn Clock>,
    engine: Rrc<dyn RenderEngine>,
    resources: Rrc<Resources>,
    runner: Box<dyn Runner>,

    pre_render: Carrier<PreRender>,
    post_render: Carrier<PostRender>,
    tictac: Carrier<Tictac>,
    add_entity: Carrier<AddEntity>,
    remove_entity: Carrier<RemoveEntity>,
    update_component: Carrier<UpdateComponent>,
    add_component: Carrier<AddComponent>,
    remove_component: Carrier<RemoveComponent>,
    replace_component: Carrier<ReplaceComponent>,
}

impl App {
    pub fn new<R, CLK, RE>(app_config: AppConfig) -> Self
    where
        R: Runner + 'static,
        CLK: Clock + 'static,
        RE: RenderEngine + 'static,
    {
        let app = Self {
            scene: Rc::new(RefCell::new(Scene::new(&app_config))),
            clock: Rc::new(RefCell::new(CLK::new(&app_config))),
            engine: Rc::new(RefCell::new(RE::new(&app_config))),
            runner: Box::new(R::new(&app_config)),

            resources: Rc::new(RefCell::new(app_config.resources)),

            pre_render: app_config.pre_render,
            post_render: app_config.post_render,
            tictac: app_config.tictac,
            add_entity: app_config.add_entity,
            remove_entity: app_config.remove_entity,
            update_component: app_config.update_component,
            add_component: app_config.add_component,
            remove_component: app_config.remove_component,
            replace_component: app_config.replace_component,
        };

        app_config.initialize.send(&Initialize {
            scene: Rc::clone(&app.scene),
            clock: Rc::clone(&app.clock),
            engine: Rc::clone(&app.engine),
            resources: Rc::clone(&app.resources),
        });

        app.clock
            .borrow()
            .on_tick()
            .register(ClockListener::new(&app));

        app.tictac.register(TestSystem);
        app.pre_render.register(TestSystem);

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

    pub fn on_pre_render(&self) -> &Carrier<PreRender> {
        &self.pre_render
    }

    pub fn on_post_render(&self) -> &Carrier<PostRender> {
        &self.post_render
    }

    pub fn on_tictac(&self) -> &Carrier<Tictac> {
        &self.tictac
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
    scene: Rrc<Scene>,
    clock: Rrc<dyn Clock>,
    engine: Rrc<dyn RenderEngine>,
    resources: Rrc<Resources>,
    pre_render: Carrier<PreRender>,
    post_render: Carrier<PostRender>,
}

impl AppJob {
    fn new(app: &App) -> Self {
        Self {
            scene: Rc::clone(&app.scene),
            clock: Rc::clone(&app.clock),
            engine: Rc::clone(&app.engine),
            resources: Rc::clone(&app.resources),
            pre_render: app.pre_render.clone(),
            post_render: app.post_render.clone(),
        }
    }
}

impl Job for AppJob {
    fn execute(&mut self) {
        self.pre_render.send(&PreRender {
            scene: Rc::clone(&self.scene),
            clock: Rc::clone(&self.clock),
            engine: Rc::clone(&self.engine),
            resources: Rc::clone(&self.resources),
        });

        self.engine.borrow_mut().render(RenderContext {
            scene: Rc::clone(&self.scene),
            clock: Rc::clone(&self.clock),
            resources: Rc::clone(&self.resources),
        });

        self.post_render.send(&PostRender {
            scene: Rc::clone(&self.scene),
            clock: Rc::clone(&self.clock),
            engine: Rc::clone(&self.engine),
            resources: Rc::clone(&self.resources),
        });
    }
}

struct ClockListener {
    tictac: Carrier<Tictac>,
    scene: Rrc<Scene>,
    clock: Rrc<dyn Clock>,
    engine: Rrc<dyn RenderEngine>,
    resources: Rrc<Resources>,
}

impl ClockListener {
    fn new(app: &App) -> Self {
        Self {
            tictac: app.tictac.clone(),
            scene: Rc::clone(&app.scene),
            clock: Rc::clone(&app.clock),
            engine: Rc::clone(&app.engine),
            resources: Rc::clone(&app.resources),
        }
    }
}

impl Listener<Tick> for ClockListener {
    fn execute(&self, tick: &Tick) {
        self.tictac.send(&Tictac {
            tick: *tick,
            scene: Rc::clone(&self.scene),
            clock: Rc::clone(&self.clock),
            engine: Rc::clone(&self.engine),
            resources: Rc::clone(&self.resources),
        });
    }
}

struct TestSystem;

impl System<Tictac> for TestSystem {
    fn execute(
        &self,
        Tictac {
            tick,
            scene,
            clock,
            engine,
            resources,
        }: &Tictac,
    ) {
        let entity = scene.borrow_mut().entity_manager_mut().create_empty_entity();
    }
}

impl System<PreRender> for TestSystem {
    fn execute(&self, message: &PreRender) {
        todo!()
    }
}
