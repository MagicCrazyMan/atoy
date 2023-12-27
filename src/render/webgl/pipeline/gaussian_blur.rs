use web_sys::{
    js_sys::Float32Array, WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlTexture,
    WebGlUniformLocation,
};

use crate::render::{
    pp::{Executor, ResourceKey, Resources, State, Stuff},
    webgl::{
        error::Error,
        offscreen::{
            FramebufferAttachment, FramebufferTarget, OffscreenFramebuffer,
            OffscreenTextureProvider,
        },
        program::{compile_shaders, create_program, ShaderSource},
        texture::{TextureDataType, TextureFormat, TextureInternalFormat}, uniform::UBO_GAUSSIAN_BLUR_BINDING,
    },
};

#[rustfmt::skip]
const BUFFER_DATA: [f32; 16] = [
    // vertices
    1.0,-1.0,  1.0,1.0, -1.0,1.0, -1.0,-1.0,
    // textures coordinates
    1.0, 0.0,  1.0,1.0,  0.0,1.0,  0.0, 0.0
];

#[rustfmt::skip]
const GAUSSIAN_KERNEL: [f32; 81] = [
    0.0002629586560000000, 0.0008765396640000000, 0.001972215865600000, 0.0031555460336000003, 0.0036814698320000003, 0.0031555460336000003, 0.001972215865600000, 0.0008765396640000000, 0.0002629586560000000,
    0.0008765396640000000, 0.0029218349159999997, 0.006574133966399999, 0.0105186165084000000, 0.0122717174580000000, 0.0105186165084000000, 0.006574133966399999, 0.0029218349159999997, 0.0008765396640000000,
    0.0019722158656000000, 0.0065741339663999990, 0.014791813586560000, 0.0236669066033600000, 0.0276113869832000000, 0.0236669066033600000, 0.014791813586560000, 0.0065741339663999990, 0.0019722158656000000,
    0.0031555460336000003, 0.0105186165084000000, 0.023666906603360000, 0.0378670583491600000, 0.0441782282542000000, 0.0378670583491600000, 0.023666906603360000, 0.0105186165084000000, 0.0031555460336000003,
    0.0036814698320000003, 0.0122717174580000000, 0.027611386983200000, 0.0441782282542000000, 0.0515412587290000060, 0.0441782282542000000, 0.027611386983200000, 0.0122717174580000000, 0.0036814698320000003,
    0.0031555460336000003, 0.0105186165084000000, 0.023666906603360000, 0.0378670583491600000, 0.0441782282542000000, 0.0378670583491600000, 0.023666906603360000, 0.0105186165084000000, 0.0031555460336000003,
    0.0019722158656000000, 0.0065741339663999990, 0.014791813586560000, 0.0236669066033600000, 0.0276113869832000000, 0.0236669066033600000, 0.014791813586560000, 0.0065741339663999990, 0.0019722158656000000,
    0.0008765396640000000, 0.0029218349159999997, 0.006574133966399999, 0.0105186165084000000, 0.0122717174580000000, 0.0105186165084000000, 0.006574133966399999, 0.0029218349159999997, 0.0008765396640000000,
    0.0002629586560000000, 0.0008765396640000000, 0.001972215865600000, 0.0031555460336000003, 0.0036814698320000003, 0.0031555460336000003, 0.001972215865600000, 0.0008765396640000000, 0.0002629586560000000,
];

struct Compiled {
    program: WebGlProgram,
    position_location: u32,
    texture_location: u32,
    kernel_location: u32,
    sampler_location: WebGlUniformLocation,
    attributes_buffer: WebGlBuffer,
    uniform_buffer: WebGlBuffer,
}

pub struct GaussianBlur {
    epoch: usize,
    compiled: Option<Compiled>,
    onepass_frame: OffscreenFramebuffer,
    twopass_frame: OffscreenFramebuffer,
    in_texture: ResourceKey<WebGlTexture>,
    out_texture: ResourceKey<WebGlTexture>,
}

impl GaussianBlur {
    pub fn new(
        in_texture: ResourceKey<WebGlTexture>,
        out_texture: ResourceKey<WebGlTexture>,
    ) -> Self {
        Self {
            epoch: 2,
            compiled: None,
            onepass_frame: OffscreenFramebuffer::new(
                FramebufferTarget::FRAMEBUFFER,
                [OffscreenTextureProvider::new(
                    FramebufferTarget::FRAMEBUFFER,
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA,
                    TextureFormat::RGBA,
                    TextureDataType::UNSIGNED_BYTE,
                    0,
                )],
                [],
            ),
            twopass_frame: OffscreenFramebuffer::new(
                FramebufferTarget::FRAMEBUFFER,
                [OffscreenTextureProvider::new(
                    FramebufferTarget::FRAMEBUFFER,
                    FramebufferAttachment::COLOR_ATTACHMENT0,
                    TextureInternalFormat::RGBA,
                    TextureFormat::RGBA,
                    TextureDataType::UNSIGNED_BYTE,
                    0,
                )],
                [],
            ),
            in_texture,
            out_texture,
        }
    }
}

impl Executor for GaussianBlur {
    type Error = Error;

    fn before(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        _: &mut Resources,
    ) -> Result<bool, Self::Error> {
        if self.compiled.is_none() {
            let vertex_shader = compile_shaders(
                state.gl(),
                &ShaderSource::VertexRaw(include_str!("./shaders/gaussian_blur.vert")),
            )?;
            let fragment_shader = compile_shaders(
                state.gl(),
                &ShaderSource::FragmentRaw(include_str!("./shaders/gaussian_blur.frag")),
            )?;
            let program = create_program(
                state.gl(),
                &[vertex_shader.clone(), fragment_shader.clone()],
            )?;
            let position_location = state.gl().get_attrib_location(&program, "a_Position") as u32;
            let texture_location = state.gl().get_attrib_location(&program, "a_TexCoord") as u32;
            let sampler_location = state
                .gl()
                .get_uniform_location(&program, "u_Sampler")
                .unwrap();

            let kernel_location = state.gl().get_uniform_block_index(&program, "Kernel");

            let attributes_buffer = state
                .gl()
                .create_buffer()
                .ok_or(Error::CreateBufferFailed)?;
            state.gl().bind_buffer(
                WebGl2RenderingContext::ARRAY_BUFFER,
                Some(&attributes_buffer),
            );
            state.gl().buffer_data_with_u8_array(
                WebGl2RenderingContext::ARRAY_BUFFER,
                unsafe { &std::mem::transmute_copy::<[f32; 16], [u8; 64]>(&BUFFER_DATA) },
                WebGl2RenderingContext::STATIC_DRAW,
            );

            let uniform_buffer = state
                .gl()
                .create_buffer()
                .ok_or(Error::CreateBufferFailed)?;
            // creates padded uniform buffer data for kernel
            let kernel_data = {
                let kernel_data = Float32Array::new_with_length(81 * 4);
                // pads kernel weights
                for (i, v) in GAUSSIAN_KERNEL.iter().enumerate() {
                    kernel_data.set_index((i * 4 + 0) as u32, *v);
                }

                kernel_data
            };
            state.gl().bind_buffer(
                WebGl2RenderingContext::UNIFORM_BUFFER,
                Some(&uniform_buffer),
            );
            state.gl().buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::UNIFORM_BUFFER,
                &kernel_data,
                WebGl2RenderingContext::STATIC_DRAW,
            );
            state
                .gl()
                .bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, None);

            self.compiled = Some(Compiled {
                program,
                position_location,
                texture_location,
                sampler_location,
                kernel_location,
                attributes_buffer,
                uniform_buffer,
            });
        }

        let Compiled {
            program,
            position_location,
            texture_location,
            kernel_location,
            sampler_location,
            attributes_buffer,
            uniform_buffer,
        } = self.compiled.as_ref().unwrap();

        state.gl().use_program(Some(program));

        state.gl().bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(attributes_buffer),
        );
        state.gl().vertex_attrib_pointer_with_i32(
            *position_location,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        state.gl().enable_vertex_attrib_array(*position_location);
        state.gl().vertex_attrib_pointer_with_i32(
            *texture_location,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            32,
        );
        state.gl().enable_vertex_attrib_array(*texture_location);
        state.gl().uniform1i(Some(sampler_location), 0);

        state
            .gl()
            .uniform_block_binding(program, *kernel_location, UBO_GAUSSIAN_BLUR_BINDING);
        state.gl().bind_buffer_base(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            UBO_GAUSSIAN_BLUR_BINDING,
            Some(&uniform_buffer),
        );

        Ok(true)
    }

    fn after(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        _: &mut Resources,
    ) -> Result<(), Self::Error> {
        let Compiled {
            position_location,
            texture_location,
            ..
        } = self.compiled.as_ref().unwrap();

        state.gl().disable_vertex_attrib_array(*position_location);
        state.gl().disable_vertex_attrib_array(*texture_location);

        Ok(())
    }

    fn execute(
        &mut self,
        state: &mut State,
        _: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<(), Self::Error> {
        let Some(in_texture) = resources.get(&self.in_texture) else {
            return Ok(());
        };

        for i in 0..self.epoch {
            if i % 2 == 0 {
                self.onepass_frame.bind(state.gl())?;

                if i == 0 {
                    state
                        .gl()
                        .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(in_texture));
                } else {
                    state.gl().bind_texture(
                        WebGl2RenderingContext::TEXTURE_2D,
                        Some(
                            &self
                                .twopass_frame
                                .textures()
                                .unwrap()
                                .get(0)
                                .as_ref()
                                .unwrap()
                                .0,
                        ),
                    );
                }
            } else {
                self.twopass_frame.bind(state.gl())?;

                state.gl().bind_texture(
                    WebGl2RenderingContext::TEXTURE_2D,
                    Some(
                        &self
                            .onepass_frame
                            .textures()
                            .unwrap()
                            .get(0)
                            .as_ref()
                            .unwrap()
                            .0,
                    ),
                );
            }

            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                WebGl2RenderingContext::NEAREST as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_S,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );
            state.gl().tex_parameteri(
                WebGl2RenderingContext::TEXTURE_2D,
                WebGl2RenderingContext::TEXTURE_WRAP_T,
                WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
            );

            state
                .gl()
                .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);
        }

        if self.epoch % 2 == 0 {
            resources.insert(
                self.out_texture.clone(),
                self.onepass_frame
                    .textures()
                    .unwrap()
                    .get(0)
                    .as_ref()
                    .unwrap()
                    .0
                    .clone(),
            );
        } else {
            resources.insert(
                self.out_texture.clone(),
                self.twopass_frame
                    .textures()
                    .unwrap()
                    .get(0)
                    .as_ref()
                    .unwrap()
                    .0
                    .clone(),
            );
        }

        Ok(())
    }
}
