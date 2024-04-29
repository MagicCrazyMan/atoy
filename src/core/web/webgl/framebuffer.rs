use std::{cell::RefCell, iter::FromIterator, rc::Rc};

use hashbrown::{hash_map::Entry, HashMap, HashSet};
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
    buffer::BufferTarget,
    conversion::ToGlEnum,
    error::Error,
    renderbuffer::RenderbufferInternalFormat,
    texture::{
        TextureTarget, TextureUncompressedInternalFormat, TexturePixelDataType,
        TexturePixelFormat, TextureUnit,
    },
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

// impl AttachmentSource {
//     fn create_attachment(
//         &self,
//         gl: &WebGl2RenderingContext,
//         width: usize,
//         height: usize,
//         renderbuffer_samples: Option<usize>,
//     ) -> Result<Attachment, Error> {
//         let attach = match self {
//             AttachmentSource::CreateTexture {
//                 internal_format,
//                 clear_policy,
//             } => {
//                 let binding = gl.texture_binding_2d();

//                 let texture = gl.create_texture().ok_or(Error::CreateTextureFailure)?;
//                 gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
//                 gl.tex_storage_2d(
//                     WebGl2RenderingContext::TEXTURE_2D,
//                     1,
//                     internal_format.gl_enum(),
//                     width as i32,
//                     height as i32,
//                 );
//                 gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, binding.as_ref());

//                 Attachment::Texture {
//                     texture,
//                     level: 0,
//                     clear_policy: *clear_policy,
//                     owned: true,
//                 }
//             }
//             AttachmentSource::FromTexture {
//                 texture,
//                 level,
//                 clear_policy,
//             } => Attachment::Texture {
//                 texture: texture.clone(),
//                 level: *level,
//                 clear_policy: *clear_policy,
//                 owned: false,
//             },
//             AttachmentSource::CreateRenderbuffer {
//                 internal_format,
//                 clear_policy,
//             } => {
//                 let binding = gl.renderbuffer_binding();

//                 let renderbuffer = gl
//                     .create_renderbuffer()
//                     .ok_or(Error::CreateRenderbufferFailure)?;
//                 gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
//                 match renderbuffer_samples {
//                     Some(samples) => gl.renderbuffer_storage_multisample(
//                         WebGl2RenderingContext::RENDERBUFFER,
//                         samples as i32,
//                         internal_format.gl_enum(),
//                         width as i32,
//                         height as i32,
//                     ),
//                     None => gl.renderbuffer_storage(
//                         WebGl2RenderingContext::RENDERBUFFER,
//                         internal_format.gl_enum(),
//                         width as i32,
//                         height as i32,
//                     ),
//                 };
//                 gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, binding.as_ref());

//                 Attachment::Renderbuffer {
//                     renderbuffer,
//                     clear_policy: *clear_policy,
//                     owned: true,
//                 }
//             }
//             AttachmentSource::FromRenderbuffer {
//                 renderbuffer,
//                 clear_policy,
//             } => Attachment::Renderbuffer {
//                 renderbuffer: renderbuffer.clone(),
//                 clear_policy: *clear_policy,
//                 owned: false,
//             },
//         };

//         Ok(attach)
//     }
// }

// #[derive(Debug)]
// enum Attachment {
//     Texture {
//         texture: WebGlTexture,
//         clear_policy: ClearPolicy,
//         level: usize,
//         owned: bool,
//     },
//     Renderbuffer {
//         renderbuffer: WebGlRenderbuffer,
//         clear_policy: ClearPolicy,
//         owned: bool,
//     },
// }

// impl Attachment {
//     fn clear_policy(&self) -> &ClearPolicy {
//         match self {
//             Attachment::Texture { clear_policy, .. }
//             | Attachment::Renderbuffer { clear_policy, .. } => clear_policy,
//         }
//     }

//     fn is_owned(&self) -> bool {
//         match self {
//             Attachment::Texture { owned, .. } => *owned,
//             Attachment::Renderbuffer { owned, .. } => *owned,
//         }
//     }

//     fn attach(
//         &self,
//         gl: &WebGl2RenderingContext,
//         target: FramebufferTarget,
//         attachment_target: FramebufferAttachment,
//     ) {
//         match self {
//             Attachment::Texture { texture, level, .. } => {
//                 let binding = gl.texture_binding_2d();
//                 gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
//                 gl.framebuffer_texture_2d(
//                     target.gl_enum(),
//                     attachment_target.gl_enum(),
//                     WebGl2RenderingContext::TEXTURE_2D,
//                     Some(texture),
//                     *level as i32,
//                 );
//                 gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, binding.as_ref());
//             }
//             Attachment::Renderbuffer { renderbuffer, .. } => {
//                 let binding = gl.renderbuffer_binding();
//                 gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(renderbuffer));
//                 gl.framebuffer_renderbuffer(
//                     target.gl_enum(),
//                     attachment_target.gl_enum(),
//                     WebGl2RenderingContext::RENDERBUFFER,
//                     Some(renderbuffer),
//                 );
//                 gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, binding.as_ref());
//             }
//         }
//     }

//     fn delete(&self, gl: &WebGl2RenderingContext) {
//         if self.is_owned() {
//             match self {
//                 Attachment::Texture { texture, .. } => {
//                     gl.delete_texture(Some(texture));
//                 }
//                 Attachment::Renderbuffer { renderbuffer, .. } => {
//                     gl.delete_renderbuffer(Some(renderbuffer));
//                 }
//             }
//         }
//     }
// }

// #[derive(Debug)]
// struct Attach {
//     width: usize,
//     height: usize,
//     framebuffer: WebGlFramebuffer,
//     attachments: HashMap<FramebufferAttachment, Attachment>,
//     default_draw_buffers: Array,
//     default_read_buffer: u32,
// }

// #[derive(Debug)]
// struct Runtime {
//     gl: WebGl2RenderingContext,

//     attach: Option<Attach>,
//     bindings: HashSet<FramebufferTarget>,
// }

// impl Runtime {
//     fn bind(
//         &mut self,
//         target: FramebufferTarget,
//         size_policy: &SizePolicy,
//         renderbuffer_samples: &Option<usize>,
//         sources: &HashMap<FramebufferAttachment, AttachmentSource>,
//     ) -> Result<(), Error> {
//         let (width, height) = size_policy.size(&self.gl);
//         if let Some(attach) = self.attach.as_ref() {
//             // recreates attach if size changed
//             if attach.width != width || attach.height != height {
//                 self.delete();
//             } else {
//                 if self.bindings.contains(&target) {
//                     return Ok(());
//                 } else {
//                     self.gl
//                         .bind_framebuffer(target.gl_enum(), Some(&attach.framebuffer));

//                     if target == FramebufferTarget::DrawFramebuffer {
//                         self.gl.draw_buffers(&attach.default_draw_buffers);
//                     } else if target == FramebufferTarget::ReadFramebuffer {
//                         self.gl.read_buffer(attach.default_read_buffer);
//                     }

//                     self.bindings.insert(target);
//                     return Ok(());
//                 }
//             }
//         }

//         let framebuffer = self
//             .gl
//             .create_framebuffer()
//             .ok_or(Error::CreateFramebufferFailure)?;
//         self.gl
//             .bind_framebuffer(target.gl_enum(), Some(&framebuffer));
//         let (attachments, default_draw_buffers) = sources
//             .iter()
//             .try_fold(
//                 (HashMap::new(), Array::new()),
//                 |(mut attachements, default_draw_buffers), (attachment_target, source)| {
//                     let attachment =
//                         source.create_attachment(&self.gl, width, height, *renderbuffer_samples)?;
//                     attachment.attach(&self.gl, target, *attachment_target);
//                     attachements.insert(*attachment_target, attachment);

//                     if let Some(operable_buffer) = attachment_target.to_operable_buffer() {
//                         default_draw_buffers
//                             .push(&JsValue::from_f64(operable_buffer.gl_enum() as f64));
//                     }

//                     Ok((attachements, default_draw_buffers))
//                 },
//             )
//             .or_else(|err| {
//                 self.gl.bind_framebuffer(target.gl_enum(), None);
//                 Err(err)
//             })?;
//         default_draw_buffers.sort();
//         let default_read_buffer = default_draw_buffers
//             .get(0)
//             .as_f64()
//             .map(|b| b as u32)
//             .unwrap_or(WebGl2RenderingContext::NONE);

//         if target == FramebufferTarget::DrawFramebuffer {
//             self.gl.draw_buffers(&default_draw_buffers);
//         } else if target == FramebufferTarget::ReadFramebuffer {
//             self.gl.read_buffer(default_read_buffer);
//         }

//         self.attach = Some(Attach {
//             width,
//             height,
//             framebuffer,
//             attachments,
//             default_draw_buffers,
//             default_read_buffer,
//         });

//         self.bindings.insert(target);

//         Ok(())
//     }

//     fn unbind(&mut self, target: FramebufferTarget) {
//         if self.bindings.remove(&target) {
//             self.gl.bind_framebuffer(target.gl_enum(), None);
//         }
//     }

//     fn unbind_all(&mut self) {
//         self.unbind(FramebufferTarget::DrawFramebuffer);
//         self.unbind(FramebufferTarget::ReadFramebuffer);
//     }

//     fn delete(&mut self) {
//         if let Some(attach) = self.attach.take() {
//             self.gl.delete_framebuffer(Some(&attach.framebuffer));
//             attach
//                 .attachments
//                 .iter()
//                 .for_each(|(_, attachment)| attachment.delete(&self.gl));
//         }
//     }

//     fn is_bound_as_draw(&self) -> Result<(), Error> {
//         if !self.bindings.contains(&FramebufferTarget::DrawFramebuffer) {
//             return Err(Error::FramebufferUnboundAsDraw);
//         }

//         Ok(())
//     }

//     fn is_bound_as_read(&self) -> Result<(), Error> {
//         if !self.bindings.contains(&FramebufferTarget::ReadFramebuffer) {
//             return Err(Error::FramebufferUnboundAsRead);
//         }

//         Ok(())
//     }
// }

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

    pub fn blit(&self, to: &Self) -> Result<(), Error> {
        self.blit_with_params(
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

    pub fn blit_with_params(
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

        from.blit(
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

    //     /// Initializes framebuffer.
    //     pub fn init(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
    //         if let Some(runtime) = self.runtime.as_ref() {
    //             if &runtime.gl != gl {
    //                 return Err(Error::FramebufferAlreadyInitialized);
    //             } else {
    //                 return Ok(());
    //             }
    //         }

    //         self.runtime = Some(Runtime {
    //             gl: gl.clone(),
    //             attach: None,
    //             bindings: HashSet::new(),
    //         });
    //         Ok(())
    //     }

    //     /// Binds framebuffer.
    //     pub fn bind(&mut self, target: FramebufferTarget) -> Result<(), Error> {
    //         let runtime = self.runtime.as_mut().ok_or(Error::FramebufferRegistered)?;

    //         runtime.bind(
    //             target,
    //             &self.size_policy,
    //             &self.renderbuffer_samples,
    //             &self.sources,
    //         )?;

    //         Ok(())
    //     }

    //     /// Unbinds framebuffer.
    //     pub fn unbind(&mut self, target: FramebufferTarget) -> Result<(), Error> {
    //         let runtime = self.runtime.as_mut().ok_or(Error::FramebufferRegistered)?;
    //         runtime.unbind(target);
    //         Ok(())
    //     }

    //     pub fn unbind_all(&mut self) -> Result<(), Error> {
    //         let runtime = self.runtime.as_mut().ok_or(Error::FramebufferRegistered)?;
    //         runtime.unbind_all();
    //         Ok(())
    //     }

    //     /// Clear specified attached buffer.
    //     pub fn clear_buffer(&self, attachment_target: FramebufferAttachment) -> Result<(), Error> {
    //         let runtime = self.runtime.as_ref().ok_or(Error::FramebufferRegistered)?;
    //         runtime.is_bound_as_draw()?;

    //         if let Some(attach) = runtime.attach.as_ref() {
    //             if let Some(attachment) = attach.attachments.get(&attachment_target) {
    //                 attachment
    //                     .clear_policy()
    //                     .clear(&runtime.gl, attachment_target.to_draw_buffer_index());
    //             }
    //         }

    //         Ok(())
    //     }

    //     /// Clears all attached buffers.
    //     pub fn clear_buffers(&self) -> Result<(), Error> {
    //         let runtime = self.runtime.as_ref().ok_or(Error::FramebufferRegistered)?;
    //         runtime.is_bound_as_draw()?;

    //         if let Some(attach) = runtime.attach.as_ref() {
    //             attach
    //                 .attachments
    //                 .iter()
    //                 .for_each(|(attachment_target, attachment)| {
    //                     attachment
    //                         .clear_policy()
    //                         .clear(&runtime.gl, attachment_target.to_draw_buffer_index());
    //                 });
    //         }

    //         Ok(())
    //     }

    //     pub fn set_read_buffer(&mut self, read_buffer: OperableBuffer) -> Result<(), Error> {
    //         let runtime = self.runtime.as_mut().ok_or(Error::FramebufferRegistered)?;
    //         runtime.is_bound_as_read()?;
    //         runtime.gl.read_buffer(read_buffer.gl_enum());

    //         Ok(())
    //     }

    //     pub fn set_draw_buffers<I>(&mut self, draw_buffers: I) -> Result<(), Error>
    //     where
    //         I: IntoIterator<Item = OperableBuffer>,
    //     {
    //         let runtime = self.runtime.as_mut().ok_or(Error::FramebufferRegistered)?;
    //         runtime.is_bound_as_draw()?;

    //         let draw_buffers = Array::from_iter(
    //             draw_buffers
    //                 .into_iter()
    //                 .map(|b| JsValue::from_f64(b.gl_enum() as f64)),
    //         );
    //         runtime.gl.draw_buffers(&draw_buffers);

    //         Ok(())
    //     }

    //     /// Reads pixels.
    //     pub fn read_pixels(
    //         &mut self,
    //         x: i32,
    //         y: i32,
    //         width: i32,
    //         height: i32,
    //         pixel_format: TextureUncompressedPixelFormat,
    //         data_type: TextureUncompressedPixelDataType,
    //         dst_data: &Object,
    //         dst_offset: u32,
    //     ) -> Result<(), Error> {
    //         let runtime = self.runtime.as_ref().ok_or(Error::FramebufferRegistered)?;
    //         runtime.is_bound_as_read()?;
    //         runtime
    //             .gl
    //             .read_pixels_with_array_buffer_view_and_dst_offset(
    //                 x,
    //                 y,
    //                 width,
    //                 height,
    //                 pixel_format.gl_enum(),
    //                 data_type.gl_enum(),
    //                 dst_data,
    //                 dst_offset,
    //             )
    //             .or_else(|err| Err(Error::ReadPixelsFailure(err.as_string())))?;

    //         Ok(())
    //     }

    //     /// Returns number of sample of the render buffers if multisample is enabled.
    //     pub fn renderbuffer_samples(&self) -> usize {
    //         self.renderbuffer_samples
    //     }

    //     /// Returns framebuffer width.
    //     pub fn width(&self) -> Option<usize> {
    //         self.runtime
    //             .as_ref()
    //             .and_then(|runtime| runtime.attach.as_ref())
    //             .map(|attach| attach.width)
    //     }

    //     /// Returns framebuffer height.
    //     pub fn height(&self) -> Option<usize> {
    //         self.runtime
    //             .as_ref()
    //             .and_then(|runtime| runtime.attach.as_ref())
    //             .map(|attach| attach.height)
    //     }

    //     /// Returns [`WebGlTexture`] by [`FramebufferAttachmentTarget`].
    //     /// Returns `None` if [`FramebufferAttachmentTarget`] does not attach with a texture (even it is attached with a renderbuffer).
    //     pub fn texture(
    //         &self,
    //         attachment_target: FramebufferAttachment,
    //     ) -> Result<Option<&WebGlTexture>, Error> {
    //         let runtime = self.runtime.as_ref().ok_or(Error::FramebufferRegistered)?;
    //         let Some(attach) = runtime.attach.as_ref() else {
    //             return Ok(None);
    //         };

    //         let texture = attach
    //             .attachments
    //             .get(&attachment_target)
    //             .and_then(|attachment| match attachment {
    //                 Attachment::Texture { texture, .. } => Some(texture),
    //                 Attachment::Renderbuffer { .. } => None,
    //             });
    //         Ok(texture)
    //     }

    //     /// Returns a [`WebGlRenderbuffer`] by [`FramebufferAttachmentTarget`].
    //     /// Returns `None` if [`FramebufferAttachmentTarget`] does not attach with a renderbuffer (even it is attached with a texture).
    //     pub fn renderbuffer(
    //         &self,
    //         attachment_target: FramebufferAttachment,
    //     ) -> Result<Option<&WebGlRenderbuffer>, Error> {
    //         let runtime = self.runtime.as_ref().ok_or(Error::FramebufferRegistered)?;
    //         let Some(attach) = runtime.attach.as_ref() else {
    //             return Ok(None);
    //         };

    //         let renderbuffer = attach
    //             .attachments
    //             .get(&attachment_target)
    //             .and_then(|attachment| match attachment {
    //                 Attachment::Texture { .. } => None,
    //                 Attachment::Renderbuffer { renderbuffer, .. } => Some(renderbuffer),
    //             });
    //         Ok(renderbuffer)
    //     }

    //     /// Sets render buffer samples. Disabling multisamples by providing `0` or `None`.
    //     pub fn set_renderbuffer_samples(&mut self, samples: Option<usize>) {
    //         let samples = samples.and_then(|samples| if samples == 0 { None } else { Some(samples) });

    //         if samples == self.renderbuffer_samples {
    //             return;
    //         }

    //         self.renderbuffer_samples = samples;
    //         if let Some(runtime) = self.runtime.as_mut() {
    //             runtime.delete();
    //         }
    //     }

    //     pub fn set_attachment(
    //         &mut self,
    //         attachment_target: FramebufferAttachment,
    //         source: Option<AttachmentSource>,
    //     ) -> Result<(), Error> {
    //         let rebuild = match source {
    //             Some(source) => match self.sources.entry(attachment_target) {
    //                 Entry::Occupied(o) => {
    //                     if o.get() == &source {
    //                         false
    //                     } else {
    //                         o.replace_entry(source);
    //                         true
    //                     }
    //                 }
    //                 Entry::Vacant(v) => {
    //                     v.insert(source);
    //                     true
    //                 }
    //             },
    //             None => self.sources.remove(&attachment_target).is_some(),
    //         };

    //         if rebuild {
    //             if let Some(runtime) = self.runtime.as_mut() {
    //                 runtime.delete();
    //             }
    //         }

    //         Ok(())
    //     }
    // }

    // pub struct FramebufferBuilder {
    //     size_policy: SizePolicy,
    //     sources: HashMap<FramebufferAttachment, AttachmentSource>,
    //     renderbuffer_samples: Option<usize>,
    // }

    // impl FramebufferBuilder {
    //     pub fn new() -> Self {
    //         Self {
    //             size_policy: SizePolicy::FollowDrawingBuffer,
    //             sources: HashMap::new(),
    //             renderbuffer_samples: None,
    //         }
    //     }

    //     pub fn set_size_policy(mut self, size_policy: SizePolicy) -> Self {
    //         self.size_policy = size_policy;
    //         self
    //     }

    //     pub fn set_renderbuffer_samples(mut self, samples: usize) -> Self {
    //         let samples = if samples == 0 { None } else { Some(samples) };
    //         self.renderbuffer_samples = samples;
    //         self
    //     }

    //     pub fn build(self) -> Framebuffer {
    //         Framebuffer {
    //             size_policy: self.size_policy,
    //             sources: self.sources,
    //             renderbuffer_samples: self.renderbuffer_samples,

    //             runtime: None,
    //         }
    //     }
    // }

    // macro_rules! framebuffer_build_attachments {
    //     ($(($attachment:expr, $func: ident)),+) => {
    //        impl FramebufferBuilder {
    //         $(
    //             pub fn $func(mut self, source: AttachmentSource) -> Self {
    //                 self.sources.insert($attachment, source);
    //                 self
    //             }
    //         )+
    //        }
    //     };
}

// framebuffer_build_attachments! {
//     (FramebufferAttachment::ColorAttachment0, set_color_attachment0),
//     (FramebufferAttachment::ColorAttachment1, set_color_attachment1),
//     (FramebufferAttachment::ColorAttachment2, set_color_attachment2),
//     (FramebufferAttachment::ColorAttachment3, set_color_attachment3),
//     (FramebufferAttachment::ColorAttachment4, set_color_attachment4),
//     (FramebufferAttachment::ColorAttachment5, set_color_attachment5),
//     (FramebufferAttachment::ColorAttachment6, set_color_attachment6),
//     (FramebufferAttachment::ColorAttachment7, set_color_attachment7),
//     (FramebufferAttachment::ColorAttachment8, set_color_attachment8),
//     (FramebufferAttachment::ColorAttachment9, set_color_attachment9),
//     (FramebufferAttachment::ColorAttachment10, set_color_attachment10),
//     (FramebufferAttachment::ColorAttachment11, set_color_attachment11),
//     (FramebufferAttachment::ColorAttachment12, set_color_attachment12),
//     (FramebufferAttachment::ColorAttachment13, set_color_attachment13),
//     (FramebufferAttachment::ColorAttachment14, set_color_attachment14),
//     (FramebufferAttachment::ColorAttachment15, set_color_attachment15),
//     (FramebufferAttachment::DepthAttachment, set_depth_attachment),
//     (FramebufferAttachment::StencilAttachment, set_stencil_attachment),
//     (FramebufferAttachment::DepthStencilAttachment, set_depth_stencil_attachment)
// }

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
    reg_active_unit: Rc<RefCell<TextureUnit>>,
    reg_texture_bounds: Rc<RefCell<HashMap<(TextureUnit, TextureTarget), WebGlTexture>>>,
    reg_buffer_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,

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

        self.gl
            .bind_framebuffer(target.gl_enum(), Some(&self.gl_framebuffer));

        // bind read buffers and draw buffers
        let read_buffer = read_buffer.map(|read_buffer| read_buffer.gl_enum());
        let draw_buffers = draw_buffers.map(|draw_buffers| draw_buffers.to_array());
        if target == FramebufferTarget::ReadFramebuffer {
            if let Some(read_buffer) = read_buffer {
                self.gl.read_buffer(read_buffer);
            }
        }
        // binds draw buffers
        else if target == FramebufferTarget::DrawFramebuffer {
            if let Some(draw_buffers) = draw_buffers {
                self.gl.draw_buffers(&draw_buffers);
            }
        }

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
                            self.reg_active_unit.borrow().clone(),
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

    fn blit(
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
        self.gl.bind_framebuffer(
            FramebufferTarget::ReadFramebuffer.gl_enum(),
            Some(&self.gl_framebuffer),
        );
        self.gl.bind_framebuffer(
            FramebufferTarget::DrawFramebuffer.gl_enum(),
            Some(&to.gl_framebuffer),
        );
        if let Some(read_buffer) = read_buffer {
            self.gl.read_buffer(read_buffer.gl_enum());
        }
        if let Some(draw_buffers) = draw_buffers {
            self.gl.draw_buffers(&draw_buffers.to_array());
        }

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

        self.gl.read_buffer(self.gl_origin_read_buffer);
        self.gl.draw_buffers(&to.gl_origin_write_buffers);

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

        Ok(())
    }

    fn read_pixels(
        &mut self,
        pixel_format: TexturePixelFormat,
        pixel_data_type: TexturePixelDataType,
        read_buffer: Option<OperableBuffer>,
        x: Option<usize>,
        y: Option<usize>,
        width: Option<usize>,
        height: Option<usize>,
        dst_offset: Option<usize>
    ) -> Result<(), Error> {
        self.gl.bind_framebuffer(
            FramebufferTarget::ReadFramebuffer.gl_enum(),
            Some(&self.gl_framebuffer),
        );
        if let Some(read_buffer) = read_buffer {
            self.gl.read_buffer(read_buffer.gl_enum());
        }

        todo!();

        
        self.gl.read_buffer(self.gl_origin_read_buffer);
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
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct FramebufferRegistry {
    id: Uuid,
    gl: WebGl2RenderingContext,
    framebuffer_bounds: Rc<RefCell<HashMap<FramebufferTarget, WebGlFramebuffer>>>,
    active_unit: Rc<RefCell<TextureUnit>>,
    texture_bounds: Rc<RefCell<HashMap<(TextureUnit, TextureTarget), WebGlTexture>>>,
    buffer_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
}

impl FramebufferRegistry {
    pub fn new(
        gl: WebGl2RenderingContext,
        buffer_bounds: Rc<RefCell<HashMap<BufferTarget, WebGlBuffer>>>,
        active_unit: Rc<RefCell<TextureUnit>>,
        texture_bounds: Rc<RefCell<HashMap<(TextureUnit, TextureTarget), WebGlTexture>>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            gl,
            framebuffer_bounds: Rc::new(RefCell::new(HashMap::new())),
            active_unit,
            texture_bounds,
            buffer_bounds,
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
            reg_active_unit: Rc::clone(&self.active_unit),
            reg_texture_bounds: Rc::clone(&self.texture_bounds),
            reg_buffer_bounds: Rc::clone(&self.buffer_bounds),

            framebuffer_size_policy: framebuffer.size_policy,
            framebuffer_size: None,
            renderbuffer_samples: framebuffer.renderbuffer_samples,
            framebuffer_sources: framebuffer.sources.clone(),
        };

        *framebuffer.registered.borrow_mut() = Some(registered);

        Ok(())
    }
}
