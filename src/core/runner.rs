use super::AsAny;

pub trait Runner: AsAny {
    fn start(&mut self, job: Box<dyn Job>);

    fn stop(&mut self);

    fn running(&self) -> bool;
}

pub trait Job {
    fn execute(&mut self);
}
