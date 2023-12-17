//! This module provides convenient functions to create
//! [`WebGlFramebuffer`], [`WebGlRenderbuffer`], [`WebGlTexture`] and other stuffs to draw to offscreen.
//!
//! Unlike [`BufferStore`](super::buffer::BufferStore), [`TextureStore`](super::texture::TextureStore) and
//! [`ProgramStore`](super::program::ProgramStore), stuffs created from here does not manage automatically,
//! you should cleanups everything by yourself when finishing drawing.

use web_sys::{WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture};

use super::{
    conversion::ToGlEnum,
    error::Error,
    renderbuffer::RenderbufferInternalFormat,
    texture::{TextureDataType, TextureFormat, TextureInternalFormat},
};

/// Creates a [`WebGlFramebuffer`].
pub fn create_framebuffer(gl: &WebGl2RenderingContext) -> Result<WebGlFramebuffer, Error> {
    gl.create_framebuffer()
        .ok_or(Error::CreateFramebufferFailure)
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
        .ok_or(Error::CreateRenderbufferFailure)?;

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
    let texture = gl.create_texture().ok_or(Error::CreateTextureFailure)?;

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
pub struct OffscreenFrame {
    width: i32,
    height: i32,

    framebuffer_providers: Vec<OffscreenFramebufferProvider>,
    texture_providers: Vec<OffscreenTextureProvider>,
    renderbuffer_providers: Vec<OffscreenRenderbufferProvider>,

    framebuffers: Option<Vec<(WebGlFramebuffer, OffscreenFramebufferProvider)>>,
    textures: Option<Vec<(WebGlTexture, OffscreenTextureProvider)>>,
    renderbuffers: Option<Vec<(WebGlRenderbuffer, OffscreenRenderbufferProvider)>>,
}

impl OffscreenFrame {
    /// Constructs a new offscreen frame.
    pub fn new<
        FI: IntoIterator<Item = OffscreenFramebufferProvider>,
        TI: IntoIterator<Item = OffscreenTextureProvider>,
        RI: IntoIterator<Item = OffscreenRenderbufferProvider>,
    >(
        framebuffer_providers: FI,
        texture_providers: TI,
        renderbuffer_providers: RI,
    ) -> Self {
        Self {
            width: 0,
            height: 0,

            framebuffers: None,
            textures: None,
            renderbuffers: None,

            framebuffer_providers: framebuffer_providers.into_iter().collect(),
            texture_providers: texture_providers.into_iter().collect(),
            renderbuffer_providers: renderbuffer_providers.into_iter().collect(),
        }
    }

    /// Binds offscreen frame to WebGL.
    pub fn bind(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        // delete previous framebuffers, textures and renderbuffers if size changed
        if gl.drawing_buffer_width() != self.width || gl.drawing_buffer_height() != self.height {
            if let Some(framebuffers) = &mut self.framebuffers {
                for (framebuffer, _) in framebuffers {
                    gl.delete_framebuffer(Some(&framebuffer));
                }
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

            self.framebuffers = None;
            self.textures = None;
            self.renderbuffers = None;
        }

        self.create_framebuffers(gl)?;
        self.create_textures(gl)?;
        self.create_renderbuffers(gl)?;

        for (framebuffer, provider) in self.framebuffers.as_ref().unwrap() {
            gl.bind_framebuffer(provider.target.gl_enum(), Some(framebuffer))
        }

        for (texture, provider) in self.textures.as_ref().unwrap() {
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
            gl.framebuffer_texture_2d(
                provider.target.gl_enum(),
                provider.attachment.gl_enum(),
                WebGl2RenderingContext::TEXTURE_2D,
                Some(texture),
                provider.level,
            )
        }

        for (renderbuffer, provider) in self.renderbuffers.as_ref().unwrap() {
            gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(renderbuffer));
            gl.framebuffer_renderbuffer(
                provider.target.gl_enum(),
                provider.attachment.gl_enum(),
                WebGl2RenderingContext::RENDERBUFFER,
                Some(renderbuffer),
            )
        }

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

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
    pub fn set_read_buffer(self, gl: &WebGl2RenderingContext, attachment: FramebufferAttachment) {
        gl.read_buffer(attachment.gl_enum());
    }

    fn create_framebuffers(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        if self.framebuffers.is_some() {
            return Ok(());
        }

        let mut framebuffers = Vec::with_capacity(self.framebuffer_providers.len());
        for provider in &self.framebuffer_providers {
            let framebuffer = gl
                .create_framebuffer()
                .ok_or(Error::CreateFramebufferFailure)?;
            gl.bind_framebuffer(provider.target.gl_enum(), Some(&framebuffer));
            framebuffers.push((framebuffer, *provider));
        }

        self.framebuffers = Some(framebuffers);

        Ok(())
    }

    fn create_textures(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        if self.textures.is_some() {
            return Ok(());
        }

        gl.active_texture(WebGl2RenderingContext::TEXTURE0);

        let mut textures = Vec::with_capacity(self.texture_providers.len());
        for provider in &self.texture_providers {
            let texture = gl.create_texture().ok_or(Error::CreateTextureFailure)?;
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                provider.internal_format.gl_enum() as i32,
                self.width,
                self.height,
                0,
                provider.format.gl_enum(),
                provider.data_type.gl_enum(),
                None,
            )
            .or_else(|err| Err(Error::TexImageFailure(err.as_string())))?;

            textures.push((texture, *provider));
        }

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
        self.textures = Some(textures);

        Ok(())
    }

    fn create_renderbuffers(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        if self.textures.is_some() {
            return Ok(());
        }

        let mut renderbuffers = Vec::with_capacity(self.renderbuffer_providers.len());
        for provider in &self.renderbuffer_providers {
            let renderbuffer = gl
                .create_renderbuffer()
                .ok_or(Error::CreateRenderbufferFailure)?;
            gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
            gl.renderbuffer_storage(
                WebGl2RenderingContext::RENDERBUFFER,
                provider.internal_format.gl_enum(),
                self.width,
                self.height,
            );

            renderbuffers.push((renderbuffer, *provider));
        }

        gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);
        self.renderbuffers = Some(renderbuffers);

        Ok(())
    }
}

/// Offscreen texture provider specifies texture configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OffscreenFramebufferProvider {
    target: FramebufferTarget,
}

impl OffscreenFramebufferProvider {
    /// Constructs a new offscreen framebuffer provider.
    pub fn new(target: FramebufferTarget) -> Self { Self { target } }

    /// Constructs a new framebuffer provider for [`FramebufferTarget::FRAMEBUFFER`].
    pub fn framebuffer() -> Self {
        Self {
            target: FramebufferTarget::FRAMEBUFFER,
        }
    }

    /// Constructs a new framebuffer provider for [`FramebufferTarget::READ_FRAMEBUFFER`].
    pub fn read_framebuffer() -> Self {
        Self {
            target: FramebufferTarget::READ_FRAMEBUFFER,
        }
    }

    /// Constructs a new framebuffer provider for [`FramebufferTarget::DRAW_FRAMEBUFFER`].
    pub fn draw_framebuffer() -> Self {
        Self {
            target: FramebufferTarget::DRAW_FRAMEBUFFER,
        }
    }

    /// Gets [`FramebufferTarget`].
    pub fn target(&self) -> FramebufferTarget {
        self.target
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
