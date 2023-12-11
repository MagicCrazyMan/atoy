use std::{any::Any, collections::HashMap, sync::OnceLock};

use wasm_bindgen::{Clamped, JsCast};
use web_sys::{
    js_sys::Float32Array, CanvasRenderingContext2d, HtmlCanvasElement, ImageData,
    WebGl2RenderingContext, WebGlFramebuffer, WebGlTexture,
};

use crate::{
    bounding::BoundingVolumeNative,
    document,
    entity::{BorrowedMut, Weak},
    geometry::Geometry,
    material::{Material, Transparency},
    render::webgl::{
        attribute::{bind_attributes, AttributeBinding, AttributeValue},
        buffer::{
            BufferComponentSize, BufferDataType, BufferDescriptor, BufferSource, BufferTarget,
            BufferUsage, MemoryPolicy,
        },
        draw::{draw, Draw, DrawMode},
        error::Error,
        program::ShaderSource,
        uniform::{
            bind_uniforms, UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue,
        },
    },
};

use super::{Executor, ResourceSource, State, Stuff};

pub struct Outlining {
    entity: ResourceSource,
    outline_color: [f32; 4],
    outline_material: OutliningMaterial,
    outline_blur_geometry: OutliningBlurGeometry,
    outline_blur_material: OutliningBlurMaterial,
    blur_times: usize,
    framebuffer: Option<WebGlFramebuffer>,
    depth_stencil_texture: Option<(WebGlTexture, u32, u32)>,
    color_texture: Option<(WebGlTexture, u32, u32)>,

    blur_h_framebuffer: Option<WebGlFramebuffer>,
    blur_h_texture: Option<(WebGlTexture, u32, u32)>,
    blur_v_framebuffer: Option<WebGlFramebuffer>,
    blur_v_texture: Option<(WebGlTexture, u32, u32)>,
    test: bool,
}

impl Outlining {
    pub fn new(entity: ResourceSource) -> Self {
        Self {
            entity,
            outline_color: [1.0, 0.0, 0.0, 1.0],
            outline_material: OutliningMaterial {
                outline_width: 10,
                outline_color: [0.0, 0.0, 0.0, 0.0],
                should_scale: true,
            },
            outline_blur_geometry: OutliningBlurGeometry::new(),
            outline_blur_material: OutliningBlurMaterial::new(),
            blur_times: 2 * 5,
            framebuffer: None,
            depth_stencil_texture: None,
            color_texture: None,
            blur_h_framebuffer: None,
            blur_h_texture: None,
            blur_v_framebuffer: None,
            blur_v_texture: None,
            test: false,
        }
    }

    fn use_framebuffer(&mut self, state: &State) -> Result<WebGlFramebuffer, Error> {
        let framebuffer = &mut self.framebuffer;
        let framebuffer = match framebuffer {
            Some(framebuffer) => framebuffer.clone(),
            None => {
                let fbo = state
                    .gl
                    .create_framebuffer()
                    .ok_or(Error::CreateFramebufferFailure)?;
                *framebuffer = Some(fbo.clone());
                fbo
            }
        };

        Ok(framebuffer)
    }

    fn use_blur_onepass_framebuffer(&mut self, state: &State) -> Result<WebGlFramebuffer, Error> {
        let framebuffer = &mut self.blur_h_framebuffer;
        let framebuffer = match framebuffer {
            Some(framebuffer) => framebuffer.clone(),
            None => {
                let fbo = state
                    .gl
                    .create_framebuffer()
                    .ok_or(Error::CreateFramebufferFailure)?;
                *framebuffer = Some(fbo.clone());
                fbo
            }
        };

        Ok(framebuffer)
    }

    fn use_blur_twopass_framebuffer(&mut self, state: &State) -> Result<WebGlFramebuffer, Error> {
        let framebuffer = &mut self.blur_v_framebuffer;
        let framebuffer = match framebuffer {
            Some(framebuffer) => framebuffer.clone(),
            None => {
                let fbo = state
                    .gl
                    .create_framebuffer()
                    .ok_or(Error::CreateFramebufferFailure)?;
                *framebuffer = Some(fbo.clone());
                fbo
            }
        };

        Ok(framebuffer)
    }

    fn use_depth_stencil_texture(&mut self, state: &State) -> Result<WebGlTexture, Error> {
        let w = state.canvas.width();
        let h = state.canvas.height();

        if let Some((texture, width, height)) = &self.depth_stencil_texture {
            if w == *width && h == *height {
                return Ok(texture.clone());
            } else {
                state.gl.delete_texture(Some(texture));
            }
        }

        let texture = state
            .gl
            .create_texture()
            .ok_or(Error::CreateTextureFailure)?;

        state.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        state
            .gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::DEPTH32F_STENCIL8 as i32,
                w as i32,
                h as i32,
                0,
                WebGl2RenderingContext::DEPTH_STENCIL,
                WebGl2RenderingContext::FLOAT_32_UNSIGNED_INT_24_8_REV,
                None,
            )
            .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        self.depth_stencil_texture = Some((texture.clone(), w, h));

        Ok(texture)
    }

    fn use_color_texture(&mut self, state: &State) -> Result<WebGlTexture, Error> {
        let w = state.canvas.width();
        let h = state.canvas.height();

        if let Some((texture, width, height)) = &self.color_texture {
            if w == *width && h == *height {
                return Ok(texture.clone());
            } else {
                state.gl.delete_texture(Some(texture));
            }
        }

        let tx = state
            .gl
            .create_texture()
            .ok_or(Error::CreateTextureFailure)?;

        state.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&tx));

        state
            .gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                w as i32,
                h as i32,
                0,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                None,
            )
            .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        self.color_texture = Some((tx.clone(), w, h));

        Ok(tx)
    }

    fn use_blur_onepass_texture(&mut self, state: &State) -> Result<WebGlTexture, Error> {
        let w = state.canvas.width();
        let h = state.canvas.height();

        if let Some((texture, width, height)) = &self.blur_h_texture {
            if w == *width && h == *height {
                return Ok(texture.clone());
            } else {
                state.gl.delete_texture(Some(texture));
            }
        }

        let texture = state
            .gl
            .create_texture()
            .ok_or(Error::CreateTextureFailure)?;

        state.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        state
            .gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                w as i32,
                h as i32,
                0,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                None,
            )
            .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        self.blur_h_texture = Some((texture.clone(), w, h));

        Ok(texture)
    }

    fn use_blur_twopass_texture(&mut self, state: &State) -> Result<WebGlTexture, Error> {
        let w = state.canvas.width();
        let h = state.canvas.height();

        if let Some((texture, width, height)) = &self.blur_v_texture {
            if w == *width && h == *height {
                return Ok(texture.clone());
            } else {
                state.gl.delete_texture(Some(texture));
            }
        }

        let texture = state
            .gl
            .create_texture()
            .ok_or(Error::CreateTextureFailure)?;

        state.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        state
            .gl
            .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                w as i32,
                h as i32,
                0,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                None,
            )
            .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        self.blur_v_texture = Some((texture.clone(), w, h));

        Ok(texture)
    }
}

impl Executor for Outlining {
    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        runtime_resources: &mut HashMap<String, Box<dyn Any>>,
        persist_resources: &mut HashMap<String, Box<dyn Any>>,
    ) -> Result<(), Error> {
        if self.test {
            return Ok(());
        }

        let entity = match &self.entity {
            ResourceSource::Runtime(key) => runtime_resources.get(key.as_str()),
            ResourceSource::Persist(key) => persist_resources.get(key.as_str()),
        };
        let Some(entity) = entity
            .and_then(|resource| resource.downcast_ref::<Weak>())
            .and_then(|e| e.upgrade())
        else {
            return Ok(());
        };

        let entity = entity.borrow_mut();
        let Some(geometry) = entity.geometry() else {
            return Ok(());
        };

        // setups framebuffer
        let framebuffer = self.use_framebuffer(state)?;
        let depth_stencil_texture = self.use_depth_stencil_texture(state)?;
        let color_texture = self.use_color_texture(state)?;
        state.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        state
            .gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&framebuffer));
        state
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&color_texture));
        state.gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::COLOR_ATTACHMENT0,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&color_texture),
            0,
        );
        state.gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&depth_stencil_texture),
        );
        state.gl.framebuffer_texture_2d(
            WebGl2RenderingContext::FRAMEBUFFER,
            WebGl2RenderingContext::DEPTH_STENCIL_ATTACHMENT,
            WebGl2RenderingContext::TEXTURE_2D,
            Some(&depth_stencil_texture),
            0,
        );

        state.gl.clear_bufferfv_with_f32_array(
            WebGl2RenderingContext::COLOR,
            0,
            &[0.0, 0.0, 0.0, 0.0],
        );
        state
            .gl
            .clear_bufferfi(WebGl2RenderingContext::DEPTH_STENCIL, 0, 1.0, 0);

        // draws outline into framebuffer
        {
            // prepares material
            let program = state.program_store.use_program(&self.outline_material)?;
            state.gl.use_program(Some(program.gl_program()));

            // setups webgl
            state.gl.enable(WebGl2RenderingContext::STENCIL_TEST);

            // only have to binds attribute once
            let items = bind_attributes(state, &entity, geometry, &self.outline_material, &program);

            // one pass, enable stencil test, disable depth test, draw entity with scaling up, sets stencil values to 1
            {
                self.outline_material.should_scale = true;
                self.outline_material.outline_color = [0.0, 0.0, 0.0, 0.0];

                state.gl.depth_mask(false);
                state.gl.stencil_mask(0xFF);
                state
                    .gl
                    .stencil_func(WebGl2RenderingContext::ALWAYS, 1, 0xff);
                state.gl.stencil_op(
                    WebGl2RenderingContext::KEEP,
                    WebGl2RenderingContext::REPLACE,
                    WebGl2RenderingContext::REPLACE,
                );

                bind_uniforms(
                    state,
                    stuff,
                    &entity,
                    geometry,
                    &self.outline_material,
                    &program,
                );
                draw(state, geometry, &self.outline_material);
            }

            // two pass, enable stencil test, disable depth test, draw entity with no scaling, sets stencil values to 0
            {
                self.outline_material.should_scale = false;
                self.outline_material.outline_color = [0.0, 0.0, 0.0, 0.0];

                state.gl.depth_mask(false);
                state.gl.stencil_mask(0xFF);
                state
                    .gl
                    .stencil_func(WebGl2RenderingContext::ALWAYS, 0, 0xff);
                state.gl.stencil_op(
                    WebGl2RenderingContext::KEEP,
                    WebGl2RenderingContext::REPLACE,
                    WebGl2RenderingContext::REPLACE,
                );

                bind_uniforms(
                    state,
                    stuff,
                    &entity,
                    geometry,
                    &self.outline_material,
                    &program,
                );
                draw(state, geometry, &self.outline_material);
            }

            // three pass, disable stencil test, enable depth test, draw entity with scaling up, draws depth of where stencil value is 1
            {
                self.outline_material.should_scale = true;
                self.outline_material.outline_color = self.outline_color;

                state.gl.depth_mask(true);
                state.gl.stencil_mask(0);
                state
                    .gl
                    .stencil_func(WebGl2RenderingContext::EQUAL, 1, 0xff);
                state.gl.stencil_op(
                    WebGl2RenderingContext::KEEP,
                    WebGl2RenderingContext::KEEP,
                    WebGl2RenderingContext::KEEP,
                );

                bind_uniforms(
                    state,
                    stuff,
                    &entity,
                    geometry,
                    &self.outline_material,
                    &program,
                );
                draw(state, geometry, &self.outline_material);
            }

            // resets webgl
            state.gl.disable(WebGl2RenderingContext::STENCIL_TEST);
            state
                .gl
                .stencil_func(WebGl2RenderingContext::EQUAL, 0, 0xff);

            drop(items);
        }

        // let mut binary =
        //     vec![0u8; 4 * 1 * state.canvas.width() as usize * state.canvas.height() as usize];
        // state
        //     .gl
        //     .read_pixels_with_u8_array_and_dst_offset(
        //         0,
        //         0,
        //         state.canvas.width() as i32,
        //         state.canvas.height() as i32,
        //         WebGl2RenderingContext::RGBA,
        //         WebGl2RenderingContext::UNSIGNED_BYTE,
        //         &mut binary,
        //         0,
        //     )
        //     .unwrap();
        // let image = ImageData::new_with_u8_clamped_array_and_sh(
        //     Clamped(&binary),
        //     state.canvas.width(),
        //     state.canvas.height(),
        // )
        // .unwrap();
        // let canvas = document()
        //     .create_element("canvas")
        //     .unwrap()
        //     .dyn_into::<HtmlCanvasElement>()
        //     .unwrap();
        // canvas.set_width(state.canvas.width());
        // canvas.set_height(state.canvas.height());
        // let ctx = canvas
        //     .get_context("2d")
        //     .unwrap()
        //     .unwrap()
        //     .dyn_into::<CanvasRenderingContext2d>()
        //     .unwrap();
        // ctx.put_image_data(&image, 0.0, 0.0).unwrap();
        // document().body().unwrap().append_child(&canvas).unwrap();
        // self.test = true;

        // draws gaussian blur
        {
            state.gl.disable(WebGl2RenderingContext::DEPTH_TEST);
            state.gl.disable(WebGl2RenderingContext::BLEND);

            let blur_onepass_framebuffer = self.use_blur_onepass_framebuffer(state).unwrap();
            let blur_onepass_texture = self.use_blur_onepass_texture(state).unwrap();
            state.gl.bind_framebuffer(
                WebGl2RenderingContext::FRAMEBUFFER,
                Some(&blur_onepass_framebuffer),
            );
            state.gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&blur_onepass_texture),
            );
            state.gl.framebuffer_texture_2d(
                WebGl2RenderingContext::FRAMEBUFFER,
                WebGl2RenderingContext::COLOR_ATTACHMENT0,
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&blur_onepass_texture),
                0,
            );

            let blur_twopass_texture = self.use_blur_twopass_texture(state).unwrap();
            state.gl.bind_texture(
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&blur_twopass_texture),
            );
            let blur_twopass_framebuffer = self.use_blur_twopass_framebuffer(state).unwrap();
            state.gl.bind_framebuffer(
                WebGl2RenderingContext::FRAMEBUFFER,
                Some(&blur_twopass_framebuffer),
            );
            state.gl.framebuffer_texture_2d(
                WebGl2RenderingContext::FRAMEBUFFER,
                WebGl2RenderingContext::COLOR_ATTACHMENT0,
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&blur_twopass_texture),
                0,
            );

            state.gl.disable(WebGl2RenderingContext::DEPTH_TEST);

            // prepares material
            let program = state
                .program_store
                .use_program(&self.outline_blur_material)?;
            state.gl.use_program(Some(program.gl_program()));

            // only have to bind attribute once
            let items = bind_attributes(
                state,
                &entity,
                &self.outline_blur_geometry,
                &self.outline_blur_material,
                &program,
            );

            for i in 0..self.blur_times {
                if i % 2 == 0 {
                    state.gl.bind_framebuffer(
                        WebGl2RenderingContext::FRAMEBUFFER,
                        Some(&blur_onepass_framebuffer),
                    );

                    if i == 0 {
                        // use color texture for the first time
                        state
                            .gl
                            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&color_texture));
                    } else {
                        state.gl.bind_texture(
                            WebGl2RenderingContext::TEXTURE_2D,
                            Some(&blur_twopass_texture),
                        );
                    }
                } else {
                    if i == self.blur_times - 1 {
                        // for the last draw, we draw it to canvas framebuffer
                        state
                            .gl
                            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
                    } else {
                        state.gl.bind_framebuffer(
                            WebGl2RenderingContext::FRAMEBUFFER,
                            Some(&blur_twopass_framebuffer),
                        );
                    }
                    state.gl.bind_texture(
                        WebGl2RenderingContext::TEXTURE_2D,
                        Some(&blur_onepass_texture),
                    );
                }

                state.gl.clear_bufferfv_with_f32_array(
                    WebGl2RenderingContext::COLOR,
                    0,
                    &[0.0, 0.0, 0.0, 0.0],
                );

                state.gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
                    WebGl2RenderingContext::LINEAR as i32,
                );
                state.gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
                    WebGl2RenderingContext::LINEAR as i32,
                );
                state.gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    WebGl2RenderingContext::TEXTURE_WRAP_S,
                    WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
                );
                state.gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    WebGl2RenderingContext::TEXTURE_WRAP_T,
                    WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
                );
                bind_uniforms(
                    state,
                    stuff,
                    &entity,
                    &self.outline_blur_geometry,
                    &self.outline_blur_material,
                    &program,
                );
                draw(
                    state,
                    &self.outline_blur_geometry,
                    &self.outline_blur_material,
                );
            }

            state
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            // enable depth test
            state.gl.enable(WebGl2RenderingContext::DEPTH_TEST);
            state.gl.enable(WebGl2RenderingContext::BLEND);

            drop(items);
        }

        Ok(())
    }
}

struct OutliningMaterial {
    outline_width: u32,
    outline_color: [f32; 4],
    should_scale: bool,
}

impl Material for OutliningMaterial {
    fn name(&self) -> &'static str {
        "OutliningMaterial"
    }

    fn transparency(&self) -> Transparency {
        Transparency::Opaque
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[AttributeBinding::GeometryPosition]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        if self.should_scale {
            &[
                UniformBinding::ModelMatrix,
                UniformBinding::ViewProjMatrix,
                UniformBinding::FromMaterial("u_Color"),
                UniformBinding::FromMaterial("u_ShouldScale"),
                UniformBinding::CanvasSize,
                UniformBinding::FromMaterial("u_OutlineWidth"),
            ]
        } else {
            &[
                UniformBinding::ModelMatrix,
                UniformBinding::ViewProjMatrix,
                UniformBinding::FromMaterial("u_Color"),
                UniformBinding::FromMaterial("u_ShouldScale"),
            ]
        }
    }

    fn sources<'a>(&'a self) -> &[ShaderSource<'a>] {
        &[
            ShaderSource::Vertex(include_str!("./outlining.vert")),
            ShaderSource::Fragment(include_str!("./outlining.frag")),
        ]
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &BorrowedMut) -> Option<UniformValue> {
        match name {
            "u_ShouldScale" => Some(UniformValue::UnsignedInteger1(if self.should_scale {
                1
            } else {
                0
            })),
            "u_Color" => Some(UniformValue::FloatVector4(self.outline_color)),
            "u_OutlineWidth" => Some(UniformValue::UnsignedInteger1(self.outline_width)),
            _ => None,
        }
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct OutliningBlurGeometry {
    vertices: AttributeValue,
    texture_coordinates: AttributeValue,
}

impl OutliningBlurGeometry {
    fn new() -> Self {
        #[rustfmt::skip]
        const VERTICES: [f32; 12] = [
            1.0,-1.0,  1.0, 1.0, -1.0, 1.0,
           -1.0, 1.0, -1.0,-1.0,  1.0,-1.0,
        ];
        #[rustfmt::skip]
        const TEXTURE_COORDINATES: [f32; 12] = [
            1.0, 0.0,  1.0, 1.0,  0.0, 1.0,
            0.0, 1.0,  0.0, 0.0,  1.0, 0.0,
        ];

        Self {
            vertices: AttributeValue::Buffer {
                descriptor: BufferDescriptor::new(
                    BufferSource::from_binary(
                        unsafe { std::mem::transmute_copy::<[f32; 12], [u8; 48]>(&VERTICES) },
                        0,
                        48,
                    ),
                    BufferUsage::StaticDraw,
                ),
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::Two,
                data_type: BufferDataType::Float,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: 0,
            },
            texture_coordinates: AttributeValue::Buffer {
                descriptor: BufferDescriptor::new(
                    BufferSource::from_binary(
                        unsafe {
                            std::mem::transmute_copy::<[f32; 12], [u8; 48]>(&TEXTURE_COORDINATES)
                        },
                        0,
                        48,
                    ),
                    BufferUsage::StaticDraw,
                ),
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::Two,
                data_type: BufferDataType::Float,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: 0,
            },
        }
    }
}

impl Geometry for OutliningBlurGeometry {
    fn draw(&self) -> Draw {
        Draw::Arrays {
            mode: DrawMode::Triangles,
            first: 0,
            count: 6,
        }
    }

    fn bounding_volume_native(&self) -> Option<BoundingVolumeNative> {
        None
    }

    fn vertices(&self) -> Option<AttributeValue> {
        Some(self.vertices.clone())
    }

    fn normals(&self) -> Option<AttributeValue> {
        None
    }

    fn texture_coordinates(&self) -> Option<AttributeValue> {
        Some(self.texture_coordinates.clone())
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, _: &str, _: &BorrowedMut) -> Option<UniformValue> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[rustfmt::skip]
const GAUSSIAN_KERNEL: [f32; 81] = [
    0.000262958656, 0.000876539664, 0.0019722158656, 0.0031555460336000003, 0.0036814698320000003, 0.0031555460336000003, 0.0019722158656, 0.000876539664, 0.000262958656,
    0.000876539664, 0.0029218349159999997, 0.006574133966399999, 0.0105186165084, 0.012271717458, 0.0105186165084, 0.006574133966399999, 0.0029218349159999997, 0.000876539664,
    0.0019722158656, 0.006574133966399999, 0.01479181358656, 0.02366690660336, 0.0276113869832, 0.02366690660336, 0.01479181358656, 0.006574133966399999, 0.0019722158656,
    0.0031555460336000003, 0.0105186165084, 0.02366690660336, 0.03786705834916, 0.0441782282542, 0.03786705834916, 0.02366690660336, 0.0105186165084, 0.0031555460336000003,
    0.0036814698320000003, 0.012271717458, 0.0276113869832, 0.0441782282542, 0.051541258729000006, 0.0441782282542, 0.0276113869832, 0.012271717458, 0.0036814698320000003,
    0.0031555460336000003, 0.0105186165084, 0.02366690660336, 0.03786705834916, 0.0441782282542, 0.03786705834916, 0.02366690660336, 0.0105186165084, 0.0031555460336000003,
    0.0019722158656, 0.006574133966399999, 0.01479181358656, 0.02366690660336, 0.0276113869832, 0.02366690660336, 0.01479181358656, 0.006574133966399999, 0.0019722158656,
    0.000876539664, 0.0029218349159999997, 0.006574133966399999, 0.0105186165084, 0.012271717458, 0.0105186165084, 0.006574133966399999, 0.0029218349159999997, 0.000876539664,
    0.000262958656, 0.000876539664, 0.0019722158656, 0.0031555460336000003, 0.0036814698320000003, 0.0031555460336000003, 0.0019722158656, 0.000876539664, 0.000262958656,
];

struct OutliningBlurMaterial {
    kernel: UniformBlockValue,
}

impl OutliningBlurMaterial {
    fn new() -> Self {
        // creates padded uniform buffer data for kernel
        let kernel_uniform_buffer = {
            let kernel_uniform_buffer = Float32Array::new_with_length(81 * 4);
            // pads kernel weights
            for (i, v) in GAUSSIAN_KERNEL.iter().enumerate() {
                kernel_uniform_buffer.set_index((i * 4 + 0) as u32, *v);
            }

            kernel_uniform_buffer
        };

        Self {
            kernel: UniformBlockValue::BufferBase {
                descriptor: BufferDescriptor::with_memory_policy(
                    BufferSource::from_float32_array(kernel_uniform_buffer.clone(), 0, 81 * 4),
                    // BufferSource::from_binary([0; 81], 0, 81 * 4 * 4),
                    BufferUsage::StaticDraw,
                    MemoryPolicy::restorable(move || {
                        BufferSource::from_float32_array(kernel_uniform_buffer.clone(), 0, 81 * 4)
                    }),
                ),
                target: BufferTarget::UniformBuffer,
                binding: 0,
            },
        }
    }
}

impl Material for OutliningBlurMaterial {
    fn name(&self) -> &'static str {
        "OutliningBlurMaterial"
    }

    fn transparency(&self) -> Transparency {
        Transparency::Opaque
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryTextureCoordinate,
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[UniformBinding::FromMaterial("u_ColorSampler")]
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[UniformBlockBinding::FromMaterial("Kernel")]
    }

    fn sources(&self) -> &[ShaderSource] {
        &[
            ShaderSource::Vertex(include_str!("./outlining_blur.vert")),
            ShaderSource::Fragment(include_str!("./outlining_blur.frag")),
        ]
    }

    fn attribute_value(&self, _: &str, _: &BorrowedMut) -> Option<AttributeValue> {
        None
    }

    fn uniform_value(&self, name: &str, _: &BorrowedMut) -> Option<UniformValue> {
        match name {
            "u_ColorSampler" => Some(UniformValue::Integer1(0)),
            _ => None,
        }
    }

    fn uniform_block_value(&self, name: &str, _: &BorrowedMut) -> Option<UniformBlockValue> {
        match name {
            "Kernel" => Some(self.kernel.clone()),
            _ => None,
        }
    }

    fn ready(&self) -> bool {
        true
    }

    fn instanced(&self) -> Option<i32> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
