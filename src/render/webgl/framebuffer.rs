//! This module provides convenient functions to create
//! [`WebGlFramebuffer`], [`WebGlRenderbuffer`], [`WebGlTexture`] and other stuffs to draw.
//!
//! Unlike [`BufferStore`](super::buffer::BufferStore), [`TextureStore`](super::texture::TextureStore) and
//! [`ProgramStore`](super::program::ProgramStore), stuffs created from here does not manage automatically,
//! you should cleanups everything by yourself when finishing drawing.

use log::warn;
use wasm_bindgen::JsValue;
use web_sys::{
    js_sys::{Array, Object},
    WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture,
};

use super::{
    conversion::ToGlEnum,
    error::Error,
    renderbuffer::RenderbufferInternalFormat,
    texture::{TextureDataType, TextureFormat, TextureInternalFormat},
};

/// Available framebuffer size policies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FramebufferSizePolicy {
    FollowDrawingBuffer,
    ScaleDrawingBuffer(f64),
    Custom { width: i32, height: i32 },
}

impl FramebufferSizePolicy {
    fn size(&self, gl: &WebGl2RenderingContext) -> (i32, i32) {
        match self {
            Self::FollowDrawingBuffer => {
                let width = gl.drawing_buffer_width();
                let height = gl.drawing_buffer_height();
                (width, height)
            }
            Self::ScaleDrawingBuffer(scale) => {
                let width = gl.drawing_buffer_width();
                let height = gl.drawing_buffer_height();
                (
                    (width as f64 * scale).round() as i32,
                    (height as f64 * scale).round() as i32,
                )
            }
            Self::Custom { width, height } => (*width, *height),
        }
    }
}

/// Available framebuffer targets mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramebufferTarget {
    FRAMEBUFFER,
    READ_FRAMEBUFFER,
    DRAW_FRAMEBUFFER,
}

/// Available framebuffer attachments mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramebufferAttachment {
    COLOR_ATTACHMENT0,
    COLOR_ATTACHMENT1,
    COLOR_ATTACHMENT2,
    COLOR_ATTACHMENT3,
    COLOR_ATTACHMENT4,
    COLOR_ATTACHMENT5,
    COLOR_ATTACHMENT6,
    COLOR_ATTACHMENT7,
    COLOR_ATTACHMENT8,
    COLOR_ATTACHMENT9,
    COLOR_ATTACHMENT10,
    COLOR_ATTACHMENT11,
    COLOR_ATTACHMENT12,
    COLOR_ATTACHMENT13,
    COLOR_ATTACHMENT14,
    COLOR_ATTACHMENT15,
    DEPTH_ATTACHMENT,
    STENCIL_ATTACHMENT,
    DEPTH_STENCIL_ATTACHMENT,
}

impl FramebufferAttachment {
    fn to_draw_buffer_index(&self) -> i32 {
        match self {
            FramebufferAttachment::COLOR_ATTACHMENT0 => 0,
            FramebufferAttachment::COLOR_ATTACHMENT1 => 1,
            FramebufferAttachment::COLOR_ATTACHMENT2 => 2,
            FramebufferAttachment::COLOR_ATTACHMENT3 => 3,
            FramebufferAttachment::COLOR_ATTACHMENT4 => 4,
            FramebufferAttachment::COLOR_ATTACHMENT5 => 5,
            FramebufferAttachment::COLOR_ATTACHMENT6 => 6,
            FramebufferAttachment::COLOR_ATTACHMENT7 => 7,
            FramebufferAttachment::COLOR_ATTACHMENT8 => 8,
            FramebufferAttachment::COLOR_ATTACHMENT9 => 9,
            FramebufferAttachment::COLOR_ATTACHMENT10 => 10,
            FramebufferAttachment::COLOR_ATTACHMENT11 => 11,
            FramebufferAttachment::COLOR_ATTACHMENT12 => 12,
            FramebufferAttachment::COLOR_ATTACHMENT13 => 13,
            FramebufferAttachment::COLOR_ATTACHMENT14 => 14,
            FramebufferAttachment::COLOR_ATTACHMENT15 => 15,
            FramebufferAttachment::DEPTH_ATTACHMENT => 0,
            FramebufferAttachment::STENCIL_ATTACHMENT => 0,
            FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT => 0,
        }
    }
}

/// Available draw buffer source mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramebufferDrawBuffer {
    NONE,
    BACK,
    COLOR_ATTACHMENT0,
    COLOR_ATTACHMENT1,
    COLOR_ATTACHMENT2,
    COLOR_ATTACHMENT3,
    COLOR_ATTACHMENT4,
    COLOR_ATTACHMENT5,
    COLOR_ATTACHMENT6,
    COLOR_ATTACHMENT7,
    COLOR_ATTACHMENT8,
    COLOR_ATTACHMENT9,
    COLOR_ATTACHMENT10,
    COLOR_ATTACHMENT11,
    COLOR_ATTACHMENT12,
    COLOR_ATTACHMENT13,
    COLOR_ATTACHMENT14,
    COLOR_ATTACHMENT15,
}

/// Available blit framebuffer masks mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlitMask {
    COLOR_BUFFER_BIT,
    DEPTH_BUFFER_BIT,
    STENCIL_BUFFER_BIT,
}

/// Available blit framebuffer filters mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlitFlilter {
    NEAREST,
    LINEAR,
}

/// Offscreen frame containing [`WebGlFramebuffer`], [`WebGlRenderbuffer`], [`WebGlTexture`] and other stuffs
/// to make WebGl draw entities to framebuffer.
///
/// Offscreen frame holds a drawing buffer width and drawing buffer height from [`WebGl2RenderingContext`]
/// to ensure [`WebGlRenderbuffer`] and [`WebGlTexture`] always fit into a same size.
/// When width and height in [`WebGl2RenderingContext`] changed,
/// new [`WebGlRenderbuffer`] and [`WebGlTexture`] are recreated as well.
///
/// [`drawBuffers`](https://developer.mozilla.org/en-US/docs/Web/API/WebGL2RenderingContext/drawBuffers)
pub struct Framebuffer {
    gl: WebGl2RenderingContext,
    size_policy: FramebufferSizePolicy,

    texture_providers: Vec<TextureProvider>,
    renderbuffer_providers: Vec<RenderbufferProvider>,
    renderbuffer_samples: Option<i32>,
    draw_buffers: Array,

    size: Option<(i32, i32)>,
    framebuffer: Option<WebGlFramebuffer>,
    textures: Option<Vec<(WebGlTexture, FramebufferAttachment, ClearPolicy)>>,
    renderbuffers: Option<Vec<(WebGlRenderbuffer, FramebufferAttachment, ClearPolicy)>>,

    binding_target: Option<FramebufferTarget>,
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        let (Some(framebuffer), Some(textures), Some(renderbuffers)) = (
            self.framebuffer.as_ref(),
            self.textures.as_ref(),
            self.renderbuffers.as_ref(),
        ) else {
            return;
        };

        self.gl.delete_framebuffer(Some(framebuffer));
        textures
            .iter()
            .for_each(|(texture, _, _)| self.gl.delete_texture(Some(texture)));
        renderbuffers
            .iter()
            .for_each(|(renderbuffer, _, _)| self.gl.delete_renderbuffer(Some(renderbuffer)));
    }
}

impl Framebuffer {
    /// Constructs a new framebuffer object.
    ///
    /// Multisamples does not works on [`TextureProvider`] and [`RenderbufferProvider::FromExisting`].
    /// Size policy does not works on [`TextureProvider::FromExisting`] and [`RenderbufferProvider::FromExisting`].
    pub fn new<
        TI: IntoIterator<Item = TextureProvider>,
        RI: IntoIterator<Item = RenderbufferProvider>,
        DI: IntoIterator<Item = FramebufferDrawBuffer>,
    >(
        gl: WebGl2RenderingContext,
        size_policy: FramebufferSizePolicy,
        texture_providers: TI,
        renderbuffer_providers: RI,
        draw_buffers: DI,
        renderbuffer_samples: Option<i32>,
    ) -> Self {
        let draw_buffers_array = Array::new();
        for draw_buffer in draw_buffers.into_iter() {
            draw_buffers_array.push(&JsValue::from_f64(draw_buffer.gl_enum() as f64));
        }

        Self {
            gl,
            size_policy,

            size: None,
            texture_providers: texture_providers.into_iter().collect(),
            renderbuffer_providers: renderbuffer_providers.into_iter().collect(),
            renderbuffer_samples,
            draw_buffers: draw_buffers_array,

            binding_target: None,
            framebuffer: None,
            textures: None,
            renderbuffers: None,
        }
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    /// Clears framebuffer and its associated renderbuffers and textures.
    pub fn clear(&mut self) {
        if let Some(framebuffer) = &mut self.framebuffer {
            self.gl.delete_framebuffer(Some(&framebuffer));
        }

        if let Some(textures) = &mut self.textures {
            for (texture, _, _) in textures {
                self.gl.delete_texture(Some(&texture));
            }
        }

        if let Some(renderbuffers) = &mut self.renderbuffers {
            for (renderbuffer, _, _) in renderbuffers {
                self.gl.delete_renderbuffer(Some(&renderbuffer));
            }
        }

        self.size = None;
        self.framebuffer = None;
        self.textures = None;
        self.renderbuffers = None;
    }

    pub fn clear_buffers(&self) {
        if self.binding_target.is_none() {
            warn!("can not clear buffer bits of a unbound framebuffer");
            return;
        }

        if let Some(textures) = &self.textures {
            for (_, attachment, clear_policy) in textures {
                clear_policy.clear(&self.gl, attachment.to_draw_buffer_index());
            }
        }
        if let Some(renderbuffers) = &self.renderbuffers {
            for (_, attachment, clear_policy) in renderbuffers {
                clear_policy.clear(&self.gl, attachment.to_draw_buffer_index());
            }
        }
    }

    /// Binds framebuffer to WebGL.
    pub fn bind(&mut self, target: FramebufferTarget) -> Result<(), Error> {
        self.unbind();
        self.use_framebuffer(target)?;
        if self.draw_buffers.length() > 0 {
            self.gl.draw_buffers(&self.draw_buffers);
        }

        self.binding_target = Some(target);

        Ok(())
    }

    /// Unbinds framebuffer from WebGL.
    pub fn unbind(&mut self) {
        if let Some(binding_target) = self.binding_target.take() {
            self.gl.bind_framebuffer(binding_target.gl_enum(), None);
        }
    }

    /// Reads pixels.
    pub fn read_pixels(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: TextureFormat,
        data_type: TextureDataType,
        dst_data: &Object,
        dst_offset: u32,
    ) -> Result<(), Error> {
        self.read_pixels_with_read_buffer(
            FramebufferDrawBuffer::COLOR_ATTACHMENT0,
            x,
            y,
            width,
            height,
            format,
            data_type,
            dst_data,
            dst_offset,
        )
    }

    /// Reads pixels.
    pub fn read_pixels_with_read_buffer(
        &mut self,
        read_buffer: FramebufferDrawBuffer,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: TextureFormat,
        data_type: TextureDataType,
        dst_data: &Object,
        dst_offset: u32,
    ) -> Result<(), Error> {
        let Some(framebuffer) = self.framebuffer.as_ref() else {
            return Ok(());
        };
        self.gl
            .bind_framebuffer(WebGl2RenderingContext::READ_FRAMEBUFFER, Some(framebuffer));
        self.gl.read_buffer(read_buffer.gl_enum());
        self.gl
            .read_pixels_with_array_buffer_view_and_dst_offset(
                x,
                y,
                width,
                height,
                format.gl_enum(),
                data_type.gl_enum(),
                dst_data,
                dst_offset,
            )
            .or_else(|err| Err(Error::ReadPixelsFailure(err.as_string())))?;
        self.gl
            .bind_framebuffer(WebGl2RenderingContext::READ_FRAMEBUFFER, None);
        self.gl.read_buffer(WebGl2RenderingContext::BACK);
        Ok(())
    }

    /// Returns draw buffers associated with this framebuffer.
    pub fn draw_buffers(&self) -> &Array {
        &self.draw_buffers
    }

    /// Returns number of sample of the render buffers if multisample is enabled.
    pub fn renderbuffer_samples(&self) -> Option<i32> {
        self.renderbuffer_samples
            .and_then(|samples| if samples == 0 { None } else { Some(samples) })
    }

    /// Sets render buffer samples. Disabling multisamples by providing `0` or `None`.
    pub fn set_renderbuffer_samples(&mut self, samples: Option<i32>) {
        let samples = match samples {
            Some(samples) => {
                if samples == 0 {
                    None
                } else {
                    Some(samples)
                }
            }
            None => None,
        };
        if samples != self.renderbuffer_samples {
            self.renderbuffer_samples = samples;
            self.clear();
        }
    }

    fn verify_size_policy(&mut self) -> (i32, i32) {
        let Some((width, height)) = self.size else {
            return self.size_policy.size(&self.gl);
        };

        let (twidth, theight) = self.size_policy.size(&self.gl);
        if twidth != width || theight != height {
            self.clear();
        }

        (twidth, theight)
    }

    fn use_framebuffer(&mut self, target: FramebufferTarget) -> Result<(), Error> {
        let (width, height) = self.verify_size_policy();

        match self.framebuffer.as_ref() {
            Some(framebuffer) => {
                self.gl
                    .bind_framebuffer(target.gl_enum(), Some(&framebuffer));
            }
            None => {
                let framebuffer = self.create_framebuffer()?;
                self.gl
                    .bind_framebuffer(target.gl_enum(), Some(&framebuffer));
                self.create_textures(target, width, height)?;
                self.create_renderbuffers(target, width, height)?;

                self.framebuffer = Some(framebuffer);
                self.size = Some((width, height));
            }
        }

        Ok(())
    }

    fn create_framebuffer(&mut self) -> Result<WebGlFramebuffer, Error> {
        let framebuffer = self
            .gl
            .create_framebuffer()
            .ok_or(Error::CreateFramebufferFailure)?;
        Ok(framebuffer)
    }

    fn create_textures(
        &mut self,
        target: FramebufferTarget,
        width: i32,
        height: i32,
    ) -> Result<(), Error> {
        if self.textures.is_some() {
            return Ok(());
        }

        self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);

        let mut textures = Vec::with_capacity(self.texture_providers.len());
        for provider in &self.texture_providers {
            let (texture, attachment, clear_policy) = match provider {
                TextureProvider::FromNew {
                    attachment,
                    internal_format,
                    format,
                    data_type,
                    clear_policy,
                } => {
                    let texture = self
                        .gl
                        .create_texture()
                        .ok_or(Error::CreateTextureFailure)?;
                    self.gl
                        .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
                    self.gl
                        .tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                            WebGl2RenderingContext::TEXTURE_2D,
                            0,
                            internal_format.gl_enum() as i32,
                            width,
                            height,
                            0,
                            format.gl_enum(),
                            data_type.gl_enum(),
                            None,
                        )
                        .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;

                    (texture, *attachment, *clear_policy)
                }
                TextureProvider::FromExisting {
                    attachment,
                    texture,
                    clear_policy,
                } => {
                    self.gl
                        .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));

                    (texture.clone(), *attachment, *clear_policy)
                }
            };

            self.gl.framebuffer_texture_2d(
                target.gl_enum(),
                attachment.gl_enum(),
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&texture),
                0,
            );

            textures.push((texture, attachment, clear_policy));
        }

        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        self.textures = Some(textures);

        Ok(())
    }

    fn create_renderbuffers(
        &mut self,
        target: FramebufferTarget,
        width: i32,
        height: i32,
    ) -> Result<(), Error> {
        if self.renderbuffers.is_some() {
            return Ok(());
        }

        let mut renderbuffers = Vec::with_capacity(self.renderbuffer_providers.len());
        for provider in &self.renderbuffer_providers {
            let (renderbuffer, attachment, clear_policy) = match provider {
                RenderbufferProvider::FromNew {
                    attachment,
                    internal_format,
                    clear_policy,
                } => {
                    let renderbuffer = self
                        .gl
                        .create_renderbuffer()
                        .ok_or(Error::CreateRenderbufferFailure)?;
                    self.gl.bind_renderbuffer(
                        WebGl2RenderingContext::RENDERBUFFER,
                        Some(&renderbuffer),
                    );
                    match self.renderbuffer_samples() {
                        Some(samples) => self.gl.renderbuffer_storage_multisample(
                            WebGl2RenderingContext::RENDERBUFFER,
                            samples,
                            internal_format.gl_enum(),
                            width,
                            height,
                        ),
                        None => self.gl.renderbuffer_storage(
                            WebGl2RenderingContext::RENDERBUFFER,
                            internal_format.gl_enum(),
                            width,
                            height,
                        ),
                    }
                    (renderbuffer, *attachment, *clear_policy)
                }
                RenderbufferProvider::FromExisting {
                    attachment,
                    renderbuffer,
                    clear_policy,
                } => (renderbuffer.clone(), *attachment, *clear_policy),
            };

            self.gl.framebuffer_renderbuffer(
                target.gl_enum(),
                attachment.gl_enum(),
                WebGl2RenderingContext::RENDERBUFFER,
                Some(&renderbuffer),
            );

            renderbuffers.push((renderbuffer, attachment, clear_policy));
        }

        self.gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
        self.renderbuffers = Some(renderbuffers);

        Ok(())
    }

    /// Returns framebuffer width.
    pub fn width(&self) -> Option<i32> {
        self.size.map(|(width, _)| width)
    }

    /// Returns framebuffer height.
    pub fn height(&self) -> Option<i32> {
        self.size.map(|(_, height)| height)
    }

    /// Returns [`FramebufferTarget`] currently binding to this framebuffer.
    pub fn binding_target(&self) -> Option<FramebufferTarget> {
        self.binding_target.clone()
    }

    /// Returns [`WebGlFramebuffer`],
    pub fn framebuffer(&self) -> Option<&WebGlFramebuffer> {
        self.framebuffer.as_ref()
    }

    /// Returns a [`WebGlTexture`] by index.
    pub fn texture(&self, index: usize) -> Option<&WebGlTexture> {
        self.textures
            .as_ref()
            .and_then(|list| list.get(index))
            .map(|(texture, _, _)| texture)
    }

    /// Returns a [`WebGlRenderbuffer`] by index.
    pub fn renderbuffer(&self, index: usize) -> Option<&WebGlRenderbuffer> {
        self.renderbuffers
            .as_ref()
            .and_then(|list| list.get(index))
            .map(|(renderbuffer, _, _)| renderbuffer)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClearPolicy {
    ColorFloat([f32; 4]),
    ColorInteger([i32; 4]),
    ColorUnsignedInteger([u32; 4]),
    Depth(f32),
    Stencil(i32),
    DepthStencil(f32, i32),
}

impl ClearPolicy {
    fn clear(&self, gl: &WebGl2RenderingContext, draw_buffer: i32) {
        match self {
            ClearPolicy::ColorFloat(values) => {
                gl.clear_bufferfv_with_f32_array(WebGl2RenderingContext::COLOR, draw_buffer, values)
            }
            ClearPolicy::ColorInteger(values) => {
                gl.clear_bufferiv_with_i32_array(WebGl2RenderingContext::COLOR, draw_buffer, values)
            }
            ClearPolicy::ColorUnsignedInteger(values) => gl.clear_bufferuiv_with_u32_array(
                WebGl2RenderingContext::COLOR,
                draw_buffer,
                values,
            ),
            ClearPolicy::Depth(depth) => gl.clear_bufferfv_with_f32_array(
                WebGl2RenderingContext::DEPTH,
                draw_buffer,
                &[*depth],
            ),
            ClearPolicy::Stencil(stencil) => gl.clear_bufferiv_with_i32_array(
                WebGl2RenderingContext::STENCIL,
                draw_buffer,
                &[*stencil],
            ),
            ClearPolicy::DepthStencil(depth, stencil) => gl.clear_bufferfi(
                WebGl2RenderingContext::DEPTH_STENCIL,
                draw_buffer,
                *depth,
                *stencil,
            ),
        }
    }
}

/// Offscreen texture provider specifies texture configurations.
#[derive(Debug, Clone, PartialEq)]
pub enum TextureProvider {
    FromNew {
        attachment: FramebufferAttachment,
        internal_format: TextureInternalFormat,
        format: TextureFormat,
        data_type: TextureDataType,
        clear_policy: ClearPolicy,
    },
    FromExisting {
        attachment: FramebufferAttachment,
        texture: WebGlTexture,
        clear_policy: ClearPolicy,
    },
}

impl TextureProvider {
    /// Constructs a new texture provider by creating new [`WebGlTexture`].
    pub fn new(
        attachment: FramebufferAttachment,
        internal_format: TextureInternalFormat,
        format: TextureFormat,
        data_type: TextureDataType,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromNew {
            attachment,
            internal_format,
            format,
            data_type,
            clear_policy,
        }
    }

    /// Constructs a new renderbuffer provider by using a existing [`WebGlTexture`].
    pub fn from_existing(
        attachment: FramebufferAttachment,
        texture: WebGlTexture,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromExisting {
            attachment,
            texture,
            clear_policy,
        }
    }

    /// Returns [`ClearPolicy`] for this texture.
    pub fn clear_policy(&self) -> &ClearPolicy {
        match self {
            TextureProvider::FromNew { clear_policy, .. } => clear_policy,
            TextureProvider::FromExisting { clear_policy, .. } => clear_policy,
        }
    }
}

/// Offscreen renderbuffer provider specifies renderbuffer configurations.
#[derive(Debug, Clone, PartialEq)]
pub enum RenderbufferProvider {
    FromNew {
        attachment: FramebufferAttachment,
        internal_format: RenderbufferInternalFormat,
        clear_policy: ClearPolicy,
    },
    FromExisting {
        attachment: FramebufferAttachment,
        renderbuffer: WebGlRenderbuffer,
        clear_policy: ClearPolicy,
    },
}

impl RenderbufferProvider {
    /// Constructs a new renderbuffer provider by creating new [`WebGlRenderbuffer`].
    pub fn new(
        attachment: FramebufferAttachment,
        internal_format: RenderbufferInternalFormat,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromNew {
            internal_format,
            attachment,
            clear_policy,
        }
    }

    /// Constructs a new renderbuffer provider by using a existing [`WebGlRenderbuffer`].
    pub fn from_existing(
        attachment: FramebufferAttachment,
        renderbuffer: WebGlRenderbuffer,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromExisting {
            attachment,
            renderbuffer,
            clear_policy,
        }
    }

    /// Returns [`ClearPolicy`] for this renderbuffer.
    pub fn clear_policy(&self) -> &ClearPolicy {
        match self {
            RenderbufferProvider::FromNew { clear_policy, .. } => clear_policy,
            RenderbufferProvider::FromExisting { clear_policy, .. } => clear_policy,
        }
    }
}
