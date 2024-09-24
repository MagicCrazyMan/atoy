use log::warn;
use proc::GlEnum;
use smallvec::SmallVec;
use web_sys::{WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture};

use super::{
    capabilities::WebGlCapabilities, error::Error, renderbuffer::WebGlRenderbufferInternalFormat,
    texture::WebGlTextureInternalFormat,
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

    // fn to_operable_buffer(&self) -> Option<OperableBuffer> {
    //     match self {
    //         FramebufferAttachmentTarget::ColorAttachment0 => Some(OperableBuffer::ColorAttachment0),
    //         FramebufferAttachmentTarget::ColorAttachment1 => Some(OperableBuffer::ColorAttachment1),
    //         FramebufferAttachmentTarget::ColorAttachment2 => Some(OperableBuffer::ColorAttachment2),
    //         FramebufferAttachmentTarget::ColorAttachment3 => Some(OperableBuffer::ColorAttachment3),
    //         FramebufferAttachmentTarget::ColorAttachment4 => Some(OperableBuffer::ColorAttachment4),
    //         FramebufferAttachmentTarget::ColorAttachment5 => Some(OperableBuffer::ColorAttachment5),
    //         FramebufferAttachmentTarget::ColorAttachment6 => Some(OperableBuffer::ColorAttachment6),
    //         FramebufferAttachmentTarget::ColorAttachment7 => Some(OperableBuffer::ColorAttachment7),
    //         FramebufferAttachmentTarget::ColorAttachment8 => Some(OperableBuffer::ColorAttachment8),
    //         FramebufferAttachmentTarget::ColorAttachment9 => Some(OperableBuffer::ColorAttachment9),
    //         FramebufferAttachmentTarget::ColorAttachment10 => {
    //             Some(OperableBuffer::ColorAttachment10)
    //         }
    //         FramebufferAttachmentTarget::ColorAttachment11 => {
    //             Some(OperableBuffer::ColorAttachment11)
    //         }
    //         FramebufferAttachmentTarget::ColorAttachment12 => {
    //             Some(OperableBuffer::ColorAttachment12)
    //         }
    //         FramebufferAttachmentTarget::ColorAttachment13 => {
    //             Some(OperableBuffer::ColorAttachment13)
    //         }
    //         FramebufferAttachmentTarget::ColorAttachment14 => {
    //             Some(OperableBuffer::ColorAttachment14)
    //         }
    //         FramebufferAttachmentTarget::ColorAttachment15 => {
    //             Some(OperableBuffer::ColorAttachment15)
    //         }
    //         FramebufferAttachmentTarget::DepthAttachment => None,
    //         FramebufferAttachmentTarget::StencilAttachment => None,
    //         FramebufferAttachmentTarget::DepthStencilAttachment => None,
    //     }
    // }
}

// /// Available drawable or readable buffer attachment mapped from [`WebGl2RenderingContext`].
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
// pub enum OperableBuffer {
//     None,
//     /// [`WebGl2RenderingContext::BACK`] only works for Canvas Draw Buffer.
//     /// Do not bind this attachment to FBO.
//     Back,
//     #[gl_enum(COLOR_ATTACHMENT0)]
//     ColorAttachment0,
//     #[gl_enum(COLOR_ATTACHMENT1)]
//     ColorAttachment1,
//     #[gl_enum(COLOR_ATTACHMENT2)]
//     ColorAttachment2,
//     #[gl_enum(COLOR_ATTACHMENT3)]
//     ColorAttachment3,
//     #[gl_enum(COLOR_ATTACHMENT4)]
//     ColorAttachment4,
//     #[gl_enum(COLOR_ATTACHMENT5)]
//     ColorAttachment5,
//     #[gl_enum(COLOR_ATTACHMENT6)]
//     ColorAttachment6,
//     #[gl_enum(COLOR_ATTACHMENT7)]
//     ColorAttachment7,
//     #[gl_enum(COLOR_ATTACHMENT8)]
//     ColorAttachment8,
//     #[gl_enum(COLOR_ATTACHMENT9)]
//     ColorAttachment9,
//     #[gl_enum(COLOR_ATTACHMENT10)]
//     ColorAttachment10,
//     #[gl_enum(COLOR_ATTACHMENT11)]
//     ColorAttachment11,
//     #[gl_enum(COLOR_ATTACHMENT12)]
//     ColorAttachment12,
//     #[gl_enum(COLOR_ATTACHMENT13)]
//     ColorAttachment13,
//     #[gl_enum(COLOR_ATTACHMENT14)]
//     ColorAttachment14,
//     #[gl_enum(COLOR_ATTACHMENT15)]
//     ColorAttachment15,
// }

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
    /// Uses the size of current drawing buffer.
    FollowDrawingBuffer,
    /// Ceiling scales the size of current drawing buffer.
    ScaleDrawingBuffer(f64),
    /// Uses a custom size.
    Custom { width: usize, height: usize },
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
        internal_format: WebGlTextureInternalFormat,
    },
    CreateRenderbuffer {
        internal_format: WebGlRenderbufferInternalFormat,
    },
    ExternalTexture {
        internal_format: Option<WebGlTextureInternalFormat>,
        texture: WebGlTexture,
    },
    ExternalRenderbuffer {
        internal_format: Option<WebGlRenderbufferInternalFormat>,
        renderbuffer: WebGlRenderbuffer,
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
        internal_format: Option<WebGlFramebufferDepthStencilFormat>,
        level: usize,
        texture: WebGlTexture,
    },
    ExternalRenderbuffer {
        internal_format: Option<WebGlFramebufferDepthStencilFormat>,
        level: usize,
        renderbuffer: WebGlRenderbuffer,
    },
}

/// WebGL framebuffer create options.
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
    pub colors: Vec<WebGlFramebufferColorSource>,
    /// Depth and stencil attachment.
    ///
    /// Factory decides what kind of attachment should be used automatically
    /// by the internal format of the texture or renderbuffer.
    pub depth_stencil: Option<WebGlFramebufferDepthStencilSource>,
}

enum ColorAttachment {
    Texture {
        internal_format: WebGlTextureInternalFormat,
        texture: WebGlTexture,
    },
    Renderbuffer {
        internal_format: WebGlRenderbufferInternalFormat,
        renderbuffer: WebGlRenderbuffer,
    },
}

enum DepthStencilAttachment {
    Texture {
        internal_format: WebGlFramebufferDepthStencilFormat,
        texture: WebGlTexture,
    },
    Renderbuffer {
        internal_format: WebGlFramebufferDepthStencilFormat,
        renderbuffer: WebGlRenderbuffer,
    },
}

pub struct WebGlFramebufferItem {
    size_policy: SizePolicy,
    current_with: usize,
    current_height: usize,
    gl_framebuffer: WebGlFramebuffer,
    colors: Vec<ColorAttachment>,
    depth: Option<DepthStencilAttachment>,
    stencil: Option<DepthStencilAttachment>,
}

impl WebGlFramebufferItem {
    /// Returns framebuffer.
    pub fn gl_framebuffer(&self) -> &WebGlFramebuffer {
        &self.gl_framebuffer
    }

    /// Returns size policy.
    pub fn size_policy(&self) -> SizePolicy {
        self.size_policy
    }

    /// Returns current width of framebuffer.
    pub fn current_with(&self) -> usize {
        self.current_with
    }

    /// Returns current height of framebuffer.
    pub fn current_height(&self) -> usize {
        self.current_height
    }

    /// Returns color texture by index.
    ///
    /// Returns `None` if no color attachment at specified index or color attachment is renderbuffer.
    pub fn color_texture(
        &self,
        index: usize,
    ) -> Option<(WebGlTexture, WebGlTextureInternalFormat)> {
        self.colors.get(index).and_then(|a| match a {
            ColorAttachment::Texture {
                internal_format,
                texture,
            } => Some((texture.clone(), *internal_format)),
            ColorAttachment::Renderbuffer { .. } => None,
        })
    }

    /// Returns color renderbuffer by index.
    ///
    /// Returns `None` if no color attachment at specified index or color attachment is texture.
    pub fn color_renderbuffer(
        &self,
        index: usize,
    ) -> Option<(WebGlRenderbuffer, WebGlRenderbufferInternalFormat)> {
        self.colors.get(index).and_then(|a| match a {
            ColorAttachment::Texture { .. } => None,
            ColorAttachment::Renderbuffer {
                internal_format,
                renderbuffer,
            } => Some((renderbuffer.clone(), *internal_format)),
        })
    }

    /// Returns depth texture.
    ///
    /// Returns `None` if no depth attachment or depth attachment is renderbuffer.
    ///
    /// Returns as same as [`stencil_texture`](WebGlFramebufferItem::stencil_texture)
    /// if this is a depth stencil combination format.
    pub fn depth_texture(&self) -> Option<(WebGlTexture, WebGlFramebufferDepthStencilFormat)> {
        self.depth.as_ref().and_then(|a| match a {
            DepthStencilAttachment::Texture {
                internal_format,
                texture,
            } => Some((texture.clone(), *internal_format)),
            DepthStencilAttachment::Renderbuffer { .. } => None,
        })
    }

    /// Returns depth renderbuffer.
    ///
    /// Returns `None` if no depth attachment or depth attachment is texture.
    ///
    /// Returns as same as [`stencil_renderbuffer`](WebGlFramebufferItem::stencil_renderbuffer)
    /// if this is a depth stencil combination format.
    pub fn depth_renderbuffer(
        &self,
    ) -> Option<(WebGlRenderbuffer, WebGlFramebufferDepthStencilFormat)> {
        self.depth.as_ref().and_then(|a| match a {
            DepthStencilAttachment::Texture { .. } => None,
            DepthStencilAttachment::Renderbuffer {
                internal_format,
                renderbuffer,
            } => Some((renderbuffer.clone(), *internal_format)),
        })
    }

    /// Returns stencil texture.
    ///
    /// Returns `None` if no stencil attachment or stencil attachment is renderbuffer.
    ///
    /// Returns as same as [`depth_texture`](WebGlFramebufferItem::depth_texture)
    /// if this is a depth stencil combination format.
    pub fn stencil_texture(&self) -> Option<(WebGlTexture, WebGlFramebufferDepthStencilFormat)> {
        self.stencil.as_ref().and_then(|a| match a {
            DepthStencilAttachment::Texture {
                internal_format,
                texture,
            } => Some((texture.clone(), *internal_format)),
            DepthStencilAttachment::Renderbuffer { .. } => None,
        })
    }

    /// Returns stencil renderbuffer.
    ///
    /// Returns `None` if no stencil attachment or stencil attachment is texture.
    ///
    /// Returns as same as [`depth_renderbuffer`](WebGlFramebufferItem::depth_renderbuffer)
    /// if this is a depth stencil combination format.
    pub fn stencil_renderbuffer(
        &self,
    ) -> Option<(WebGlRenderbuffer, WebGlFramebufferDepthStencilFormat)> {
        self.stencil.as_ref().and_then(|a| match a {
            DepthStencilAttachment::Texture { .. } => None,
            DepthStencilAttachment::Renderbuffer {
                internal_format,
                renderbuffer,
            } => Some((renderbuffer.clone(), *internal_format)),
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
        capabilities: &WebGlCapabilities,
    ) -> Result<WebGlFramebufferItem, Error> {
        let framebuffer = self
            .gl
            .create_framebuffer()
            .ok_or(Error::CreateFramebufferFailure)?;

        let max_color_attachments = capabilities.max_color_attachments();
        if max_color_attachments < options.colors.len() {
            warn!("color attachments exceed the maximum color attachments");
        }
        let color_len = max_color_attachments.min(options.colors.len());
        let mut colors = Vec::with_capacity(color_len);
        for i in 0..color_len {
            let source = &options.colors[i];
            let attachment = match source {
                WebGlFramebufferColorSource::CreateTexture { internal_format } => {
                   let texture= self.gl
                        .create_texture()
                        .ok_or(Error::CreateTextureFailure)?;
                    self.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
                    self.gl.tex_storage_2d(WebGl2RenderingContext::TEXTURE_2D, 1, internal_format.to_gl_enum(), width, height);
                    self.gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
                }
                WebGlFramebufferColorSource::CreateRenderbuffer { internal_format } => todo!(),
                WebGlFramebufferColorSource::ExternalTexture {
                    internal_format,
                    texture,
                } => todo!(),
                WebGlFramebufferColorSource::ExternalRenderbuffer {
                    internal_format,
                    renderbuffer,
                } => todo!(),
            };
            colors.push(attachment);
        }
        todo!()
    }
}
