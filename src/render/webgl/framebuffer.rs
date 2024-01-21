use std::iter::FromIterator;

use hashbrown::HashMap;
use log::warn;
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

impl FramebufferAttachment {
    #[rustfmt::skip]
    #[inline]
    fn to_bit_field(&self) -> u32 {
        match self {
            FramebufferAttachment::COLOR_ATTACHMENT0 =>        0b0000000000000000001,
            FramebufferAttachment::COLOR_ATTACHMENT1 =>        0b0000000000000000010,
            FramebufferAttachment::COLOR_ATTACHMENT2 =>        0b0000000000000000100,
            FramebufferAttachment::COLOR_ATTACHMENT3 =>        0b0000000000000001000,
            FramebufferAttachment::COLOR_ATTACHMENT4 =>        0b0000000000000010000,
            FramebufferAttachment::COLOR_ATTACHMENT5 =>        0b0000000000000100000,
            FramebufferAttachment::COLOR_ATTACHMENT6 =>        0b0000000000001000000,
            FramebufferAttachment::COLOR_ATTACHMENT7 =>        0b0000000000010000000,
            FramebufferAttachment::COLOR_ATTACHMENT8 =>        0b0000000000100000000,
            FramebufferAttachment::COLOR_ATTACHMENT9 =>        0b0000000001000000000,
            FramebufferAttachment::COLOR_ATTACHMENT10 =>       0b0000000010000000000,
            FramebufferAttachment::COLOR_ATTACHMENT11 =>       0b0000000100000000000,
            FramebufferAttachment::COLOR_ATTACHMENT12 =>       0b0000001000000000000,
            FramebufferAttachment::COLOR_ATTACHMENT13 =>       0b0000010000000000000,
            FramebufferAttachment::COLOR_ATTACHMENT14 =>       0b0000100000000000000,
            FramebufferAttachment::COLOR_ATTACHMENT15 =>       0b0001000000000000000,
            FramebufferAttachment::DEPTH_ATTACHMENT =>         0b0010000000000000000,
            FramebufferAttachment::STENCIL_ATTACHMENT =>       0b0100000000000000000,
            FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT => 0b1000000000000000000,
        }
    }

    #[inline]
    fn from_index(index: u8) -> Self {
        match index {
            0 => FramebufferAttachment::COLOR_ATTACHMENT0,
            1 => FramebufferAttachment::COLOR_ATTACHMENT1,
            2 => FramebufferAttachment::COLOR_ATTACHMENT2,
            3 => FramebufferAttachment::COLOR_ATTACHMENT3,
            4 => FramebufferAttachment::COLOR_ATTACHMENT4,
            5 => FramebufferAttachment::COLOR_ATTACHMENT5,
            6 => FramebufferAttachment::COLOR_ATTACHMENT6,
            7 => FramebufferAttachment::COLOR_ATTACHMENT7,
            8 => FramebufferAttachment::COLOR_ATTACHMENT8,
            9 => FramebufferAttachment::COLOR_ATTACHMENT9,
            10 => FramebufferAttachment::COLOR_ATTACHMENT10,
            11 => FramebufferAttachment::COLOR_ATTACHMENT11,
            12 => FramebufferAttachment::COLOR_ATTACHMENT12,
            13 => FramebufferAttachment::COLOR_ATTACHMENT13,
            14 => FramebufferAttachment::COLOR_ATTACHMENT14,
            15 => FramebufferAttachment::COLOR_ATTACHMENT15,
            16 => FramebufferAttachment::DEPTH_ATTACHMENT,
            17 => FramebufferAttachment::STENCIL_ATTACHMENT,
            18 => FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT,
            _ => unreachable!(),
        }
    }

    fn extract_bit_field(bit_field: u32) -> Vec<Self> {
        let mut attachments = Vec::new();
        for i in 0..18u8 {
            if (bit_field >> i) & 1 != 0 {
                attachments.push(Self::from_index(i));
            }
        }

        attachments
    }

    #[inline]
    fn to_draw_buffer_index(&self) -> i32 {
        match self {
            FramebufferAttachment::COLOR_ATTACHMENT0 => 0,
            FramebufferAttachment::COLOR_ATTACHMENT1 => 1,
            FramebufferAttachment::COLOR_ATTACHMENT2 => 2,
            FramebufferAttachment::COLOR_ATTACHMENT3 => 3,
            FramebufferAttachment::COLOR_ATTACHMENT4 => 4,
            FramebufferAttachment::COLOR_ATTACHMENT5 => 5,
            FramebufferAttachment::COLOR_ATTACHMENT6 => 6,
            FramebufferAttachment::COLOR_ATTACHMENT7 => 7,
            FramebufferAttachment::COLOR_ATTACHMENT8 => 8,
            FramebufferAttachment::COLOR_ATTACHMENT9 => 9,
            FramebufferAttachment::COLOR_ATTACHMENT10 => 10,
            FramebufferAttachment::COLOR_ATTACHMENT11 => 11,
            FramebufferAttachment::COLOR_ATTACHMENT12 => 12,
            FramebufferAttachment::COLOR_ATTACHMENT13 => 13,
            FramebufferAttachment::COLOR_ATTACHMENT14 => 14,
            FramebufferAttachment::COLOR_ATTACHMENT15 => 15,
            FramebufferAttachment::DEPTH_ATTACHMENT => 0,
            FramebufferAttachment::STENCIL_ATTACHMENT => 0,
            FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT => 0,
        }
    }

    #[inline]
    fn to_draw_buffer(&self) -> Option<FramebufferDrawBuffer> {
        match self {
            FramebufferAttachment::COLOR_ATTACHMENT0 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT0)
            }
            FramebufferAttachment::COLOR_ATTACHMENT1 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT1)
            }
            FramebufferAttachment::COLOR_ATTACHMENT2 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT2)
            }
            FramebufferAttachment::COLOR_ATTACHMENT3 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT3)
            }
            FramebufferAttachment::COLOR_ATTACHMENT4 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT4)
            }
            FramebufferAttachment::COLOR_ATTACHMENT5 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT5)
            }
            FramebufferAttachment::COLOR_ATTACHMENT6 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT6)
            }
            FramebufferAttachment::COLOR_ATTACHMENT7 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT7)
            }
            FramebufferAttachment::COLOR_ATTACHMENT8 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT8)
            }
            FramebufferAttachment::COLOR_ATTACHMENT9 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT9)
            }
            FramebufferAttachment::COLOR_ATTACHMENT10 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT10)
            }
            FramebufferAttachment::COLOR_ATTACHMENT11 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT11)
            }
            FramebufferAttachment::COLOR_ATTACHMENT12 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT12)
            }
            FramebufferAttachment::COLOR_ATTACHMENT13 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT13)
            }
            FramebufferAttachment::COLOR_ATTACHMENT14 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT14)
            }
            FramebufferAttachment::COLOR_ATTACHMENT15 => {
                Some(FramebufferDrawBuffer::COLOR_ATTACHMENT15)
            }
            FramebufferAttachment::DEPTH_ATTACHMENT => None,
            FramebufferAttachment::STENCIL_ATTACHMENT => None,
            FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT => None,
        }
    }

    fn as_message(&self) -> &'static str {
        match self {
            FramebufferAttachment::COLOR_ATTACHMENT0 => "COLOR_ATTACHMENT0",
            FramebufferAttachment::COLOR_ATTACHMENT1 => "COLOR_ATTACHMENT1",
            FramebufferAttachment::COLOR_ATTACHMENT2 => "COLOR_ATTACHMENT2",
            FramebufferAttachment::COLOR_ATTACHMENT3 => "COLOR_ATTACHMENT3",
            FramebufferAttachment::COLOR_ATTACHMENT4 => "COLOR_ATTACHMENT4",
            FramebufferAttachment::COLOR_ATTACHMENT5 => "COLOR_ATTACHMENT5",
            FramebufferAttachment::COLOR_ATTACHMENT6 => "COLOR_ATTACHMENT6",
            FramebufferAttachment::COLOR_ATTACHMENT7 => "COLOR_ATTACHMENT7",
            FramebufferAttachment::COLOR_ATTACHMENT8 => "COLOR_ATTACHMENT8",
            FramebufferAttachment::COLOR_ATTACHMENT9 => "COLOR_ATTACHMENT9",
            FramebufferAttachment::COLOR_ATTACHMENT10 => "COLOR_ATTACHMENT10",
            FramebufferAttachment::COLOR_ATTACHMENT11 => "COLOR_ATTACHMENT11",
            FramebufferAttachment::COLOR_ATTACHMENT12 => "COLOR_ATTACHMENT12",
            FramebufferAttachment::COLOR_ATTACHMENT13 => "COLOR_ATTACHMENT13",
            FramebufferAttachment::COLOR_ATTACHMENT14 => "COLOR_ATTACHMENT14",
            FramebufferAttachment::COLOR_ATTACHMENT15 => "COLOR_ATTACHMENT15",
            FramebufferAttachment::DEPTH_ATTACHMENT => "DEPTH_ATTACHMENT",
            FramebufferAttachment::STENCIL_ATTACHMENT => "STENCIL_ATTACHMENT",
            FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT => "DEPTH_STENCIL_ATTACHMENT",
        }
    }
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

/// Available framebuffer size policies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizePolicy {
    FollowDrawingBuffer,
    ScaleDrawingBuffer(f64),
    Custom { width: i32, height: i32 },
}

impl SizePolicy {
    fn size(&self, gl: &WebGl2RenderingContext) -> (i32, i32) {
        match self {
            Self::FollowDrawingBuffer => {
                let width = gl.drawing_buffer_width();
                let height = gl.drawing_buffer_height();
                (width, height)
            }
            Self::ScaleDrawingBuffer(scale) => {
                let width = gl.drawing_buffer_width();
                let height = gl.drawing_buffer_height();
                (
                    (width as f64 * scale).round() as i32,
                    (height as f64 * scale).round() as i32,
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

#[derive(Debug, Clone)]
pub enum AttachmentProvider {
    FromNewTexture {
        internal_format: TextureInternalFormat,
        format: TextureFormat,
        data_type: TextureDataType,
        clear_policy: ClearPolicy,
    },
    FromExistingTexture {
        texture: WebGlTexture,
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

impl AttachmentProvider {
    pub fn new_texture(
        internal_format: TextureInternalFormat,
        format: TextureFormat,
        data_type: TextureDataType,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromNewTexture {
            internal_format,
            format,
            data_type,
            clear_policy,
        }
    }

    pub fn new_renderbuffer(
        internal_format: RenderbufferInternalFormat,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromNewRenderbuffer {
            internal_format,
            clear_policy,
        }
    }

    pub fn from_texture(texture: WebGlTexture, clear_policy: ClearPolicy) -> Self {
        Self::FromExistingTexture {
            texture,
            clear_policy,
        }
    }

    pub fn from_renderbuffer(renderbuffer: WebGlRenderbuffer, clear_policy: ClearPolicy) -> Self {
        Self::FromExistingRenderbuffer {
            renderbuffer,
            clear_policy,
        }
    }
}

impl AttachmentProvider {
    fn create_attach(
        &self,
        gl: &WebGl2RenderingContext,
        target: FramebufferTarget,
        attachment: FramebufferAttachment,
        width: i32,
        height: i32,
        renderbuffer_samples: Option<i32>,
    ) -> Result<Attach, Error> {
        let attach = match self {
            AttachmentProvider::FromNewTexture {
                internal_format,
                format,
                data_type,
                clear_policy,
            } => {
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
                gl.framebuffer_texture_2d(
                    target.gl_enum(),
                    attachment.gl_enum(),
                    WebGl2RenderingContext::TEXTURE_2D,
                    Some(&texture),
                    0,
                );

                Attach::Texture {
                    texture,
                    clear_policy: *clear_policy,
                }
            }
            AttachmentProvider::FromExistingTexture {
                texture,
                clear_policy,
            } => {
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
                gl.framebuffer_texture_2d(
                    target.gl_enum(),
                    attachment.gl_enum(),
                    WebGl2RenderingContext::TEXTURE_2D,
                    Some(&texture),
                    0,
                );

                Attach::Texture {
                    texture: texture.clone(),
                    clear_policy: *clear_policy,
                }
            }
            AttachmentProvider::FromNewRenderbuffer {
                internal_format,
                clear_policy,
            } => {
                let renderbuffer = gl
                    .create_renderbuffer()
                    .ok_or(Error::CreateRenderbufferFailure)?;
                gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));
                match renderbuffer_samples {
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
                };
                gl.framebuffer_renderbuffer(
                    target.gl_enum(),
                    attachment.gl_enum(),
                    WebGl2RenderingContext::RENDERBUFFER,
                    Some(&renderbuffer),
                );

                Attach::Renderbuffer {
                    renderbuffer,
                    clear_policy: *clear_policy,
                }
            }
            AttachmentProvider::FromExistingRenderbuffer {
                renderbuffer,
                clear_policy,
            } => {
                gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(renderbuffer));
                gl.framebuffer_renderbuffer(
                    target.gl_enum(),
                    attachment.gl_enum(),
                    WebGl2RenderingContext::RENDERBUFFER,
                    Some(renderbuffer),
                );
                Attach::Renderbuffer {
                    renderbuffer: renderbuffer.clone(),
                    clear_policy: *clear_policy,
                }
            }
        };

        Ok(attach)
    }
}

enum Attach {
    Texture {
        texture: WebGlTexture,
        clear_policy: ClearPolicy,
    },
    Renderbuffer {
        renderbuffer: WebGlRenderbuffer,
        clear_policy: ClearPolicy,
    },
}

impl Attach {
    fn clear_policy(&self) -> &ClearPolicy {
        match self {
            Attach::Texture { clear_policy, .. } => clear_policy,
            Attach::Renderbuffer { clear_policy, .. } => clear_policy,
        }
    }
}

struct Bound {
    target: FramebufferTarget,
    read_buffer: Option<FramebufferDrawBuffer>,
    draw_buffers: Option<Array>,
}

struct Runtime {
    width: i32,
    height: i32,
    framebuffer: WebGlFramebuffer,
    attaches: HashMap<FramebufferAttachment, Attach>,
    attachment_bit_field: u32,
    bound: Option<Bound>,
}

impl Runtime {
    // fn new(
    //     gl: &WebGl2RenderingContext,
    //     providers: &HashMap<FramebufferAttachment, AttachmentProvider>,
    //     renderbuffer_samples: Option<i32>,
    //     target: FramebufferTarget,
    //     width: i32,
    //     height: i32,
    // ) -> Result<Self, Error> {
    //     let framebuffer = gl
    //         .create_framebuffer()
    //         .ok_or(Error::CreateFramebufferFailure)?;
    //     gl.bind_framebuffer(target.gl_enum(), Some(&framebuffer));
    //     gl.active_texture(WebGl2RenderingContext::TEXTURE0);

    //     let mut attaches = HashMap::with_capacity(providers.len());
    //     for (attachment, provider) in providers {
    //         let attach = provider.create_attach(
    //             gl,
    //             target,
    //             *attachment,
    //             width,
    //             height,
    //             renderbuffer_samples,
    //         )?;
    //         attaches.insert(*attachment, attach);
    //     }

    //     gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
    //     gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, None);

    //     Ok(Runtime {
    //         width,
    //         height,
    //         attaches,
    //         framebuffer,
    //         attachment_bit_field: 0,
    //         bound: Some(Bound {
    //             target,
    //             read_buffer: None,
    //             draw_buffers: None,
    //         }),
    //     })
    // }

    // fn update_attachment_lazily(
    //     &mut self,
    //     gl: &WebGl2RenderingContext,
    //     attachment: FramebufferAttachment,
    //     provider: &AttachmentProvider,
    //     renderbuffer_samples: Option<i32>,
    // ) -> Result<(), Error> {
    //     match self.bound.as_ref() {
    //         Some(bound) => {
    //             if let Some(attach) = self.attaches.remove(&attachment) {
    //                 match attach {
    //                     Attach::Texture { texture, .. } => {
    //                         gl.framebuffer_texture_2d(
    //                             bound.target.gl_enum(),
    //                             attachment.gl_enum(),
    //                             WebGl2RenderingContext::TEXTURE_2D,
    //                             None,
    //                             0,
    //                         );
    //                         gl.delete_texture(Some(&texture));
    //                     }
    //                     Attach::Renderbuffer { renderbuffer, .. } => {
    //                         gl.framebuffer_renderbuffer(
    //                             bound.target.gl_enum(),
    //                             attachment.gl_enum(),
    //                             WebGl2RenderingContext::RENDERBUFFER,
    //                             None,
    //                         );
    //                         gl.delete_renderbuffer(Some(&renderbuffer));
    //                     }
    //                 }
    //             }

    //             let attach = provider.create_attach(
    //                 gl,
    //                 bound.target,
    //                 attachment,
    //                 self.width,
    //                 self.height,
    //                 renderbuffer_samples,
    //             )?;
    //         }
    //         None => {
    //             self.attaches.remove(&attachment);
    //         }
    //     }

    //     Ok(())
    // }
}

pub struct Framebuffer {
    gl: WebGl2RenderingContext,
    size_policy: SizePolicy,

    providers: HashMap<FramebufferAttachment, AttachmentProvider>,
    draw_buffers: Array,
    attachment_bit_field: u32,
    renderbuffer_samples: Option<i32>,

    runtime: Option<Runtime>,
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        self.unbind();
        if let Some(runtime) = self.runtime.take() {
            self.gl.delete_framebuffer(Some(&runtime.framebuffer));
            runtime
                .attaches
                .iter()
                .for_each(|(_, attach)| match attach {
                    Attach::Texture { texture, .. } => self.gl.delete_texture(Some(texture)),
                    Attach::Renderbuffer { renderbuffer, .. } => {
                        self.gl.delete_renderbuffer(Some(renderbuffer))
                    }
                });
        }
    }
}

impl Framebuffer {
    /// Constructs a new framebuffer object.
    pub fn new<P>(
        gl: WebGl2RenderingContext,
        size_policy: SizePolicy,
        providers: P,
        renderbuffer_samples: Option<i32>,
    ) -> Self
    where
        P: IntoIterator<Item = (FramebufferAttachment, AttachmentProvider)>,
    {
        let mut attachment_bit_field = 0b0000000000000000000;
        let draw_buffers = Array::new();
        let mut ps = HashMap::new();
        for (attachment, provider) in providers {
            if ps.insert(attachment, provider).is_some() {
                warn!("more than one attachment for {}", attachment.as_message());
            } else {
                if let Some(draw_buffer) = attachment.to_draw_buffer() {
                    draw_buffers.push(&JsValue::from_f64(draw_buffer.gl_enum() as f64));
                }
            }
            attachment_bit_field |= attachment.to_bit_field();
        }

        Self {
            gl,
            size_policy,

            providers: ps,
            draw_buffers,
            attachment_bit_field,
            renderbuffer_samples,

            runtime: None,
        }
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn clear_buffer_bits(&self) -> Result<(), Error> {
        let Some(runtime) = self.runtime.as_ref() else {
            return Err(Error::FramebufferUninitialized);
        };
        if runtime.bound.is_none() {
            return Err(Error::FramebufferUnbound);
        }

        runtime.attaches.iter().for_each(|(attachment, attach)| {
            attach
                .clear_policy()
                .clear(&self.gl, attachment.to_draw_buffer_index());
        });

        Ok(())
    }

    pub fn clear_buffer_bits_of_attachment(
        &self,
        attachment: FramebufferAttachment,
    ) -> Result<(), Error> {
        let Some(runtime) = self.runtime.as_ref() else {
            return Err(Error::FramebufferUninitialized);
        };
        if runtime.bound.is_none() {
            return Err(Error::FramebufferUnbound);
        }

        if let Some(attach) = runtime.attaches.get(&attachment) {
            match attach {
                Attach::Texture { clear_policy, .. }
                | Attach::Renderbuffer { clear_policy, .. } => {
                    clear_policy.clear(&self.gl, attachment.to_draw_buffer_index())
                }
            }
        }

        Ok(())
    }

    /// Binds framebuffer to WebGL.
    pub fn bind(&mut self, target: FramebufferTarget) -> Result<(), Error> {
        let (width, height) = self.size_policy.size(&self.gl);

        let runtime = match self.runtime.as_mut() {
            Some(runtime) => runtime,
            None => {
                let framebuffer = self
                    .gl
                    .create_framebuffer()
                    .ok_or(Error::CreateFramebufferFailure)?;
                self.runtime = Some(Runtime {
                    width,
                    height,
                    framebuffer,
                    attachment_bit_field: 0b0000000000000000000,
                    attaches: HashMap::new(),
                    bound: None,
                });
                self.runtime.as_mut().unwrap()
            }
        };

        if let Some(bound) = runtime.bound.as_ref() {
            if bound.target != target {
                return Err(Error::FramebufferBinding(target));
            } else {
                return Ok(());
            }
        }

        if width != runtime.width || height != runtime.height {
            for (attachment, attach) in runtime.attaches.drain() {
                match attach {
                    Attach::Texture { texture, .. } => {
                        self.gl.framebuffer_texture_2d(
                            target.gl_enum(),
                            attachment.gl_enum(),
                            WebGl2RenderingContext::TEXTURE_2D,
                            None,
                            0,
                        );
                        self.gl.delete_texture(Some(&texture));
                    }
                    Attach::Renderbuffer { renderbuffer, .. } => {
                        self.gl.framebuffer_renderbuffer(
                            target.gl_enum(),
                            attachment.gl_enum(),
                            WebGl2RenderingContext::RENDERBUFFER,
                            None,
                        );
                        self.gl.delete_renderbuffer(Some(&renderbuffer));
                    }
                }
            }
            runtime.width = width;
            runtime.height = height;
            runtime.attachment_bit_field = 0b0000000000000000000;
        }

        if self.attachment_bit_field != runtime.attachment_bit_field {
            log::info!("111");
            let should_removes = FramebufferAttachment::extract_bit_field(
                !self.attachment_bit_field & runtime.attachment_bit_field,
            );
            for attachment in should_removes {
                let attach = runtime.attaches.remove(&attachment).unwrap();
                match attach {
                    Attach::Texture { texture, .. } => {
                        self.gl.framebuffer_texture_2d(
                            target.gl_enum(),
                            attachment.gl_enum(),
                            WebGl2RenderingContext::TEXTURE_2D,
                            None,
                            0,
                        );
                        self.gl.delete_texture(Some(&texture));
                    }
                    Attach::Renderbuffer { renderbuffer, .. } => {
                        self.gl.framebuffer_renderbuffer(
                            target.gl_enum(),
                            attachment.gl_enum(),
                            WebGl2RenderingContext::RENDERBUFFER,
                            None,
                        );
                        self.gl.delete_renderbuffer(Some(&renderbuffer));
                    }
                }
            }

            let should_adds = FramebufferAttachment::extract_bit_field(
                self.attachment_bit_field & !runtime.attachment_bit_field,
            );
            for attachment in should_adds {
                let provider = self.providers.get(&attachment).unwrap();
                let attach = provider.create_attach(
                    &self.gl,
                    target,
                    attachment,
                    width,
                    height,
                    self.renderbuffer_samples,
                )?;
                runtime.attaches.insert(attachment, attach);
            }
            runtime.attachment_bit_field = self.attachment_bit_field;
        }

        self.gl.draw_buffers(&self.draw_buffers);

        Ok(())
    }

    pub fn set_read_buffer(&mut self, read_buffer: FramebufferDrawBuffer) -> Result<(), Error> {
        let Some(runtime) = self.runtime.as_mut() else {
            return Err(Error::FramebufferUninitialized);
        };
        let Some(bound) = runtime.bound.as_mut() else {
            return Err(Error::FramebufferUnbound);
        };

        self.gl.read_buffer(read_buffer.gl_enum());
        bound.read_buffer = Some(read_buffer);

        Ok(())
    }

    pub fn set_draw_buffers<I>(&mut self, draw_buffers: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = FramebufferDrawBuffer>,
    {
        let Some(runtime) = self.runtime.as_mut() else {
            return Err(Error::FramebufferUninitialized);
        };
        let Some(bound) = runtime.bound.as_mut() else {
            return Err(Error::FramebufferUnbound);
        };

        let draw_buffers = Array::from_iter(
            draw_buffers
                .into_iter()
                .map(|b| JsValue::from_f64(b.gl_enum() as f64)),
        );
        self.gl.draw_buffers(&draw_buffers);
        bound.draw_buffers = Some(draw_buffers);

        Ok(())
    }

    /// Unbinds framebuffer from WebGL.
    pub fn unbind(&mut self) {
        let Some(mut bound) = self
            .runtime
            .as_mut()
            .and_then(|runtime| runtime.bound.take())
        else {
            return;
        };

        if bound.draw_buffers.take().is_some() {
            self.gl.draw_buffers(&self.draw_buffers);
        }

        self.gl.bind_framebuffer(bound.target.gl_enum(), None);

        if bound.read_buffer.take().is_some() {
            self.gl.read_buffer(WebGl2RenderingContext::BACK);
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
        if self.runtime.is_none() {
            return Err(Error::FramebufferUninitialized);
        };
        let Some(bound) = self.bound_target() else {
            return Err(Error::FramebufferUnbound);
        };
        if bound != FramebufferTarget::READ_FRAMEBUFFER || bound != FramebufferTarget::FRAMEBUFFER {
            return Err(Error::FramebufferUnbound);
        }

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

    /// Returns framebuffer width.
    pub fn width(&self) -> Option<i32> {
        self.runtime.as_ref().map(|runtime| runtime.width)
    }

    /// Returns framebuffer height.
    pub fn height(&self) -> Option<i32> {
        self.runtime.as_ref().map(|runtime| runtime.height)
    }

    /// Returns [`FramebufferTarget`] currently binding to this framebuffer.
    pub fn bound_target(&self) -> Option<FramebufferTarget> {
        self.runtime
            .as_ref()
            .and_then(|runtime| runtime.bound.as_ref())
            .map(|bound| bound.target)
    }

    /// Returns [`WebGlFramebuffer`],
    pub fn framebuffer(&self) -> Option<&WebGlFramebuffer> {
        self.runtime
            .as_ref()
            .map(|runtime| runtime.framebuffer.as_ref())
    }

    /// Returns [`WebGlTexture`] by [`FramebufferAttachment`].
    /// Returns `None` if [`FramebufferAttachment`] does not attach with a texture (even attached with a renderbuffer).
    pub fn texture(&self, attachment: FramebufferAttachment) -> Option<&WebGlTexture> {
        self.runtime
            .as_ref()
            .and_then(|runtime| runtime.attaches.get(&attachment))
            .and_then(|attach| match attach {
                Attach::Texture { texture, .. } => Some(texture),
                Attach::Renderbuffer { .. } => None,
            })
    }

    /// Returns a [`WebGlRenderbuffer`] by [`FramebufferAttachment`].
    /// Returns `None` if [`FramebufferAttachment`] does not attach with a renderbuffer (even attached with a texture).
    pub fn renderbuffer(&self, attachment: FramebufferAttachment) -> Option<&WebGlRenderbuffer> {
        self.runtime
            .as_ref()
            .and_then(|runtime| runtime.attaches.get(&attachment))
            .and_then(|attach| match attach {
                Attach::Texture { .. } => None,
                Attach::Renderbuffer { renderbuffer, .. } => Some(renderbuffer),
            })
    }

    /// Sets render buffer samples. Disabling multisamples by providing `0` or `None`.
    pub fn set_renderbuffer_samples(&mut self, samples: Option<i32>) -> Result<(), Error> {

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
        
        let Some(runtime) = self.runtime.as_mut() else {
            self.renderbuffer_samples = samples;
            return Ok(());
        };
        if let Some(bound) = runtime.bound.as_ref() {
            return Err(Error::FramebufferBinding(bound.target));
        }

        if samples != self.renderbuffer_samples {
            self.renderbuffer_samples = samples;

            if let Some(runtime) = self.runtime.as_mut() {
                for (_, attach) in runtime.attaches.drain() {
                    match attach {
                        Attach::Texture { texture, .. } => {
                            self.gl.delete_texture(Some(&texture));
                        }
                        Attach::Renderbuffer { renderbuffer, .. } => {
                            self.gl.delete_renderbuffer(Some(&renderbuffer));
                        }
                    }
                }
                runtime.attachment_bit_field = 0b0000000000000000000;
            }
        }

        Ok(())
    }

    pub fn set_attachment(
        &mut self,
        attachment: FramebufferAttachment,
        provider: AttachmentProvider,
    ) -> Result<(), Error> {
        if let Some(bound) = self
            .runtime
            .as_ref()
            .and_then(|runtime| runtime.bound.as_ref())
        {
            return Err(Error::FramebufferBinding(bound.target));
        }

        // match self.runtime.as_mut() {
        //     Some(runtime) => {
        //         runtime.update_attachment_lazily(
        //             &self.gl,
        //             attachment,
        //             &provider,
        //             self.renderbuffer_samples(),
        //         )?;
        //         self.providers.insert(attachment, provider);
        //     }
        //     None => {
        //         self.providers.insert(attachment, provider);
        //     }
        // };

        Ok(())
    }
}

pub struct FramebufferBuilder {
    size_policy: SizePolicy,
    providers: HashMap<FramebufferAttachment, AttachmentProvider>,
    draw_buffers: Array,
    renderbuffer_samples: Option<i32>,
}

impl FramebufferBuilder {
    pub fn new() -> Self {
        Self {
            size_policy: SizePolicy::FollowDrawingBuffer,
            providers: HashMap::new(),
            draw_buffers: Array::new(),
            renderbuffer_samples: None,
        }
    }

    pub fn with_size_policy(mut self, size_policy: SizePolicy) -> Self {
        self.size_policy = size_policy;
        self
    }

    pub fn with_samples(mut self, samples: i32) -> Self {
        self.renderbuffer_samples = Some(samples);
        self
    }

    pub fn without_samples(mut self) -> Self {
        self.renderbuffer_samples = None;
        self
    }

    pub fn build(self, gl: WebGl2RenderingContext) -> Framebuffer {
        Framebuffer {
            gl,
            size_policy: self.size_policy,
            providers: self.providers,
            draw_buffers: self.draw_buffers,
            renderbuffer_samples: self.renderbuffer_samples,

            attachment_bit_field: todo!(),
            runtime: None,
        }
    }
}

macro_rules! framebuffer_build_attachments {
    ($(($attachment:expr, $with_func: ident, $without_func: ident)),+) => {
       impl FramebufferBuilder {
        $(
            pub fn $with_func(mut self, provider: AttachmentProvider) -> Self {
                self.providers.insert($attachment, provider);
                if let Some(draw_buffer) = $attachment.to_draw_buffer() {
                    self.draw_buffers.push(&JsValue::from_f64(draw_buffer.gl_enum() as f64));
                }
                self
            }

            pub fn $without_func(mut self) -> Self {
                self.providers.remove(&$attachment);
                self.draw_buffers = self.draw_buffers.filter(&mut |v, _, _| {
                    v.as_f64()
                        != $attachment
                            .to_draw_buffer()
                            .map(|draw_buffer| draw_buffer.gl_enum() as f64)
                });
                self
            }
        )+
       }
    };
}

framebuffer_build_attachments! {
    (FramebufferAttachment::COLOR_ATTACHMENT0, with_color_attachment0, without_color_attachment0),
    (FramebufferAttachment::COLOR_ATTACHMENT1, with_color_attachment1, without_color_attachment1),
    (FramebufferAttachment::COLOR_ATTACHMENT2, with_color_attachment2, without_color_attachment2),
    (FramebufferAttachment::COLOR_ATTACHMENT3, with_color_attachment3, without_color_attachment3),
    (FramebufferAttachment::COLOR_ATTACHMENT4, with_color_attachment4, without_color_attachment4),
    (FramebufferAttachment::COLOR_ATTACHMENT5, with_color_attachment5, without_color_attachment5),
    (FramebufferAttachment::COLOR_ATTACHMENT6, with_color_attachment6, without_color_attachment6),
    (FramebufferAttachment::COLOR_ATTACHMENT7, with_color_attachment7, without_color_attachment7),
    (FramebufferAttachment::COLOR_ATTACHMENT8, with_color_attachment8, without_color_attachment8),
    (FramebufferAttachment::COLOR_ATTACHMENT9, with_color_attachment9, without_color_attachment9),
    (FramebufferAttachment::COLOR_ATTACHMENT10, with_color_attachment10, without_color_attachment10),
    (FramebufferAttachment::COLOR_ATTACHMENT11, with_color_attachment11, without_color_attachment11),
    (FramebufferAttachment::COLOR_ATTACHMENT12, with_color_attachment12, without_color_attachment12),
    (FramebufferAttachment::COLOR_ATTACHMENT13, with_color_attachment13, without_color_attachment13),
    (FramebufferAttachment::COLOR_ATTACHMENT14, with_color_attachment14, without_color_attachment14),
    (FramebufferAttachment::COLOR_ATTACHMENT15, with_color_attachment15, without_color_attachment15),
    (FramebufferAttachment::DEPTH_ATTACHMENT, with_depth_attachment, without_depth_attachment),
    (FramebufferAttachment::STENCIL_ATTACHMENT, with_stencil_attachment, without_stencil_attachment),
    (FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT, with_depth_stencil_attachment, without_depth_stencil_attachment)
}
