use web_sys::js_sys::{ArrayBuffer, Float32Array};

use crate::{
    light::{
        ambient_light::AmbientLight, area_light::AreaLight, attenuation::Attenuation,
        directional_light::DirectionalLight, point_light::PointLight, spot_light::SpotLight,
    },
    renderer::webgl::{
        buffer::{Buffer, Preallocation},
        error::Error,
        matrix::GlF32,
        state::FrameState,
    },
    scene::{Scene, MAX_AREA_LIGHTS, MAX_DIRECTIONAL_LIGHTS, MAX_POINT_LIGHTS, MAX_SPOT_LIGHTS},
};

use super::{
    UBO_LIGHTS_AMBIENT_LIGHT_BYTE_LENGTH, UBO_LIGHTS_AMBIENT_LIGHT_BYTE_OFFSET,
    UBO_LIGHTS_AREA_LIGHTS_BYTE_OFFSET, UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH,
    UBO_LIGHTS_ATTENUATIONS_BYTE_OFFSET, UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTE_OFFSET,
    UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTE_LENGTH, UBO_LIGHTS_POINT_LIGHTS_BYTE_OFFSET,
    UBO_LIGHTS_POINT_LIGHT_BYTE_LENGTH, UBO_LIGHTS_SPOT_LIGHTS_BYTE_OFFSET,
    UBO_LIGHTS_SPOT_LIGHT_BYTE_LENGTH, UBO_LIGHTS_UNIFORM_BLOCK_MOUNT_POINT,
    UBO_UNIVERSAL_UNIFORMS_BYTE_LENGTH, UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTE_OFFSET,
    UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTE_OFFSET, UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTE_OFFSET,
    UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTE_OFFSET,
    UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTE_OFFSET, UBO_UNIVERSAL_UNIFORM_BLOCK_MOUNT_POINT,
};

pub struct StandardPreparation {
    universal_uniforms: ArrayBuffer,

    last_light_attenuation: Option<Attenuation>,
    last_ambient_light: Option<AmbientLight>,
    last_directional_lights: Option<Vec<DirectionalLight>>,
    last_point_lights: Option<Vec<PointLight>>,
    last_spot_lights: Option<Vec<SpotLight>>,
    last_area_lights: Option<Vec<AreaLight>>,
}

impl StandardPreparation {
    pub fn new() -> Self {
        Self {
            universal_uniforms: ArrayBuffer::new(UBO_UNIVERSAL_UNIFORMS_BYTE_LENGTH as u32),

            last_light_attenuation: None,
            last_ambient_light: None,
            last_directional_lights: None,
            last_point_lights: None,
            last_spot_lights: None,
            last_area_lights: None,
        }
    }

    fn update_universal_ubo(
        &mut self,
        universal_ubo: &mut Buffer,
        state: &mut FrameState,
    ) -> Result<(), Error> {
        state.buffer_store().register(universal_ubo)?;

        // u_RenderTime
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_uniforms,
            UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTE_OFFSET as u32,
            1,
        )
        .set_index(0, state.timestamp() as f32);

        // u_CameraPosition
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_uniforms,
            UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTE_OFFSET as u32,
            3,
        )
        .copy_from(&state.camera().position().to_f32_array());

        // u_ViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_uniforms,
            UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTE_OFFSET as u32,
            16,
        )
        .copy_from(&state.camera().view_matrix().to_f32_array());

        // u_ProjMatrix
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_uniforms,
            UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTE_OFFSET as u32,
            16,
        )
        .copy_from(&state.camera().proj_matrix().to_f32_array());

        // u_ProjViewMatrix
        Float32Array::new_with_byte_offset_and_length(
            &self.universal_uniforms,
            UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTE_OFFSET as u32,
            16,
        )
        .copy_from(&state.camera().view_proj_matrix().to_f32_array());

        universal_ubo.buffer_sub_data(self.universal_uniforms.clone(), 0);
        universal_ubo.bind_ubo(UBO_UNIVERSAL_UNIFORM_BLOCK_MOUNT_POINT)?;

        Ok(())
    }

    fn update_lights_ubo(
        &mut self,
        lights_ubo: &mut Buffer,
        state: &mut FrameState,
        scene: &mut Scene,
    ) -> Result<(), Error> {
        state.buffer_store().register(lights_ubo)?;

        // u_Attenuations
        if self
            .last_light_attenuation
            .as_ref()
            .map(|last| last != scene.light_attenuation())
            .unwrap_or(true)
        {
            let mut data = [0u8; 12];
            data[0..4].copy_from_slice(scene.light_attenuation().a().to_ne_bytes().as_slice());
            data[4..8].copy_from_slice(scene.light_attenuation().b().to_ne_bytes().as_slice());
            data[8..12].copy_from_slice(scene.light_attenuation().c().to_ne_bytes().as_slice());

            lights_ubo.buffer_sub_data(data, UBO_LIGHTS_ATTENUATIONS_BYTE_OFFSET);
            self.last_light_attenuation = Some(scene.light_attenuation().clone());
        }

        // u_AmbientLight
        if &self.last_ambient_light != scene.ambient_light() {
            match scene.ambient_light() {
                Some(light) => {
                    lights_ubo.buffer_sub_data(light.ubo(), UBO_LIGHTS_AMBIENT_LIGHT_BYTE_OFFSET);
                }
                None => {
                    lights_ubo.buffer_sub_data(
                        Preallocation::new(UBO_LIGHTS_AMBIENT_LIGHT_BYTE_LENGTH),
                        UBO_LIGHTS_AMBIENT_LIGHT_BYTE_OFFSET,
                    );
                }
            }
            self.last_ambient_light = scene.ambient_light().clone();
        }

        // uses for sending empty data
        macro_rules! update_lights {
            ($(($last:ident, $lights:ident, $count:tt, $len:tt, $offset:tt))+) => {
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
                                    light.ubo(),
                                    $offset + index * $len,
                                );
                                last_lights.insert(index, light.clone());
                            }

                            // clears the rest
                            let removed = last_lights.drain(lights.len()..);
                            if removed.len() != 0 {
                                let clear_len = $len * ($count - lights.len());
                                let clear_offset = $offset + lights.len() * $len;
                                lights_ubo.buffer_sub_data(
                                    Preallocation::new(clear_len),
                                    clear_offset,
                                );
                            }
                        }
                        None => {
                            let lights = scene.$lights();
                            let mut last_lights = Vec::with_capacity(lights.len());

                            // clears first
                            lights_ubo.buffer_sub_data(
                                Preallocation::new($len * $count),
                                $offset,
                            );

                            // buffers each
                            for (index, light) in lights.into_iter().enumerate() {
                                lights_ubo.buffer_sub_data(
                                    light.ubo(),
                                    $offset + index * $len,
                                );
                                last_lights.push(light.clone());
                            }
                            self.$last = Some(last_lights);
                        }
                    }
                )+
            };
        }

        update_lights! {
            (last_directional_lights, directional_lights, MAX_DIRECTIONAL_LIGHTS, UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTE_LENGTH, UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTE_OFFSET)
            (last_point_lights, point_lights, MAX_POINT_LIGHTS, UBO_LIGHTS_POINT_LIGHT_BYTE_LENGTH, UBO_LIGHTS_POINT_LIGHTS_BYTE_OFFSET)
            (last_spot_lights, spot_lights, MAX_SPOT_LIGHTS, UBO_LIGHTS_SPOT_LIGHT_BYTE_LENGTH, UBO_LIGHTS_SPOT_LIGHTS_BYTE_OFFSET)
            (last_area_lights, area_lights, MAX_AREA_LIGHTS, UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH, UBO_LIGHTS_AREA_LIGHTS_BYTE_OFFSET)
        }

        lights_ubo.bind_ubo(UBO_LIGHTS_UNIFORM_BLOCK_MOUNT_POINT)?;

        Ok(())
    }
}

impl StandardPreparation {
    pub fn prepare(
        &mut self,
        state: &mut FrameState,
        scene: &mut Scene,
        universal_ubo: &mut Buffer,
        lights_ubo: &mut Buffer,
    ) -> Result<(), Error> {
        state.gl().viewport(
            0,
            0,
            state.canvas().width() as i32,
            state.canvas().height() as i32,
        );
        self.update_universal_ubo(universal_ubo, state)?;
        self.update_lights_ubo(lights_ubo, state, scene)?;
        Ok(())
    }
}

impl AmbientLight {
    fn ubo(&self) -> [u8; UBO_LIGHTS_AMBIENT_LIGHT_BYTE_LENGTH] {
        let mut ubo = [0.0f32; UBO_LIGHTS_AMBIENT_LIGHT_BYTE_LENGTH / 4];
        ubo[0..3].copy_from_slice(&self.color().to_f32_array());
        ubo[3] = if self.enabled() { 1.0 } else { 0.0 };

        unsafe {
            std::mem::transmute::<
                [f32; UBO_LIGHTS_AMBIENT_LIGHT_BYTE_LENGTH / 4],
                [u8; UBO_LIGHTS_AMBIENT_LIGHT_BYTE_LENGTH],
            >(ubo)
        }
    }
}

impl DirectionalLight {
    fn ubo(&self) -> [u8; UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTE_LENGTH] {
        let mut ubo = [0.0f32; UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTE_LENGTH / 4];
        ubo[0..3].copy_from_slice(&self.direction().to_f32_array());
        ubo[3] = if self.enabled() { 1.0 } else { 0.0 };
        ubo[4..7].copy_from_slice(&self.ambient().to_f32_array());
        ubo[8..11].copy_from_slice(&self.diffuse().to_f32_array());
        ubo[12..15].copy_from_slice(&self.specular().to_f32_array());

        unsafe {
            std::mem::transmute::<
                [f32; UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTE_LENGTH / 4],
                [u8; UBO_LIGHTS_DIRECTIONAL_LIGHT_BYTE_LENGTH],
            >(ubo)
        }
    }
}

impl PointLight {
    fn ubo(&self) -> [u8; UBO_LIGHTS_POINT_LIGHT_BYTE_LENGTH] {
        let mut ubo = [0.0f32; UBO_LIGHTS_POINT_LIGHT_BYTE_LENGTH / 4];
        ubo[0..3].copy_from_slice(&self.position().to_f32_array());
        ubo[3] = if self.enabled() { 1.0 } else { 0.0 };
        ubo[4..7].copy_from_slice(&self.ambient().to_f32_array());
        ubo[8..11].copy_from_slice(&self.diffuse().to_f32_array());
        ubo[12..15].copy_from_slice(&self.specular().to_f32_array());

        unsafe {
            std::mem::transmute::<
                [f32; UBO_LIGHTS_POINT_LIGHT_BYTE_LENGTH / 4],
                [u8; UBO_LIGHTS_POINT_LIGHT_BYTE_LENGTH],
            >(ubo)
        }
    }
}

impl AreaLight {
    fn ubo(&self) -> [u8; UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH] {
        let mut ubo = [0.0f32; UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH / 4];
        ubo[0..3].copy_from_slice(&self.direction().to_f32_array());
        ubo[3] = if self.enabled() { 1.0 } else { 0.0 };
        ubo[4..7].copy_from_slice(&self.up().to_f32_array());
        ubo[7] = self.inner_width();
        ubo[8..11].copy_from_slice(&self.right().to_f32_array());
        ubo[11] = self.inner_height();
        ubo[12..15].copy_from_slice(&self.position().to_f32_array());
        ubo[15] = self.offset();
        ubo[16..19].copy_from_slice(&self.ambient().to_f32_array());
        ubo[19] = self.outer_width();
        ubo[20..23].copy_from_slice(&self.diffuse().to_f32_array());
        ubo[23] = self.outer_height();
        ubo[24..27].copy_from_slice(&self.specular().to_f32_array());

        unsafe {
            std::mem::transmute::<
                [f32; UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH / 4],
                [u8; UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH],
            >(ubo)
        }
    }
}

impl SpotLight {
    fn ubo(&self) -> [u8; UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH] {
        let mut ubo = [0.0f32; UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH / 4];
        ubo[0..3].copy_from_slice(&self.direction().to_f32_array());
        ubo[3] = if self.enabled() { 1.0 } else { 0.0 };
        ubo[4..7].copy_from_slice(&self.position().to_f32_array());
        ubo[7] = 0.0;
        ubo[8..11].copy_from_slice(&self.ambient().to_f32_array());
        ubo[11] = self.inner_cutoff().cos();
        ubo[12..15].copy_from_slice(&self.diffuse().to_f32_array());
        ubo[15] = self.outer_cutoff().cos();
        ubo[16..19].copy_from_slice(&self.specular().to_f32_array());

        unsafe {
            std::mem::transmute::<
                [f32; UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH / 4],
                [u8; UBO_LIGHTS_AREA_LIGHT_BYTE_LENGTH],
            >(ubo)
        }
    }
}
