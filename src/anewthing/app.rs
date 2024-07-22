use super::{channel::Channel, clock::Clock};

pub struct App {
    channel: Channel,
    clock: Box<dyn Clock>,
}

impl App {
    pub fn new<C>(clock: C) -> Self
    where
        C: Clock + 'static,
    {
        let channel = Channel::new();

        Self {
            channel,
            clock: Box::new(clock),
        }
    }

    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    pub fn clock(&self) -> &dyn Clock {
        self.clock.as_ref()
    }

    pub fn clock_mut(&mut self) -> &mut dyn Clock {
        self.clock.as_mut()
    }

    pub fn run(&mut self) {
        let channel = self.channel.clone();
        self.clock_mut().start(channel);
    }

    pub fn terminate(&mut self) {
        self.clock_mut().stop();
    }
}
