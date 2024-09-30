use std::{cell::RefCell, rc::Rc};

use hashbrown::HashMap;
use log::warn;
use proc::GlEnum;
use web_sys::{
    WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer, WebGlSampler, WebGlTexture,
};

use super::{
    capabilities::WebGlCapabilities,
    error::Error,
    renderbuffer::WebGlRenderbufferInternalFormat,
    texture::{WebGlTextureLayout, WebGlTexturePlainInternalFormat, WebGlTextureUnit},
};

/// Available framebuffer targets mapped from [`WebGl2RenderingContext`].
/// In WebGL 2.0, framebuffer target splits from [`WebGl2RenderingContext::FRAMEBUFFER`]
/// into [`WebGl2RenderingContext::READ_FRAMEBUFFER`] and [`WebGl2RenderingContext::DRAW_FRAMEBUFFER`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlFramebufferTarget {
    ReadFramebuffer,
    DrawFramebuffer,
}

/// Available framebuffer attachment targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlFramebufferAttachTarget {
    #[gl_enum(COLOR_ATTACHMENT0)]
    ColorAttachment0,
    #[gl_enum(COLOR_ATTACHMENT1)]
    ColorAttachment1,
    #[gl_enum(COLOR_ATTACHMENT2)]
    ColorAttachment2,
    #[gl_enum(COLOR_ATTACHMENT3)]
    ColorAttachment3,
    #[gl_enum(COLOR_ATTACHMENT4)]
    ColorAttachment4,
    #[gl_enum(COLOR_ATTACHMENT5)]
    ColorAttachment5,
    #[gl_enum(COLOR_ATTACHMENT6)]
    ColorAttachment6,
    #[gl_enum(COLOR_ATTACHMENT7)]
    ColorAttachment7,
    #[gl_enum(COLOR_ATTACHMENT8)]
    ColorAttachment8,
    #[gl_enum(COLOR_ATTACHMENT9)]
    ColorAttachment9,
    #[gl_enum(COLOR_ATTACHMENT10)]
    ColorAttachment10,
    #[gl_enum(COLOR_ATTACHMENT11)]
    ColorAttachment11,
    #[gl_enum(COLOR_ATTACHMENT12)]
    ColorAttachment12,
    #[gl_enum(COLOR_ATTACHMENT13)]
    ColorAttachment13,
    #[gl_enum(COLOR_ATTACHMENT14)]
    ColorAttachment14,
    #[gl_enum(COLOR_ATTACHMENT15)]
    ColorAttachment15,
    DepthAttachment,
    StencilAttachment,
    DepthStencilAttachment,
}

impl WebGlFramebufferAttachTarget {
    /// Returns the sequence index of color attachments.
    /// Always returns `0` for depth and stencil attachments.
    #[inline]
    pub fn as_index(&self) -> usize {
        match self {
            WebGlFramebufferAttachTarget::ColorAttachment0 => 0,
            WebGlFramebufferAttachTarget::ColorAttachment1 => 1,
            WebGlFramebufferAttachTarget::ColorAttachment2 => 2,
            WebGlFramebufferAttachTarget::ColorAttachment3 => 3,
            WebGlFramebufferAttachTarget::ColorAttachment4 => 4,
            WebGlFramebufferAttachTarget::ColorAttachment5 => 5,
            WebGlFramebufferAttachTarget::ColorAttachment6 => 6,
            WebGlFramebufferAttachTarget::ColorAttachment7 => 7,
            WebGlFramebufferAttachTarget::ColorAttachment8 => 8,
            WebGlFramebufferAttachTarget::ColorAttachment9 => 9,
            WebGlFramebufferAttachTarget::ColorAttachment10 => 10,
            WebGlFramebufferAttachTarget::ColorAttachment11 => 11,
            WebGlFramebufferAttachTarget::ColorAttachment12 => 12,
            WebGlFramebufferAttachTarget::ColorAttachment13 => 13,
            WebGlFramebufferAttachTarget::ColorAttachment14 => 14,
            WebGlFramebufferAttachTarget::ColorAttachment15 => 15,
            WebGlFramebufferAttachTarget::DepthAttachment => 0,
            WebGlFramebufferAttachTarget::StencilAttachment => 0,
            WebGlFramebufferAttachTarget::DepthStencilAttachment => 0,
        }
    }
}

/// Available depth and stencil format mapped from [`WebGl2RenderingContext`](web_sys::WebGl2RenderingContext).
/// Available both on renderbuffer and texture.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum WebGlFramebufferDepthStencilFormat {
    #[gl_enum(STENCIL_INDEX8)]
    STENCIL_INDEX8,
    #[gl_enum(DEPTH_COMPONENT32F)]
    DEPTH_COMPONENT32F,
    #[gl_enum(DEPTH_COMPONENT24)]
    DEPTH_COMPONENT24,
    #[gl_enum(DEPTH_COMPONENT16)]
    DEPTH_COMPONENT16,
    #[gl_enum(DEPTH32F_STENCIL8)]
    DEPTH32F_STENCIL8,
    #[gl_enum(DEPTH24_STENCIL8)]
    DEPTH24_STENCIL8,
}

/// Available framebuffer size policies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizePolicy {
    /// Uses the size of WebGL context drawing buffer.
    FollowDrawingBuffer,
    /// Ceiling scales the size of WebGL context drawing buffer.
    ScaleDrawingBuffer(f64),
    /// Uses a custom size.
    Custom { width: usize, height: usize },
}

impl SizePolicy {
    fn size_of(&self, gl: &WebGl2RenderingContext) -> (usize, usize) {
        match self {
            SizePolicy::FollowDrawingBuffer => (
                gl.drawing_buffer_width() as usize,
                gl.drawing_buffer_height() as usize,
            ),
            SizePolicy::ScaleDrawingBuffer(scale) => (
                (gl.drawing_buffer_width() as f64 * scale).ceil() as usize,
                (gl.drawing_buffer_height() as f64 * scale).ceil() as usize,
            ),
            SizePolicy::Custom { width, height } => (*width, *height),
        }
    }
}

/// Available texture targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebGlFramebufferTextureTarget {
    Texture2D,
    TextureCubeMapPositiveX,
    TextureCubeMapNegativeX,
    TextureCubeMapPositiveY,
    TextureCubeMapNegativeY,
    TextureCubeMapPositiveZ,
    TextureCubeMapNegativeZ,
    Texture2DArray { index: usize },
    Texture3D { depth: usize },
}

impl WebGlFramebufferTextureTarget {
    pub fn to_gl_enum(&self) -> u32 {
        match self {
            WebGlFramebufferTextureTarget::Texture2D => WebGl2RenderingContext::TEXTURE_2D,
            WebGlFramebufferTextureTarget::TextureCubeMapPositiveX => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X
            }
            WebGlFramebufferTextureTarget::TextureCubeMapNegativeX => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X
            }
            WebGlFramebufferTextureTarget::TextureCubeMapPositiveY => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y
            }
            WebGlFramebufferTextureTarget::TextureCubeMapNegativeY => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y
            }
            WebGlFramebufferTextureTarget::TextureCubeMapPositiveZ => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z
            }
            WebGlFramebufferTextureTarget::TextureCubeMapNegativeZ => {
                WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z
            }
            WebGlFramebufferTextureTarget::Texture2DArray { .. } => {
                WebGl2RenderingContext::TEXTURE_2D_ARRAY
            }
            WebGlFramebufferTextureTarget::Texture3D { .. } => WebGl2RenderingContext::TEXTURE_3D,
        }
    }
}

/// Framebuffer color attachment source.
/// Developer could create a owned texture or renderbuffer for an attachment
/// or use an external texture or renderbuffer.
///
/// When using an external texture or rbo,
/// developer must ensure the size of texture or rbo
/// match the size of framebuffer by [`SizePolicy`].
/// If internal format of an external texture or rbo is not provided, it is queried from WebGL context.
pub enum WebGlFramebufferColorSource {
    CreateTexture {
        internal_format: WebGlTexturePlainInternalFormat,
    },
    CreateRenderbuffer {
        internal_format: WebGlRenderbufferInternalFormat,
    },
    ExternalTexture {
        target: WebGlFramebufferTextureTarget,
        level: usize,
        gl_texture: WebGlTexture,
    },
    ExternalRenderbuffer {
        gl_renderbuffer: WebGlRenderbuffer,
    },
}

/// Framebuffer color attachment source.
/// Developer could create a owned texture or renderbuffer for an attachment
/// or use an external texture or renderbuffer.
///
/// When using an external texture or rbo,
/// developer must ensure the size of texture or rbo
/// match the size of framebuffer by [`SizePolicy`].
/// If internal format of an external texture or rbo is not provided, it is queried from WebGL context.
pub enum WebGlFramebufferDepthStencilSource {
    CreateTexture {
        internal_format: WebGlFramebufferDepthStencilFormat,
    },
    CreateRenderbuffer {
        internal_format: WebGlFramebufferDepthStencilFormat,
    },
    ExternalTexture {
        internal_format: WebGlFramebufferDepthStencilFormat,
        target: WebGlFramebufferTextureTarget,
        level: usize,
        gl_texture: WebGlTexture,
    },
    ExternalRenderbuffer {
        internal_format: WebGlFramebufferDepthStencilFormat,
        gl_renderbuffer: WebGlRenderbuffer,
    },
}

impl WebGlFramebufferDepthStencilSource {
    fn internal_format(&self) -> WebGlFramebufferDepthStencilFormat {
        match self {
            WebGlFramebufferDepthStencilSource::CreateTexture {
                internal_format, ..
            } => *internal_format,
            WebGlFramebufferDepthStencilSource::CreateRenderbuffer { internal_format } => {
                *internal_format
            }
            WebGlFramebufferDepthStencilSource::ExternalTexture {
                internal_format, ..
            } => *internal_format,
            WebGlFramebufferDepthStencilSource::ExternalRenderbuffer {
                internal_format, ..
            } => *internal_format,
        }
    }
}

/// WebGL framebuffer create options.
///
/// When creating a framebuffer,
/// factory does not check whether the combination of attachments is valid or not.
/// Developer should learns the rules from WebGL specification and take care of it by yourself.
pub struct WebGlFramebufferCreateOptions {
    /// Size policy telling factory the size to use when creates textures or renderbuffers.
    ///
    /// External textures and renderbuffers does not follow the policy,
    /// Developer should verify them by yourself.
    pub size_policy: SizePolicy,
    /// A list of textures or renderbuffers to attach to color attachments.
    /// Factory attaches each to `COLOR_ATTACHMENT{i}` by index in order.
    ///
    /// If the length of this list is greater than the maximum color attachments,
    /// the rest of the attachments are ignored.
    ///
    /// If a floating format is used, factory will try to enable `EXT_color_buffer_float` automatically.
    /// But no error is thrown even if it is not supported.
    pub colors: Vec<WebGlFramebufferColorSource>,
    /// Depth and stencil attachment.
    ///
    /// Factory decides what kind of attachment should be used automatically
    /// by the internal format of the texture or renderbuffer.
    pub depth_stencil: Option<WebGlFramebufferDepthStencilSource>,
    /// MSAA sample count, sets a value greater than zero to enable.
    ///
    /// When MASS enabled, all attachment must be renderbuffer,
    /// since multisampling texture does not support in WebGL.
    /// But no error is thrown even texture attachment is used.
    pub multisample: Option<usize>,
}

#[derive(Clone)]
enum Attachment {
    Texture { gl_texture: WebGlTexture },
    Renderbuffer { gl_renderbuffer: WebGlRenderbuffer },
}

#[derive(Clone)]
pub struct WebGlFramebufferItem {
    gl_framebuffer: WebGlFramebuffer,
    create_options: Rc<WebGlFramebufferCreateOptions>,
    current_width: Rc<RefCell<usize>>,
    current_height: Rc<RefCell<usize>>,
    color_attachments: Vec<Attachment>,
    depth_attachment: Option<Attachment>,
    stencil_attachment: Option<Attachment>,
}

impl WebGlFramebufferItem {
    /// Returns framebuffer.
    pub fn gl_framebuffer(&self) -> &WebGlFramebuffer {
        &self.gl_framebuffer
    }

    /// Returns size policy.
    pub fn size_policy(&self) -> SizePolicy {
        self.create_options.size_policy
    }

    /// Returns MSAA samples count.
    pub fn multisample(&self) -> Option<usize> {
        self.create_options.multisample
    }

    /// Returns current width of framebuffer.
    pub fn current_width(&self) -> usize {
        *self.current_width.borrow()
    }

    /// Returns current height of framebuffer.
    pub fn current_height(&self) -> usize {
        *self.current_height.borrow()
    }

    /// Returns the length of color attachments.
    pub fn color_attachment_len(&self) -> usize {
        self.color_attachments.len()
    }

    /// Returns color texture by index.
    ///
    /// Returns `None` if no color attachment at specified index or color attachment is renderbuffer.
    pub fn color_texture(&self, index: usize) -> Option<WebGlTexture> {
        self.color_attachments.get(index).and_then(|a| match a {
            Attachment::Texture { gl_texture, .. } => Some(gl_texture.clone()),
            Attachment::Renderbuffer { .. } => None,
        })
    }

    /// Returns color renderbuffer by index.
    ///
    /// Returns `None` if no color attachment at specified index or color attachment is texture.
    pub fn color_renderbuffer(&self, index: usize) -> Option<WebGlRenderbuffer> {
        self.color_attachments.get(index).and_then(|a| match a {
            Attachment::Texture { .. } => None,
            Attachment::Renderbuffer {
                gl_renderbuffer, ..
            } => Some(gl_renderbuffer.clone()),
        })
    }

    /// Returns depth texture.
    ///
    /// Returns `None` if no depth attachment or depth attachment is renderbuffer.
    ///
    /// Returns as same as [`stencil_texture`](WebGlFramebufferItem::stencil_texture)
    /// if this is a depth stencil combination format.
    pub fn depth_texture(&self) -> Option<WebGlTexture> {
        self.depth_attachment.as_ref().and_then(|a| match a {
            Attachment::Texture { gl_texture, .. } => Some(gl_texture.clone()),
            Attachment::Renderbuffer { .. } => None,
        })
    }

    /// Returns depth renderbuffer.
    ///
    /// Returns `None` if no depth attachment or depth attachment is texture.
    ///
    /// Returns as same as [`stencil_renderbuffer`](WebGlFramebufferItem::stencil_renderbuffer)
    /// if this is a depth stencil combination format.
    pub fn depth_renderbuffer(&self) -> Option<WebGlRenderbuffer> {
        self.depth_attachment.as_ref().and_then(|a| match a {
            Attachment::Texture { .. } => None,
            Attachment::Renderbuffer {
                gl_renderbuffer, ..
            } => Some(gl_renderbuffer.clone()),
        })
    }

    /// Returns stencil texture.
    ///
    /// Returns `None` if no stencil attachment or stencil attachment is renderbuffer.
    ///
    /// Returns as same as [`depth_texture`](WebGlFramebufferItem::depth_texture)
    /// if this is a depth stencil combination format.
    pub fn stencil_texture(&self) -> Option<WebGlTexture> {
        self.stencil_attachment.as_ref().and_then(|a| match a {
            Attachment::Texture { gl_texture, .. } => Some(gl_texture.clone()),
            Attachment::Renderbuffer { .. } => None,
        })
    }

    /// Returns stencil renderbuffer.
    ///
    /// Returns `None` if no stencil attachment or stencil attachment is texture.
    ///
    /// Returns as same as [`depth_renderbuffer`](WebGlFramebufferItem::depth_renderbuffer)
    /// if this is a depth stencil combination format.
    pub fn stencil_renderbuffer(&self) -> Option<WebGlRenderbuffer> {
        self.stencil_attachment.as_ref().and_then(|a| match a {
            Attachment::Texture { .. } => None,
            Attachment::Renderbuffer {
                gl_renderbuffer, ..
            } => Some(gl_renderbuffer.clone()),
        })
    }
}

/// WebGL framebuffer factory.
///
/// Different from other managers like [`WebGlBufferManager`](super::buffer::WebGlBufferManager),
/// framebuffer factory only creates and updates framebuffer items, but not caches them.
/// Creates framebuffer using a same [`WebGlFramebufferCreateOptions`] always resulting a new one.
pub struct WebGlFramebufferFactory {
    gl: WebGl2RenderingContext,
}

impl WebGlFramebufferFactory {
    /// Constructs a new framebuffer factory.
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self { gl }
    }

    /// Creates a new framebuffer item by a [`WebGlFramebufferCreateOptions`].
    pub fn create_framebuffer(
        &self,
        options: WebGlFramebufferCreateOptions,
        using_draw_framebuffer: &Option<WebGlFramebufferItem>,
        activating_texture_unit: WebGlTextureUnit,
        using_textures: &HashMap<
            (WebGlTextureUnit, WebGlTextureLayout),
            (WebGlTexture, WebGlSampler),
        >,
        capabilities: &WebGlCapabilities,
    ) -> Result<WebGlFramebufferItem, Error> {
        let gl_framebuffer = self
            .gl
            .create_framebuffer()
            .ok_or(Error::CreateFramebufferFailure)?;
        let (width, height) = options.size_policy.size_of(&self.gl);
        let mut item = WebGlFramebufferItem {
            gl_framebuffer,
            create_options: Rc::new(options),
            current_width: Rc::new(RefCell::new(width)),
            current_height: Rc::new(RefCell::new(height)),
            color_attachments: Vec::new(),
            depth_attachment: None,
            stencil_attachment: None,
        };
        self.create_attachments(
            &mut item,
            using_draw_framebuffer,
            activating_texture_unit,
            using_textures,
            capabilities,
        )?;

        Ok(item)
    }

    /// Updates framebuffer.
    /// Recreates self-hosted texture and renderbuffer if size changed.
    pub fn update_framebuffer(
        &self,
        item: &mut WebGlFramebufferItem,
        using_draw_framebuffer: &Option<WebGlFramebufferItem>,
        activating_texture_unit: WebGlTextureUnit,
        using_textures: &HashMap<
            (WebGlTextureUnit, WebGlTextureLayout),
            (WebGlTexture, WebGlSampler),
        >,
        capabilities: &WebGlCapabilities,
    ) -> Result<(), Error> {
        let (width, height) = item.create_options.size_policy.size_of(&self.gl);
        let mut current_width = item.current_width.borrow_mut();
        let mut current_height = item.current_height.borrow_mut();

        if *current_width == width && *current_height == height {
            return Ok(());
        }

        *current_width = width;
        *current_height = height;
        drop(current_width);
        drop(current_height);

        self.create_attachments(
            item,
            using_draw_framebuffer,
            activating_texture_unit,
            using_textures,
            capabilities,
        )?;

        Ok(())
    }

    fn create_attachments(
        &self,
        item: &mut WebGlFramebufferItem,
        using_draw_framebuffer: &Option<WebGlFramebufferItem>,
        activating_texture_unit: WebGlTextureUnit,
        using_textures: &HashMap<
            (WebGlTextureUnit, WebGlTextureLayout),
            (WebGlTexture, WebGlSampler),
        >,
        capabilities: &WebGlCapabilities,
    ) -> Result<(), Error> {
        self.gl.bind_framebuffer(
            WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
            Some(&item.gl_framebuffer),
        );

        let WebGlFramebufferCreateOptions {
            colors,
            depth_stencil,
            multisample,
            ..
        } = &*item.create_options;
        let width = *item.current_width.borrow();
        let height = *item.current_height.borrow();
        let multisample = multisample.and_then(|m| if m == 0 { None } else { Some(m) });
        let using_gl_draw_framebuffer = using_draw_framebuffer.as_ref().map(|i| i.gl_framebuffer());
        let using_gl_texture = using_textures
            .get(&(activating_texture_unit, WebGlTextureLayout::Texture2D))
            .map(|(t, _)| t);

        // Creates color attachments
        let max_color_attachments = capabilities.max_color_attachments();
        if max_color_attachments < colors.len() {
            warn!(
                "color attachments exceed the maximum color attachments ({max_color_attachments})"
            );
        }
        let color_attachment_len = max_color_attachments.min(colors.len());
        let mut color_attachments = Vec::with_capacity(color_attachment_len);
        for (i, color) in colors.into_iter().enumerate() {
            if i >= color_attachment_len {
                break;
            }
            let attachment = match color {
                WebGlFramebufferColorSource::CreateTexture { internal_format } => {
                    internal_format.check_color_buffer_float_supported(capabilities);

                    let gl_texture = self
                        .gl
                        .create_texture()
                        .ok_or(Error::CreateTextureFailure)?;
                    self.gl
                        .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&gl_texture));
                    self.gl.tex_storage_2d(
                        WebGl2RenderingContext::TEXTURE_2D,
                        1,
                        internal_format.to_gl_enum(),
                        width as i32,
                        height as i32,
                    );
                    self.gl.framebuffer_texture_2d(
                        WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
                        WebGl2RenderingContext::TEXTURE0 + i as u32,
                        WebGl2RenderingContext::TEXTURE_2D,
                        Some(&gl_texture),
                        0,
                    );
                    self.gl
                        .bind_texture(WebGl2RenderingContext::TEXTURE_2D, using_gl_texture);

                    Attachment::Texture { gl_texture }
                }
                WebGlFramebufferColorSource::CreateRenderbuffer { internal_format } => {
                    internal_format.check_color_buffer_float_supported(capabilities);

                    let gl_renderbuffer = self
                        .gl
                        .create_renderbuffer()
                        .ok_or(Error::CreateRenderbufferFailure)?;
                    self.gl.bind_renderbuffer(
                        WebGl2RenderingContext::RENDERBUFFER,
                        Some(&gl_renderbuffer),
                    );
                    match multisample {
                        Some(multisample) => self.gl.renderbuffer_storage_multisample(
                            WebGl2RenderingContext::RENDERBUFFER,
                            multisample as i32,
                            internal_format.to_gl_enum(),
                            width as i32,
                            height as i32,
                        ),
                        None => self.gl.renderbuffer_storage(
                            WebGl2RenderingContext::RENDERBUFFER,
                            internal_format.to_gl_enum(),
                            width as i32,
                            height as i32,
                        ),
                    };
                    self.gl.framebuffer_renderbuffer(
                        WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
                        WebGl2RenderingContext::TEXTURE0 + i as u32,
                        WebGl2RenderingContext::RENDERBUFFER,
                        Some(&gl_renderbuffer),
                    );
                    self.gl
                        .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

                    Attachment::Renderbuffer { gl_renderbuffer }
                }
                WebGlFramebufferColorSource::ExternalTexture {
                    target,
                    level,
                    gl_texture,
                } => {
                    self.gl.bind_texture(target.to_gl_enum(), Some(&gl_texture));
                    match target {
                        WebGlFramebufferTextureTarget::Texture2D
                        | WebGlFramebufferTextureTarget::TextureCubeMapPositiveX
                        | WebGlFramebufferTextureTarget::TextureCubeMapNegativeX
                        | WebGlFramebufferTextureTarget::TextureCubeMapPositiveY
                        | WebGlFramebufferTextureTarget::TextureCubeMapNegativeY
                        | WebGlFramebufferTextureTarget::TextureCubeMapPositiveZ
                        | WebGlFramebufferTextureTarget::TextureCubeMapNegativeZ => {
                            self.gl.framebuffer_texture_2d(
                                WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
                                WebGl2RenderingContext::TEXTURE0 + i as u32,
                                target.to_gl_enum(),
                                Some(&gl_texture),
                                *level as i32,
                            );
                        }
                        WebGlFramebufferTextureTarget::Texture2DArray { index: depth }
                        | WebGlFramebufferTextureTarget::Texture3D { depth } => {
                            self.gl.framebuffer_texture_layer(
                                WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
                                WebGl2RenderingContext::TEXTURE0 + i as u32,
                                Some(&gl_texture),
                                *level as i32,
                                *depth as i32,
                            );
                        }
                    }
                    self.gl.bind_texture(target.to_gl_enum(), using_gl_texture);

                    Attachment::Texture {
                        gl_texture: gl_texture.clone(),
                    }
                }
                WebGlFramebufferColorSource::ExternalRenderbuffer { gl_renderbuffer } => {
                    self.gl.bind_renderbuffer(
                        WebGl2RenderingContext::RENDERBUFFER,
                        Some(&gl_renderbuffer),
                    );
                    self.gl.framebuffer_renderbuffer(
                        WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
                        WebGl2RenderingContext::TEXTURE0 + i as u32,
                        WebGl2RenderingContext::RENDERBUFFER,
                        Some(&gl_renderbuffer),
                    );
                    self.gl
                        .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

                    Attachment::Renderbuffer {
                        gl_renderbuffer: gl_renderbuffer.clone(),
                    }
                }
            };
            color_attachments.push(attachment);
        }

        // Creates depth stencil attachments
        if let Some(depth_stencil) = depth_stencil {
            let internal_format = depth_stencil.internal_format();
            let attachment_target = match internal_format {
                WebGlFramebufferDepthStencilFormat::STENCIL_INDEX8 => {
                    WebGl2RenderingContext::STENCIL_ATTACHMENT
                }
                WebGlFramebufferDepthStencilFormat::DEPTH_COMPONENT32F
                | WebGlFramebufferDepthStencilFormat::DEPTH_COMPONENT24
                | WebGlFramebufferDepthStencilFormat::DEPTH_COMPONENT16 => {
                    WebGl2RenderingContext::DEPTH_ATTACHMENT
                }
                WebGlFramebufferDepthStencilFormat::DEPTH32F_STENCIL8
                | WebGlFramebufferDepthStencilFormat::DEPTH24_STENCIL8 => {
                    WebGl2RenderingContext::DEPTH_STENCIL_ATTACHMENT
                }
            };
            let attachment = match depth_stencil {
                WebGlFramebufferDepthStencilSource::CreateTexture { internal_format } => {
                    let gl_texture = self
                        .gl
                        .create_texture()
                        .ok_or(Error::CreateTextureFailure)?;
                    self.gl
                        .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&gl_texture));
                    self.gl.tex_storage_2d(
                        WebGl2RenderingContext::TEXTURE_2D,
                        1,
                        internal_format.to_gl_enum(),
                        width as i32,
                        height as i32,
                    );
                    self.gl.framebuffer_texture_2d(
                        WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
                        attachment_target,
                        WebGl2RenderingContext::TEXTURE_2D,
                        Some(&gl_texture),
                        0,
                    );
                    self.gl
                        .bind_texture(WebGl2RenderingContext::TEXTURE_2D, using_gl_texture);

                    Attachment::Texture { gl_texture }
                }
                WebGlFramebufferDepthStencilSource::CreateRenderbuffer { internal_format } => {
                    let gl_renderbuffer = self
                        .gl
                        .create_renderbuffer()
                        .ok_or(Error::CreateRenderbufferFailure)?;
                    self.gl.bind_renderbuffer(
                        WebGl2RenderingContext::RENDERBUFFER,
                        Some(&gl_renderbuffer),
                    );
                    match multisample {
                        Some(multisample) => self.gl.renderbuffer_storage_multisample(
                            WebGl2RenderingContext::RENDERBUFFER,
                            multisample as i32,
                            internal_format.to_gl_enum(),
                            width as i32,
                            height as i32,
                        ),
                        None => self.gl.renderbuffer_storage(
                            WebGl2RenderingContext::RENDERBUFFER,
                            internal_format.to_gl_enum(),
                            width as i32,
                            height as i32,
                        ),
                    };
                    self.gl.framebuffer_renderbuffer(
                        WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
                        attachment_target,
                        WebGl2RenderingContext::RENDERBUFFER,
                        Some(&gl_renderbuffer),
                    );
                    self.gl
                        .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

                    Attachment::Renderbuffer { gl_renderbuffer }
                }
                WebGlFramebufferDepthStencilSource::ExternalTexture {
                    target,
                    level,
                    gl_texture,
                    ..
                } => {
                    self.gl.bind_texture(target.to_gl_enum(), Some(&gl_texture));
                    match target {
                        WebGlFramebufferTextureTarget::Texture2D
                        | WebGlFramebufferTextureTarget::TextureCubeMapPositiveX
                        | WebGlFramebufferTextureTarget::TextureCubeMapNegativeX
                        | WebGlFramebufferTextureTarget::TextureCubeMapPositiveY
                        | WebGlFramebufferTextureTarget::TextureCubeMapNegativeY
                        | WebGlFramebufferTextureTarget::TextureCubeMapPositiveZ
                        | WebGlFramebufferTextureTarget::TextureCubeMapNegativeZ => {
                            self.gl.framebuffer_texture_2d(
                                WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
                                attachment_target,
                                target.to_gl_enum(),
                                Some(&gl_texture),
                                *level as i32,
                            );
                        }
                        WebGlFramebufferTextureTarget::Texture2DArray { index: depth }
                        | WebGlFramebufferTextureTarget::Texture3D { depth } => {
                            self.gl.framebuffer_texture_layer(
                                WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
                                attachment_target,
                                Some(&gl_texture),
                                *level as i32,
                                *depth as i32,
                            );
                        }
                    }
                    self.gl.bind_texture(target.to_gl_enum(), using_gl_texture);

                    Attachment::Texture {
                        gl_texture: gl_texture.clone(),
                    }
                }
                WebGlFramebufferDepthStencilSource::ExternalRenderbuffer {
                    gl_renderbuffer,
                    ..
                } => {
                    self.gl.bind_renderbuffer(
                        WebGl2RenderingContext::RENDERBUFFER,
                        Some(&gl_renderbuffer),
                    );
                    self.gl.framebuffer_renderbuffer(
                        WebGlFramebufferTarget::DrawFramebuffer.to_gl_enum(),
                        attachment_target,
                        WebGl2RenderingContext::RENDERBUFFER,
                        Some(&gl_renderbuffer),
                    );
                    self.gl
                        .bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

                    Attachment::Renderbuffer {
                        gl_renderbuffer: gl_renderbuffer.clone(),
                    }
                }
            };
            match internal_format {
                WebGlFramebufferDepthStencilFormat::STENCIL_INDEX8 => {
                    item.stencil_attachment = Some(attachment);
                }
                WebGlFramebufferDepthStencilFormat::DEPTH_COMPONENT32F
                | WebGlFramebufferDepthStencilFormat::DEPTH_COMPONENT24
                | WebGlFramebufferDepthStencilFormat::DEPTH_COMPONENT16 => {
                    item.depth_attachment = Some(attachment);
                }
                WebGlFramebufferDepthStencilFormat::DEPTH32F_STENCIL8
                | WebGlFramebufferDepthStencilFormat::DEPTH24_STENCIL8 => {
                    item.stencil_attachment = Some(attachment.clone());
                    item.depth_attachment = Some(attachment);
                }
            }
        };

        self.gl.bind_framebuffer(
            WebGl2RenderingContext::RENDERBUFFER,
            using_gl_draw_framebuffer,
        );

        Ok(())
    }
}

impl WebGlTexturePlainInternalFormat {
    fn check_color_buffer_float_supported(&self, capabilities: &WebGlCapabilities) {
        let supported = match self {
            WebGlTexturePlainInternalFormat::R16F
            | WebGlTexturePlainInternalFormat::RG16F
            | WebGlTexturePlainInternalFormat::RGBA16F
            | WebGlTexturePlainInternalFormat::R32F
            | WebGlTexturePlainInternalFormat::RG32F
            | WebGlTexturePlainInternalFormat::RGBA32F
            | WebGlTexturePlainInternalFormat::R11F_G11F_B10F => {
                capabilities.color_buffer_float_supported()
            }
            _ => return,
        };

        if !supported {
            warn!("EXT_color_buffer_float does not supported.");
        }
    }
}

impl WebGlRenderbufferInternalFormat {
    fn check_color_buffer_float_supported(&self, capabilities: &WebGlCapabilities) {
        let supported = match self {
            WebGlRenderbufferInternalFormat::R16F
            | WebGlRenderbufferInternalFormat::RG16F
            | WebGlRenderbufferInternalFormat::RGBA16F
            | WebGlRenderbufferInternalFormat::R32F
            | WebGlRenderbufferInternalFormat::RG32F
            | WebGlRenderbufferInternalFormat::RGBA32F
            | WebGlRenderbufferInternalFormat::R11F_G11F_B10F => {
                capabilities.color_buffer_float_supported()
            }
            _ => return,
        };

        if !supported {
            warn!("EXT_color_buffer_float does not supported.");
        }
    }
}
