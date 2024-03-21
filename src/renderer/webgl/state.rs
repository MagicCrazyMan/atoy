use std::{iter::FromIterator, ptr::NonNull};

use wasm_bindgen::JsValue;
use web_sys::{
    js_sys::{Array, Object},
    HtmlCanvasElement, WebGl2RenderingContext, WebGlFramebuffer, WebGlTexture,
};

use crate::camera::Camera;

use super::{
    buffer::{BufferStore, BufferTarget},
    capabilities::Capabilities,
    conversion::ToGlEnum,
    draw::Draw,
    error::Error,
    framebuffer::{
        AttachmentProvider, BlitFlilter, BlitMask, Framebuffer, FramebufferAttachment,
        FramebufferBuilder, FramebufferTarget, OperableBuffer, SizePolicy,
    },
    params::GetWebGlParameters,
    program::ProgramStore,
    texture::{TextureStore, TextureUnit},
};

pub struct FrameState {
    timestamp: f64,
    gl: WebGl2RenderingContext,
    canvas: HtmlCanvasElement,
    camera: NonNull<(dyn Camera + 'static)>,

    program_store: NonNull<ProgramStore>,
    buffer_store: NonNull<BufferStore>,
    texture_store: NonNull<TextureStore>,
    capabilities: NonNull<Capabilities>,
}

impl FrameState {
    /// Constructs a new rendering state.
    pub(crate) fn new(
        timestamp: f64,
        camera: &mut (dyn Camera + 'static),
        gl: WebGl2RenderingContext,
        canvas: HtmlCanvasElement,
        program_store: &mut ProgramStore,
        buffer_store: &mut BufferStore,
        texture_store: &mut TextureStore,
        capabilities: &mut Capabilities,
    ) -> Self {
        unsafe {
            Self {
                timestamp,
                gl,
                canvas,
                camera: NonNull::new_unchecked(camera),
                program_store: NonNull::new_unchecked(program_store),
                buffer_store: NonNull::new_unchecked(buffer_store),
                texture_store: NonNull::new_unchecked(texture_store),
                capabilities: NonNull::new_unchecked(capabilities),
            }
        }
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn timestamp(&self) -> f64 {
        self.timestamp
    }

    /// Returns the [`Camera`].
    pub fn camera(&self) -> &dyn Camera {
        unsafe { self.camera.as_ref() }
    }

    /// Returns the [`ProgramStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn program_store(&self) -> &ProgramStore {
        unsafe { self.program_store.as_ref() }
    }

    /// Returns the mutable [`ProgramStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn program_store_mut(&mut self) -> &mut ProgramStore {
        unsafe { self.program_store.as_mut() }
    }

    /// Returns the [`BufferStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn buffer_store(&self) -> &BufferStore {
        unsafe { self.buffer_store.as_ref() }
    }

    /// Returns the [`TextureStore`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn texture_store(&self) -> &TextureStore {
        unsafe { self.texture_store.as_ref() }
    }

    /// Returns the [`Capabilities`] provided by the [`WebGL2Render`](crate::render::webgl::WebGL2Render).
    pub fn capabilities(&self) -> &Capabilities {
        unsafe { self.capabilities.as_ref() }
    }

    pub fn draw(&mut self, draw: &Draw) -> Result<(), Error> {
        match draw {
            Draw::Arrays { mode, first, count } => {
                self.gl.draw_arrays(mode.gl_enum(), *first, *count)
            }
            Draw::Elements {
                mode,
                count,
                offset,
                indices,
                indices_data_type,
            } => {
                self.buffer_store().register(&indices)?;

                indices.bind(BufferTarget::ELEMENT_ARRAY_BUFFER)?;
                self.gl.draw_elements_with_i32(
                    mode.gl_enum(),
                    *count,
                    indices_data_type.gl_enum(),
                    *offset,
                );
                indices.unbind(BufferTarget::ELEMENT_ARRAY_BUFFER)?;
            }
        }

        Ok(())
    }

    pub fn create_framebuffer<P>(
        &self,
        size_policy: SizePolicy,
        providers: P,
        renderbuffer_samples: Option<i32>,
    ) -> Framebuffer
    where
        P: IntoIterator<Item = (FramebufferAttachment, AttachmentProvider)>,
    {
        Framebuffer::new(
            self.gl.clone(),
            size_policy,
            providers,
            renderbuffer_samples,
        )
    }

    pub fn create_framebuffer_with_builder(&self, builder: FramebufferBuilder) -> Framebuffer {
        builder.build(self.gl.clone())
    }

    /// Reads pixels from current binding framebuffer.
    pub fn read_pixels(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: u32,
        type_: u32,
        dst_data: &Object,
        dst_offset: u32,
    ) -> Result<(), Error> {
        self.gl
            .read_pixels_with_array_buffer_view_and_dst_offset(
                x, y, width, height, format, type_, dst_data, dst_offset,
            )
            .or_else(|err| Err(Error::ReadPixelsFailure(err.as_string())))?;
        Ok(())
    }

    /// Applies computation using current binding framebuffer and program.
    pub fn do_computation<'a, I>(&self, textures: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = (&'a WebGlTexture, TextureUnit)>,
    {
        let sampler = self.capabilities().computation_sampler()?;
        let mut states = Vec::new();
        for (texture, unit) in textures {
            self.gl.active_texture(unit.gl_enum());
            let binding = self.gl.texture_binding_2d();
            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
            self.gl
                .bind_sampler(unit.unit_index() as u32, Some(&sampler));
            states.push((unit, binding));
        }

        self.gl
            .draw_arrays(WebGl2RenderingContext::TRIANGLE_FAN, 0, 4);

        for (unit, binding) in states {
            self.gl.active_texture(unit.gl_enum());
            self.gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, binding.as_ref());
            self.gl.bind_sampler(unit.unit_index() as u32, None);
        }

        Ok(())
    }

    /// Blits between read [`Framebuffer`] and draw [`Framebuffer`].
    pub fn blit_framebuffers(
        &self,
        read_framebuffer: &mut Framebuffer,
        draw_framebuffer: &mut Framebuffer,
        mask: BlitMask,
        filter: BlitFlilter,
    ) -> Result<(), Error> {
        draw_framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        let dst_width = draw_framebuffer
            .width()
            .ok_or(Error::FramebufferUninitialized)?;
        let dst_height = draw_framebuffer
            .height()
            .ok_or(Error::FramebufferUninitialized)?;

        read_framebuffer.bind(FramebufferTarget::READ_FRAMEBUFFER)?;
        let src_width = read_framebuffer
            .width()
            .ok_or(Error::FramebufferUninitialized)?;
        let src_height = read_framebuffer
            .height()
            .ok_or(Error::FramebufferUninitialized)?;

        self.gl.blit_framebuffer(
            0,
            0,
            src_width,
            src_height,
            0,
            0,
            dst_width,
            dst_height,
            mask.gl_enum(),
            filter.gl_enum(),
        );

        read_framebuffer.unbind();
        draw_framebuffer.unbind();

        Ok(())
    }

    /// Blits between read [`Framebuffer`] and draw [`Framebuffer`].
    pub fn blit_framebuffers_with_buffers<I>(
        &self,
        read_framebuffer: &mut Framebuffer,
        read_buffer: OperableBuffer,
        draw_framebuffer: &mut Framebuffer,
        draw_buffers: I,
        mask: BlitMask,
        filter: BlitFlilter,
    ) -> Result<(), Error>
    where
        I: IntoIterator<Item = OperableBuffer>,
    {
        draw_framebuffer.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        draw_framebuffer.set_draw_buffers(draw_buffers)?;
        read_framebuffer.bind(FramebufferTarget::READ_FRAMEBUFFER)?;
        read_framebuffer.set_read_buffer(read_buffer)?;
        let dst_width = draw_framebuffer
            .width()
            .ok_or(Error::FramebufferUninitialized)?;
        let dst_height = draw_framebuffer
            .height()
            .ok_or(Error::FramebufferUninitialized)?;
        let src_width = read_framebuffer
            .width()
            .ok_or(Error::FramebufferUninitialized)?;
        let src_height = read_framebuffer
            .height()
            .ok_or(Error::FramebufferUninitialized)?;

        self.gl.blit_framebuffer(
            0,
            0,
            src_width,
            src_height,
            0,
            0,
            dst_width,
            dst_height,
            mask.gl_enum(),
            filter.gl_enum(),
        );

        draw_framebuffer.unbind();
        read_framebuffer.unbind();

        Ok(())
    }

    /// Blits between read [`WebGlFramebuffer`](WebGlFramebuffer) and draw [`WebGlFramebuffer`](WebGlFramebuffer).
    pub fn blit_framebuffers_native<I1, I2>(
        &self,
        read_framebuffer: &WebGlFramebuffer,
        read_buffer: OperableBuffer,
        draw_framebuffer: &WebGlFramebuffer,
        draw_buffers: I1,
        reset_draw_buffers: I2,
        src_x0: i32,
        src_y0: i32,
        src_x1: i32,
        src_y1: i32,
        dst_x0: i32,
        dst_y0: i32,
        dst_x1: i32,
        dst_y1: i32,
        mask: BlitMask,
        filter: BlitFlilter,
    ) -> Result<(), Error>
    where
        I1: IntoIterator<Item = OperableBuffer>,
        I2: IntoIterator<Item = OperableBuffer>,
    {
        self.gl.bind_framebuffer(
            WebGl2RenderingContext::DRAW_FRAMEBUFFER,
            Some(draw_framebuffer),
        );
        self.gl.bind_framebuffer(
            WebGl2RenderingContext::READ_FRAMEBUFFER,
            Some(read_framebuffer),
        );

        let draw_buffers = Array::from_iter(
            draw_buffers
                .into_iter()
                .map(|v| JsValue::from_f64(v.gl_enum() as f64)),
        );
        self.gl.draw_buffers(&draw_buffers);
        self.gl.read_buffer(read_buffer.gl_enum());

        self.gl.blit_framebuffer(
            src_x0,
            src_y0,
            src_x1,
            src_y1,
            dst_x0,
            dst_y0,
            dst_x1,
            dst_y1,
            mask.gl_enum(),
            filter.gl_enum(),
        );

        let draw_buffers = Array::from_iter(
            reset_draw_buffers
                .into_iter()
                .map(|v| JsValue::from_f64(v.gl_enum() as f64)),
        );
        self.gl.draw_buffers(&draw_buffers);
        self.gl
            .bind_framebuffer(WebGl2RenderingContext::DRAW_FRAMEBUFFER, None);

        self.gl
            .bind_framebuffer(WebGl2RenderingContext::READ_FRAMEBUFFER, None);
        self.gl.read_buffer(WebGl2RenderingContext::BACK);

        Ok(())
    }
}
