use crate::core::channel::MessageChannel;

pub struct App {
    channel: MessageChannel,
}

impl App {
    pub fn new() -> Self {
        Self {
            channel: MessageChannel::new(),
        }
    }
}
