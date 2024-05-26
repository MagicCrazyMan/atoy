pub trait JobLooper {
    fn start<J>(&mut self, job: J)
    where
        J: FnMut() + 'static;

    fn stop(&mut self);

    fn is_running(&self) -> bool;
}
