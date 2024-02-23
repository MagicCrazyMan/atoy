use gl_matrix4rust::{GLF32Borrowed, GLF32};
use web_sys::js_sys::{ArrayBuffer, Float32Array};

use crate::{
    light::{
        ambient_light::AmbientLight, area_light::AreaLight, attenuation::Attenuation,
        directional_light::DirectionalLight, point_light::PointLight, spot_light::SpotLight,
    },
    pipeline::webgl::UBO_LIGHTS_BYTES_LENGTH,
    renderer::webgl::{
        buffer::{BufferDescriptor, BufferSource},
        error::Error,
        state::FrameState,
    },
    scene::{Scene, MAX_AREA_LIGHTS, MAX_DIRECTIONAL_LIGHTS, MAX_POINT_LIGHTS, MAX_SPOT_LIGHTS},
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
    universal_uniforms: ArrayBuffer,

    last_light_attenuations: Option<Attenuation>,
    last_ambient_light: Option<AmbientLight>,
    last_directional_lights: Option<Vec<DirectionalLight>>,
    last_point_lights: Option<Vec<PointLight>>,
    last_spot_lights: Option<Vec<SpotLight>>,
    last_area_lights: Option<Vec<AreaLight>>,
}

impl StandardPreparation {
    pub fn new() -> Self {
        Self {
            universal_uniforms: ArrayBuffer::new(UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH as u32),

            last_light_attenuations: None,
            last_ambient_light: None,
            last_directional_lights: None,
            last_point_lights: None,
            last_spot_lights: None,
            last_area_lights: None,
        }
    }

    fn update_universal_ubo(
        &mut self,
        universal_ubo: &mut BufferDescriptor,
        state: &mut FrameState,
    ) -> Result<(), Error> {
        // u_RenderTime
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_uniforms,
            UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET as u32,
            1,
        )
        .set_index(0, state.timestamp() as f32);

        // u_CameraPosition
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_uniforms,
            UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET as u32,
            3,
        )
        .copy_from(&state.camera().position().gl_f32());

        // u_ViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_uniforms,
            UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET as u32,
            16,
        )
        .copy_from(&state.camera().view_matrix().gl_f32());

        // u_ProjMatrix
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_uniforms,
            UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET as u32,
            16,
        )
        .copy_from(&state.camera().proj_matrix().gl_f32());

        // u_ProjViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_uniforms,
            UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET as u32,
            16,
        )
        .copy_from(&state.camera().view_proj_matrix().gl_f32());

        universal_ubo.buffer_sub_data(
            BufferSource::from_array_buffer(self.universal_uniforms.clone()),
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
                    unsafe {
                        std::mem::transmute::<[f32; 3], [u8; 12]>([
                            scene.light_attenuations().a(),
                            scene.light_attenuations().b(),
                            scene.light_attenuations().c(),
                        ])
                    },
                    0,
                    12,
                ),
                UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET,
            );
            self.last_light_attenuations = Some(scene.light_attenuations().clone());
        }

        // u_AmbientLight
        if self.last_ambient_light.as_ref() != scene.ambient_light() {
            match scene.ambient_light() {
                Some(light) => {
                    lights_ubo.buffer_sub_data(
                        BufferSource::from_binary(
                            light.ubo(),
                            0,
                            UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH,
                        ),
                        UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET,
                    );
                }
                None => {
                    lights_ubo.buffer_sub_data(
                        BufferSource::from_binary(
                            [0; UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH],
                            0,
                            UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH,
                        ),
                        UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET,
                    );
                }
            }
            self.last_ambient_light = scene.ambient_light().cloned();
        }

        // uses for sending empty data
        const MAX_UBO_LIGHTS_LENGTH: [u8; UBO_LIGHTS_BYTES_LENGTH] = [0; UBO_LIGHTS_BYTES_LENGTH];
        macro_rules! update_lights {
            ($(($last:ident, $lights:ident, $max:tt, $len:tt, $offset:tt))+) => {
                $(
                    match &mut self.$last {
                        Some(last_lights) => {
                            let lights = scene.$lights();

                            for (index, light) in lights.into_iter().enumerate() {
                                let last = last_lights.get(index);
                                if last.map(|last| last == light).unwrap_or(false) {
                                    continue;
                                }

                                lights_ubo.buffer_sub_data(
                                    BufferSource::from_binary(
                                        light.ubo(),
                                        0,
                                        $len,
                                    ),
                                    $offset + index * $len,
                                );
                                last_lights.insert(index, light.clone());
                            }

                            // clears the rest
                            let removed = last_lights.drain(lights.len()..);
                            if removed.len() != 0 {
                                let clear_len = $len * ($max - lights.len());
                                let clear_offset = $offset + lights.len() * $len;
                                lights_ubo.buffer_sub_data(
                                    BufferSource::from_binary(&MAX_UBO_LIGHTS_LENGTH[0..clear_len], 0, clear_len),
                                    clear_offset,
                                );
                            }
                        }
                        None => {
                            let lights = scene.$lights();

                            lights_ubo.buffer_sub_data(
                                BufferSource::from_binary(
                                    &MAX_UBO_LIGHTS_LENGTH[0..$len * $max],
                                    0,
                                    $len * $max,
                                ),
                                $offset,
                            );
                            lights
                                .into_iter()
                                .enumerate()
                                .for_each(|(index, light)| {
                                    lights_ubo.buffer_sub_data(
                                        BufferSource::from_binary(
                                            light.ubo(),
                                            0,
                                            $len,
                                        ),
                                        $offset + index * $len,
                                    );
                                });
                            self.$last = Some(lights.to_vec());
                        }
                    }
                )+
            };
        }

        update_lights! {
            (last_directional_lights, directional_lights, MAX_DIRECTIONAL_LIGHTS, UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH, UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET)
            (last_point_lights, point_lights, MAX_POINT_LIGHTS, UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH, UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET)
            (last_spot_lights, spot_lights, MAX_SPOT_LIGHTS, UBO_LIGHTS_SPOT_LIGHT_BYTES_LENGTH, UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET)
            (last_area_lights, area_lights, MAX_AREA_LIGHTS, UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH, UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET)
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

impl AmbientLight {
    fn ubo(&self) -> [u8; UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH] {
        let mut ubo = [0.0f32; UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH / 4];
        ubo[0..3].copy_from_slice(self.color().gl_f32_borrowed());
        ubo[3] = if self.enabled() { 1.0 } else { 0.0 };

        unsafe {
            std::mem::transmute::<
                [f32; UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH / 4],
                [u8; UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH],
            >(ubo)
        }
    }
}

impl DirectionalLight {
    fn ubo(&self) -> [u8; UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH] {
        let mut ubo = [0.0f32; UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH / 4];
        ubo[0..3].copy_from_slice(self.direction().gl_f32_borrowed());
        ubo[3] = if self.enabled() { 1.0 } else { 0.0 };
        ubo[4..7].copy_from_slice(self.ambient().gl_f32_borrowed());
        ubo[8..11].copy_from_slice(self.diffuse().gl_f32_borrowed());
        ubo[12..15].copy_from_slice(self.specular().gl_f32_borrowed());

        unsafe {
            std::mem::transmute::<
                [f32; UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH / 4],
                [u8; UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTES_LENGTH],
            >(ubo)
        }
    }
}

impl PointLight {
    fn ubo(&self) -> [u8; UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH] {
        let mut ubo = [0.0f32; UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH / 4];
        ubo[0..3].copy_from_slice(&self.position().gl_f32());
        ubo[3] = if self.enabled() { 1.0 } else { 0.0 };
        ubo[4..7].copy_from_slice(self.ambient().gl_f32_borrowed());
        ubo[8..11].copy_from_slice(self.diffuse().gl_f32_borrowed());
        ubo[12..15].copy_from_slice(self.specular().gl_f32_borrowed());

        unsafe {
            std::mem::transmute::<
                [f32; UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH / 4],
                [u8; UBO_LIGHTS_POINT_LIGHT_BYTES_LENGTH],
            >(ubo)
        }
    }
}

impl AreaLight {
    fn ubo(&self) -> [u8; UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH] {
        let mut ubo = [0.0f32; UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH / 4];
        ubo[0..3].copy_from_slice(self.direction().gl_f32_borrowed());
        ubo[3] = if self.enabled() { 1.0 } else { 0.0 };
        ubo[4..7].copy_from_slice(self.up().gl_f32_borrowed());
        ubo[7] = self.inner_width();
        ubo[8..11].copy_from_slice(self.right().gl_f32_borrowed());
        ubo[11] = self.inner_height();
        ubo[12..15].copy_from_slice(&self.position().gl_f32());
        ubo[15] = self.offset();
        ubo[16..19].copy_from_slice(self.ambient().gl_f32_borrowed());
        ubo[19] = self.outer_width();
        ubo[20..23].copy_from_slice(self.diffuse().gl_f32_borrowed());
        ubo[23] = self.outer_height();
        ubo[24..27].copy_from_slice(self.specular().gl_f32_borrowed());

        unsafe {
            std::mem::transmute::<
                [f32; UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH / 4],
                [u8; UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH],
            >(ubo)
        }
    }
}

impl SpotLight {
    fn ubo(&self) -> [u8; UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH] {
        let mut ubo = [0.0f32; UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH / 4];
        ubo[0..3].copy_from_slice(self.direction().gl_f32_borrowed());
        ubo[3] = if self.enabled() { 1.0 } else { 0.0 };
        ubo[4..7].copy_from_slice(&self.position().gl_f32());
        ubo[7] = 0.0;
        ubo[8..11].copy_from_slice(self.ambient().gl_f32_borrowed());
        ubo[11] = self.inner_cutoff().cos();
        ubo[12..15].copy_from_slice(self.diffuse().gl_f32_borrowed());
        ubo[15] = self.outer_cutoff().cos();
        ubo[16..19].copy_from_slice(self.specular().gl_f32_borrowed());

        unsafe {
            std::mem::transmute::<
                [f32; UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH / 4],
                [u8; UBO_LIGHTS_AREA_LIGHT_BYTES_LENGTH],
            >(ubo)
        }
    }
}
