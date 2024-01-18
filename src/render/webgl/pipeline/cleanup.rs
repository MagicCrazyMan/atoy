use crate::render::webgl::{
    state::FrameState,
    uniform::{UBO_LIGHTS_BINDING, UBO_UNIVERSAL_UNIFORMS_BINDING},
};

pub struct StandardCleanup;

impl StandardCleanup {
    pub fn new() -> Self {
        Self
    }
}

impl StandardCleanup {
    pub fn cleanup(&mut self, state: &mut FrameState) {
        state.gl().viewport(
            0,
            0,
            state.canvas().width() as i32,
            state.canvas().height() as i32,
        );
        state
            .buffer_store_mut()
            .unbind_uniform_buffer_object(UBO_UNIVERSAL_UNIFORMS_BINDING);
        state
            .buffer_store_mut()
            .unbind_uniform_buffer_object(UBO_LIGHTS_BINDING);
    }
}
