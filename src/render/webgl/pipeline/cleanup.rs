use crate::render::webgl::state::FrameState;

use super::{UBO_LIGHTS_BINDING, UBO_UNIVERSAL_UNIFORMS_BINDING};

pub struct StandardCleanup;

impl StandardCleanup {
    pub fn new() -> Self {
        Self
    }
}

impl StandardCleanup {
    pub fn cleanup(&mut self, state: &mut FrameState) {
        state
            .buffer_store_mut()
            .unbind_uniform_buffer_object(UBO_UNIVERSAL_UNIFORMS_BINDING);
        state
            .buffer_store_mut()
            .unbind_uniform_buffer_object(UBO_LIGHTS_BINDING);
    }
}
