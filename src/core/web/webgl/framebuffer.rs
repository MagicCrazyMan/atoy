use std::{cell::RefCell, collections::VecDeque, iter::FromIterator, rc::Rc};

use hashbrown::{HashMap, HashSet};
use js_sys::{ArrayBuffer, Uint8Array};
use proc::GlEnum;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::{
    js_sys::Array, WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlRenderbuffer,
    WebGlTexture,
};

use crate::core::web::webgl::{renderbuffer::RenderbufferTarget, texture::TextureRegistry};

use super::{
    blit::{BlitFilter, BlitMask},
    buffer::{
        Buffer, BufferRegistered, BufferRegisteredUndrop, BufferRegistry, BufferTarget, BufferUsage,
    },
    client_wait::ClientWaitAsync,
    error::Error,
    pixel::{PixelDataType, PixelFormat, PixelPackStorage},
    renderbuffer::RenderbufferInternalFormat,
    texture::{
        Texture, Texture2D, Texture2DArray, Texture3D, TextureCubeMap, TextureCubeMapFace,
        TextureTarget, TextureUncompressedInternalFormat,
    },
};

/// Available framebuffer targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum FramebufferTarget {
    ReadFramebuffer,
    DrawFramebuffer,
}

/// Available framebuffer attachment targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum FramebufferAttachment {
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

impl FramebufferAttachment {
    fn to_operable_buffer(&self) -> Option<OperableBuffer> {
        match self {
            FramebufferAttachment::ColorAttachment0 => Some(OperableBuffer::ColorAttachment0),
            FramebufferAttachment::ColorAttachment1 => Some(OperableBuffer::ColorAttachment1),
            FramebufferAttachment::ColorAttachment2 => Some(OperableBuffer::ColorAttachment2),
            FramebufferAttachment::ColorAttachment3 => Some(OperableBuffer::ColorAttachment3),
            FramebufferAttachment::ColorAttachment4 => Some(OperableBuffer::ColorAttachment4),
            FramebufferAttachment::ColorAttachment5 => Some(OperableBuffer::ColorAttachment5),
            FramebufferAttachment::ColorAttachment6 => Some(OperableBuffer::ColorAttachment6),
            FramebufferAttachment::ColorAttachment7 => Some(OperableBuffer::ColorAttachment7),
            FramebufferAttachment::ColorAttachment8 => Some(OperableBuffer::ColorAttachment8),
            FramebufferAttachment::ColorAttachment9 => Some(OperableBuffer::ColorAttachment9),
            FramebufferAttachment::ColorAttachment10 => Some(OperableBuffer::ColorAttachment10),
            FramebufferAttachment::ColorAttachment11 => Some(OperableBuffer::ColorAttachment11),
            FramebufferAttachment::ColorAttachment12 => Some(OperableBuffer::ColorAttachment12),
            FramebufferAttachment::ColorAttachment13 => Some(OperableBuffer::ColorAttachment13),
            FramebufferAttachment::ColorAttachment14 => Some(OperableBuffer::ColorAttachment14),
            FramebufferAttachment::ColorAttachment15 => Some(OperableBuffer::ColorAttachment15),
            FramebufferAttachment::DepthAttachment => None,
            FramebufferAttachment::StencilAttachment => None,
            FramebufferAttachment::DepthStencilAttachment => None,
        }
    }

    fn to_clear_index(&self) -> i32 {
        match self {
            FramebufferAttachment::ColorAttachment0 => 0,
            FramebufferAttachment::ColorAttachment1 => 1,
            FramebufferAttachment::ColorAttachment2 => 2,
            FramebufferAttachment::ColorAttachment3 => 3,
            FramebufferAttachment::ColorAttachment4 => 4,
            FramebufferAttachment::ColorAttachment5 => 5,
            FramebufferAttachment::ColorAttachment6 => 6,
            FramebufferAttachment::ColorAttachment7 => 7,
            FramebufferAttachment::ColorAttachment8 => 8,
            FramebufferAttachment::ColorAttachment9 => 9,
            FramebufferAttachment::ColorAttachment10 => 10,
            FramebufferAttachment::ColorAttachment11 => 11,
            FramebufferAttachment::ColorAttachment12 => 12,
            FramebufferAttachment::ColorAttachment13 => 13,
            FramebufferAttachment::ColorAttachment14 => 14,
            FramebufferAttachment::ColorAttachment15 => 15,
            FramebufferAttachment::DepthAttachment => 0,
            FramebufferAttachment::StencilAttachment => 0,
            FramebufferAttachment::DepthStencilAttachment => 0,
        }
    }
}

/// Available drawable or readable buffer attachment mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, GlEnum)]
pub enum OperableBuffer {
    None,
    /// [`WebGl2RenderingContext::BACK`] only works for Canvas Draw Buffer.
    /// Do not bind this attachment to FBO.
    Back,
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
}

/// Available framebuffer size policies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizePolicy {
    FollowDrawingBuffer,
    ScaleDrawingBuffer(f64),
    Custom { width: usize, height: usize },
}

impl SizePolicy {
    fn size(&self, gl: &WebGl2RenderingContext) -> (usize, usize) {
        match self {
            Self::FollowDrawingBuffer => {
                let width = gl.drawing_buffer_width() as usize;
                let height = gl.drawing_buffer_height() as usize;
                (width, height)
            }
            Self::ScaleDrawingBuffer(scale) => {
                let width = gl.drawing_buffer_width();
                let height = gl.drawing_buffer_height();
                (
                    (width as f64 * scale).round() as usize,
                    (height as f64 * scale).round() as usize,
                )
            }
            Self::Custom { width, height } => (*width, *height),
        }
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
    fn clear(&self, gl: &WebGl2RenderingContext, attachment: FramebufferAttachment) {
        let index = attachment.to_clear_index();
        match self {
            ClearPolicy::ColorFloat(values) => {
                gl.clear_bufferfv_with_f32_array(WebGl2RenderingContext::COLOR, index, values)
            }
            ClearPolicy::ColorInteger(values) => {
                gl.clear_bufferiv_with_i32_array(WebGl2RenderingContext::COLOR, index, values)
            }
            ClearPolicy::ColorUnsignedInteger(values) => {
                gl.clear_bufferuiv_with_u32_array(WebGl2RenderingContext::COLOR, index, values)
            }
            ClearPolicy::Depth(depth) => {
                gl.clear_bufferfv_with_f32_array(WebGl2RenderingContext::DEPTH, index, &[*depth])
            }
            ClearPolicy::Stencil(stencil) => gl.clear_bufferiv_with_i32_array(
                WebGl2RenderingContext::STENCIL,
                index,
                &[*stencil],
            ),
            ClearPolicy::DepthStencil(depth, stencil) => gl.clear_bufferfi(
                WebGl2RenderingContext::DEPTH_STENCIL,
                index,
                *depth,
                *stencil,
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttachmentSource {
    CreateTexture2D {
        internal_format: TextureUncompressedInternalFormat,
        clear_policy: ClearPolicy,
    },
    FromTexture2D {
        texture: WebGlTexture,
        level: usize,
        clear_policy: ClearPolicy,
    },
    FromTextureCubeMap {
        texture: WebGlTexture,
        level: usize,
        face: TextureCubeMapFace,
        clear_policy: ClearPolicy,
    },
    FromTextureLayer {
        texture: WebGlTexture,
        level: usize,
        layer: usize,
        clear_policy: ClearPolicy,
    },
    CreateRenderbuffer {
        internal_format: RenderbufferInternalFormat,
        clear_policy: ClearPolicy,
    },
    FromRenderbuffer {
        renderbuffer: WebGlRenderbuffer,
        clear_policy: ClearPolicy,
    },
}

impl AttachmentSource {
    pub fn new_texture(internal_format: TextureUncompressedInternalFormat) -> Self {
        let clear_policy = match internal_format {
            TextureUncompressedInternalFormat::RGBA32I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::RGBA32UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::RGBA16I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::RGBA16UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::RGBA8 => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGBA8I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::RGBA8UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::SRGB8_ALPHA8 => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGB10_A2 => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGB10_A2UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::RGBA4 => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGB5_A1 => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGB8 => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGB565 => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RG32I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::RG32UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::RG16I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::RG16UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::RG8 => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            TextureUncompressedInternalFormat::RG8I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::RG8UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::R32I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::R32UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::R16I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::R16UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::R8 => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            TextureUncompressedInternalFormat::R8I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::R8UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::RGBA32F => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGBA16F => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGBA8_SNORM => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGB32F => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGB32I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::RGB32UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::RGB16F => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGB16I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::RGB16UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::RGB8_SNORM => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGB8I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            TextureUncompressedInternalFormat::RGB8UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            TextureUncompressedInternalFormat::SRGB8 => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::R11F_G11F_B10F => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RGB9_E5 => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RG32F => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RG16F => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::RG8_SNORM => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::R32F => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::R16F => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::R8_SNORM => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            TextureUncompressedInternalFormat::DEPTH_COMPONENT32F => ClearPolicy::Depth(1.0),
            TextureUncompressedInternalFormat::DEPTH_COMPONENT24 => ClearPolicy::Depth(1.0),
            TextureUncompressedInternalFormat::DEPTH_COMPONENT16 => ClearPolicy::Depth(1.0),
            TextureUncompressedInternalFormat::DEPTH32F_STENCIL8 => {
                ClearPolicy::DepthStencil(1.0, 0)
            }
            TextureUncompressedInternalFormat::DEPTH24_STENCIL8 => {
                ClearPolicy::DepthStencil(1.0, 0)
            }
        };

        Self::CreateTexture2D {
            internal_format,
            clear_policy,
        }
    }

    pub fn new_renderbuffer(internal_format: RenderbufferInternalFormat) -> Self {
        let clear_policy = match internal_format {
            RenderbufferInternalFormat::RGBA32I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::RGBA32UI => ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::RGBA16I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::RGBA16UI => ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::RGBA8 => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::RGBA8I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::RGBA8UI => ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::SRGB8_ALPHA8 => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
            RenderbufferInternalFormat::RGB10_A2 => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::RGB10_A2UI => {
                ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0])
            }
            RenderbufferInternalFormat::RGBA4 => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::RGB5_A1 => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::RGB8 => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::RGB565 => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::RG32I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::RG32UI => ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::RG16I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::RG16UI => ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::RG8 => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::RG8I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::RG8UI => ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::R32I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::R32UI => ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::R16I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::R16UI => ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::R8 => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::R8I => ClearPolicy::ColorInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::R8UI => ClearPolicy::ColorUnsignedInteger([0, 0, 0, 0]),
            RenderbufferInternalFormat::DEPTH_COMPONENT32F => ClearPolicy::Depth(1.0),
            RenderbufferInternalFormat::DEPTH_COMPONENT24 => ClearPolicy::Depth(1.0),
            RenderbufferInternalFormat::DEPTH_COMPONENT16 => ClearPolicy::Depth(1.0),
            RenderbufferInternalFormat::DEPTH32F_STENCIL8 => ClearPolicy::DepthStencil(1.0, 0),
            RenderbufferInternalFormat::DEPTH24_STENCIL8 => ClearPolicy::DepthStencil(1.0, 0),
            RenderbufferInternalFormat::R16F => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::RG16F => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::RGBA16F => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::R32F => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::RG32F => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::RGBA32F => ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]),
            RenderbufferInternalFormat::R11F_G11F_B10F => {
                ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0])
            }
        };

        Self::CreateRenderbuffer {
            internal_format,
            clear_policy,
        }
    }

    pub fn new_texture_with_clear_policy(
        internal_format: TextureUncompressedInternalFormat,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::CreateTexture2D {
            internal_format,
            clear_policy,
        }
    }

    pub fn new_renderbuffer_with_clear_policy(
        internal_format: RenderbufferInternalFormat,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::CreateRenderbuffer {
            internal_format,
            clear_policy,
        }
    }

    pub fn from_texture(texture: WebGlTexture, level: usize, clear_policy: ClearPolicy) -> Self {
        Self::FromTexture2D {
            texture,
            level,
            clear_policy,
        }
    }

    pub fn from_renderbuffer(renderbuffer: WebGlRenderbuffer, clear_policy: ClearPolicy) -> Self {
        Self::FromRenderbuffer {
            renderbuffer,
            clear_policy,
        }
    }

    fn clear_policy(&self) -> &ClearPolicy {
        match &self {
            AttachmentSource::CreateTexture2D { clear_policy, .. }
            | AttachmentSource::FromTexture2D { clear_policy, .. }
            | AttachmentSource::FromTextureCubeMap { clear_policy, .. }
            | AttachmentSource::FromTextureLayer { clear_policy, .. }
            | AttachmentSource::CreateRenderbuffer { clear_policy, .. }
            | AttachmentSource::FromRenderbuffer { clear_policy, .. } => clear_policy,
        }
    }
}

#[derive(Debug)]
pub struct Framebuffer {
    id: Uuid,
    size_policy: SizePolicy,
    renderbuffer_samples: Option<usize>,
    sources: HashMap<FramebufferAttachment, AttachmentSource>,

    registered: Rc<RefCell<Option<FramebufferRegistered>>>,
}

impl Framebuffer {
    /// Constructs a new framebuffer object.
    pub fn new<S>(sources: S, size_policy: SizePolicy, renderbuffer_samples: Option<usize>) -> Self
    where
        S: IntoIterator<Item = (FramebufferAttachment, AttachmentSource)>,
    {
        let sources = HashMap::from_iter(sources);

        let renderbuffer_samples =
            renderbuffer_samples
                .and_then(|samples| if samples == 0 { None } else { Some(samples) });

        Self {
            id: Uuid::new_v4(),
            size_policy,
            sources,
            renderbuffer_samples,

            registered: Rc::new(RefCell::new(None)),
        }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn size_policy(&self) -> SizePolicy {
        self.size_policy
    }

    pub fn renderbuffer_samples(&self) -> Option<usize> {
        self.renderbuffer_samples
    }

    pub fn sources(&self) -> &HashMap<FramebufferAttachment, AttachmentSource> {
        &self.sources
    }

    pub fn gl_framebuffer(&self) -> Result<WebGlFramebuffer, Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .map(|r| r.gl_framebuffer.clone())
            .ok_or(Error::FramebufferUnregistered)
    }

    pub fn gl_textures(&self) -> Result<HashMap<FramebufferAttachment, WebGlTexture>, Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .map(|r| {
                r.gl_textures
                    .iter()
                    .map(|(attachment, (gl_texture, _, _))| (*attachment, gl_texture.clone()))
                    .collect()
            })
            .ok_or(Error::FramebufferUnregistered)
    }

    pub fn gl_renderbuffers(
        &self,
    ) -> Result<HashMap<FramebufferAttachment, WebGlRenderbuffer>, Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .map(|r| {
                r.gl_renderbuffers
                    .iter()
                    .map(|(attachment, (gl_renderbuffer, _, _))| {
                        (*attachment, gl_renderbuffer.clone())
                    })
                    .collect()
            })
            .ok_or(Error::FramebufferUnregistered)
    }

    pub fn gl_texture(&self, attachment: FramebufferAttachment) -> Result<WebGlTexture, Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .and_then(|r| r.gl_textures.get(&attachment))
            .map(|(gl_texture, _, _)| gl_texture.clone())
            .ok_or(Error::FramebufferUnregistered)
    }

    pub fn gl_renderbuffer(
        &self,
        attachment: FramebufferAttachment,
    ) -> Result<WebGlRenderbuffer, Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .and_then(|r| r.gl_renderbuffers.get(&attachment))
            .map(|(gl_renderbuffer, _, _)| gl_renderbuffer.clone())
            .ok_or(Error::FramebufferUnregistered)
    }

    pub fn bind_as_draw(&self, draw_buffers: Option<Vec<OperableBuffer>>) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .bind(FramebufferTarget::ReadFramebuffer, None, draw_buffers)
    }

    pub fn bind_as_read(&self, read_buffer: Option<OperableBuffer>) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .bind(FramebufferTarget::ReadFramebuffer, read_buffer, None)
    }

    pub fn unbind_as_draw(&self) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .unbind(FramebufferTarget::DrawFramebuffer);

        Ok(())
    }

    pub fn unbind_as_read(&self) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .unbind(FramebufferTarget::ReadFramebuffer);

        Ok(())
    }

    pub fn unbind_all(&self) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .unbind_all();

        Ok(())
    }

    pub fn clear(&self, attachment: FramebufferAttachment) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .clear(attachment)
    }

    pub fn clear_all(&self) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .clear_all()
    }

    pub fn size(&self) -> Result<(usize, usize), Error> {
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::FramebufferUnregistered)?;
        registered.temp_bind(FramebufferTarget::DrawFramebuffer, &None, &None)?;
        registered.build(FramebufferTarget::DrawFramebuffer)?;
        registered.temp_unbind(FramebufferTarget::DrawFramebuffer, &None, &None);

        Ok(registered.framebuffer_size.unwrap().clone())
    }

    pub fn blit_to(&self, to: &Self) -> Result<(), Error> {
        self.blit_to_with_params(
            None,
            None,
            None,
            None,
            None,
            to,
            None,
            None,
            None,
            None,
            None,
            BlitMask::ColorBufferBit,
            BlitFilter::Linear,
        )
    }

    pub fn blit_to_with_params(
        &self,
        read_buffer: Option<OperableBuffer>,
        src_x0: Option<usize>,
        src_y0: Option<usize>,
        src_x1: Option<usize>,
        src_y1: Option<usize>,
        to: &Self,
        draw_buffers: Option<Vec<OperableBuffer>>,
        dst_x0: Option<usize>,
        dst_y0: Option<usize>,
        dst_x1: Option<usize>,
        dst_y1: Option<usize>,
        mask: BlitMask,
        filter: BlitFilter,
    ) -> Result<(), Error> {
        let mut from = self.registered.borrow_mut();
        let mut to = to.registered.borrow_mut();
        let (from, to) = (
            from.as_mut().ok_or(Error::FramebufferUnregistered)?,
            to.as_mut().ok_or(Error::FramebufferUnregistered)?,
        );

        from.blit_to(
            read_buffer,
            src_x0,
            src_y0,
            src_x1,
            src_y1,
            to,
            draw_buffers,
            dst_x0,
            dst_y0,
            dst_x1,
            dst_y1,
            mask,
            filter,
        )
    }

    pub fn read_pixels_to_array_buffer(
        &self,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
    ) -> Result<ArrayBuffer, Error> {
        self.read_pixels_to_array_buffer_with_params(
            pixel_format,
            pixel_data_type,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    pub fn read_pixels_to_array_buffer_with_params(
        &self,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        pixel_pack_storages: Option<Vec<PixelPackStorage>>,
        read_buffer: Option<OperableBuffer>,
        x: Option<usize>,
        y: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        dst_offset: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        let readback = self
            .registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .read_pixels(
                ReadPixelsKind::NewArrayBuffer,
                pixel_format,
                pixel_data_type,
                pixel_pack_storages,
                read_buffer,
                x,
                y,
                width,
                height,
                dst_offset,
            )?;
        match readback {
            ReadPixels::ArrayBuffer(array_buffer) => Ok(array_buffer),
            ReadPixels::NewPixelBufferObject(_, _, _) | ReadPixels::ToPixelBufferObject => {
                unreachable!()
            }
        }
    }

    pub fn read_pixels_to_new_array_buffer(
        &self,
        to: ArrayBuffer,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
    ) -> Result<ArrayBuffer, Error> {
        self.read_pixels_to_new_array_buffer_with_params(
            to,
            pixel_format,
            pixel_data_type,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    pub fn read_pixels_to_new_array_buffer_with_params(
        &self,
        to: ArrayBuffer,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        pixel_pack_storages: Option<Vec<PixelPackStorage>>,
        read_buffer: Option<OperableBuffer>,
        x: Option<usize>,
        y: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        dst_offset: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        let readback = self
            .registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .read_pixels(
                ReadPixelsKind::ToArrayBuffer(to),
                pixel_format,
                pixel_data_type,
                pixel_pack_storages,
                read_buffer,
                x,
                y,
                width,
                height,
                dst_offset,
            )?;
        match readback {
            ReadPixels::ArrayBuffer(array_buffer) => Ok(array_buffer),
            ReadPixels::NewPixelBufferObject(_, _, _) | ReadPixels::ToPixelBufferObject => {
                unreachable!()
            }
        }
    }

    pub async fn read_pixels_to_array_buffer_async(
        &self,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
    ) -> Result<ArrayBuffer, Error> {
        self.read_pixels_to_array_buffer_with_params_async(
            pixel_format,
            pixel_data_type,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
    }

    pub async fn read_pixels_to_array_buffer_with_params_async(
        &self,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        pixel_pack_storages: Option<Vec<PixelPackStorage>>,
        read_buffer: Option<OperableBuffer>,
        x: Option<usize>,
        y: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        dst_offset: Option<usize>,
        max_retries: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        let readback = self
            .registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .read_pixels_async(
                ReadPixelsKind::NewArrayBuffer,
                pixel_format,
                pixel_data_type,
                pixel_pack_storages,
                read_buffer,
                x,
                y,
                width,
                height,
                dst_offset,
                max_retries,
            )
            .await?;

        match readback {
            ReadPixels::ArrayBuffer(array_buffer) => Ok(array_buffer),
            ReadPixels::NewPixelBufferObject(_, _, _) | ReadPixels::ToPixelBufferObject => {
                unreachable!()
            }
        }
    }

    pub async fn read_pixels_to_new_array_buffer_async(
        &self,
        to: ArrayBuffer,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
    ) -> Result<ArrayBuffer, Error> {
        self.read_pixels_to_new_array_buffer_with_params_async(
            to,
            pixel_format,
            pixel_data_type,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
    }

    pub async fn read_pixels_to_new_array_buffer_with_params_async(
        &self,
        to: ArrayBuffer,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        pixel_pack_storages: Option<Vec<PixelPackStorage>>,
        read_buffer: Option<OperableBuffer>,
        x: Option<usize>,
        y: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        dst_offset: Option<usize>,
        max_retries: Option<usize>,
    ) -> Result<ArrayBuffer, Error> {
        let readback = self
            .registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .read_pixels_async(
                ReadPixelsKind::ToArrayBuffer(to),
                pixel_format,
                pixel_data_type,
                pixel_pack_storages,
                read_buffer,
                x,
                y,
                width,
                height,
                dst_offset,
                max_retries,
            )
            .await?;

        match readback {
            ReadPixels::ArrayBuffer(array_buffer) => Ok(array_buffer),
            ReadPixels::NewPixelBufferObject(_, _, _) | ReadPixels::ToPixelBufferObject => {
                unreachable!()
            }
        }
    }

    pub fn read_pixels_to_new_pbo(
        &self,
        pixel_buffer_object_usage: BufferUsage,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
    ) -> Result<Buffer, Error> {
        self.read_pixels_to_new_pbo_with_params(
            pixel_buffer_object_usage,
            pixel_format,
            pixel_data_type,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    pub fn read_pixels_to_new_pbo_with_params(
        &self,
        pixel_buffer_object_usage: BufferUsage,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        pixel_pack_storages: Option<Vec<PixelPackStorage>>,
        read_buffer: Option<OperableBuffer>,
        x: Option<usize>,
        y: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        dst_offset: Option<usize>,
    ) -> Result<Buffer, Error> {
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::FramebufferUnregistered)?;
        let readback = registered.read_pixels(
            ReadPixelsKind::NewPixelBufferObject(pixel_buffer_object_usage),
            pixel_format,
            pixel_data_type,
            pixel_pack_storages,
            read_buffer,
            x,
            y,
            width,
            height,
            dst_offset,
        )?;

        match readback {
            ReadPixels::NewPixelBufferObject(gl_buffer, size, usage) => {
                // wraps native WebGlBuffer to Buffer
                *registered.buffer_registry.used_size.borrow_mut() += size;

                let queue = Rc::new(RefCell::new(VecDeque::new()));
                let registered = BufferRegistered(BufferRegisteredUndrop {
                    gl: registered.gl.clone(),
                    gl_buffer,
                    gl_bounds: HashSet::new(),

                    reg_id: registered.buffer_registry.id,
                    reg_bounds: Rc::clone(&registered.buffer_registry.bounds),
                    reg_used_size: Rc::downgrade(&registered.buffer_registry.used_size),

                    buffer_size: size,
                    buffer_usage: pixel_buffer_object_usage,
                    buffer_queue: Rc::downgrade(&queue),
                    buffer_async_upload: Rc::new(RefCell::new(None)),

                    restore_when_drop: false,
                });

                Ok(Buffer {
                    id: Uuid::new_v4(),
                    usage,
                    queue,
                    registered: Rc::new(RefCell::new(Some(registered))),
                })
            }
            ReadPixels::ArrayBuffer(_) | ReadPixels::ToPixelBufferObject => unreachable!(),
        }
    }

    pub fn read_pixels_to_pbo(
        &self,
        to: Buffer,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
    ) -> Result<Buffer, Error> {
        self.read_pixels_to_pbo_with_params(
            to,
            pixel_format,
            pixel_data_type,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    pub fn read_pixels_to_pbo_with_params(
        &self,
        to: Buffer,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        pixel_pack_storages: Option<Vec<PixelPackStorage>>,
        read_buffer: Option<OperableBuffer>,
        x: Option<usize>,
        y: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        dst_offset: Option<usize>,
    ) -> Result<Buffer, Error> {
        let readback = self
            .registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .read_pixels(
                ReadPixelsKind::ToPixelBufferObject(
                    to.registered
                        .borrow_mut()
                        .as_mut()
                        .ok_or(Error::FramebufferUnregistered)?
                        .0
                        .gl_buffer
                        .clone(),
                ),
                pixel_format,
                pixel_data_type,
                pixel_pack_storages,
                read_buffer,
                x,
                y,
                width,
                height,
                dst_offset,
            )?;

        match readback {
            ReadPixels::ToPixelBufferObject => Ok(to),
            ReadPixels::ArrayBuffer(_) | ReadPixels::NewPixelBufferObject(_, _, _) => {
                unreachable!()
            }
        }
    }

    pub fn copy_to_new_texture_2d(
        &self,
        internal_format: TextureUncompressedInternalFormat,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<Texture<Texture2D, TextureUncompressedInternalFormat>, Error> {
        self.copy_to_new_texture_2d_with_params(
            internal_format,
            x,
            y,
            width,
            height,
            None,
            None,
            None,
            None,
        )
    }

    pub fn copy_to_new_texture_2d_with_params(
        &self,
        internal_format: TextureUncompressedInternalFormat,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        read_buffer: Option<OperableBuffer>,
        level: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
    ) -> Result<Texture<Texture2D, TextureUncompressedInternalFormat>, Error> {
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::FramebufferUnregistered)?;
        let result = registered.copy_to_texture(
            CopyTextureKind::NewTexture2D { internal_format },
            x,
            y,
            width,
            height,
            read_buffer,
            level,
            x_offset,
            y_offset,
            None,
        )?;

        match result {
            CopyTexture::NewTexture2D {
                gl_texture,
                levels,
                width,
                height,
            } => registered.texture_registry.capture_2d(
                gl_texture,
                Texture2D::new(levels, width, height),
                internal_format,
            ),
            _ => unreachable!(),
        }
    }

    pub fn copy_to_texture_2d(
        &self,
        texture: Texture<Texture2D, TextureUncompressedInternalFormat>,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<Texture<Texture2D, TextureUncompressedInternalFormat>, Error> {
        self.copy_to_texture_2d_with_params(texture, x, y, width, height, None, None, None, None)
    }

    pub fn copy_to_texture_2d_with_params(
        &self,
        texture: Texture<Texture2D, TextureUncompressedInternalFormat>,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        read_buffer: Option<OperableBuffer>,
        level: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
    ) -> Result<Texture<Texture2D, TextureUncompressedInternalFormat>, Error> {
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::FramebufferUnregistered)?;
        let result = registered.copy_to_texture(
            CopyTextureKind::ToTexture2D {
                gl_texture: texture.gl_texture()?,
            },
            x,
            y,
            width,
            height,
            read_buffer,
            level,
            x_offset,
            y_offset,
            None,
        )?;

        match result {
            CopyTexture::ToTexture2D => Ok(texture),
            _ => unreachable!(),
        }
    }

    pub fn copy_to_new_texture_cube_map(
        &self,
        face: TextureCubeMapFace,
        internal_format: TextureUncompressedInternalFormat,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<Texture<TextureCubeMap, TextureUncompressedInternalFormat>, Error> {
        self.copy_to_new_texture_cube_map_with_params(
            face,
            internal_format,
            x,
            y,
            width,
            height,
            None,
            None,
            None,
            None,
        )
    }

    pub fn copy_to_new_texture_cube_map_with_params(
        &self,
        face: TextureCubeMapFace,
        internal_format: TextureUncompressedInternalFormat,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        read_buffer: Option<OperableBuffer>,
        level: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
    ) -> Result<Texture<TextureCubeMap, TextureUncompressedInternalFormat>, Error> {
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::FramebufferUnregistered)?;
        let result = registered.copy_to_texture(
            CopyTextureKind::NewTextureCubeMap {
                face,
                internal_format,
            },
            x,
            y,
            width,
            height,
            read_buffer,
            level,
            x_offset,
            y_offset,
            None,
        )?;

        match result {
            CopyTexture::NewTextureCubeMap {
                gl_texture,
                levels,
                width,
                height,
            } => registered.texture_registry.capture_cube_map(
                gl_texture,
                TextureCubeMap::new(levels, width, height),
                internal_format,
            ),
            _ => unreachable!(),
        }
    }

    pub fn copy_to_texture_cube_map(
        &self,
        face: TextureCubeMapFace,
        texture: Texture<TextureCubeMap, TextureUncompressedInternalFormat>,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<Texture<TextureCubeMap, TextureUncompressedInternalFormat>, Error> {
        self.copy_to_texture_cube_map_with_params(
            face, texture, x, y, width, height, None, None, None, None,
        )
    }

    pub fn copy_to_texture_cube_map_with_params(
        &self,
        face: TextureCubeMapFace,
        texture: Texture<TextureCubeMap, TextureUncompressedInternalFormat>,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        read_buffer: Option<OperableBuffer>,
        level: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
    ) -> Result<Texture<TextureCubeMap, TextureUncompressedInternalFormat>, Error> {
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::FramebufferUnregistered)?;
        let result = registered.copy_to_texture(
            CopyTextureKind::ToTextureCubeMap {
                face,
                gl_texture: texture.gl_texture()?,
            },
            x,
            y,
            width,
            height,
            read_buffer,
            level,
            x_offset,
            y_offset,
            None,
        )?;

        match result {
            CopyTexture::ToTextureCubeMap => Ok(texture),
            _ => unreachable!(),
        }
    }

    pub fn copy_to_new_texture_3d(
        &self,
        internal_format: TextureUncompressedInternalFormat,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<Texture<Texture3D, TextureUncompressedInternalFormat>, Error> {
        self.copy_to_new_texture_3d_with_params(
            internal_format,
            x,
            y,
            width,
            height,
            None,
            None,
            None,
            None,
            None,
        )
    }

    pub fn copy_to_new_texture_3d_with_params(
        &self,
        internal_format: TextureUncompressedInternalFormat,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        read_buffer: Option<OperableBuffer>,
        level: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
        z_offset: Option<usize>,
    ) -> Result<Texture<Texture3D, TextureUncompressedInternalFormat>, Error> {
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::FramebufferUnregistered)?;
        let result = registered.copy_to_texture(
            CopyTextureKind::NewTexture3D { internal_format },
            x,
            y,
            width,
            height,
            read_buffer,
            level,
            x_offset,
            y_offset,
            z_offset,
        )?;

        match result {
            CopyTexture::NewTexture3D {
                gl_texture,
                levels,
                width,
                height,
                depth,
            } => registered.texture_registry.capture_3d(
                gl_texture,
                Texture3D::new(levels, width, height, depth),
                internal_format,
            ),
            _ => unreachable!(),
        }
    }

    pub fn copy_to_texture_3d(
        &self,
        texture: Texture<Texture3D, TextureUncompressedInternalFormat>,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<Texture<Texture3D, TextureUncompressedInternalFormat>, Error> {
        self.copy_to_texture_3d_with_params(
            texture, x, y, width, height, None, None, None, None, None,
        )
    }

    pub fn copy_to_texture_3d_with_params(
        &self,
        texture: Texture<Texture3D, TextureUncompressedInternalFormat>,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        read_buffer: Option<OperableBuffer>,
        level: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
        z_offset: Option<usize>,
    ) -> Result<Texture<Texture3D, TextureUncompressedInternalFormat>, Error> {
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::FramebufferUnregistered)?;
        let result = registered.copy_to_texture(
            CopyTextureKind::ToTexture3D {
                gl_texture: texture.gl_texture()?,
            },
            x,
            y,
            width,
            height,
            read_buffer,
            level,
            x_offset,
            y_offset,
            z_offset,
        )?;

        match result {
            CopyTexture::ToTexture3D => Ok(texture),
            _ => unreachable!(),
        }
    }

    pub fn copy_to_new_texture_2d_array(
        &self,
        internal_format: TextureUncompressedInternalFormat,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<Texture<Texture2DArray, TextureUncompressedInternalFormat>, Error> {
        self.copy_to_new_texture_2d_array_with_params(
            internal_format,
            x,
            y,
            width,
            height,
            None,
            None,
            None,
            None,
            None,
        )
    }

    pub fn copy_to_new_texture_2d_array_with_params(
        &self,
        internal_format: TextureUncompressedInternalFormat,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        read_buffer: Option<OperableBuffer>,
        level: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
        z_offset: Option<usize>,
    ) -> Result<Texture<Texture2DArray, TextureUncompressedInternalFormat>, Error> {
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::FramebufferUnregistered)?;
        let result = registered.copy_to_texture(
            CopyTextureKind::NewTexture2DArray { internal_format },
            x,
            y,
            width,
            height,
            read_buffer,
            level,
            x_offset,
            y_offset,
            z_offset,
        )?;

        match result {
            CopyTexture::NewTexture2DArray {
                gl_texture,
                levels,
                width,
                height,
                length,
            } => registered.texture_registry.capture_2d_array(
                gl_texture,
                Texture2DArray::new(levels, width, height, length),
                internal_format,
            ),
            _ => unreachable!(),
        }
    }

    pub fn copy_to_texture_2d_array(
        &self,
        texture: Texture<Texture2DArray, TextureUncompressedInternalFormat>,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Result<Texture<Texture2DArray, TextureUncompressedInternalFormat>, Error> {
        self.copy_to_texture_2d_array_with_params(
            texture, x, y, width, height, None, None, None, None, None,
        )
    }

    pub fn copy_to_texture_2d_array_with_params(
        &self,
        texture: Texture<Texture2DArray, TextureUncompressedInternalFormat>,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        read_buffer: Option<OperableBuffer>,
        level: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
        z_offset: Option<usize>,
    ) -> Result<Texture<Texture2DArray, TextureUncompressedInternalFormat>, Error> {
        let mut registered = self.registered.borrow_mut();
        let registered = registered.as_mut().ok_or(Error::FramebufferUnregistered)?;
        let result = registered.copy_to_texture(
            CopyTextureKind::ToTexture2DArray {
                gl_texture: texture.gl_texture()?,
            },
            x,
            y,
            width,
            height,
            read_buffer,
            level,
            x_offset,
            y_offset,
            z_offset,
        )?;

        match result {
            CopyTexture::ToTexture2DArray => Ok(texture),
            _ => unreachable!(),
        }
    }
}

fn operable_buffers_to_array(operable_buffers: &[OperableBuffer]) -> Array {
    let array = Array::new_with_length(operable_buffers.len() as u32);
    for (index, draw_buffer) in operable_buffers.iter().enumerate() {
        array.set(
            index as u32,
            JsValue::from_f64(draw_buffer.to_gl_enum() as f64),
        );
    }

    array
}

enum ReadPixelsKind {
    NewArrayBuffer,
    ToArrayBuffer(ArrayBuffer),
    NewPixelBufferObject(BufferUsage),
    ToPixelBufferObject(WebGlBuffer),
}

enum ReadPixels {
    ArrayBuffer(ArrayBuffer),
    NewPixelBufferObject(WebGlBuffer, usize, BufferUsage),
    ToPixelBufferObject,
}

enum CopyTextureKind {
    NewTexture2D {
        internal_format: TextureUncompressedInternalFormat,
    },
    ToTexture2D {
        gl_texture: WebGlTexture,
    },
    NewTextureCubeMap {
        face: TextureCubeMapFace,
        internal_format: TextureUncompressedInternalFormat,
    },
    ToTextureCubeMap {
        face: TextureCubeMapFace,
        gl_texture: WebGlTexture,
    },
    NewTexture3D {
        internal_format: TextureUncompressedInternalFormat,
    },
    ToTexture3D {
        gl_texture: WebGlTexture,
    },
    NewTexture2DArray {
        internal_format: TextureUncompressedInternalFormat,
    },
    ToTexture2DArray {
        gl_texture: WebGlTexture,
    },
}

enum CopyTexture {
    NewTexture2D {
        gl_texture: WebGlTexture,
        levels: usize,
        width: usize,
        height: usize,
    },
    ToTexture2D,
    NewTextureCubeMap {
        gl_texture: WebGlTexture,
        levels: usize,
        width: usize,
        height: usize,
    },
    ToTextureCubeMap,
    NewTexture3D {
        gl_texture: WebGlTexture,
        levels: usize,
        width: usize,
        height: usize,
        depth: usize,
    },
    ToTexture3D,
    NewTexture2DArray {
        gl_texture: WebGlTexture,
        levels: usize,
        width: usize,
        height: usize,
        length: usize,
    },
    ToTexture2DArray,
}

#[derive(Debug)]
struct FramebufferRegistered {
    gl: WebGl2RenderingContext,
    gl_framebuffer: WebGlFramebuffer,
    gl_textures: HashMap<FramebufferAttachment, (WebGlTexture, ClearPolicy, bool)>,
    gl_renderbuffers: HashMap<FramebufferAttachment, (WebGlRenderbuffer, ClearPolicy, bool)>,
    gl_bounds: HashSet<FramebufferTarget>,
    gl_origin_read_buffer: u32,
    gl_origin_write_buffers: Array,

    reg_id: Uuid,
    reg_framebuffer_bounds: Rc<RefCell<HashMap<FramebufferTarget, (WebGlFramebuffer, u32, Array)>>>,

    buffer_registry: BufferRegistry,
    texture_registry: TextureRegistry,

    framebuffer_size_policy: SizePolicy,
    framebuffer_size: Option<(usize, usize)>,
    renderbuffer_samples: Option<usize>,
    framebuffer_sources: HashMap<FramebufferAttachment, AttachmentSource>,
}

impl FramebufferRegistered {
    fn bind(
        &mut self,
        target: FramebufferTarget,
        read_buffer: Option<OperableBuffer>,
        draw_buffers: Option<Vec<OperableBuffer>>,
    ) -> Result<(), Error> {
        if let Some((gl_framebuffer, _, _)) = self.reg_framebuffer_bounds.borrow().get(&target) {
            if gl_framebuffer == &self.gl_framebuffer {
                return Ok(());
            } else {
                return Err(Error::FramebufferTargetOccupied(target));
            }
        }

        self.gl
            .bind_framebuffer(target.to_gl_enum(), Some(&self.gl_framebuffer));

        self.build(target)?;

        // bind read buffers
        let read_buffer = read_buffer.map(|read_buffer| read_buffer.to_gl_enum());
        let draw_buffers =
            draw_buffers.map(|draw_buffers| operable_buffers_to_array(&draw_buffers));
        if target == FramebufferTarget::ReadFramebuffer {
            if let Some(read_buffer) = read_buffer {
                self.gl.read_buffer(read_buffer);
            }
        }
        // binds draw buffers
        else if target == FramebufferTarget::DrawFramebuffer {
            if let Some(draw_buffers) = &draw_buffers {
                self.gl.draw_buffers(draw_buffers);
            }
        }

        self.gl_bounds.insert_unique_unchecked(target);
        self.reg_framebuffer_bounds
            .borrow_mut()
            .insert_unique_unchecked(
                target,
                (
                    self.gl_framebuffer.clone(),
                    read_buffer.unwrap_or(self.gl_origin_read_buffer),
                    draw_buffers.unwrap_or(self.gl_origin_write_buffers.clone()),
                ),
            );

        Ok(())
    }

    fn temp_bind(
        &mut self,
        target: FramebufferTarget,
        read_buffer: &Option<OperableBuffer>,
        draw_buffers: &Option<Vec<OperableBuffer>>,
    ) -> Result<(), Error> {
        self.gl
            .bind_framebuffer(target.to_gl_enum(), Some(&self.gl_framebuffer));

        self.build(target)?;

        if target == FramebufferTarget::DrawFramebuffer {
            if let Some(draw_buffers) = draw_buffers {
                self.gl
                    .draw_buffers(&operable_buffers_to_array(draw_buffers));
            }
        } else if target == FramebufferTarget::ReadFramebuffer {
            if let Some(read_buffer) = read_buffer {
                self.gl.read_buffer(read_buffer.to_gl_enum());
            }
        }

        Ok(())
    }

    fn temp_unbind(
        &mut self,
        target: FramebufferTarget,
        read_buffer: &Option<OperableBuffer>,
        draw_buffers: &Option<Vec<OperableBuffer>>,
    ) {
        if target == FramebufferTarget::DrawFramebuffer {
            if draw_buffers.is_some() {
                self.gl.draw_buffers(&self.gl_origin_write_buffers);
            }

            if let Some((gl_framebuffer, _, draw_buffers)) = self
                .reg_framebuffer_bounds
                .borrow()
                .get(&FramebufferTarget::DrawFramebuffer)
            {
                self.gl.bind_framebuffer(
                    FramebufferTarget::DrawFramebuffer.to_gl_enum(),
                    Some(gl_framebuffer),
                );
                self.gl.draw_buffers(draw_buffers);
            } else {
                self.gl
                    .bind_framebuffer(FramebufferTarget::DrawFramebuffer.to_gl_enum(), None);
            }
        } else if target == FramebufferTarget::ReadFramebuffer {
            if read_buffer.is_some() {
                self.gl.read_buffer(self.gl_origin_read_buffer);
            }

            if let Some((gl_framebuffer, read_buffer, _)) = self
                .reg_framebuffer_bounds
                .borrow()
                .get(&FramebufferTarget::ReadFramebuffer)
            {
                self.gl.bind_framebuffer(
                    FramebufferTarget::ReadFramebuffer.to_gl_enum(),
                    Some(gl_framebuffer),
                );
                self.gl.read_buffer(*read_buffer);
            } else {
                self.gl
                    .bind_framebuffer(FramebufferTarget::ReadFramebuffer.to_gl_enum(), None);
            }
        }
    }

    fn build(&mut self, target: FramebufferTarget) -> Result<(), Error> {
        let (width, height) = match self.framebuffer_size {
            Some((width, height)) => {
                let (twidth, theight) = self.framebuffer_size_policy.size(&self.gl);
                if twidth != width || theight != height {
                    (twidth, theight)
                } else {
                    return Ok(());
                }
            }
            None => {
                return Ok(());
            }
        };

        for (attachment, source) in self.framebuffer_sources.iter() {
            enum Attach {
                Texture {
                    texture: WebGlTexture,
                    level: usize,
                    layer: Option<usize>,
                    cube_map_face: Option<TextureCubeMapFace>,
                    clear_policy: ClearPolicy,
                    owned: bool,
                },
                Renderbuffer(WebGlRenderbuffer, ClearPolicy, bool),
            }

            let attach = match source {
                AttachmentSource::CreateTexture2D {
                    internal_format,
                    clear_policy,
                } => {
                    let gl_texture = self
                        .gl
                        .create_texture()
                        .ok_or(Error::CreateTextureFailure)?;
                    self.gl
                        .bind_texture(TextureTarget::Texture2D.to_gl_enum(), Some(&gl_texture));
                    self.gl.tex_storage_2d(
                        TextureTarget::Texture2D.to_gl_enum(),
                        1,
                        internal_format.to_gl_enum(),
                        width as i32,
                        height as i32,
                    );

                    Attach::Texture {
                        texture: gl_texture.clone(),
                        level: 0,
                        layer: None,
                        cube_map_face: None,
                        clear_policy: *clear_policy,
                        owned: true,
                    }
                }
                AttachmentSource::FromTexture2D {
                    texture,
                    level,
                    clear_policy,
                } => Attach::Texture {
                    texture: texture.clone(),
                    level: *level,
                    layer: None,
                    cube_map_face: None,
                    clear_policy: *clear_policy,
                    owned: false,
                },
                AttachmentSource::FromTextureCubeMap {
                    texture,
                    level,
                    face,
                    clear_policy,
                } => Attach::Texture {
                    texture: texture.clone(),
                    level: *level,
                    layer: None,
                    cube_map_face: Some(*face),
                    clear_policy: *clear_policy,
                    owned: false,
                },
                AttachmentSource::FromTextureLayer {
                    texture,
                    level,
                    layer,
                    clear_policy,
                } => Attach::Texture {
                    texture: texture.clone(),
                    level: *level,
                    layer: Some(*layer),
                    cube_map_face: None,
                    clear_policy: *clear_policy,
                    owned: false,
                },
                AttachmentSource::CreateRenderbuffer {
                    internal_format,
                    clear_policy,
                } => {
                    let gl_renderbuffer = self
                        .gl
                        .create_renderbuffer()
                        .ok_or(Error::CreateRenderbufferFailure)?;
                    self.gl.bind_renderbuffer(
                        RenderbufferTarget::Renderbuffer.to_gl_enum(),
                        Some(&gl_renderbuffer),
                    );
                    match self.renderbuffer_samples {
                        Some(samples) => {
                            self.gl.renderbuffer_storage_multisample(
                                RenderbufferTarget::Renderbuffer.to_gl_enum(),
                                samples as i32,
                                internal_format.to_gl_enum(),
                                width as i32,
                                height as i32,
                            );
                        }
                        None => {
                            self.gl.renderbuffer_storage(
                                RenderbufferTarget::Renderbuffer.to_gl_enum(),
                                internal_format.to_gl_enum(),
                                width as i32,
                                height as i32,
                            );
                        }
                    }

                    Attach::Renderbuffer(gl_renderbuffer, *clear_policy, true)
                }
                AttachmentSource::FromRenderbuffer {
                    renderbuffer,
                    clear_policy,
                } => Attach::Renderbuffer(renderbuffer.clone(), *clear_policy, false),
            };

            match attach {
                Attach::Texture {
                    texture,
                    level,
                    layer,
                    cube_map_face,
                    clear_policy,
                    owned,
                } => {
                    match layer {
                        Some(layer) => {
                            self.gl.framebuffer_texture_layer(
                                target.to_gl_enum(),
                                attachment.to_gl_enum(),
                                Some(&texture),
                                level as i32,
                                layer as i32,
                            );
                        }
                        None => {
                            self.gl.bind_texture(
                                TextureTarget::Texture2D.to_gl_enum(),
                                Some(&texture),
                            );
                            let textarget = cube_map_face
                                .map(|face| face.to_gl_enum())
                                .unwrap_or(TextureTarget::Texture2D.to_gl_enum());
                            self.gl.framebuffer_texture_2d(
                                target.to_gl_enum(),
                                attachment.to_gl_enum(),
                                textarget,
                                Some(&texture),
                                level as i32,
                            );
                            self.gl.bind_texture(
                                TextureTarget::Texture2D.to_gl_enum(),
                                self.texture_registry.bounds.borrow().get(&(
                                    self.texture_registry.active_unit.borrow().clone(),
                                    TextureTarget::Texture2D,
                                )),
                            );
                        }
                    }

                    if let Some((removed, _, owned)) = self
                        .gl_textures
                        .insert(*attachment, (texture, clear_policy, owned))
                    {
                        if owned {
                            self.gl.delete_texture(Some(&removed))
                        }
                    }
                }
                Attach::Renderbuffer(gl_renderbuffer, clear_policy, owned) => {
                    self.gl.bind_renderbuffer(
                        RenderbufferTarget::Renderbuffer.to_gl_enum(),
                        Some(&gl_renderbuffer),
                    );
                    self.gl.framebuffer_renderbuffer(
                        target.to_gl_enum(),
                        attachment.to_gl_enum(),
                        RenderbufferTarget::Renderbuffer.to_gl_enum(),
                        Some(&gl_renderbuffer),
                    );
                    self.gl
                        .bind_renderbuffer(RenderbufferTarget::Renderbuffer.to_gl_enum(), None);

                    if let Some((removed, _, owned)) = self
                        .gl_renderbuffers
                        .insert(*attachment, (gl_renderbuffer, clear_policy, owned))
                    {
                        if owned {
                            self.gl.delete_renderbuffer(Some(&removed));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn unbind(&mut self, target: FramebufferTarget) {
        if self.gl_bounds.remove(&target) {
            // restores to default read and write buffers
            if target == FramebufferTarget::DrawFramebuffer {
                self.gl.draw_buffers(&self.gl_origin_write_buffers);
            } else if target == FramebufferTarget::ReadFramebuffer {
                self.gl.read_buffer(self.gl_origin_read_buffer);
            }

            self.gl.bind_framebuffer(target.to_gl_enum(), None);
            self.reg_framebuffer_bounds.borrow_mut().remove(&target);
        }
    }

    fn unbind_all(&mut self) {
        for target in self.gl_bounds.drain() {
            // restores to default read and write buffers
            if target == FramebufferTarget::DrawFramebuffer {
                self.gl.draw_buffers(&self.gl_origin_write_buffers);
            } else if target == FramebufferTarget::ReadFramebuffer {
                self.gl.read_buffer(self.gl_origin_read_buffer);
            }

            self.gl.bind_framebuffer(target.to_gl_enum(), None);
            self.reg_framebuffer_bounds.borrow_mut().remove(&target);
        }
    }

    fn clear(&mut self, attachment: FramebufferAttachment) -> Result<(), Error> {
        if let Some(source) = self.framebuffer_sources.get(&attachment) {
            let clear_buffer = source.clear_policy().clone();
            self.temp_bind(FramebufferTarget::DrawFramebuffer, &None, &None)?;
            clear_buffer.clear(&self.gl, attachment);
            self.temp_unbind(FramebufferTarget::DrawFramebuffer, &None, &None);
        }
        Ok(())
    }

    fn clear_all(&mut self) -> Result<(), Error> {
        self.temp_bind(FramebufferTarget::DrawFramebuffer, &None, &None)?;
        for (attachment, source) in self.framebuffer_sources.iter() {
            source.clear_policy().clear(&self.gl, *attachment);
        }
        self.temp_unbind(FramebufferTarget::DrawFramebuffer, &None, &None);
        Ok(())
    }

    fn blit_to(
        &mut self,
        read_buffer: Option<OperableBuffer>,
        src_x0: Option<usize>,
        src_y0: Option<usize>,
        src_x1: Option<usize>,
        src_y1: Option<usize>,
        to: &mut Self,
        draw_buffers: Option<Vec<OperableBuffer>>,
        dst_x0: Option<usize>,
        dst_y0: Option<usize>,
        dst_x1: Option<usize>,
        dst_y1: Option<usize>,
        mask: BlitMask,
        filter: BlitFilter,
    ) -> Result<(), Error> {
        self.temp_bind(FramebufferTarget::ReadFramebuffer, &read_buffer, &None)?;
        to.temp_bind(FramebufferTarget::DrawFramebuffer, &None, &draw_buffers)?;

        let src_x0 = src_x0.unwrap_or(0);
        let src_y0 = src_y0.unwrap_or(0);
        let src_x1 = src_x1.unwrap_or(self.framebuffer_size.as_ref().unwrap().0);
        let src_y1 = src_y1.unwrap_or(self.framebuffer_size.as_ref().unwrap().1);
        let dst_x0 = dst_x0.unwrap_or(0);
        let dst_y0 = dst_y0.unwrap_or(0);
        let dst_x1 = dst_x1.unwrap_or(to.framebuffer_size.as_ref().unwrap().0);
        let dst_y1 = dst_y1.unwrap_or(to.framebuffer_size.as_ref().unwrap().1);
        self.gl.blit_framebuffer(
            src_x0 as i32,
            src_y0 as i32,
            src_x1 as i32,
            src_y1 as i32,
            dst_x0 as i32,
            dst_y0 as i32,
            dst_x1 as i32,
            dst_y1 as i32,
            mask.to_gl_enum(),
            filter.to_gl_enum(),
        );

        self.temp_unbind(FramebufferTarget::ReadFramebuffer, &read_buffer, &None);
        to.temp_unbind(FramebufferTarget::DrawFramebuffer, &None, &draw_buffers);

        Ok(())
    }

    /// Reads pixels to PixelBufferObject or ArrayBuffer by [`ReadBackKind`].
    fn read_pixels(
        &mut self,
        read_pixels_kind: ReadPixelsKind,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        pixel_pack_storages: Option<Vec<PixelPackStorage>>,
        read_buffer: Option<OperableBuffer>,
        x: Option<usize>,
        y: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        dst_offset: Option<usize>,
    ) -> Result<ReadPixels, Error> {
        self.temp_bind(FramebufferTarget::ReadFramebuffer, &read_buffer, &None)?;

        let default_storages = if let Some(pixel_pack_storages) = pixel_pack_storages {
            let defaults = pixel_pack_storages
                .into_iter()
                .map(|storage| storage.pixel_store(&self.gl));
            Some(defaults)
        } else {
            None
        };

        let x = x.unwrap_or(0);
        let y = y.unwrap_or(0);
        let width = width.unwrap_or(self.framebuffer_size.as_ref().unwrap().0);
        let height = height.unwrap_or(self.framebuffer_size.as_ref().unwrap().1);
        let size = width
            * height
            * pixel_format.channels_per_pixel()
            * pixel_data_type.bytes_per_channel();

        let readback = match read_pixels_kind {
            ReadPixelsKind::NewArrayBuffer | ReadPixelsKind::ToArrayBuffer(_) => {
                // reads into array buffer
                let array_buffer = match read_pixels_kind {
                    ReadPixelsKind::NewArrayBuffer => ArrayBuffer::new(size as u32),
                    ReadPixelsKind::ToArrayBuffer(array_buffer) => array_buffer,
                    _ => unreachable!(),
                };
                match dst_offset {
                    Some(dst_offset) => {
                        self.gl
                            .read_pixels_with_array_buffer_view_and_dst_offset(
                                x as i32,
                                y as i32,
                                width as i32,
                                height as i32,
                                pixel_format.to_gl_enum(),
                                pixel_data_type.to_gl_enum(),
                                &Uint8Array::new(&array_buffer),
                                dst_offset as u32,
                            )
                            .or(Err(Error::ReadPixelsFailure))?;
                    }
                    None => {
                        self.gl
                            .read_pixels_with_opt_array_buffer_view(
                                x as i32,
                                y as i32,
                                width as i32,
                                height as i32,
                                pixel_format.to_gl_enum(),
                                pixel_data_type.to_gl_enum(),
                                Some(&Uint8Array::new(&array_buffer)),
                            )
                            .or(Err(Error::ReadPixelsFailure))?;
                    }
                };

                ReadPixels::ArrayBuffer(array_buffer)
            }
            ReadPixelsKind::NewPixelBufferObject(_) | ReadPixelsKind::ToPixelBufferObject(_) => {
                // reads into pbo
                let (gl_buffer, usage) = match &read_pixels_kind {
                    ReadPixelsKind::NewPixelBufferObject(usage) => {
                        let gl_buffer =
                            self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
                        self.gl.bind_buffer(
                            BufferTarget::PixelPackBuffer.to_gl_enum(),
                            Some(&gl_buffer),
                        );
                        self.gl.buffer_data_with_i32(
                            BufferTarget::PixelPackBuffer.to_gl_enum(),
                            size as i32,
                            usage.to_gl_enum(),
                        );

                        (gl_buffer, *usage)
                    }
                    ReadPixelsKind::ToPixelBufferObject(gl_buffer) => {
                        self.gl.bind_buffer(
                            BufferTarget::PixelPackBuffer.to_gl_enum(),
                            Some(&gl_buffer),
                        );

                        (gl_buffer.clone(), BufferUsage::StaticDraw)
                    }
                    _ => unreachable!(),
                };

                let dst_offset = dst_offset.unwrap_or(0);
                self.gl
                    .read_pixels_with_i32(
                        x as i32,
                        y as i32,
                        width as i32,
                        height as i32,
                        pixel_format.to_gl_enum(),
                        pixel_data_type.to_gl_enum(),
                        dst_offset as i32,
                    )
                    .or(Err(Error::ReadPixelsFailure))?;

                self.gl.bind_buffer(
                    BufferTarget::PixelPackBuffer.to_gl_enum(),
                    self.buffer_registry
                        .bounds
                        .borrow()
                        .get(&BufferTarget::PixelPackBuffer),
                );

                match read_pixels_kind {
                    ReadPixelsKind::NewPixelBufferObject(_) => {
                        ReadPixels::NewPixelBufferObject(gl_buffer, size, usage)
                    }
                    ReadPixelsKind::ToPixelBufferObject(_) => ReadPixels::ToPixelBufferObject,
                    _ => unreachable!(),
                }
            }
        };

        if let Some(defaults) = default_storages {
            defaults.into_iter().for_each(|storage| {
                storage.pixel_store(&self.gl);
            });
        }

        self.temp_unbind(FramebufferTarget::ReadFramebuffer, &read_buffer, &None);

        Ok(readback)
    }

    /// Asynchrony reads pixels, refers to [`read_pixels`](Self::read_pixels) for more details.
    async fn read_pixels_async(
        &mut self,
        read_pixels_kind: ReadPixelsKind,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        pixel_pack_storages: Option<Vec<PixelPackStorage>>,
        read_buffer: Option<OperableBuffer>,
        x: Option<usize>,
        y: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        dst_offset: Option<usize>,
        max_retries: Option<usize>,
    ) -> Result<ReadPixels, Error> {
        ClientWaitAsync::new(self.gl.clone(), 0, 5, max_retries)
            .wait()
            .await?;

        self.read_pixels(
            read_pixels_kind,
            pixel_format,
            pixel_data_type,
            pixel_pack_storages,
            read_buffer,
            x,
            y,
            width,
            height,
            dst_offset,
        )
    }

    fn copy_to_texture(
        &mut self,
        copy_texture_kind: CopyTextureKind,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        read_buffer: Option<OperableBuffer>,
        level: Option<usize>,
        x_offset: Option<usize>,
        y_offset: Option<usize>,
        z_offset: Option<usize>,
    ) -> Result<CopyTexture, Error> {
        self.temp_bind(FramebufferTarget::ReadFramebuffer, &read_buffer, &None)?;

        let (texture_width, texture_height) = self.framebuffer_size.as_ref().unwrap().clone();
        let (target, is_3d) = match &copy_texture_kind {
            CopyTextureKind::NewTexture2D { .. } | CopyTextureKind::ToTexture2D { .. } => {
                (TextureTarget::Texture2D, false)
            }
            CopyTextureKind::NewTextureCubeMap { .. }
            | CopyTextureKind::ToTextureCubeMap { .. } => (TextureTarget::TextureCubeMap, false),
            CopyTextureKind::NewTexture3D { .. } | CopyTextureKind::ToTexture3D { .. } => {
                (TextureTarget::Texture3D, true)
            }
            CopyTextureKind::NewTexture2DArray { .. }
            | CopyTextureKind::ToTexture2DArray { .. } => (TextureTarget::Texture2DArray, true),
        };

        let gl_texture = match &copy_texture_kind {
            CopyTextureKind::NewTexture2D {
                internal_format, ..
            }
            | CopyTextureKind::NewTextureCubeMap {
                internal_format, ..
            }
            | CopyTextureKind::NewTexture3D {
                internal_format, ..
            }
            | CopyTextureKind::NewTexture2DArray {
                internal_format, ..
            } => {
                let gl_texture = self
                    .gl
                    .create_texture()
                    .ok_or(Error::CreateTextureFailure)?;
                self.gl.bind_texture(target.to_gl_enum(), Some(&gl_texture));

                if is_3d {
                    self.gl.tex_storage_3d(
                        target.to_gl_enum(),
                        1,
                        internal_format.to_gl_enum(),
                        texture_width as i32,
                        texture_height as i32,
                        1,
                    );
                } else {
                    self.gl.tex_storage_2d(
                        target.to_gl_enum(),
                        1,
                        internal_format.to_gl_enum(),
                        texture_width as i32,
                        texture_height as i32,
                    );
                }

                gl_texture
            }
            CopyTextureKind::ToTexture2D { gl_texture }
            | CopyTextureKind::ToTextureCubeMap { gl_texture, .. }
            | CopyTextureKind::ToTexture3D { gl_texture }
            | CopyTextureKind::ToTexture2DArray { gl_texture, .. } => {
                self.gl.bind_texture(target.to_gl_enum(), Some(&gl_texture));

                gl_texture.clone()
            }
        };

        let level = level.unwrap_or(0);
        let x_offset = x_offset.unwrap_or(0);
        let y_offset = y_offset.unwrap_or(0);
        let z_offset = z_offset.unwrap_or(0);
        match &copy_texture_kind {
            CopyTextureKind::NewTexture2D { .. }
            | CopyTextureKind::ToTexture2D { .. }
            | CopyTextureKind::NewTexture3D { .. }
            | CopyTextureKind::ToTexture3D { .. }
            | CopyTextureKind::NewTexture2DArray { .. }
            | CopyTextureKind::ToTexture2DArray { .. } => {
                if is_3d {
                    self.gl.copy_tex_sub_image_3d(
                        target.to_gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        z_offset as i32,
                        x as i32,
                        y as i32,
                        width as i32,
                        height as i32,
                    );
                } else {
                    self.gl.copy_tex_sub_image_2d(
                        target.to_gl_enum(),
                        level as i32,
                        x_offset as i32,
                        y_offset as i32,
                        x as i32,
                        y as i32,
                        width as i32,
                        height as i32,
                    );
                }
            }
            CopyTextureKind::NewTextureCubeMap { face, .. }
            | CopyTextureKind::ToTextureCubeMap { face, .. } => {
                self.gl.copy_tex_sub_image_2d(
                    face.to_gl_enum(),
                    level as i32,
                    x_offset as i32,
                    y_offset as i32,
                    x as i32,
                    y as i32,
                    width as i32,
                    height as i32,
                );
            }
        }

        self.gl.bind_texture(
            target.to_gl_enum(),
            self.texture_registry
                .bounds
                .borrow()
                .get(&(self.texture_registry.active_unit.borrow().clone(), target)),
        );

        self.temp_unbind(FramebufferTarget::ReadFramebuffer, &read_buffer, &None);

        let result = match copy_texture_kind {
            CopyTextureKind::NewTexture2D { .. } => CopyTexture::NewTexture2D {
                gl_texture,
                levels: 1,
                width: texture_width,
                height: texture_height,
            },
            CopyTextureKind::ToTexture2D { .. } => CopyTexture::ToTexture2D,
            CopyTextureKind::NewTextureCubeMap { .. } => CopyTexture::NewTextureCubeMap {
                gl_texture,
                levels: 1,
                width: texture_width,
                height: texture_height,
            },
            CopyTextureKind::ToTextureCubeMap { .. } => CopyTexture::ToTextureCubeMap,
            CopyTextureKind::NewTexture3D { .. } => CopyTexture::NewTexture3D {
                gl_texture,
                levels: 1,
                width: texture_width,
                height: texture_height,
                depth: 1,
            },
            CopyTextureKind::ToTexture3D { .. } => CopyTexture::ToTexture3D,
            CopyTextureKind::NewTexture2DArray { .. } => CopyTexture::NewTexture2DArray {
                gl_texture,
                levels: 1,
                width: texture_width,
                height: texture_height,
                length: 1,
            },
            CopyTextureKind::ToTexture2DArray { .. } => CopyTexture::ToTexture2DArray,
        };
        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct FramebufferRegistry {
    id: Uuid,
    gl: WebGl2RenderingContext,
    framebuffer_bounds: Rc<RefCell<HashMap<FramebufferTarget, (WebGlFramebuffer, u32, Array)>>>,

    buffer_registry: BufferRegistry,
    texture_registry: TextureRegistry,
}

impl FramebufferRegistry {
    pub fn new(
        gl: WebGl2RenderingContext,
        buffer_registry: BufferRegistry,
        texture_registry: TextureRegistry,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            gl,
            framebuffer_bounds: Rc::new(RefCell::new(HashMap::new())),

            buffer_registry,
            texture_registry,
        }
    }

    pub fn register(&self, framebuffer: &Framebuffer) -> Result<(), Error> {
        if let Some(registered) = &*framebuffer.registered.borrow() {
            if &registered.reg_id != &self.id {
                return Err(Error::RegisterFramebufferToMultipleRepositoryUnsupported);
            } else {
                return Ok(());
            }
        }

        let gl_framebuffer = self
            .gl
            .create_framebuffer()
            .ok_or(Error::CreateFramebufferFailure)?;

        // finds origin read buffer and draw buffers
        // read buffer refers to the color attachment with the smallest index
        let draw_buffers = Array::new();
        for (attachment, _) in framebuffer.sources.iter() {
            let Some(v) = attachment
                .to_operable_buffer()
                .map(|buffer| buffer.to_gl_enum())
            else {
                continue;
            };
            draw_buffers.push(&JsValue::from_f64(v as f64));
        }
        let read_buffer = draw_buffers
            .get(0)
            .as_f64()
            .map(|v| v as u32)
            .unwrap_or(WebGl2RenderingContext::NONE);

        let registered = FramebufferRegistered {
            gl: self.gl.clone(),
            gl_framebuffer,
            gl_textures: HashMap::new(),
            gl_renderbuffers: HashMap::new(),
            gl_bounds: HashSet::new(),
            gl_origin_read_buffer: read_buffer,
            gl_origin_write_buffers: draw_buffers,

            reg_id: self.id,
            reg_framebuffer_bounds: Rc::clone(&self.framebuffer_bounds),

            buffer_registry: self.buffer_registry.clone(),
            texture_registry: self.texture_registry.clone(),

            framebuffer_size_policy: framebuffer.size_policy,
            framebuffer_size: None,
            renderbuffer_samples: framebuffer.renderbuffer_samples,
            framebuffer_sources: framebuffer.sources.clone(),
        };

        *framebuffer.registered.borrow_mut() = Some(registered);

        Ok(())
    }
}
