use std::{any::Any, collections::HashMap, sync::OnceLock};

use wasm_bindgen::{Clamped, JsCast};
use wasm_bindgen_test::console_log;
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
        attribute::{bind_attributes, AttributeBinding, AttributeValue, unbind_attributes},
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

use super::{Executor, ResourceKey, State, Stuff, Resources};

pub struct Outlining {
    entity: ResourceKey<Weak>,
    outline_material: OutliningMaterial,
    outline_blur_geometry: OutliningBlurGeometry,
    outline_blur_material: OutliningBlurMaterial,
    blur_times: usize,
    framebuffer: Option<WebGlFramebuffer>,
    depth_stencil_texture: Option<(WebGlTexture, u32, u32)>,
    texture_one: Option<(WebGlTexture, u32, u32)>,
    texture_two: Option<(WebGlTexture, u32, u32)>,

    blur_h_framebuffer: Option<WebGlFramebuffer>,
    blur_h_texture: Option<(WebGlTexture, u32, u32)>,
    blur_v_framebuffer: Option<WebGlFramebuffer>,
    blur_v_texture: Option<(WebGlTexture, u32, u32)>,
    test: bool,
}

impl Outlining {
    pub fn new(entity: ResourceKey<Weak>) -> Self {
        Self {
            entity,
            outline_material: OutliningMaterial {
                outline_color: [1.0, 0.0, 0.0, 1.0],
                stage: 0,
                outline_width: 5,
            },
            outline_blur_geometry: OutliningBlurGeometry::new(),
            outline_blur_material: OutliningBlurMaterial::new(),
            blur_times: 2 * 1,
            framebuffer: None,
            depth_stencil_texture: None,
            texture_one: None,
            texture_two: None,
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

    fn use_texture_one(&mut self, state: &State) -> Result<WebGlTexture, Error> {
        let w = state.canvas.width();
        let h = state.canvas.height();

        if let Some((texture, width, height)) = &self.texture_one {
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

        self.texture_one = Some((tx.clone(), w, h));

        Ok(tx)
    }

    fn use_texture_two(&mut self, state: &State) -> Result<WebGlTexture, Error> {
        let w = state.canvas.width();
        let h = state.canvas.height();

        if let Some((texture, width, height)) = &self.texture_two {
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

        self.texture_two = Some((tx.clone(), w, h));

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
    fn before(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<bool, Error> {
        if self.test {
            return Ok(false);
        }

        Ok(true)
    }

    fn execute(
        &mut self,
        state: &mut State,
        stuff: &mut dyn Stuff,
        resources: &mut Resources,
    ) -> Result<(), Error> {

        let Some(entity) = resources.get(&self.entity).and_then(|entity| entity.upgrade()) else {
            return Ok(());
        };

        let entity = entity.borrow_mut();
        let Some(geometry) = entity.geometry() else {
            return Ok(());
        };

        // setups framebuffer
        let framebuffer = self.use_framebuffer(state)?;
        let depth_stencil_texture = self.use_depth_stencil_texture(state)?;
        let texture_one = self.use_texture_one(state)?;
        let texture_two = self.use_texture_two(state)?;
        state.gl.active_texture(WebGl2RenderingContext::TEXTURE0);
        state
            .gl
            .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&framebuffer));
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

        state
            .gl
            .clear_bufferfi(WebGl2RenderingContext::DEPTH_STENCIL, 0, 1.0, 0);

        // draws outline into framebuffer
        {
            // prepares material
            let program = state.program_store.use_program(&self.outline_material)?;
            state.gl.use_program(Some(program.gl_program()));

            // draw once with outline color
            self.outline_material.stage = 0;
            state
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture_one));
            state.gl.framebuffer_texture_2d(
                WebGl2RenderingContext::FRAMEBUFFER,
                WebGl2RenderingContext::COLOR_ATTACHMENT0,
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&texture_one),
                0,
            );
            state
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            state.gl.clear_bufferfv_with_f32_array(
                WebGl2RenderingContext::COLOR,
                0,
                &[0.0, 0.0, 0.0, 0.0],
            );
            let _items =
                bind_attributes(state, &entity, geometry, &self.outline_material, &program);
            bind_uniforms(
                state,
                stuff,
                &entity,
                geometry,
                &self.outline_material,
                &program,
            );
            draw(state, geometry, &self.outline_material);

            // swap texture, draw outline with convolution
            self.outline_material.stage = 1;
            state
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture_two));
            state.gl.framebuffer_texture_2d(
                WebGl2RenderingContext::FRAMEBUFFER,
                WebGl2RenderingContext::COLOR_ATTACHMENT0,
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&texture_two),
                0,
            );
            state
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
            state.gl.clear_bufferfv_with_f32_array(
                WebGl2RenderingContext::COLOR,
                0,
                &[0.0, 0.0, 0.0, 0.0],
            );
            state
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture_one));
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
            let _items = bind_attributes(
                state,
                &entity,
                &self.outline_blur_geometry,
                &self.outline_material,
                &program,
            );
            bind_uniforms(
                state,
                stuff,
                &entity,
                &self.outline_blur_geometry,
                &self.outline_material,
                &program,
            );
            draw(state, &self.outline_blur_geometry, &self.outline_material);

            // swap texture again, clear base draw
            self.outline_material.stage = 2;
            let _items =
                bind_attributes(state, &entity, geometry, &self.outline_material, &program);
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
                            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture_two));
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

            unbind_attributes(state, items);
        }

        Ok(())
    }
}

struct OutliningMaterial {
    stage: i32,
    outline_color: [f32; 4],
    outline_width: i32,
}

impl Material for OutliningMaterial {
    fn name(&self) -> &'static str {
        "OutliningMaterial"
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
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::ViewProjMatrix,
            UniformBinding::FromMaterial("u_StageVertex"),
            UniformBinding::FromMaterial("u_StageFrag"),
            UniformBinding::FromMaterial("u_OutlineColor"),
            UniformBinding::FromMaterial("u_OutlineWidth"),
            UniformBinding::FromMaterial("u_OutlineSampler"),
        ]
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
            "u_StageVertex" => Some(UniformValue::Integer1(self.stage)),
            "u_StageFrag" => Some(UniformValue::Integer1(self.stage)),
            "u_OutlineColor" => Some(UniformValue::FloatVector4(self.outline_color)),
            "u_OutlineWidth" => Some(UniformValue::Integer1(self.outline_width)),
            "u_OutlineSampler" => Some(UniformValue::Integer1(0)),
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
        const COMPOSITIONS: [f32; 16] = [
            // vertices
            1.0,-1.0,  1.0, 1.0, -1.0, 1.0, -1.0,-1.0, 
            // textures
            1.0, 0.0,  1.0, 1.0,  0.0, 1.0,  0.0, 0.0,
        ];
        let descriptor  = BufferDescriptor::new(
            BufferSource::from_binary(
                unsafe { std::mem::transmute_copy::<[f32; 16], [u8; 64]>(&COMPOSITIONS) },
                0,
                64,
            ),
            BufferUsage::StaticDraw,
        );

        Self {
            vertices: AttributeValue::Buffer {
                descriptor: descriptor.clone(),
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::Two,
                data_type: BufferDataType::Float,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: 0,
            },
            texture_coordinates: AttributeValue::Buffer {
                descriptor,
                target: BufferTarget::ArrayBuffer,
                component_size: BufferComponentSize::Two,
                data_type: BufferDataType::Float,
                normalized: false,
                bytes_stride: 0,
                bytes_offset: 32,
            },
        }
    }
}

impl Geometry for OutliningBlurGeometry {
    fn draw(&self) -> Draw {
        Draw::Arrays {
            mode: DrawMode::TriangleFan,
            first: 0,
            count: 4,
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
                    BufferUsage::StaticDraw,
                    MemoryPolicy::from_restorable(move || {
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
