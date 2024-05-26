use super::{
    channel::MessageChannel,
    clock::Clock,
    command::{Commands, Context},
    engine::RenderEngine,
    looper::JobLooper,
    resource::Resources,
    scene::Scene,
};

pub struct App<JL, CLK, RE> {
    scene: Scene,
    clock: CLK,
    engine: RE,
    channel: MessageChannel,

    current_commands: Commands<CLK, RE>,
    next_commands: Commands<CLK, RE>,

    resources: Resources,
    temp_resources: Resources,

    job_looper: JL,
}

impl<JL, CLK, RE> App<JL, CLK, RE> {
    pub fn new(job_looper: JL, scene: Scene, clock: CLK, engine: RE) -> Self {
        Self {
            scene,
            clock,
            engine,
            channel: MessageChannel::new(),

            current_commands: Commands::new(),
            next_commands: Commands::new(),

            resources: Resources::new(),
            temp_resources: Resources::new(),

            job_looper,
        }
    }

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

    pub fn channel(&self) -> &MessageChannel {
        &self.channel
    }

    pub fn commands(&self) -> &Commands<CLK, RE> {
        &self.current_commands
    }

    pub fn commands_mut(&mut self) -> &mut Commands<CLK, RE> {
        &mut self.current_commands
    }

    pub fn resources(&self) -> &Resources {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Resources {
        &mut self.resources
    }
}

impl<JL, CLK, RE> App<JL, CLK, RE>
where
    JL: JobLooper,
    CLK: Clock + 'static,
    RE: RenderEngine + 'static,
{
    pub fn start(&mut self) {
        struct CommandExecutor<CLK, RE> {
            scene: *mut Scene,
            clock: *mut CLK,
            engine: *mut RE,
            channel: *mut MessageChannel,

            current_commands: *mut Commands<CLK, RE>,
            next_commands: *mut Commands<CLK, RE>,

            resources: *mut Resources,
            temp_resources: *mut Resources,
        }

        impl<CLK, RE> CommandExecutor<CLK, RE>
        where
            CLK: Clock + 'static,
            RE: RenderEngine + 'static,
        {
            unsafe fn execute(&mut self) {
                while let Some(mut command) = (*self.current_commands).pop_front() {
                    let context = Context {
                        scene: &mut *self.scene,
                        clock: &mut *self.clock,
                        engine: &mut *self.engine,
                        channel: &*self.channel,
                        resources: &mut *self.resources,
                        temp_resources: &mut *self.temp_resources,

                        current_commands: &mut *self.current_commands,
                        next_commands: &mut *self.next_commands,
                    };

                    command.execute(&context);
                }

                std::mem::swap(&mut *self.current_commands, &mut *self.next_commands);
                (*self.temp_resources).clear();
            }
        }

        let mut executor = CommandExecutor {
            scene: &mut self.scene,
            clock: &mut self.clock,
            engine: &mut self.engine,
            channel: &mut self.channel,

            current_commands: &mut self.current_commands,
            next_commands: &mut self.next_commands,

            resources: &mut self.resources,
            temp_resources: &mut self.temp_resources,
        };
        self.job_looper.start(move || unsafe {
            executor.execute();
        });
    }

    pub fn stop(&mut self) {
        self.job_looper.stop();
    }
}
