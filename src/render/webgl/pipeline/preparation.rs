use gl_matrix4rust::{vec3::Vec3, GLU8Borrowed, GLF32};
use web_sys::js_sys::{ArrayBuffer, Float32Array};

use crate::{
    render::webgl::{
        buffer::{BufferDescriptor, BufferSource},
        error::Error,
        state::FrameState,
    },
    scene::Scene,
};

use super::{
    UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH, UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET,
    UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET, UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH,
    UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET, UBO_LIGHTS_BINDING,
    UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET, UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH,
    UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET, UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH,
    UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET, UBO_LIGHTS_SPOT_LIGHT_BYTES_LENGTH,
    UBO_UNIVERSAL_UNIFORMS_BINDING, UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH,
    UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET,
    UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET,
    UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET,
    UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET,
    UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET,
};

pub struct StandardPreparation {
    universal_data: ArrayBuffer,
    last_light_attenuations: Option<Vec3<f32>>,
}

impl StandardPreparation {
    pub fn new() -> Self {
        Self {
            universal_data: ArrayBuffer::new(UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH),
            last_light_attenuations: None,
        }
    }

    fn update_universal_ubo(
        &mut self,
        universal_ubo: &mut BufferDescriptor,
        state: &mut FrameState,
    ) -> Result<(), Error> {
        // u_RenderTime
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_data,
            UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET,
            1,
        )
        .set_index(0, state.timestamp() as f32);

        // u_CameraPosition
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_data,
            UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET,
            3,
        )
        .copy_from(&state.camera().position().gl_f32());

        // u_ViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_data,
            UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET,
            16,
        )
        .copy_from(&state.camera().view_matrix().gl_f32());

        // u_ProjMatrix
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_data,
            UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET,
            16,
        )
        .copy_from(&state.camera().proj_matrix().gl_f32());

        // u_ProjViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_data,
            UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET,
            16,
        )
        .copy_from(&state.camera().view_proj_matrix().gl_f32());

        universal_ubo.buffer_sub_data(
            BufferSource::from_array_buffer(self.universal_data.clone()),
            0,
        );
        state.buffer_store_mut().bind_uniform_buffer_object(
            universal_ubo,
            UBO_UNIVERSAL_UNIFORMS_BINDING,
            None,
        )?;
        Ok(())
    }

    fn update_lights_ubo(
        &mut self,
        lights_ubo: &mut BufferDescriptor,
        state: &mut FrameState,
        scene: &mut Scene,
    ) -> Result<(), Error> {
        // u_Attenuations
        if self
            .last_light_attenuations
            .as_ref()
            .map(|a| a != scene.light_attenuations())
            .unwrap_or(true)
        {
            lights_ubo.buffer_sub_data(
                BufferSource::from_binary(
                    scene.light_attenuations().gl_u8_borrowed().clone(),
                    0,
                    12,
                ),
                UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET as i32,
            );
            self.last_light_attenuations = Some(scene.light_attenuations().clone());
        }

        // u_AmbientLight
        if let Some(light) = scene.ambient_light_mut() {
            if light.ubo_dirty() {
                light.update_ubo();

                lights_ubo.buffer_sub_data(
                    BufferSource::from_binary(
                        light.ubo().clone(),
                        0,
                        UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH,
                    ),
                    UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET as i32,
                );
            }
        }

        // u_DirectionalLights
        for (index, light) in scene.directional_lights_mut().into_iter().enumerate() {
            if light.ubo_dirty() {
                light.update_ubo();

                lights_ubo.buffer_sub_data(
                    BufferSource::from_binary(
                        light.ubo().clone(),
                        0,
                        UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH,
                    ),
                    UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET as i32
                        + index as i32 * UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH as i32,
                );
            }
        }

        // u_PointLights
        for (index, light) in scene.point_lights_mut().into_iter().enumerate() {
            if light.ubo_dirty() {
                light.update_ubo();

                lights_ubo.buffer_sub_data(
                    BufferSource::from_binary(
                        light.ubo().clone(),
                        0,
                        UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH,
                    ),
                    UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET as i32
                        + index as i32 * UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH as i32,
                );
            }
        }

        // u_SpotLights
        for (index, light) in scene.spot_lights_mut().into_iter().enumerate() {
            if light.ubo_dirty() {
                light.update_ubo();

                lights_ubo.buffer_sub_data(
                    BufferSource::from_binary(
                        light.ubo().clone(),
                        0,
                        UBO_LIGHTS_SPOT_LIGHT_BYTES_LENGTH,
                    ),
                    UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET as i32
                        + index as i32 * UBO_LIGHTS_SPOT_LIGHT_BYTES_LENGTH as i32,
                );
            }
        }

        // u_AreaLights
        for (index, light) in scene.area_lights_mut().into_iter().enumerate() {
            if light.ubo_dirty() {
                light.update_ubo();

                lights_ubo.buffer_sub_data(
                    BufferSource::from_binary(
                        light.ubo().clone(),
                        0,
                        UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH,
                    ),
                    UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET as i32
                        + index as i32 * UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH as i32,
                );
            }
        }

        state.buffer_store_mut().bind_uniform_buffer_object(
            lights_ubo,
            UBO_LIGHTS_BINDING,
            None,
        )?;
        Ok(())
    }
}

impl StandardPreparation {
    pub fn prepare(
        &mut self,
        state: &mut FrameState,
        scene: &mut Scene,
        universal_ubo: &mut BufferDescriptor,
        mut lights_ubo: Option<&mut BufferDescriptor>,
    ) -> Result<(), Error> {
        state.gl().viewport(
            0,
            0,
            state.canvas().width() as i32,
            state.canvas().height() as i32,
        );
        self.update_universal_ubo(universal_ubo, state)?;
        if let Some(lights_ubo) = lights_ubo.as_mut() {
            self.update_lights_ubo(lights_ubo, state, scene)?;
        }
        Ok(())
    }
}
