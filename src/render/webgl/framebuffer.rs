//! This module provides convenient functions to create
//! [`WebGlFramebuffer`], [`WebGlRenderbuffer`], [`WebGlTexture`] and other stuffs to draw.
//!
//! Unlike [`BufferStore`](super::buffer::BufferStore), [`TextureStore`](super::texture::TextureStore) and
//! [`ProgramStore`](super::program::ProgramStore), stuffs created from here does not manage automatically,
//! you should cleanups everything by yourself when finishing drawing.

use wasm_bindgen::JsValue;
use web_sys::{
    js_sys::Array, WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture,
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

/// Creates a [`WebGlFramebuffer`].
pub fn create_framebuffer(gl: &WebGl2RenderingContext) -> Result<WebGlFramebuffer, Error> {
    gl.create_framebuffer()
        .ok_or(Error::CreateFramebufferFailed)
}

/// Creates a [`WebGlFramebuffer`] with [`RenderbufferInternalFormat`], width and height.
pub fn create_renderbuffer(
    gl: &WebGl2RenderingContext,
    internal_format: RenderbufferInternalFormat,
    samples: Option<i32>,
    width: i32,
    height: i32,
) -> Result<WebGlRenderbuffer, Error> {
    let renderbuffer = gl
        .create_renderbuffer()
        .ok_or(Error::CreateRenderbufferFailed)?;

    gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
    match samples {
        Some(samples) => gl.renderbuffer_storage_multisample(
            WebGl2RenderingContext::RENDERBUFFER,
            samples,
            internal_format.gl_enum(),
            width,
            height,
        ),
        None => gl.renderbuffer_storage(
            WebGl2RenderingContext::RENDERBUFFER,
            internal_format.gl_enum(),
            width,
            height,
        ),
    }
    gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

    Ok(renderbuffer)
}

/// Creates a [`WebGlTexture`] with [`TextureInternalFormat`], width and height.
pub fn create_texture_2d(
    gl: &WebGl2RenderingContext,
    internal_format: TextureInternalFormat,
    format: TextureFormat,
    data_type: TextureDataType,
    width: i32,
    height: i32,
) -> Result<WebGlTexture, Error> {
    let texture = gl.create_texture().ok_or(Error::CreateTextureFailed)?;

    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
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
    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

    Ok(texture)
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
    width: i32,
    height: i32,

    texture_providers: Vec<TextureProvider>,
    renderbuffer_providers: Vec<RenderbufferProvider>,
    renderbuffer_samples: Option<i32>,

    gl: Option<WebGl2RenderingContext>,
    binding_target: Option<FramebufferTarget>,
    reading_buffer: Option<FramebufferDrawBuffer>,
    framebuffer: Option<WebGlFramebuffer>,
    textures: Option<Vec<(WebGlTexture, TextureProvider)>>,
    renderbuffers: Option<Vec<(WebGlRenderbuffer, RenderbufferProvider)>>,
    attachments: Array,

    draw_buffers: Array,
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        let (Some(gl), Some(framebuffer), Some(textures), Some(renderbuffers)) = (
            self.gl.as_ref(),
            self.framebuffer.as_ref(),
            self.textures.as_ref(),
            self.renderbuffers.as_ref(),
        ) else {
            return;
        };

        gl.delete_framebuffer(Some(framebuffer));
        textures
            .iter()
            .for_each(|(texture, _)| gl.delete_texture(Some(texture)));
        renderbuffers
            .iter()
            .for_each(|(renderbuffer, _)| gl.delete_renderbuffer(Some(renderbuffer)));
    }
}

impl Framebuffer {
    /// Constructs a new framebuffer object.
    pub fn new<
        TI: IntoIterator<Item = TextureProvider>,
        RI: IntoIterator<Item = RenderbufferProvider>,
        DI: IntoIterator<Item = FramebufferDrawBuffer>,
    >(
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
            width: 0,
            height: 0,

            texture_providers: texture_providers.into_iter().collect(),
            renderbuffer_providers: renderbuffer_providers.into_iter().collect(),
            renderbuffer_samples,

            gl: None,
            binding_target: None,
            reading_buffer: None,
            framebuffer: None,
            textures: None,
            renderbuffers: None,
            attachments: Array::new(),

            draw_buffers: draw_buffers_array,
        }
    }

    /// Binds framebuffer to WebGL.
    pub fn bind(
        &mut self,
        gl: &WebGl2RenderingContext,
        target: FramebufferTarget,
    ) -> Result<(), Error> {
        if let Some(sgl) = self.gl.as_ref() {
            if sgl != gl {
                panic!("share framebuffer between different WebGL is not allowed");
            }
        }

        let drawing_buffer_width = gl.drawing_buffer_width();
        let drawing_buffer_height = gl.drawing_buffer_height();

        // delete previous framebuffers, textures and renderbuffers if size changed
        if drawing_buffer_width != self.width || drawing_buffer_height != self.height {
            if let Some(framebuffer) = &mut self.framebuffer {
                gl.delete_framebuffer(Some(&framebuffer));
            }

            if let Some(textures) = &mut self.textures {
                for (texture, _) in textures {
                    gl.delete_texture(Some(&texture));
                }
            }

            if let Some(renderbuffers) = &mut self.renderbuffers {
                for (renderbuffer, _) in renderbuffers {
                    gl.delete_renderbuffer(Some(&renderbuffer));
                }
            }

            self.framebuffer = None;
            self.textures = None;
            self.renderbuffers = None;
            self.attachments.set_length(0);

            self.width = drawing_buffer_width;
            self.height = drawing_buffer_height;
        }

        self.unbind(gl);

        self.create_framebuffer(gl)?;
        gl.bind_framebuffer(target.gl_enum(), Some(self.framebuffer.as_ref().unwrap()));
        self.create_textures(gl, target)?;
        self.create_renderbuffers(gl, target)?;

        if self.draw_buffers.length() > 0 {
            gl.draw_buffers(&self.draw_buffers);
        }

        self.gl = Some(gl.clone());
        self.binding_target = Some(target);

        Ok(())
    }

    /// Unbinds framebuffer from WebGL.
    pub fn unbind(&mut self, gl: &WebGl2RenderingContext) {
        if let Some(sgl) = self.gl.as_ref() {
            if sgl != gl {
                panic!("share framebuffer between different WebGL is not allowed");
            }
        }

        if let Some(binding_target) = self.binding_target.take() {
            gl.bind_framebuffer(binding_target.gl_enum(), None);
        }
        if let Some(_) = self.reading_buffer.take() {
            gl.read_buffer(WebGl2RenderingContext::BACK);
        }
    }

    /// Sets read buffer.
    pub fn set_read_buffer(&mut self, gl: &WebGl2RenderingContext, source: FramebufferDrawBuffer) {
        gl.read_buffer(source.gl_enum());
        self.reading_buffer = Some(source);
    }

    /// Returns number of sample of the render buffers if multisample is enabled.
    pub fn renderbuffer_samples(&self) -> Option<i32> {
        self.renderbuffer_samples.clone()
    }

    fn create_framebuffer(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        if self.framebuffer.is_some() {
            return Ok(());
        }

        let framebuffer = gl
            .create_framebuffer()
            .ok_or(Error::CreateFramebufferFailed)?;
        self.framebuffer = Some(framebuffer);

        Ok(())
    }

    fn create_textures(
        &mut self,
        gl: &WebGl2RenderingContext,
        target: FramebufferTarget,
    ) -> Result<(), Error> {
        if self.textures.is_some() {
            return Ok(());
        }

        gl.active_texture(WebGl2RenderingContext::TEXTURE0);

        let mut textures = Vec::with_capacity(self.texture_providers.len());
        for provider in &self.texture_providers {
            let texture = gl.create_texture().ok_or(Error::CreateTextureFailed)?;
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                provider.level,
                provider.internal_format.gl_enum() as i32,
                self.width,
                self.height,
                0,
                provider.format.gl_enum(),
                provider.data_type.gl_enum(),
                None,
            )
            .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;
            gl.framebuffer_texture_2d(
                target.gl_enum(),
                provider.attachment.gl_enum(),
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&texture),
                provider.level,
            );

            textures.push((texture, *provider));
            self.attachments
                .push(&JsValue::from_f64(provider.attachment.gl_enum() as f64));
        }

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        self.textures = Some(textures);

        Ok(())
    }

    fn create_renderbuffers(
        &mut self,
        gl: &WebGl2RenderingContext,
        target: FramebufferTarget,
    ) -> Result<(), Error> {
        if self.renderbuffers.is_some() {
            return Ok(());
        }

        let mut renderbuffers = Vec::with_capacity(self.renderbuffer_providers.len());
        for provider in &self.renderbuffer_providers {
            let renderbuffer = gl
                .create_renderbuffer()
                .ok_or(Error::CreateRenderbufferFailed)?;
            gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
            match self.renderbuffer_samples {
                Some(samples) => gl.renderbuffer_storage_multisample(
                    WebGl2RenderingContext::RENDERBUFFER,
                    samples,
                    provider.internal_format.gl_enum(),
                    self.width,
                    self.height,
                ),
                None => gl.renderbuffer_storage(
                    WebGl2RenderingContext::RENDERBUFFER,
                    provider.internal_format.gl_enum(),
                    self.width,
                    self.height,
                ),
            }
            gl.framebuffer_renderbuffer(
                target.gl_enum(),
                provider.attachment.gl_enum(),
                WebGl2RenderingContext::RENDERBUFFER,
                Some(&renderbuffer),
            );

            renderbuffers.push((renderbuffer, *provider));
            self.attachments
                .push(&JsValue::from_f64(provider.attachment.gl_enum() as f64));
        }

        gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
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

    /// Returns [`FramebufferDrawBuffer`] currently reading from this framebuffer.
    pub fn reading_buffer(&self) -> Option<FramebufferDrawBuffer> {
        self.reading_buffer.clone()
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
            .map(|(texture, _)| texture)
    }

    /// Returns a [`WebGlRenderbuffer`] by index.
    pub fn renderbuffer(&self, index: usize) -> Option<&WebGlRenderbuffer> {
        self.renderbuffers
            .as_ref()
            .and_then(|list| list.get(index))
            .map(|(renderbuffer, _)| renderbuffer)
    }

    /// Returns list containing [`WebGlRenderbuffer`]s,
    /// following the orders of [`OffscreenRenderbufferProvider`]s.
    pub fn renderbuffers(&self) -> Option<&Vec<(WebGlRenderbuffer, RenderbufferProvider)>> {
        self.renderbuffers.as_ref()
    }

    /// Returns list containing [`WebGlTexture`]s,
    /// following the orders of [`OffscreenTextureProvider`]s.
    pub fn textures(&self) -> Option<&Vec<(WebGlTexture, TextureProvider)>> {
        self.textures.as_ref()
    }
}

/// Offscreen texture provider specifies texture configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureProvider {
    attachment: FramebufferAttachment,
    internal_format: TextureInternalFormat,
    format: TextureFormat,
    data_type: TextureDataType,
    level: i32,
}

impl TextureProvider {
    /// Constructs a new texture provider.
    pub fn new(
        attachment: FramebufferAttachment,
        internal_format: TextureInternalFormat,
        format: TextureFormat,
        data_type: TextureDataType,
        level: i32,
    ) -> Self {
        Self {
            attachment,
            internal_format,
            format,
            data_type,
            level,
        }
    }

    /// Returns [`FramebufferAttachment`].
    pub fn attachment(&self) -> FramebufferAttachment {
        self.attachment
    }

    /// Returns [`TextureInternalFormat`].
    pub fn internal_format(&self) -> TextureInternalFormat {
        self.internal_format
    }

    /// Returns [`TextureFormat`].
    pub fn format(&self) -> TextureFormat {
        self.format
    }

    /// Returns [`TextureDataType`].
    pub fn data_type(&self) -> TextureDataType {
        self.data_type
    }

    /// Returns framebuffer texture binding level.
    pub fn level(&self) -> i32 {
        self.level
    }
}

/// Offscreen renderbuffer provider specifies renderbuffer configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderbufferProvider {
    attachment: FramebufferAttachment,
    internal_format: RenderbufferInternalFormat,
}

impl RenderbufferProvider {
    /// Constructs a new renderbuffer provider.
    pub fn new(
        attachment: FramebufferAttachment,
        internal_format: RenderbufferInternalFormat,
    ) -> Self {
        Self {
            internal_format,
            attachment,
        }
    }

    /// Returns [`FramebufferAttachment`].
    pub fn attachment(&self) -> FramebufferAttachment {
        self.attachment
    }

    /// Returns [`RenderbufferInternalFormat`].
    pub fn internal_format(&self) -> RenderbufferInternalFormat {
        self.internal_format
    }
}
