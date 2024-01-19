//! This module provides convenient functions to create
//! [`WebGlFramebuffer`], [`WebGlRenderbuffer`], [`WebGlTexture`] and other stuffs to draw.
//!
//! Unlike [`BufferStore`](super::buffer::BufferStore), [`TextureStore`](super::texture::TextureStore) and
//! [`ProgramStore`](super::program::ProgramStore), stuffs created from here does not manage automatically,
//! you should cleanups everything by yourself when finishing drawing.

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

    width: i32,
    height: i32,

    texture_providers: Vec<TextureProvider>,
    renderbuffer_providers: Vec<RenderbufferProvider>,
    renderbuffer_samples: Option<i32>,

    binding_target: Option<FramebufferTarget>,
    framebuffer: Option<WebGlFramebuffer>,
    textures: Option<Vec<WebGlTexture>>,
    renderbuffers: Option<Vec<WebGlRenderbuffer>>,
    attachments: Vec<FramebufferAttachment>,

    draw_buffers: Array,
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
            .for_each(|texture| self.gl.delete_texture(Some(texture)));
        renderbuffers
            .iter()
            .for_each(|renderbuffer| self.gl.delete_renderbuffer(Some(renderbuffer)));
    }
}

impl Framebuffer {
    /// Constructs a new framebuffer object.
    ///
    /// Multisamples does not works on [`TextureProvider`] and [`RenderbufferProvider::FromExisting`].
    pub fn new<
        TI: IntoIterator<Item = TextureProvider>,
        RI: IntoIterator<Item = RenderbufferProvider>,
        DI: IntoIterator<Item = FramebufferDrawBuffer>,
    >(
        gl: WebGl2RenderingContext,
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

            width: 0,
            height: 0,

            texture_providers: texture_providers.into_iter().collect(),
            renderbuffer_providers: renderbuffer_providers.into_iter().collect(),
            renderbuffer_samples,

            binding_target: None,
            framebuffer: None,
            textures: None,
            renderbuffers: None,
            attachments: Vec::new(),

            draw_buffers: draw_buffers_array,
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
            for texture in textures {
                self.gl.delete_texture(Some(&texture));
            }
        }

        if let Some(renderbuffers) = &mut self.renderbuffers {
            for renderbuffer in renderbuffers {
                self.gl.delete_renderbuffer(Some(&renderbuffer));
            }
        }

        self.framebuffer = None;
        self.textures = None;
        self.renderbuffers = None;
        self.attachments.clear();
    }

    /// Binds framebuffer to WebGL.
    pub fn bind(&mut self, target: FramebufferTarget) -> Result<(), Error> {
        self.unbind();

        // recreates framebuffer if size changed
        let drawing_buffer_width = self.gl.drawing_buffer_width();
        let drawing_buffer_height = self.gl.drawing_buffer_height();
        if drawing_buffer_width != self.width || drawing_buffer_height != self.height {
            self.clear();
            self.width = drawing_buffer_width;
            self.height = drawing_buffer_height;
        }

        self.create_framebuffer(target)?;

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

    fn create_framebuffer(&mut self, target: FramebufferTarget) -> Result<(), Error> {
        match self.framebuffer.as_ref() {
            Some(framebuffer) => {
                self.gl
                    .bind_framebuffer(target.gl_enum(), Some(&framebuffer));
            }
            None => {
                let framebuffer = self
                    .gl
                    .create_framebuffer()
                    .ok_or(Error::CreateFramebufferFailure)?;
                self.gl
                    .bind_framebuffer(target.gl_enum(), Some(&framebuffer));
                self.framebuffer = Some(framebuffer);

                self.create_textures(target)?;
                self.create_renderbuffers(target)?;
            }
        }

        Ok(())
    }

    fn create_textures(&mut self, target: FramebufferTarget) -> Result<(), Error> {
        if self.textures.is_some() {
            return Ok(());
        }

        self.gl.active_texture(WebGl2RenderingContext::TEXTURE0);

        let mut textures = Vec::with_capacity(self.texture_providers.len());
        for provider in &self.texture_providers {
            let (texture, attachment) = match provider {
                TextureProvider::FromNew {
                    attachment,
                    internal_format,
                    format,
                    data_type,
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
                            self.width,
                            self.height,
                            0,
                            format.gl_enum(),
                            data_type.gl_enum(),
                            None,
                        )
                        .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;

                    (texture, *attachment)
                }
                TextureProvider::FromExisting {
                    attachment,
                    texture,
                } => {
                    self.gl
                        .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));

                    (texture.clone(), *attachment)
                }
            };

            self.gl.framebuffer_texture_2d(
                target.gl_enum(),
                attachment.gl_enum(),
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&texture),
                0,
            );

            textures.push(texture);
            self.attachments.push(attachment);
        }

        self.gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        self.textures = Some(textures);

        Ok(())
    }

    fn create_renderbuffers(&mut self, target: FramebufferTarget) -> Result<(), Error> {
        if self.renderbuffers.is_some() {
            return Ok(());
        }

        let mut renderbuffers = Vec::with_capacity(self.renderbuffer_providers.len());
        for provider in &self.renderbuffer_providers {
            let (attachment, renderbuffer) = match provider {
                RenderbufferProvider::FromNew {
                    attachment,
                    internal_format,
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
                            self.width,
                            self.height,
                        ),
                        None => self.gl.renderbuffer_storage(
                            WebGl2RenderingContext::RENDERBUFFER,
                            internal_format.gl_enum(),
                            self.width,
                            self.height,
                        ),
                    }
                    (*attachment, renderbuffer)
                }
                RenderbufferProvider::FromExisting {
                    attachment,
                    renderbuffer,
                } => (*attachment, renderbuffer.clone()),
            };

            self.gl.framebuffer_renderbuffer(
                target.gl_enum(),
                attachment.gl_enum(),
                WebGl2RenderingContext::RENDERBUFFER,
                Some(&renderbuffer),
            );

            renderbuffers.push(renderbuffer);
            self.attachments.push(attachment);
        }

        self.gl
            .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
        self.renderbuffers = Some(renderbuffers);

        Ok(())
    }

    /// Returns framebuffer width.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Returns framebuffer height.
    pub fn height(&self) -> i32 {
        self.height
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
            .map(|texture| texture)
    }

    /// Returns a [`WebGlRenderbuffer`] by index.
    pub fn renderbuffer(&self, index: usize) -> Option<&WebGlRenderbuffer> {
        self.renderbuffers
            .as_ref()
            .and_then(|list| list.get(index))
            .map(|renderbuffer| renderbuffer)
    }

    /// Returns a list containing [`WebGlRenderbuffer`]s.
    pub fn renderbuffers(&self) -> Option<&[WebGlRenderbuffer]> {
        match self.renderbuffers.as_ref() {
            Some(renderbuffers) => Some(&renderbuffers),
            None => None,
        }
    }

    /// Returns a list containing [`WebGlTexture`]s.
    pub fn textures(&self) -> Option<&[WebGlTexture]> {
        match self.textures.as_ref() {
            Some(textures) => Some(&textures),
            None => None,
        }
    }
}

/// Offscreen texture provider specifies texture configurations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureProvider {
    FromNew {
        attachment: FramebufferAttachment,
        internal_format: TextureInternalFormat,
        format: TextureFormat,
        data_type: TextureDataType,
    },
    FromExisting {
        attachment: FramebufferAttachment,
        texture: WebGlTexture,
    },
}

impl TextureProvider {
    /// Constructs a new texture provider by creating new [`WebGlTexture`].
    pub fn new(
        attachment: FramebufferAttachment,
        internal_format: TextureInternalFormat,
        format: TextureFormat,
        data_type: TextureDataType,
    ) -> Self {
        Self::FromNew {
            attachment,
            internal_format,
            format,
            data_type,
        }
    }

    /// Constructs a new renderbuffer provider by using a existing [`WebGlTexture`].
    pub fn from_existing(attachment: FramebufferAttachment, texture: WebGlTexture) -> Self {
        Self::FromExisting {
            attachment,
            texture,
        }
    }
}

/// Offscreen renderbuffer provider specifies renderbuffer configurations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderbufferProvider {
    FromNew {
        attachment: FramebufferAttachment,
        internal_format: RenderbufferInternalFormat,
    },
    FromExisting {
        attachment: FramebufferAttachment,
        renderbuffer: WebGlRenderbuffer,
    },
}

impl RenderbufferProvider {
    /// Constructs a new renderbuffer provider by creating new [`WebGlRenderbuffer`].
    pub fn new(
        attachment: FramebufferAttachment,
        internal_format: RenderbufferInternalFormat,
    ) -> Self {
        Self::FromNew {
            internal_format,
            attachment,
        }
    }

    /// Constructs a new renderbuffer provider by using a existing [`WebGlRenderbuffer`].
    pub fn from_existing(
        attachment: FramebufferAttachment,
        renderbuffer: WebGlRenderbuffer,
    ) -> Self {
        Self::FromExisting {
            attachment,
            renderbuffer,
        }
    }
}
