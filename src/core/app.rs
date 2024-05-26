use super::{
    carrier::Carrier,
    clock::{Clock, Tick},
    engine::{PostRender, PreRender, RenderEngine},
    resource::Resources,
    runner::{Job, Runner},
    scene::Scene,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Initialize;

pub struct AppConfig {
    pub initialize: Carrier<Initialize>,
    pub pre_render: Carrier<PreRender>,
    pub post_render: Carrier<PostRender>,
    pub tick: Carrier<Tick>,
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            initialize: Carrier::new(),
            pre_render: Carrier::new(),
            post_render: Carrier::new(),
            tick: Carrier::new(),
        }
    }

    pub fn build<R, CLK, RE>(self) -> App<R, CLK, RE>
    where
        R: Runner,
        CLK: Clock,
        RE: RenderEngine<CLK>,
    {
        App {
            scene: Scene::new(),
            clock: CLK::new(&self),
            engine: RE::new(&self),
            runner: R::new(&self),

            resources: Resources::new(),

            initialize: self.initialize,
            pre_render: self.pre_render,
            post_render: self.post_render,
            tick: self.tick,
        }
    }
}

pub struct App<R, CLK, RE> {
    scene: Scene,
    clock: CLK,
    engine: RE,
    runner: R,

    resources: Resources,

    initialize: Carrier<Initialize>,
    pre_render: Carrier<PreRender>,
    post_render: Carrier<PostRender>,
    tick: Carrier<Tick>,
}

impl<JL, CLK, RE> App<JL, CLK, RE>
where
    JL: Runner,
    CLK: Clock + 'static,
    RE: RenderEngine<CLK> + 'static,
{
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn clock(&self) -> &CLK {
        &self.clock
    }

    pub fn clock_mut(&mut self) -> &mut CLK {
        &mut self.clock
    }

    pub fn engine(&self) -> &RE {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut RE {
        &mut self.engine
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }
}

// impl<JL, CLK, RE> App<JL, CLK, RE>
// where
//     JL: JobLooper,
//     CLK: Clock + 'static,
//     RE: RenderEngine<CLK> + 'static,
// {
//     pub fn start(&mut self) {
//         let job = AppJob::new(self.context());
//         self.job_looper.start(job);
//     }

//     pub fn stop(&mut self) {
//         self.job_looper.stop();
//     }
// }

// struct AppJob<CLK, RE> {

// }

// impl<CLK, RE> AppJob<CLK, RE>
// where
//     CLK: Clock + 'static,
//     RE: RenderEngine<CLK> + 'static,
// {
//     fn new(context: Context<CLK, RE>) -> Self {
//         Self(context)
//     }
// }

// impl<CLK, RE> Job for AppJob<CLK, RE>
// where
//     CLK: Clock + 'static,
//     RE: RenderEngine<CLK> + 'static,
// {
//     fn execute(&mut self) {
//         self.0.pre.send(PreRender);
//         self.0.engine_mut().render(&self.0);
//         self.0.channel().send(PostRender, &self.0);
//     }
// }
