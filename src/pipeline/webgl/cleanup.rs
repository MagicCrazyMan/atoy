use crate::renderer::webgl::{buffer::Buffer, error::Error};

use super::{UBO_LIGHTS_BINDING_MOUNT_POINT, UBO_UNIVERSAL_UNIFORMS_BINDING_MOUNT_POINT};

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
        lights_ubo: &Buffer,
    ) -> Result<(), Error> {
        universal_ubo.unbind_ubo(UBO_UNIVERSAL_UNIFORMS_BINDING_MOUNT_POINT)?;
        lights_ubo.unbind_ubo(UBO_LIGHTS_BINDING_MOUNT_POINT)?;
        Ok(())
    }
}
