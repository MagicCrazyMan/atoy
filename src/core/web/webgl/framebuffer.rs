use std::{
    cell::RefCell, collections::VecDeque, iter::FromIterator, rc::{Rc, Weak}
};

use hashbrown::{hash_map::Entry, HashMap, HashSet};
use js_sys::{ArrayBuffer, Uint8Array};
use log::warn;
use uuid::Uuid;
use wasm_bindgen::JsValue;
use web_sys::{
    js_sys::{Array, Object},
    WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture,
};

use crate::core::web::webgl::{renderbuffer::RenderbufferTarget, texture::TextureRegistry};

use super::{
    blit::{BlitFilter, BlitMask},
    buffer::{Buffer, BufferRegistered, BufferRegistry, BufferTarget, BufferUsage},
    client_wait::ClientWaitAsync,
    conversion::ToGlEnum,
    error::Error,
    pixel::{PixelDataType, PixelFormat, PixelPackStorage},
    renderbuffer::RenderbufferInternalFormat,
    texture::{TextureTarget, TextureUncompressedInternalFormat, TextureUnit},
};

/// Available framebuffer targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramebufferTarget {
    ReadFramebuffer,
    DrawFramebuffer,
}

/// Available framebuffer attachment targets mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramebufferAttachment {
    ColorAttachment0,
    ColorAttachment1,
    ColorAttachment2,
    ColorAttachment3,
    ColorAttachment4,
    ColorAttachment5,
    ColorAttachment6,
    ColorAttachment7,
    ColorAttachment8,
    ColorAttachment9,
    ColorAttachment10,
    ColorAttachment11,
    ColorAttachment12,
    ColorAttachment13,
    ColorAttachment14,
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
}

/// Available drawable or readable buffer attachment mapped from [`WebGl2RenderingContext`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperableBuffer {
    None,
    /// [`WebGl2RenderingContext::BACK`] only works for Canvas Draw Buffer.
    /// Do not bind this attachment to FBO.
    Back,
    ColorAttachment0,
    ColorAttachment1,
    ColorAttachment2,
    ColorAttachment3,
    ColorAttachment4,
    ColorAttachment5,
    ColorAttachment6,
    ColorAttachment7,
    ColorAttachment8,
    ColorAttachment9,
    ColorAttachment10,
    ColorAttachment11,
    ColorAttachment12,
    ColorAttachment13,
    ColorAttachment14,
    ColorAttachment15,
}

impl OperableBuffer {
    fn to_index(&self) -> Option<usize> {
        match self {
            OperableBuffer::ColorAttachment0 => Some(0),
            OperableBuffer::ColorAttachment1 => Some(1),
            OperableBuffer::ColorAttachment2 => Some(2),
            OperableBuffer::ColorAttachment3 => Some(3),
            OperableBuffer::ColorAttachment4 => Some(4),
            OperableBuffer::ColorAttachment5 => Some(5),
            OperableBuffer::ColorAttachment6 => Some(6),
            OperableBuffer::ColorAttachment7 => Some(7),
            OperableBuffer::ColorAttachment8 => Some(8),
            OperableBuffer::ColorAttachment9 => Some(9),
            OperableBuffer::ColorAttachment10 => Some(10),
            OperableBuffer::ColorAttachment11 => Some(11),
            OperableBuffer::ColorAttachment12 => Some(12),
            OperableBuffer::ColorAttachment13 => Some(13),
            OperableBuffer::ColorAttachment14 => Some(14),
            OperableBuffer::ColorAttachment15 => Some(15),
            OperableBuffer::None => None,
            OperableBuffer::Back => None,
        }
    }
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
    fn clear(&self, gl: &WebGl2RenderingContext, draw_buffer_index: i32) {
        match self {
            ClearPolicy::ColorFloat(values) => gl.clear_bufferfv_with_f32_array(
                WebGl2RenderingContext::COLOR,
                draw_buffer_index,
                values,
            ),
            ClearPolicy::ColorInteger(values) => gl.clear_bufferiv_with_i32_array(
                WebGl2RenderingContext::COLOR,
                draw_buffer_index,
                values,
            ),
            ClearPolicy::ColorUnsignedInteger(values) => gl.clear_bufferuiv_with_u32_array(
                WebGl2RenderingContext::COLOR,
                draw_buffer_index,
                values,
            ),
            ClearPolicy::Depth(depth) => gl.clear_bufferfv_with_f32_array(
                WebGl2RenderingContext::DEPTH,
                draw_buffer_index,
                &[*depth],
            ),
            ClearPolicy::Stencil(stencil) => gl.clear_bufferiv_with_i32_array(
                WebGl2RenderingContext::STENCIL,
                draw_buffer_index,
                &[*stencil],
            ),
            ClearPolicy::DepthStencil(depth, stencil) => gl.clear_bufferfi(
                WebGl2RenderingContext::DEPTH_STENCIL,
                draw_buffer_index,
                *depth,
                *stencil,
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttachmentSource {
    CreateTexture {
        internal_format: TextureUncompressedInternalFormat,
        clear_policy: ClearPolicy,
    },
    FromTexture {
        texture: WebGlTexture,
        level: usize,
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

        Self::CreateTexture {
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
        Self::CreateTexture {
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
        Self::FromTexture {
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

    pub fn unbind(&self, target: FramebufferTarget) -> Result<(), Error> {
        self.registered
            .borrow_mut()
            .as_mut()
            .ok_or(Error::FramebufferUnregistered)?
            .unbind(target);

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
                ReadBackKind::ArrayBuffer,
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
            ReadBack::ArrayBuffer(array_buffer) => Ok(array_buffer),
            ReadBack::PixelBufferObject(_, _, _) => unreachable!(),
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
                ReadBackKind::ArrayBuffer,
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
            ReadBack::ArrayBuffer(array_buffer) => Ok(array_buffer),
            ReadBack::PixelBufferObject(_, _, _) => unreachable!(),
        }
    }

    pub fn read_pixels_to_pbo(
        &self,
        pixel_buffer_object_usage: BufferUsage,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
    ) -> Result<Buffer, Error> {
        self.read_pixels_to_pbo_with_params(
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

    pub fn read_pixels_to_pbo_with_params(
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
            ReadBackKind::PixelBufferObject(pixel_buffer_object_usage),
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
            ReadBack::ArrayBuffer(_) => unreachable!(),
            ReadBack::PixelBufferObject(gl_buffer, size, usage) => {
                // wraps native WebGlBuffer to Buffer
                if let Some(buffer_used_memory) = registered.reg_buffer_used_memory.upgrade() {
                    *buffer_used_memory.borrow_mut() += size;
                }

                let queue = Rc::new(RefCell::new(VecDeque::new()));
                let queue_size = Rc::new(RefCell::new(0));

                let registered = BufferRegistered {
                    gl: registered.gl.clone(),
                    gl_buffer,
                    gl_bounds: HashSet::new(),

                    reg_id: registered.reg_buffer_id,
                    reg_bounds: Rc::clone(&registered.reg_buffer_bounds),
                    reg_used_memory: Weak::clone(&registered.reg_buffer_used_memory),

                    buffer_size: size,
                    buffer_usage: todo!(),
                    buffer_queue: Rc::downgrade(&queue),
                    buffer_async_upload: Rc::new(RefCell::new(None)),

                    restore_when_drop: false,
                };

                Ok(Buffer {
                    id: Uuid::new_v4(),
                    usage,
                    queue,
                    registered: Rc::new(RefCell::new(Some(registered))),
                })
            }
        }
    }
}

trait ToArray {
    fn to_array(&self) -> Array;
}

impl ToArray for Vec<OperableBuffer> {
    fn to_array(&self) -> Array {
        let array = Array::new_with_length(self.len() as u32);
        for (index, draw_buffer) in self.iter().enumerate() {
            array.set(
                index as u32,
                JsValue::from_f64(draw_buffer.gl_enum() as f64),
            );
        }

        array
    }
}

enum ReadBackKind {
    ArrayBuffer,
    PixelBufferObject(BufferUsage),
}

enum ReadBack {
    ArrayBuffer(ArrayBuffer),
    PixelBufferObject(WebGlBuffer, usize, BufferUsage),
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

    reg_buffer_id: Uuid,
    reg_buffer_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
    reg_buffer_used_memory: Weak<RefCell<usize>>,

    reg_texture_active_unit: Rc<RefCell<TextureUnit>>,
    reg_texture_bounds: Rc<RefCell<HashMap<(TextureUnit, TextureTarget), WebGlTexture>>>,

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
            .bind_framebuffer(target.gl_enum(), Some(&self.gl_framebuffer));

        self.build(target)?;

        // bind read buffers
        let read_buffer = read_buffer.map(|read_buffer| read_buffer.gl_enum());
        let draw_buffers = draw_buffers.map(|draw_buffers| draw_buffers.to_array());
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
                Texture(WebGlTexture, usize, ClearPolicy, bool),
                Renderbuffer(WebGlRenderbuffer, ClearPolicy, bool),
            }

            let attach = match source {
                AttachmentSource::CreateTexture {
                    internal_format,
                    clear_policy,
                } => {
                    let gl_texture = self
                        .gl
                        .create_texture()
                        .ok_or(Error::CreateTextureFailure)?;
                    self.gl
                        .bind_texture(TextureTarget::Texture2D.gl_enum(), Some(&gl_texture));
                    self.gl.tex_storage_2d(
                        TextureTarget::Texture2D.gl_enum(),
                        1,
                        internal_format.gl_enum(),
                        width as i32,
                        height as i32,
                    );

                    Attach::Texture(gl_texture, 0, *clear_policy, true)
                }
                AttachmentSource::FromTexture {
                    texture,
                    level,
                    clear_policy,
                } => Attach::Texture(texture.clone(), *level, *clear_policy, false),
                AttachmentSource::CreateRenderbuffer {
                    internal_format,
                    clear_policy,
                } => {
                    let gl_renderbuffer = self
                        .gl
                        .create_renderbuffer()
                        .ok_or(Error::CreateRenderbufferFailure)?;
                    self.gl.bind_renderbuffer(
                        RenderbufferTarget::Renderbuffer.gl_enum(),
                        Some(&gl_renderbuffer),
                    );
                    match self.renderbuffer_samples {
                        Some(samples) => {
                            self.gl.renderbuffer_storage_multisample(
                                RenderbufferTarget::Renderbuffer.gl_enum(),
                                samples as i32,
                                internal_format.gl_enum(),
                                width as i32,
                                height as i32,
                            );
                        }
                        None => {
                            self.gl.renderbuffer_storage(
                                RenderbufferTarget::Renderbuffer.gl_enum(),
                                internal_format.gl_enum(),
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
                Attach::Texture(gl_texture, level, clear_policy, owned) => {
                    self.gl
                        .bind_texture(TextureTarget::Texture2D.gl_enum(), Some(&gl_texture));
                    self.gl.framebuffer_texture_2d(
                        target.gl_enum(),
                        attachment.gl_enum(),
                        TextureTarget::Texture2D.gl_enum(),
                        Some(&gl_texture),
                        level as i32,
                    );
                    self.gl.bind_texture(
                        TextureTarget::Texture2D.gl_enum(),
                        self.reg_texture_bounds.borrow().get(&(
                            self.reg_texture_active_unit.borrow().clone(),
                            TextureTarget::Texture2D,
                        )),
                    );

                    if let Some((removed, _, owned)) = self
                        .gl_textures
                        .insert(*attachment, (gl_texture, clear_policy, owned))
                    {
                        if owned {
                            self.gl.delete_texture(Some(&removed))
                        }
                    }
                }
                Attach::Renderbuffer(gl_renderbuffer, clear_policy, owned) => {
                    self.gl.bind_renderbuffer(
                        RenderbufferTarget::Renderbuffer.gl_enum(),
                        Some(&gl_renderbuffer),
                    );
                    self.gl.framebuffer_renderbuffer(
                        target.gl_enum(),
                        attachment.gl_enum(),
                        RenderbufferTarget::Renderbuffer.gl_enum(),
                        Some(&gl_renderbuffer),
                    );
                    self.gl
                        .bind_renderbuffer(RenderbufferTarget::Renderbuffer.gl_enum(), None);

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

            self.gl.bind_framebuffer(target.gl_enum(), None);
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

            self.gl.bind_framebuffer(target.gl_enum(), None);
            self.reg_framebuffer_bounds.borrow_mut().remove(&target);
        }
    }

    fn temp_bind(
        &mut self,
        target: FramebufferTarget,
        read_buffer: &Option<OperableBuffer>,
        draw_buffers: &Option<Vec<OperableBuffer>>,
    ) -> Result<(), Error> {
        self.gl
            .bind_framebuffer(target.gl_enum(), Some(&self.gl_framebuffer));

        self.build(target)?;

        if target == FramebufferTarget::DrawFramebuffer {
            if let Some(draw_buffers) = draw_buffers {
                self.gl.draw_buffers(&draw_buffers.to_array());
            }
        } else if target == FramebufferTarget::ReadFramebuffer {
            if let Some(read_buffer) = read_buffer {
                self.gl.read_buffer(read_buffer.gl_enum());
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
                    FramebufferTarget::DrawFramebuffer.gl_enum(),
                    Some(gl_framebuffer),
                );
                self.gl.draw_buffers(draw_buffers);
            } else {
                self.gl
                    .bind_framebuffer(FramebufferTarget::DrawFramebuffer.gl_enum(), None);
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
                    FramebufferTarget::ReadFramebuffer.gl_enum(),
                    Some(gl_framebuffer),
                );
                self.gl.read_buffer(*read_buffer);
            } else {
                self.gl
                    .bind_framebuffer(FramebufferTarget::ReadFramebuffer.gl_enum(), None);
            }
        }
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
            mask.gl_enum(),
            filter.gl_enum(),
        );

        self.temp_unbind(FramebufferTarget::ReadFramebuffer, &read_buffer, &None);
        to.temp_unbind(FramebufferTarget::DrawFramebuffer, &None, &draw_buffers);

        Ok(())
    }

    fn copy_to_texture_2d(&mut self, read_buffer: Option<OperableBuffer>) -> Result<(), Error> {
        self.temp_bind(FramebufferTarget::ReadFramebuffer, &read_buffer, &None)?;

        let gl_texture = self.gl.create_texture().ok_or(Error::CreateTextureFailure)?;
        self.gl.bind_texture(TextureTarget::Texture2D.gl_enum(), Some(&gl_texture));
        // self.gl.tex_storage_2d(TextureTarget::Texture2D.gl_enum(), 1, internalformat, width, height);

        self.temp_unbind(FramebufferTarget::ReadFramebuffer, &read_buffer, &None);
        Ok(())
    }

    /// Reads pixels to PixelBufferObject or ArrayBuffer by [`ReadBackKind`].
    fn read_pixels(
        &mut self,
        read_back_kind: ReadBackKind,
        pixel_format: PixelFormat,
        pixel_data_type: PixelDataType,
        pixel_pack_storages: Option<Vec<PixelPackStorage>>,
        read_buffer: Option<OperableBuffer>,
        x: Option<usize>,
        y: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        dst_offset: Option<usize>,
    ) -> Result<ReadBack, Error> {
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

        let readback = match read_back_kind {
            ReadBackKind::ArrayBuffer => {
                // reads into array buffer
                let array_buffer = ArrayBuffer::new(size as u32);
                let typed_buffer = Uint8Array::new(&array_buffer);
                match dst_offset {
                    Some(dst_offset) => {
                        self.gl
                            .read_pixels_with_array_buffer_view_and_dst_offset(
                                x as i32,
                                y as i32,
                                width as i32,
                                height as i32,
                                pixel_format.gl_enum(),
                                pixel_data_type.gl_enum(),
                                &typed_buffer,
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
                                pixel_format.gl_enum(),
                                pixel_data_type.gl_enum(),
                                Some(&typed_buffer),
                            )
                            .or(Err(Error::ReadPixelsFailure))?;
                    }
                };

                ReadBack::ArrayBuffer(array_buffer)
            }
            ReadBackKind::PixelBufferObject(usage) => {
                // reads into pbo
                let gl_buffer = self.gl.create_buffer().ok_or(Error::CreateBufferFailure)?;
                self.gl
                    .bind_buffer(BufferTarget::PixelPackBuffer.gl_enum(), Some(&gl_buffer));
                self.gl.buffer_data_with_i32(
                    BufferTarget::PixelPackBuffer.gl_enum(),
                    size as i32,
                    usage.gl_enum(),
                );

                let dst_offset = dst_offset.unwrap_or(0);
                self.gl
                    .read_pixels_with_i32(
                        x as i32,
                        y as i32,
                        width as i32,
                        height as i32,
                        pixel_format.gl_enum(),
                        pixel_data_type.gl_enum(),
                        dst_offset as i32,
                    )
                    .or(Err(Error::ReadPixelsFailure))?;

                self.gl.bind_buffer(
                    BufferTarget::PixelPackBuffer.gl_enum(),
                    self.reg_buffer_bounds
                        .borrow()
                        .get(&BufferTarget::PixelPackBuffer),
                );

                ReadBack::PixelBufferObject(gl_buffer, size, usage)
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
        read_back_kind: ReadBackKind,
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
    ) -> Result<ReadBack, Error> {
        ClientWaitAsync::new(self.gl.clone(), 0, 5, max_retries)
            .wait()
            .await?;

        self.read_pixels(
            read_back_kind,
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
}

#[derive(Debug)]
pub struct FramebufferRegistry {
    id: Uuid,
    gl: WebGl2RenderingContext,
    framebuffer_bounds: Rc<RefCell<HashMap<FramebufferTarget, (WebGlFramebuffer, u32, Array)>>>,

    reg_buffer_id: Uuid,
    reg_buffer_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
    reg_buffer_used_memory: Weak<RefCell<usize>>,

    reg_texture_active_unit: Rc<RefCell<TextureUnit>>,
    reg_texture_bounds: Rc<RefCell<HashMap<(TextureUnit, TextureTarget), WebGlTexture>>>,
    reg_texture_used_memory: Weak<RefCell<usize>>,
}

impl FramebufferRegistry {
    pub fn new(
        gl: WebGl2RenderingContext,
        buffer_registry: &BufferRegistry,
        texture_registry: &TextureRegistry,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            gl,
            framebuffer_bounds: Rc::new(RefCell::new(HashMap::new())),

            reg_buffer_id: buffer_registry.id,
            reg_buffer_bounds: Rc::clone(&buffer_registry.bounds),
            reg_buffer_used_memory: Rc::downgrade(&buffer_registry.used_memory),

            reg_texture_active_unit: Rc::clone(&texture_registry.texture_active_unit),
            reg_texture_bounds: Rc::clone(&texture_registry.texture_bounds),
            reg_texture_used_memory: Rc::downgrade(&texture_registry.used_memory),
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
                .map(|buffer| buffer.gl_enum())
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

            reg_texture_active_unit: Rc::clone(&self.reg_texture_active_unit),
            reg_texture_bounds: Rc::clone(&self.reg_texture_bounds),

            reg_buffer_id: self.reg_buffer_id,
            reg_buffer_bounds: Rc::clone(&self.reg_buffer_bounds),
            reg_buffer_used_memory: Weak::clone(&self.reg_buffer_used_memory),

            framebuffer_size_policy: framebuffer.size_policy,
            framebuffer_size: None,
            renderbuffer_samples: framebuffer.renderbuffer_samples,
            framebuffer_sources: framebuffer.sources.clone(),
        };

        *framebuffer.registered.borrow_mut() = Some(registered);

        Ok(())
    }
}
