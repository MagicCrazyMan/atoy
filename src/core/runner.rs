use super::{app::AppConfig, AsAny};

pub trait Runner: AsAny {
    fn new(app_config: &AppConfig) -> Self
    where
        Self: Sized;

    fn start(&mut self, job: Box<dyn Job>);

    fn stop(&mut self);

    fn running(&self) -> bool;
}

pub trait Job {
    fn execute(&mut self);
}
