use gl_matrix4rust::{mat4::AsMat4, vec3::AsVec3};
use web_sys::js_sys::{ArrayBuffer, Float32Array};

use crate::{
    render::{
        webgl::{
            buffer::{BufferDescriptor, BufferSource, BufferUsage, MemoryPolicy},
            error::Error,
            state::FrameState,
            uniform::{
                UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH, UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET,
                UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH, UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET,
                UBO_LIGHTS_ATTENUATIONS_BYTES_LENGTH, UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET,
                UBO_LIGHTS_BINDING, UBO_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET, UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET, UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET, UBO_UNIVERSAL_UNIFORMS_BINDING,
                UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET,
                UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_OFFSET,
                UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET,
                UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET,
                UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET,
                UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_LENGTH,
                UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET,
            },
        },
        Executor, Resources,
    },
    scene::Scene,
};

pub struct StandardPreparation {
    universal_ubo: BufferDescriptor,
    lights_ubo: BufferDescriptor,
}

impl StandardPreparation {
    pub fn new() -> Self {
        Self {
            universal_ubo: BufferDescriptor::with_memory_policy(
                BufferSource::preallocate(UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH as i32),
                BufferUsage::DynamicDraw,
                MemoryPolicy::Unfree,
            ),
            lights_ubo: BufferDescriptor::with_memory_policy(
                BufferSource::preallocate(UBO_LIGHTS_BYTES_LENGTH as i32),
                BufferUsage::DynamicDraw,
                MemoryPolicy::Unfree,
            ),
        }
    }

    fn update_universal_ubo(
        &mut self,
        state: &mut FrameState,
        scene: &mut Scene,
    ) -> Result<(), Error> {
        let data = ArrayBuffer::new(UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH);

        // u_RenderTime
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_LENGTH / 4,
        )
        .set_index(0, state.timestamp() as f32);

        // u_EnableLighting
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_LENGTH / 4,
        )
        .set_index(0, if scene.lighting_enabled() { 1.0 } else { 0.0 });

        // u_CameraPosition
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_LENGTH / 4,
        )
        .copy_from(&state.camera().position().to_gl());

        // u_ViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_LENGTH / 4,
        )
        .copy_from(&state.camera().view_matrix().to_gl());

        // u_ProjMatrix
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_LENGTH / 4,
        )
        .copy_from(&state.camera().proj_matrix().to_gl());

        // u_ProjViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET,
            UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_LENGTH / 4,
        )
        .copy_from(&state.camera().view_proj_matrix().to_gl());

        self.universal_ubo
            .buffer_sub_data(BufferSource::from_array_buffer(data), 0);
        state
            .buffer_store_mut()
            .bind_uniform_buffer_object(&self.universal_ubo, UBO_UNIVERSAL_UNIFORMS_BINDING)?;
        Ok(())
    }

    fn update_lights_ubo(
        &mut self,
        state: &mut FrameState,
        scene: &mut Scene,
    ) -> Result<(), Error> {
        let data = ArrayBuffer::new(UBO_LIGHTS_BYTES_LENGTH);

        // u_Attenuations
        Float32Array::new_with_byte_offset_and_length(
            &data,
            UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET,
            UBO_LIGHTS_ATTENUATIONS_BYTES_LENGTH / 4,
        )
        .copy_from(&scene.light_attenuations().to_gl());

        // u_AmbientLight
        if let Some(light) = scene.ambient_light() {
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET,
                UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_DirectionalLights
        for (index, light) in scene.directional_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET
                    + index * UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_PointLights
        for (index, light) in scene.point_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET + index * UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_SpotLights
        for (index, light) in scene.spot_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET + index * UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        // u_AreaLights
        for (index, light) in scene.area_lights().into_iter().enumerate() {
            let index = index as u32;
            Float32Array::new_with_byte_offset_and_length(
                &data,
                UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET + index * UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH,
                UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH / 4,
            )
            .copy_from(&light.gl_ubo());
        }

        self.lights_ubo
            .buffer_sub_data(BufferSource::from_array_buffer(data), 0);
        state
            .buffer_store_mut()
            .bind_uniform_buffer_object(&self.lights_ubo, UBO_LIGHTS_BINDING)?;
        Ok(())
    }
}

impl Executor for StandardPreparation {
    type State = FrameState;

    type Error = Error;

    fn execute(
        &mut self,
        state: &mut Self::State,
        scene: &mut Scene,
        _: &mut Resources,
    ) -> Result<(), Self::Error> {
        state.gl().viewport(
            0,
            0,
            state.canvas().width() as i32,
            state.canvas().height() as i32,
        );

        self.update_universal_ubo(state, scene)?;
        self.update_lights_ubo(state, scene)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
