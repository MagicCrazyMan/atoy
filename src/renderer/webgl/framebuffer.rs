use std::iter::FromIterator;

use hashbrown::{hash_map::Entry, HashMap, HashSet};
use log::warn;
use wasm_bindgen::JsValue;
use web_sys::{
    js_sys::{Array, Object},
    WebGl2RenderingContext, WebGlFramebuffer, WebGlRenderbuffer, WebGlTexture,
};

use super::{
    conversion::ToGlEnum,
    error::Error,
    params::GetWebGlParameters,
    renderbuffer::RenderbufferInternalFormat,
    texture::{
        TextureUncompressedInternalFormat, TextureUncompressedPixelDataType,
        TextureUncompressedPixelFormat,
    },
};

/// Available framebuffer targets mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramebufferTarget {
    READ_FRAMEBUFFER,
    DRAW_FRAMEBUFFER,
}

/// Available framebuffer attachment targets mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramebufferAttachmentTarget {
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

impl FramebufferAttachmentTarget {
    fn to_draw_buffer_index(&self) -> i32 {
        match self {
            FramebufferAttachmentTarget::COLOR_ATTACHMENT0 => 0,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT1 => 1,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT2 => 2,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT3 => 3,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT4 => 4,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT5 => 5,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT6 => 6,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT7 => 7,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT8 => 8,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT9 => 9,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT10 => 10,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT11 => 11,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT12 => 12,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT13 => 13,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT14 => 14,
            FramebufferAttachmentTarget::COLOR_ATTACHMENT15 => 15,
            FramebufferAttachmentTarget::DEPTH_ATTACHMENT => 0,
            FramebufferAttachmentTarget::STENCIL_ATTACHMENT => 0,
            FramebufferAttachmentTarget::DEPTH_STENCIL_ATTACHMENT => 0,
        }
    }

    fn to_operable_buffer(&self) -> Option<OperableBuffer> {
        match self {
            FramebufferAttachmentTarget::COLOR_ATTACHMENT0 => {
                Some(OperableBuffer::COLOR_ATTACHMENT0)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT1 => {
                Some(OperableBuffer::COLOR_ATTACHMENT1)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT2 => {
                Some(OperableBuffer::COLOR_ATTACHMENT2)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT3 => {
                Some(OperableBuffer::COLOR_ATTACHMENT3)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT4 => {
                Some(OperableBuffer::COLOR_ATTACHMENT4)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT5 => {
                Some(OperableBuffer::COLOR_ATTACHMENT5)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT6 => {
                Some(OperableBuffer::COLOR_ATTACHMENT6)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT7 => {
                Some(OperableBuffer::COLOR_ATTACHMENT7)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT8 => {
                Some(OperableBuffer::COLOR_ATTACHMENT8)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT9 => {
                Some(OperableBuffer::COLOR_ATTACHMENT9)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT10 => {
                Some(OperableBuffer::COLOR_ATTACHMENT10)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT11 => {
                Some(OperableBuffer::COLOR_ATTACHMENT11)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT12 => {
                Some(OperableBuffer::COLOR_ATTACHMENT12)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT13 => {
                Some(OperableBuffer::COLOR_ATTACHMENT13)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT14 => {
                Some(OperableBuffer::COLOR_ATTACHMENT14)
            }
            FramebufferAttachmentTarget::COLOR_ATTACHMENT15 => {
                Some(OperableBuffer::COLOR_ATTACHMENT15)
            }
            FramebufferAttachmentTarget::DEPTH_ATTACHMENT => None,
            FramebufferAttachmentTarget::STENCIL_ATTACHMENT => None,
            FramebufferAttachmentTarget::DEPTH_STENCIL_ATTACHMENT => None,
        }
    }

    fn as_message(&self) -> &'static str {
        match self {
            FramebufferAttachmentTarget::COLOR_ATTACHMENT0 => "COLOR_ATTACHMENT0",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT1 => "COLOR_ATTACHMENT1",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT2 => "COLOR_ATTACHMENT2",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT3 => "COLOR_ATTACHMENT3",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT4 => "COLOR_ATTACHMENT4",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT5 => "COLOR_ATTACHMENT5",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT6 => "COLOR_ATTACHMENT6",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT7 => "COLOR_ATTACHMENT7",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT8 => "COLOR_ATTACHMENT8",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT9 => "COLOR_ATTACHMENT9",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT10 => "COLOR_ATTACHMENT10",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT11 => "COLOR_ATTACHMENT11",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT12 => "COLOR_ATTACHMENT12",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT13 => "COLOR_ATTACHMENT13",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT14 => "COLOR_ATTACHMENT14",
            FramebufferAttachmentTarget::COLOR_ATTACHMENT15 => "COLOR_ATTACHMENT15",
            FramebufferAttachmentTarget::DEPTH_ATTACHMENT => "DEPTH_ATTACHMENT",
            FramebufferAttachmentTarget::STENCIL_ATTACHMENT => "STENCIL_ATTACHMENT",
            FramebufferAttachmentTarget::DEPTH_STENCIL_ATTACHMENT => "DEPTH_STENCIL_ATTACHMENT",
        }
    }
}

/// Available drawable or readable buffer attachment mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperableBuffer {
    NONE,
    /// [`WebGl2RenderingContext::BACK`] only works for Canvas Draw Buffer.
    /// Do not bind this attachment to FBO.
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
    FromNewTexture {
        internal_format: TextureUncompressedInternalFormat,
        clear_policy: ClearPolicy,
    },
    FromExistingTexture {
        texture: WebGlTexture,
        level: usize,
        clear_policy: ClearPolicy,
    },
    FromNewRenderbuffer {
        internal_format: RenderbufferInternalFormat,
        clear_policy: ClearPolicy,
    },
    FromExistingRenderbuffer {
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

        Self::FromNewTexture {
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

        Self::FromNewRenderbuffer {
            internal_format,
            clear_policy,
        }
    }

    pub fn from_texture(texture: WebGlTexture, level: usize, clear_policy: ClearPolicy) -> Self {
        Self::FromExistingTexture {
            texture,
            level,
            clear_policy,
        }
    }

    pub fn from_renderbuffer(renderbuffer: WebGlRenderbuffer, clear_policy: ClearPolicy) -> Self {
        Self::FromExistingRenderbuffer {
            renderbuffer,
            clear_policy,
        }
    }

    pub fn new_texture_with_clear_policy(
        internal_format: TextureUncompressedInternalFormat,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromNewTexture {
            internal_format,
            clear_policy,
        }
    }

    pub fn new_renderbuffer_with_clear_policy(
        internal_format: RenderbufferInternalFormat,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromNewRenderbuffer {
            internal_format,
            clear_policy,
        }
    }
}

impl AttachmentSource {
    fn create_attachment(
        &self,
        gl: &WebGl2RenderingContext,
        width: usize,
        height: usize,
        renderbuffer_samples: Option<usize>,
    ) -> Result<Attachment, Error> {
        let attach = match self {
            AttachmentSource::FromNewTexture {
                internal_format,
                clear_policy,
            } => {
                let binding = gl.texture_binding_2d();

                let texture = gl.create_texture().ok_or(Error::CreateTextureFailure)?;
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
                gl.tex_storage_2d(
                    WebGl2RenderingContext::TEXTURE_2D,
                    1,
                    internal_format.gl_enum(),
                    width as i32,
                    height as i32,
                );
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, binding.as_ref());

                Attachment::Texture {
                    texture,
                    level: 0,
                    clear_policy: *clear_policy,
                    owned: true,
                }
            }
            AttachmentSource::FromExistingTexture {
                texture,
                level,
                clear_policy,
            } => Attachment::Texture {
                texture: texture.clone(),
                level: *level,
                clear_policy: *clear_policy,
                owned: false,
            },
            AttachmentSource::FromNewRenderbuffer {
                internal_format,
                clear_policy,
            } => {
                let binding = gl.renderbuffer_binding();

                let renderbuffer = gl
                    .create_renderbuffer()
                    .ok_or(Error::CreateRenderbufferFailure)?;
                gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
                match renderbuffer_samples {
                    Some(samples) => gl.renderbuffer_storage_multisample(
                        WebGl2RenderingContext::RENDERBUFFER,
                        samples as i32,
                        internal_format.gl_enum(),
                        width as i32,
                        height as i32,
                    ),
                    None => gl.renderbuffer_storage(
                        WebGl2RenderingContext::RENDERBUFFER,
                        internal_format.gl_enum(),
                        width as i32,
                        height as i32,
                    ),
                };
                gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, binding.as_ref());

                Attachment::Renderbuffer {
                    renderbuffer,
                    clear_policy: *clear_policy,
                    owned: true,
                }
            }
            AttachmentSource::FromExistingRenderbuffer {
                renderbuffer,
                clear_policy,
            } => Attachment::Renderbuffer {
                renderbuffer: renderbuffer.clone(),
                clear_policy: *clear_policy,
                owned: false,
            },
        };

        Ok(attach)
    }
}

#[derive(Debug)]
enum Attachment {
    Texture {
        texture: WebGlTexture,
        clear_policy: ClearPolicy,
        level: usize,
        owned: bool,
    },
    Renderbuffer {
        renderbuffer: WebGlRenderbuffer,
        clear_policy: ClearPolicy,
        owned: bool,
    },
}

impl Attachment {
    fn clear_policy(&self) -> &ClearPolicy {
        match self {
            Attachment::Texture { clear_policy, .. }
            | Attachment::Renderbuffer { clear_policy, .. } => clear_policy,
        }
    }

    fn is_owned(&self) -> bool {
        match self {
            Attachment::Texture { owned, .. } => *owned,
            Attachment::Renderbuffer { owned, .. } => *owned,
        }
    }

    fn attach(
        &self,
        gl: &WebGl2RenderingContext,
        target: FramebufferTarget,
        attachment_target: FramebufferAttachmentTarget,
    ) {
        match self {
            Attachment::Texture { texture, level, .. } => {
                let binding = gl.texture_binding_2d();
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
                gl.framebuffer_texture_2d(
                    target.gl_enum(),
                    attachment_target.gl_enum(),
                    WebGl2RenderingContext::TEXTURE_2D,
                    Some(texture),
                    *level as i32,
                );
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, binding.as_ref());
            }
            Attachment::Renderbuffer { renderbuffer, .. } => {
                let binding = gl.renderbuffer_binding();
                gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(renderbuffer));
                gl.framebuffer_renderbuffer(
                    target.gl_enum(),
                    attachment_target.gl_enum(),
                    WebGl2RenderingContext::RENDERBUFFER,
                    Some(renderbuffer),
                );
                gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, binding.as_ref());
            }
        }
    }

    fn delete(&self, gl: &WebGl2RenderingContext) {
        if self.is_owned() {
            match self {
                Attachment::Texture { texture, .. } => {
                    gl.delete_texture(Some(texture));
                }
                Attachment::Renderbuffer { renderbuffer, .. } => {
                    gl.delete_renderbuffer(Some(renderbuffer));
                }
            }
        }
    }
}

#[derive(Debug)]
struct Attach {
    width: usize,
    height: usize,
    framebuffer: WebGlFramebuffer,
    attachments: HashMap<FramebufferAttachmentTarget, Attachment>,
    default_draw_buffers: Array,
    default_read_buffer: u32,
}

#[derive(Debug)]
struct Runtime {
    gl: WebGl2RenderingContext,

    attach: Option<Attach>,
    bindings: HashSet<FramebufferTarget>,
}

impl Runtime {
    fn bind(
        &mut self,
        target: FramebufferTarget,
        size_policy: &SizePolicy,
        renderbuffer_samples: &Option<usize>,
        sources: &HashMap<FramebufferAttachmentTarget, AttachmentSource>,
    ) -> Result<(), Error> {
        let (width, height) = size_policy.size(&self.gl);
        if let Some(attach) = self.attach.as_ref() {
            // recreates attach if size changed
            if attach.width != width || attach.height != height {
                self.delete();
            } else {
                if self.bindings.contains(&target) {
                    return Ok(());
                } else {
                    self.gl
                        .bind_framebuffer(target.gl_enum(), Some(&attach.framebuffer));

                    if target == FramebufferTarget::DRAW_FRAMEBUFFER {
                        self.gl.draw_buffers(&attach.default_draw_buffers);
                    } else if target == FramebufferTarget::READ_FRAMEBUFFER {
                        self.gl.read_buffer(attach.default_read_buffer);
                    }

                    self.bindings.insert(target);
                    return Ok(());
                }
            }
        }

        let framebuffer = self
            .gl
            .create_framebuffer()
            .ok_or(Error::CreateFramebufferFailure)?;
        self.gl
            .bind_framebuffer(target.gl_enum(), Some(&framebuffer));
        let (attachments, default_draw_buffers) = sources
            .iter()
            .try_fold(
                (HashMap::new(), Array::new()),
                |(mut attachements, default_draw_buffers), (attachment_target, source)| {
                    let attachment =
                        source.create_attachment(&self.gl, width, height, *renderbuffer_samples)?;
                    attachment.attach(&self.gl, target, *attachment_target);
                    attachements.insert(*attachment_target, attachment);

                    if let Some(operable_buffer) = attachment_target.to_operable_buffer() {
                        default_draw_buffers
                            .push(&JsValue::from_f64(operable_buffer.gl_enum() as f64));
                    }

                    Ok((attachements, default_draw_buffers))
                },
            )
            .or_else(|err| {
                self.gl.bind_framebuffer(target.gl_enum(), None);
                Err(err)
            })?;
        default_draw_buffers.sort();
        let default_read_buffer = default_draw_buffers
            .get(0)
            .as_f64()
            .map(|b| b as u32)
            .unwrap_or(WebGl2RenderingContext::NONE);

        if target == FramebufferTarget::DRAW_FRAMEBUFFER {
            self.gl.draw_buffers(&default_draw_buffers);
        } else if target == FramebufferTarget::READ_FRAMEBUFFER {
            self.gl.read_buffer(default_read_buffer);
        }

        self.attach = Some(Attach {
            width,
            height,
            framebuffer,
            attachments,
            default_draw_buffers,
            default_read_buffer,
        });

        self.bindings.insert(target);

        Ok(())
    }

    fn unbind(&mut self, target: FramebufferTarget) {
        if self.bindings.remove(&target) {
            self.gl.bind_framebuffer(target.gl_enum(), None);
        }
    }

    fn unbind_all(&mut self) {
        self.unbind(FramebufferTarget::DRAW_FRAMEBUFFER);
        self.unbind(FramebufferTarget::READ_FRAMEBUFFER);
    }

    fn delete(&mut self) {
        if let Some(attach) = self.attach.take() {
            self.gl.delete_framebuffer(Some(&attach.framebuffer));
            attach
                .attachments
                .iter()
                .for_each(|(_, attachment)| attachment.delete(&self.gl));
        }
    }

    fn is_bound_as_draw(&self) -> Result<(), Error> {
        if !self.bindings.contains(&FramebufferTarget::DRAW_FRAMEBUFFER) {
            return Err(Error::FramebufferUnboundAsDraw);
        }

        Ok(())
    }

    fn is_bound_as_read(&self) -> Result<(), Error> {
        if !self.bindings.contains(&FramebufferTarget::READ_FRAMEBUFFER) {
            return Err(Error::FramebufferUnboundAsRead);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Framebuffer {
    size_policy: SizePolicy,
    sources: HashMap<FramebufferAttachmentTarget, AttachmentSource>,
    renderbuffer_samples: Option<usize>,

    runtime: Option<Runtime>,
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.unbind_all();
            runtime.delete();
        }
    }
}

impl Framebuffer {
    /// Constructs a new framebuffer object.
    pub fn new<S>(sources: S, size_policy: SizePolicy, renderbuffer_samples: Option<usize>) -> Self
    where
        S: IntoIterator<Item = (FramebufferAttachmentTarget, AttachmentSource)>,
    {
        let sources =
            sources
                .into_iter()
                .fold(HashMap::new(), |mut sources, (attachment, source)| {
                    if sources.contains_key(&attachment) {
                        warn!(
                            "duplicated attachment for {}, older one overrode",
                            attachment.as_message()
                        );
                    }

                    sources.insert(attachment, source);
                    sources
                });

        let renderbuffer_samples =
            renderbuffer_samples
                .and_then(|samples| if samples == 0 { None } else { Some(samples) });
        Self {
            size_policy,
            sources,
            renderbuffer_samples,

            runtime: None,
        }
    }

    /// Initializes framebuffer.
    pub fn init(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        if let Some(runtime) = self.runtime.as_ref() {
            if &runtime.gl != gl {
                return Err(Error::FramebufferAlreadyInitialized);
            } else {
                return Ok(());
            }
        }

        self.runtime = Some(Runtime {
            gl: gl.clone(),
            attach: None,
            bindings: HashSet::new(),
        });
        Ok(())
    }

    /// Binds framebuffer.
    pub fn bind(&mut self, target: FramebufferTarget) -> Result<(), Error> {
        let runtime = self
            .runtime
            .as_mut()
            .ok_or(Error::FramebufferUninitialized)?;

        runtime.bind(
            target,
            &self.size_policy,
            &self.renderbuffer_samples,
            &self.sources,
        )?;

        Ok(())
    }

    /// Unbinds framebuffer.
    pub fn unbind(&mut self, target: FramebufferTarget) -> Result<(), Error> {
        let runtime = self
            .runtime
            .as_mut()
            .ok_or(Error::FramebufferUninitialized)?;
        runtime.unbind(target);
        Ok(())
    }

    pub fn unbind_all(&mut self) -> Result<(), Error> {
        let runtime = self
            .runtime
            .as_mut()
            .ok_or(Error::FramebufferUninitialized)?;
        runtime.unbind_all();
        Ok(())
    }

    /// Clear specified attached buffer.
    pub fn clear_buffer(
        &self,
        attachment_target: FramebufferAttachmentTarget,
    ) -> Result<(), Error> {
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(Error::FramebufferUninitialized)?;
        runtime.is_bound_as_draw()?;

        if let Some(attach) = runtime.attach.as_ref() {
            if let Some(attachment) = attach.attachments.get(&attachment_target) {
                attachment
                    .clear_policy()
                    .clear(&runtime.gl, attachment_target.to_draw_buffer_index());
            }
        }

        Ok(())
    }

    /// Clears all attached buffers.
    pub fn clear_buffers(&self) -> Result<(), Error> {
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(Error::FramebufferUninitialized)?;
        runtime.is_bound_as_draw()?;

        if let Some(attach) = runtime.attach.as_ref() {
            attach
                .attachments
                .iter()
                .for_each(|(attachment_target, attachment)| {
                    attachment
                        .clear_policy()
                        .clear(&runtime.gl, attachment_target.to_draw_buffer_index());
                });
        }

        Ok(())
    }

    pub fn set_read_buffer(&mut self, read_buffer: OperableBuffer) -> Result<(), Error> {
        let runtime = self
            .runtime
            .as_mut()
            .ok_or(Error::FramebufferUninitialized)?;
        runtime.is_bound_as_read()?;
        runtime.gl.read_buffer(read_buffer.gl_enum());

        Ok(())
    }

    pub fn set_draw_buffers<I>(&mut self, draw_buffers: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = OperableBuffer>,
    {
        let runtime = self
            .runtime
            .as_mut()
            .ok_or(Error::FramebufferUninitialized)?;
        runtime.is_bound_as_draw()?;

        let draw_buffers = Array::from_iter(
            draw_buffers
                .into_iter()
                .map(|b| JsValue::from_f64(b.gl_enum() as f64)),
        );
        runtime.gl.draw_buffers(&draw_buffers);

        Ok(())
    }

    /// Reads pixels.
    pub fn read_pixels(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        pixel_format: TextureUncompressedPixelFormat,
        data_type: TextureUncompressedPixelDataType,
        dst_data: &Object,
        dst_offset: u32,
    ) -> Result<(), Error> {
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(Error::FramebufferUninitialized)?;
        runtime.is_bound_as_read()?;
        runtime
            .gl
            .read_pixels_with_array_buffer_view_and_dst_offset(
                x,
                y,
                width,
                height,
                pixel_format.gl_enum(),
                data_type.gl_enum(),
                dst_data,
                dst_offset,
            )
            .or_else(|err| Err(Error::ReadPixelsFailure(err.as_string())))?;

        Ok(())
    }

    /// Returns number of sample of the render buffers if multisample is enabled.
    pub fn renderbuffer_samples(&self) -> Option<usize> {
        self.renderbuffer_samples
    }

    /// Returns framebuffer width.
    pub fn width(&self) -> Option<usize> {
        self.runtime
            .as_ref()
            .and_then(|runtime| runtime.attach.as_ref())
            .map(|attach| attach.width)
    }

    /// Returns framebuffer height.
    pub fn height(&self) -> Option<usize> {
        self.runtime
            .as_ref()
            .and_then(|runtime| runtime.attach.as_ref())
            .map(|attach| attach.height)
    }

    /// Returns [`WebGlTexture`] by [`FramebufferAttachmentTarget`].
    /// Returns `None` if [`FramebufferAttachmentTarget`] does not attach with a texture (even it is attached with a renderbuffer).
    pub fn texture(
        &self,
        attachment_target: FramebufferAttachmentTarget,
    ) -> Result<Option<&WebGlTexture>, Error> {
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(Error::FramebufferUninitialized)?;
        let Some(attach) = runtime.attach.as_ref() else {
            return Ok(None);
        };

        let texture = attach
            .attachments
            .get(&attachment_target)
            .and_then(|attachment| match attachment {
                Attachment::Texture { texture, .. } => Some(texture),
                Attachment::Renderbuffer { .. } => None,
            });
        Ok(texture)
    }

    /// Returns a [`WebGlRenderbuffer`] by [`FramebufferAttachmentTarget`].
    /// Returns `None` if [`FramebufferAttachmentTarget`] does not attach with a renderbuffer (even it is attached with a texture).
    pub fn renderbuffer(
        &self,
        attachment_target: FramebufferAttachmentTarget,
    ) -> Result<Option<&WebGlRenderbuffer>, Error> {
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(Error::FramebufferUninitialized)?;
        let Some(attach) = runtime.attach.as_ref() else {
            return Ok(None);
        };

        let renderbuffer = attach
            .attachments
            .get(&attachment_target)
            .and_then(|attachment| match attachment {
                Attachment::Texture { .. } => None,
                Attachment::Renderbuffer { renderbuffer, .. } => Some(renderbuffer),
            });
        Ok(renderbuffer)
    }

    /// Sets render buffer samples. Disabling multisamples by providing `0` or `None`.
    pub fn set_renderbuffer_samples(&mut self, samples: Option<usize>) {
        let samples = samples.and_then(|samples| if samples == 0 { None } else { Some(samples) });

        if samples == self.renderbuffer_samples {
            return;
        }

        self.renderbuffer_samples = samples;
        if let Some(runtime) = self.runtime.as_mut() {
            runtime.delete();
        }
    }

    pub fn set_attachment(
        &mut self,
        attachment_target: FramebufferAttachmentTarget,
        source: Option<AttachmentSource>,
    ) -> Result<(), Error> {
        let rebuild = match source {
            Some(source) => match self.sources.entry(attachment_target) {
                Entry::Occupied(o) => {
                    if o.get() == &source {
                        false
                    } else {
                        o.replace_entry(source);
                        true
                    }
                }
                Entry::Vacant(v) => {
                    v.insert(source);
                    true
                }
            },
            None => self.sources.remove(&attachment_target).is_some(),
        };

        if rebuild {
            if let Some(runtime) = self.runtime.as_mut() {
                runtime.delete();
            }
        }

        Ok(())
    }
}

pub struct FramebufferBuilder {
    size_policy: SizePolicy,
    sources: HashMap<FramebufferAttachmentTarget, AttachmentSource>,
    renderbuffer_samples: Option<usize>,
}

impl FramebufferBuilder {
    pub fn new() -> Self {
        Self {
            size_policy: SizePolicy::FollowDrawingBuffer,
            sources: HashMap::new(),
            renderbuffer_samples: None,
        }
    }

    pub fn set_size_policy(mut self, size_policy: SizePolicy) -> Self {
        self.size_policy = size_policy;
        self
    }

    pub fn set_renderbuffer_samples(mut self, samples: usize) -> Self {
        let samples = if samples == 0 { None } else { Some(samples) };
        self.renderbuffer_samples = samples;
        self
    }

    pub fn build(self) -> Framebuffer {
        Framebuffer {
            size_policy: self.size_policy,
            sources: self.sources,
            renderbuffer_samples: self.renderbuffer_samples,

            runtime: None,
        }
    }
}

macro_rules! framebuffer_build_attachments {
    ($(($attachment:expr, $func: ident)),+) => {
       impl FramebufferBuilder {
        $(
            pub fn $func(mut self, source: AttachmentSource) -> Self {
                self.sources.insert($attachment, source);
                self
            }
        )+
       }
    };
}

framebuffer_build_attachments! {
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT0, set_color_attachment0),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT1, set_color_attachment1),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT2, set_color_attachment2),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT3, set_color_attachment3),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT4, set_color_attachment4),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT5, set_color_attachment5),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT6, set_color_attachment6),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT7, set_color_attachment7),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT8, set_color_attachment8),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT9, set_color_attachment9),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT10, set_color_attachment10),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT11, set_color_attachment11),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT12, set_color_attachment12),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT13, set_color_attachment13),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT14, set_color_attachment14),
    (FramebufferAttachmentTarget::COLOR_ATTACHMENT15, set_color_attachment15),
    (FramebufferAttachmentTarget::DEPTH_ATTACHMENT, set_depth_attachment),
    (FramebufferAttachmentTarget::STENCIL_ATTACHMENT, set_stencil_attachment),
    (FramebufferAttachmentTarget::DEPTH_STENCIL_ATTACHMENT, set_depth_stencil_attachment)
}
