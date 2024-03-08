use crate::renderer::webgl::{buffer::Buffer, error::Error};

use super::{UBO_LIGHTS_BINDING_INDEX, UBO_UNIVERSAL_UNIFORMS_BINDING_INDEX};

pub struct StandardCleanup;

impl StandardCleanup {
    pub fn new() -> Self {
        Self
    }
}

impl StandardCleanup {
    pub fn cleanup(
        &mut self,
        universal_ubo: &Buffer,
        lights_ubo: Option<&Buffer>,
    ) -> Result<(), Error> {
        universal_ubo.unbind_ubo(UBO_UNIVERSAL_UNIFORMS_BINDING_INDEX)?;
        if let Some(lights_ubo) = lights_ubo.as_ref() {
            lights_ubo.unbind_ubo(UBO_LIGHTS_BINDING_INDEX)?;
        }
        Ok(())
    }
}
