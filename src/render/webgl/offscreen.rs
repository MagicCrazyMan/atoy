//! This module provides convenient functions to create
//! [`WebGlFramebuffer`], [`WebGlRenderbuffer`], [`WebGlTexture`] and other stuffs to draw to offscreen.
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

/// Creates a [`WebGlFramebuffer`].
pub fn create_framebuffer(gl: &WebGl2RenderingContext) -> Result<WebGlFramebuffer, Error> {
    gl.create_framebuffer()
        .ok_or(Error::CreateFramebufferFailed)
}

/// Creates a [`WebGlFramebuffer`] with [`RenderbufferInternalFormat`], width and height.
pub fn create_renderbuffer(
    gl: &WebGl2RenderingContext,
    internal_format: RenderbufferInternalFormat,
    width: i32,
    height: i32,
) -> Result<WebGlRenderbuffer, Error> {
    let renderbuffer = gl
        .create_renderbuffer()
        .ok_or(Error::CreateRenderbufferFailed)?;

    gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
    gl.renderbuffer_storage(
        WebGl2RenderingContext::RENDERBUFFER,
        internal_format.gl_enum(),
        width,
        height,
    );
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
/// to make WebGl draw entities to an offscreen framebuffer.
///
/// Offscreen frame holds a drawing buffer width and drawing buffer height from [`WebGl2RenderingContext`]
/// to ensure [`WebGlRenderbuffer`] and [`WebGlTexture`] always fit into a same size.
/// When width and height in [`WebGl2RenderingContext`] changed,
/// new [`WebGlRenderbuffer`] and [`WebGlTexture`] are recreated as well.
///
/// [`drawBuffers`](https://developer.mozilla.org/en-US/docs/Web/API/WebGL2RenderingContext/drawBuffers)
pub struct OffscreenFramebuffer {
    width: i32,
    height: i32,

    target: FramebufferTarget,
    texture_providers: Vec<OffscreenTextureProvider>,
    renderbuffer_providers: Vec<OffscreenRenderbufferProvider>,

    framebuffer: Option<WebGlFramebuffer>,
    textures: Option<Vec<(WebGlTexture, OffscreenTextureProvider)>>,
    renderbuffers: Option<Vec<(WebGlRenderbuffer, OffscreenRenderbufferProvider)>>,
    draw_buffers: Array,
}

impl OffscreenFramebuffer {
    /// Constructs a new offscreen frame.
    pub fn new<
        TI: IntoIterator<Item = OffscreenTextureProvider>,
        RI: IntoIterator<Item = OffscreenRenderbufferProvider>,
    >(
        target: FramebufferTarget,
        texture_providers: TI,
        renderbuffer_providers: RI,
    ) -> Self {
        Self::with_draw_buffers(target, texture_providers, renderbuffer_providers, [])
    }

    /// Constructs a new offscreen frame.
    pub fn with_draw_buffers<
        TI: IntoIterator<Item = OffscreenTextureProvider>,
        RI: IntoIterator<Item = OffscreenRenderbufferProvider>,
        DI: IntoIterator<Item = FramebufferSource>,
    >(
        target: FramebufferTarget,
        texture_providers: TI,
        renderbuffer_providers: RI,
        draw_buffers: DI,
    ) -> Self {
        let draw_buffers_array = Array::new();
        for draw_buffer in draw_buffers.into_iter() {
            draw_buffers_array.push(&JsValue::from_f64(draw_buffer.gl_enum() as f64));
        }

        Self {
            width: 0,
            height: 0,

            target,
            texture_providers: texture_providers.into_iter().collect(),
            renderbuffer_providers: renderbuffer_providers.into_iter().collect(),

            framebuffer: None,
            textures: None,
            renderbuffers: None,
            draw_buffers: Array::new(),
        }
    }

    /// Binds offscreen frame to WebGL.
    pub fn bind(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
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
            self.width = drawing_buffer_width;
            self.height = drawing_buffer_height;
        }

        self.create_framebuffer(gl)?;
        gl.bind_framebuffer(
            self.target.gl_enum(),
            Some(self.framebuffer.as_ref().unwrap()),
        );

        self.create_textures(gl)?;
        self.create_renderbuffers(gl)?;

        if self.draw_buffers.length() > 0 {
            gl.draw_buffers(&self.draw_buffers);
        }

        Ok(())
    }

    /// Unbinds offscreen frame from WebGL.
    pub fn unbind(&self, gl: &WebGl2RenderingContext) {
        gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        gl.bind_framebuffer(WebGl2RenderingContext::READ_FRAMEBUFFER, None);
        gl.bind_framebuffer(WebGl2RenderingContext::DRAW_FRAMEBUFFER, None);
        gl.read_buffer(WebGl2RenderingContext::NONE);
    }

    /// Sets read buffer.
    pub fn set_read_buffer(self, gl: &WebGl2RenderingContext, source: FramebufferSource) {
        gl.read_buffer(source.gl_enum());
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

    fn create_textures(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
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
                provider.target.gl_enum(),
                provider.attachment.gl_enum(),
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&texture),
                provider.level,
            );

            textures.push((texture, *provider));
        }

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        self.textures = Some(textures);

        Ok(())
    }

    fn create_renderbuffers(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        if self.renderbuffers.is_some() {
            return Ok(());
        }

        let mut renderbuffers = Vec::with_capacity(self.renderbuffer_providers.len());
        for provider in &self.renderbuffer_providers {
            let renderbuffer = gl
                .create_renderbuffer()
                .ok_or(Error::CreateRenderbufferFailed)?;
            gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
            gl.renderbuffer_storage(
                WebGl2RenderingContext::RENDERBUFFER,
                provider.internal_format.gl_enum(),
                self.width,
                self.height,
            );
            gl.framebuffer_renderbuffer(
                provider.target.gl_enum(),
                provider.attachment.gl_enum(),
                WebGl2RenderingContext::RENDERBUFFER,
                Some(&renderbuffer),
            );

            renderbuffers.push((renderbuffer, *provider));
        }

        gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
        self.renderbuffers = Some(renderbuffers);

        Ok(())
    }

    /// Returns [`FramebufferTarget`] binding to this offscreen framebuffer.
    pub fn target(&self) -> FramebufferTarget {
        self.target
    }

    /// Returns [`WebGlFramebuffer`],
    pub fn framebuffer(&self) -> Option<&WebGlFramebuffer> {
        self.framebuffer.as_ref()
    }

    /// Gets a [`WebGlTexture`] by index.
    pub fn texture(&self, index: usize) -> Option<&WebGlTexture> {
        self.textures
            .as_ref()
            .and_then(|list| list.get(index))
            .map(|(texture, _)| texture)
    }

    /// Gets a [`WebGlRenderbuffer`] by index.
    pub fn renderbuffer(&self, index: usize) -> Option<&WebGlRenderbuffer> {
        self.renderbuffers
            .as_ref()
            .and_then(|list| list.get(index))
            .map(|(renderbuffer, _)| renderbuffer)
    }

    /// Returns list containing [`WebGlRenderbuffer`]s,
    /// following the orders of [`OffscreenRenderbufferProvider`]s.
    pub fn renderbuffers(
        &self,
    ) -> Option<&Vec<(WebGlRenderbuffer, OffscreenRenderbufferProvider)>> {
        self.renderbuffers.as_ref()
    }

    /// Returns list containing [`WebGlTexture`]s,
    /// following the orders of [`OffscreenTextureProvider`]s.
    pub fn textures(&self) -> Option<&Vec<(WebGlTexture, OffscreenTextureProvider)>> {
        self.textures.as_ref()
    }
}

/// Offscreen texture provider specifies texture configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OffscreenTextureProvider {
    target: FramebufferTarget,
    attachment: FramebufferAttachment,
    internal_format: TextureInternalFormat,
    format: TextureFormat,
    data_type: TextureDataType,
    level: i32,
}

impl OffscreenTextureProvider {
    /// Constructs a new offscreen texture provider.
    pub fn new(
        target: FramebufferTarget,
        attachment: FramebufferAttachment,
        internal_format: TextureInternalFormat,
        format: TextureFormat,
        data_type: TextureDataType,
        level: i32,
    ) -> Self {
        Self {
            target,
            attachment,
            internal_format,
            format,
            data_type,
            level,
        }
    }

    /// Gets [`FramebufferTarget`].
    pub fn target(&self) -> FramebufferTarget {
        self.target
    }

    /// Gets [`FramebufferAttachment`].
    pub fn attachment(&self) -> FramebufferAttachment {
        self.attachment
    }

    /// Gets [`TextureInternalFormat`].
    pub fn internal_format(&self) -> TextureInternalFormat {
        self.internal_format
    }

    /// Gets [`TextureFormat`].
    pub fn format(&self) -> TextureFormat {
        self.format
    }

    /// Gets [`TextureDataType`].
    pub fn data_type(&self) -> TextureDataType {
        self.data_type
    }

    /// Gets framebuffer texture binding level.
    pub fn level(&self) -> i32 {
        self.level
    }
}

/// Offscreen renderbuffer provider specifies renderbuffer configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OffscreenRenderbufferProvider {
    target: FramebufferTarget,
    attachment: FramebufferAttachment,
    internal_format: RenderbufferInternalFormat,
}

impl OffscreenRenderbufferProvider {
    /// Constructs a new offscreen renderbuffer provider.
    pub fn new(
        target: FramebufferTarget,
        attachment: FramebufferAttachment,
        internal_format: RenderbufferInternalFormat,
    ) -> Self {
        Self {
            target,
            internal_format,
            attachment,
        }
    }

    /// Gets [`FramebufferTarget`].
    pub fn target(&self) -> FramebufferTarget {
        self.target
    }

    /// Gets [`FramebufferAttachment`].
    pub fn attachment(&self) -> FramebufferAttachment {
        self.attachment
    }

    /// Gets [`RenderbufferInternalFormat`].
    pub fn internal_format(&self) -> RenderbufferInternalFormat {
        self.internal_format
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

/// Available read buffer source mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramebufferSource {
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
