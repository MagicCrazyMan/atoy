use super::app::AppConfig;

pub trait Runner {
    fn new(app_config: &AppConfig) -> Self
    where
        Self: Sized;

    fn start<J>(&mut self, job: J)
    where
        J: Job + 'static;

    fn stop(&mut self);

    fn is_running(&self) -> bool;
}

pub trait Job {
    fn execute(&mut self);
}
