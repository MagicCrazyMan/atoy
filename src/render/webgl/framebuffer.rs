use std::iter::FromIterator;

use hashbrown::{HashMap, HashSet};
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
    texture::{TextureDataType, TextureFormat, TextureColorFormat},
    utils::{renderbuffer_binding, texture_binding_2d},
};

/// Available framebuffer targets mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FramebufferTarget {
    FRAMEBUFFER,
    READ_FRAMEBUFFER,
    DRAW_FRAMEBUFFER,
}

pub const DEFAULT_CLEAR_POLICY_FOR_COLOR_ATTACHMENT: ClearPolicy =
    ClearPolicy::ColorFloat([0.0, 0.0, 0.0, 0.0]);
pub const DEFAULT_CLEAR_POLICY_FOR_DEPTH_ATTACHMENT: ClearPolicy = ClearPolicy::Depth(1.0);
pub const DEFAULT_CLEAR_POLICY_FOR_STENCIL_ATTACHMENT: ClearPolicy = ClearPolicy::Stencil(0);
pub const DEFAULT_CLEAR_POLICY_FOR_DEPTH_STENCIL_ATTACHMENT: ClearPolicy =
    ClearPolicy::DepthStencil(1.0, 0);

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
    fn to_draw_buffer(&self) -> Option<OperatableBuffer> {
        match self {
            FramebufferAttachment::COLOR_ATTACHMENT0 => Some(OperatableBuffer::COLOR_ATTACHMENT0),
            FramebufferAttachment::COLOR_ATTACHMENT1 => Some(OperatableBuffer::COLOR_ATTACHMENT1),
            FramebufferAttachment::COLOR_ATTACHMENT2 => Some(OperatableBuffer::COLOR_ATTACHMENT2),
            FramebufferAttachment::COLOR_ATTACHMENT3 => Some(OperatableBuffer::COLOR_ATTACHMENT3),
            FramebufferAttachment::COLOR_ATTACHMENT4 => Some(OperatableBuffer::COLOR_ATTACHMENT4),
            FramebufferAttachment::COLOR_ATTACHMENT5 => Some(OperatableBuffer::COLOR_ATTACHMENT5),
            FramebufferAttachment::COLOR_ATTACHMENT6 => Some(OperatableBuffer::COLOR_ATTACHMENT6),
            FramebufferAttachment::COLOR_ATTACHMENT7 => Some(OperatableBuffer::COLOR_ATTACHMENT7),
            FramebufferAttachment::COLOR_ATTACHMENT8 => Some(OperatableBuffer::COLOR_ATTACHMENT8),
            FramebufferAttachment::COLOR_ATTACHMENT9 => Some(OperatableBuffer::COLOR_ATTACHMENT9),
            FramebufferAttachment::COLOR_ATTACHMENT10 => Some(OperatableBuffer::COLOR_ATTACHMENT10),
            FramebufferAttachment::COLOR_ATTACHMENT11 => Some(OperatableBuffer::COLOR_ATTACHMENT11),
            FramebufferAttachment::COLOR_ATTACHMENT12 => Some(OperatableBuffer::COLOR_ATTACHMENT12),
            FramebufferAttachment::COLOR_ATTACHMENT13 => Some(OperatableBuffer::COLOR_ATTACHMENT13),
            FramebufferAttachment::COLOR_ATTACHMENT14 => Some(OperatableBuffer::COLOR_ATTACHMENT14),
            FramebufferAttachment::COLOR_ATTACHMENT15 => Some(OperatableBuffer::COLOR_ATTACHMENT15),
            FramebufferAttachment::DEPTH_ATTACHMENT => None,
            FramebufferAttachment::STENCIL_ATTACHMENT => None,
            FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT => None,
        }
    }

    #[inline]
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

    #[inline]
    fn default_clear_policy(&self) -> &ClearPolicy {
        match self {
            FramebufferAttachment::COLOR_ATTACHMENT0
            | FramebufferAttachment::COLOR_ATTACHMENT1
            | FramebufferAttachment::COLOR_ATTACHMENT2
            | FramebufferAttachment::COLOR_ATTACHMENT3
            | FramebufferAttachment::COLOR_ATTACHMENT4
            | FramebufferAttachment::COLOR_ATTACHMENT5
            | FramebufferAttachment::COLOR_ATTACHMENT6
            | FramebufferAttachment::COLOR_ATTACHMENT7
            | FramebufferAttachment::COLOR_ATTACHMENT8
            | FramebufferAttachment::COLOR_ATTACHMENT9
            | FramebufferAttachment::COLOR_ATTACHMENT10
            | FramebufferAttachment::COLOR_ATTACHMENT11
            | FramebufferAttachment::COLOR_ATTACHMENT12
            | FramebufferAttachment::COLOR_ATTACHMENT13
            | FramebufferAttachment::COLOR_ATTACHMENT14
            | FramebufferAttachment::COLOR_ATTACHMENT15 => {
                &DEFAULT_CLEAR_POLICY_FOR_COLOR_ATTACHMENT
            }
            FramebufferAttachment::DEPTH_ATTACHMENT => &DEFAULT_CLEAR_POLICY_FOR_DEPTH_ATTACHMENT,
            FramebufferAttachment::STENCIL_ATTACHMENT => {
                &DEFAULT_CLEAR_POLICY_FOR_STENCIL_ATTACHMENT
            }
            FramebufferAttachment::DEPTH_STENCIL_ATTACHMENT => {
                &DEFAULT_CLEAR_POLICY_FOR_DEPTH_STENCIL_ATTACHMENT
            }
        }
    }
}

/// Available drawable or readable buffer attachment mapped from [`WebGl2RenderingContext`].
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperatableBuffer {
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

#[derive(Debug, Clone, PartialEq)]
pub enum AttachmentProvider {
    FromNewTexture {
        internal_format: TextureColorFormat,
        clear_policy: Option<ClearPolicy>,
    },
    FromExistingTexture {
        texture: WebGlTexture,
        clear_policy: Option<ClearPolicy>,
    },
    FromNewRenderbuffer {
        internal_format: RenderbufferInternalFormat,
        clear_policy: Option<ClearPolicy>,
    },
    FromExistingRenderbuffer {
        renderbuffer: WebGlRenderbuffer,
        clear_policy: Option<ClearPolicy>,
    },
}

impl AttachmentProvider {
    pub fn new_texture(internal_format: TextureColorFormat) -> Self {
        Self::FromNewTexture {
            internal_format,
            clear_policy: None,
        }
    }

    pub fn new_renderbuffer(internal_format: RenderbufferInternalFormat) -> Self {
        Self::FromNewRenderbuffer {
            internal_format,
            clear_policy: None,
        }
    }

    pub fn from_texture(texture: WebGlTexture) -> Self {
        Self::FromExistingTexture {
            texture,
            clear_policy: None,
        }
    }

    pub fn from_renderbuffer(renderbuffer: WebGlRenderbuffer) -> Self {
        Self::FromExistingRenderbuffer {
            renderbuffer,
            clear_policy: None,
        }
    }

    pub fn new_texture_with_clear_policy(
        internal_format: TextureColorFormat,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromNewTexture {
            internal_format,
            clear_policy: Some(clear_policy),
        }
    }

    pub fn new_renderbuffer_with_clear_policy(
        internal_format: RenderbufferInternalFormat,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromNewRenderbuffer {
            internal_format,
            clear_policy: Some(clear_policy),
        }
    }

    pub fn from_texture_with_clear_policy(
        texture: WebGlTexture,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromExistingTexture {
            texture,
            clear_policy: Some(clear_policy),
        }
    }

    pub fn from_renderbuffer_with_clear_policy(
        renderbuffer: WebGlRenderbuffer,
        clear_policy: ClearPolicy,
    ) -> Self {
        Self::FromExistingRenderbuffer {
            renderbuffer,
            clear_policy: Some(clear_policy),
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
                clear_policy,
            } => {
                let current_texture = texture_binding_2d(gl);

                let texture = gl.create_texture().ok_or(Error::CreateTextureFailure)?;
                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
                gl.tex_storage_2d(
                    WebGl2RenderingContext::TEXTURE_2D,
                    1,
                    internal_format.gl_enum(),
                    width,
                    height,
                );
                gl.framebuffer_texture_2d(
                    target.gl_enum(),
                    attachment.gl_enum(),
                    WebGl2RenderingContext::TEXTURE_2D,
                    Some(&texture),
                    0,
                );

                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, current_texture.as_ref());

                Attach::Texture {
                    texture,
                    clear_policy: *clear_policy,
                    owned: true,
                }
            }
            AttachmentProvider::FromExistingTexture {
                texture,
                clear_policy,
            } => {
                let current_texture = texture_binding_2d(gl);

                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
                gl.framebuffer_texture_2d(
                    target.gl_enum(),
                    attachment.gl_enum(),
                    WebGl2RenderingContext::TEXTURE_2D,
                    Some(&texture),
                    0,
                );

                gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, current_texture.as_ref());

                Attach::Texture {
                    texture: texture.clone(),
                    clear_policy: *clear_policy,
                    owned: false,
                }
            }
            AttachmentProvider::FromNewRenderbuffer {
                internal_format,
                clear_policy,
            } => {
                let current_renderbuffer = renderbuffer_binding(gl);

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

                gl.bind_renderbuffer(
                    WebGl2RenderingContext::RENDERBUFFER,
                    current_renderbuffer.as_ref(),
                );

                Attach::Renderbuffer {
                    renderbuffer,
                    clear_policy: *clear_policy,
                    owned: true,
                }
            }
            AttachmentProvider::FromExistingRenderbuffer {
                renderbuffer,
                clear_policy,
            } => {
                let current_renderbuffer = renderbuffer_binding(gl);

                gl.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(renderbuffer));
                gl.framebuffer_renderbuffer(
                    target.gl_enum(),
                    attachment.gl_enum(),
                    WebGl2RenderingContext::RENDERBUFFER,
                    Some(renderbuffer),
                );

                gl.bind_renderbuffer(
                    WebGl2RenderingContext::RENDERBUFFER,
                    current_renderbuffer.as_ref(),
                );

                Attach::Renderbuffer {
                    renderbuffer: renderbuffer.clone(),
                    clear_policy: *clear_policy,
                    owned: false,
                }
            }
        };

        Ok(attach)
    }
}

enum Attach {
    Texture {
        texture: WebGlTexture,
        clear_policy: Option<ClearPolicy>,
        owned: bool,
    },
    Renderbuffer {
        renderbuffer: WebGlRenderbuffer,
        clear_policy: Option<ClearPolicy>,
        owned: bool,
    },
}

impl Attach {
    #[inline]
    fn clear_policy(&self) -> Option<&ClearPolicy> {
        match self {
            Attach::Texture { clear_policy, .. } | Attach::Renderbuffer { clear_policy, .. } => {
                clear_policy.as_ref()
            }
        }
    }

    #[inline]
    fn is_owned(&self) -> bool {
        match self {
            Attach::Texture { owned, .. } => *owned,
            Attach::Renderbuffer { owned, .. } => *owned,
        }
    }

    fn delete(
        self,
        gl: &WebGl2RenderingContext,
        target: FramebufferTarget,
        attachment: FramebufferAttachment,
    ) {
        if self.is_owned() {
            match self {
                Attach::Texture { texture, .. } => {
                    gl.framebuffer_texture_2d(
                        target.gl_enum(),
                        attachment.gl_enum(),
                        WebGl2RenderingContext::TEXTURE_2D,
                        None,
                        0,
                    );
                    gl.delete_texture(Some(&texture));
                }
                Attach::Renderbuffer { renderbuffer, .. } => {
                    gl.framebuffer_renderbuffer(
                        target.gl_enum(),
                        attachment.gl_enum(),
                        WebGl2RenderingContext::RENDERBUFFER,
                        None,
                    );
                    gl.delete_renderbuffer(Some(&renderbuffer));
                }
            }
        }
    }
}

struct Bound {
    target: FramebufferTarget,
    read_buffer: Option<OperatableBuffer>,
    draw_buffers: Option<Array>,
}

struct Runtime {
    width: i32,
    height: i32,
    framebuffer: WebGlFramebuffer,
    attaches: HashMap<FramebufferAttachment, Attach>,
    removes: HashSet<FramebufferAttachment>,
    bound: Option<Bound>,
}

pub struct Framebuffer {
    gl: WebGl2RenderingContext,
    size_policy: SizePolicy,

    providers: HashMap<FramebufferAttachment, AttachmentProvider>,
    read_buffer: u32,
    draw_buffers: Array,
    adds: HashSet<FramebufferAttachment>,
    renderbuffer_samples: Option<i32>,

    runtime: Option<Runtime>,
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        self.unbind();
        if let Some(runtime) = self.runtime.take() {
            self.gl.delete_framebuffer(Some(&runtime.framebuffer));
            runtime.attaches.iter().for_each(|(_, attach)| {
                if attach.is_owned() {
                    match attach {
                        Attach::Texture { texture, .. } => self.gl.delete_texture(Some(texture)),
                        Attach::Renderbuffer { renderbuffer, .. } => {
                            self.gl.delete_renderbuffer(Some(renderbuffer))
                        }
                    }
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
        let mut adds = HashSet::new();
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
            adds.insert(attachment);
        }
        draw_buffers.sort();
        // takes the first attachment from draw buffers as default read buffer. uses NONE if no draw buffer.
        let read_buffer = draw_buffers
            .get(0)
            .as_f64()
            .map(|v| v as u32)
            .unwrap_or(WebGl2RenderingContext::NONE);

        let renderbuffer_samples = match renderbuffer_samples {
            Some(samples) => {
                if samples == 0 {
                    None
                } else {
                    Some(samples)
                }
            }
            None => None,
        };

        Self {
            gl,
            size_policy,

            providers: ps,
            read_buffer,
            draw_buffers,
            adds,
            renderbuffer_samples,

            runtime: None,
        }
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn clear_buffers(&self) -> Result<(), Error> {
        let Some(runtime) = self.runtime.as_ref() else {
            return Err(Error::FramebufferUninitialized);
        };
        if runtime.bound.is_none() {
            return Err(Error::FramebufferUnbound);
        }

        runtime.attaches.iter().for_each(|(attachment, attach)| {
            attach
                .clear_policy()
                .unwrap_or(attachment.default_clear_policy())
                .clear(&self.gl, attachment.to_draw_buffer_index());
        });

        Ok(())
    }

    pub fn clear_buffers_of_attachments<I>(&self, attachments: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = FramebufferAttachment>,
    {
        let Some(runtime) = self.runtime.as_ref() else {
            return Err(Error::FramebufferUninitialized);
        };
        if runtime.bound.is_none() {
            return Err(Error::FramebufferUnbound);
        }

        for attachment in attachments {
            if let Some(attach) = runtime.attaches.get(&attachment) {
                attach
                    .clear_policy()
                    .unwrap_or(attachment.default_clear_policy())
                    .clear(&self.gl, attachment.to_draw_buffer_index());
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
                    removes: HashSet::new(),
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

        self.gl
            .bind_framebuffer(target.gl_enum(), Some(&runtime.framebuffer));

        if width != runtime.width || height != runtime.height {
            for (attachment, attach) in runtime.attaches.drain() {
                attach.delete(&self.gl, target, attachment);
                self.adds.insert(attachment);
            }
            runtime.width = width;
            runtime.height = height;
            runtime.removes.clear();
        }

        for attachment in runtime.removes.drain() {
            let Some(attach) = runtime.attaches.remove(&attachment) else {
                continue;
            };
            attach.delete(&self.gl, target, attachment);
        }

        for attachment in self.adds.drain() {
            let Some(provider) = self.providers.get(&attachment) else {
                continue;
            };
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

        // binds draw buffers and read buffer to default.
        if target == FramebufferTarget::DRAW_FRAMEBUFFER || target == FramebufferTarget::FRAMEBUFFER
        {
            self.gl.draw_buffers(&self.draw_buffers);
        }
        if target == FramebufferTarget::READ_FRAMEBUFFER || target == FramebufferTarget::FRAMEBUFFER
        {
            self.gl.read_buffer(self.read_buffer);
        }
        runtime.bound = Some(Bound {
            target,
            read_buffer: None,
            draw_buffers: None,
        });

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

        // resets draw buffers and read buffer to default if developer manually changes them ever.
        if bound.draw_buffers.take().is_some() {
            self.gl.draw_buffers(&self.draw_buffers);
        }
        if bound.read_buffer.take().is_some() {
            self.gl.read_buffer(self.read_buffer);
        }

        self.gl.bind_framebuffer(bound.target.gl_enum(), None);
    }

    pub fn set_read_buffer(&mut self, read_buffer: OperatableBuffer) -> Result<(), Error> {
        let Some(runtime) = self.runtime.as_mut() else {
            return Err(Error::FramebufferUninitialized);
        };
        let Some(bound) = runtime.bound.as_mut() else {
            return Err(Error::FramebufferUnbound);
        };
        if bound.target != FramebufferTarget::READ_FRAMEBUFFER
            && bound.target != FramebufferTarget::FRAMEBUFFER
        {
            return Err(Error::FramebufferUnbound);
        }

        self.gl.read_buffer(read_buffer.gl_enum());
        bound.read_buffer = Some(read_buffer);

        Ok(())
    }

    pub fn set_draw_buffers<I>(&mut self, draw_buffers: I) -> Result<(), Error>
    where
        I: IntoIterator<Item = OperatableBuffer>,
    {
        let Some(runtime) = self.runtime.as_mut() else {
            return Err(Error::FramebufferUninitialized);
        };
        let Some(bound) = runtime.bound.as_mut() else {
            return Err(Error::FramebufferUnbound);
        };
        if bound.target != FramebufferTarget::DRAW_FRAMEBUFFER
            && bound.target != FramebufferTarget::FRAMEBUFFER
        {
            return Err(Error::FramebufferUnbound);
        }

        let draw_buffers = Array::from_iter(
            draw_buffers
                .into_iter()
                .map(|b| JsValue::from_f64(b.gl_enum() as f64)),
        );
        self.gl.draw_buffers(&draw_buffers);
        bound.draw_buffers = Some(draw_buffers);

        Ok(())
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
        let Some(runtime) = self.runtime.as_ref() else {
            return Err(Error::FramebufferUninitialized);
        };
        let Some(bound) = runtime.bound.as_ref() else {
            return Err(Error::FramebufferUnbound);
        };
        if bound.target != FramebufferTarget::READ_FRAMEBUFFER
            && bound.target != FramebufferTarget::FRAMEBUFFER
        {
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
    }

    /// Returns framebuffer width.
    pub fn width(&self) -> Option<i32> {
        self.runtime.as_ref().map(|runtime| runtime.width)
    }

    /// Returns framebuffer height.
    pub fn height(&self) -> Option<i32> {
        self.runtime.as_ref().map(|runtime| runtime.height)
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
            .and_then(|runtime| {
                if runtime.removes.contains(&attachment) {
                    None
                } else {
                    runtime.attaches.get(&attachment)
                }
            })
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
            .and_then(|runtime| {
                if runtime.removes.contains(&attachment) {
                    None
                } else {
                    runtime.attaches.get(&attachment)
                }
            })
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

            for (attachment, attach) in &runtime.attaches {
                if let Attach::Renderbuffer { .. } = attach {
                    runtime.removes.insert(*attachment);
                    self.adds.insert(*attachment);
                }
            }
        }

        Ok(())
    }

    pub fn set_attachment(
        &mut self,
        attachment: FramebufferAttachment,
        provider: Option<AttachmentProvider>,
    ) -> Result<(), Error> {
        let Some(runtime) = self.runtime.as_mut() else {
            return Ok(());
        };
        if let Some(bound) = runtime.bound.as_ref() {
            return Err(Error::FramebufferBinding(bound.target));
        }

        let current = self.providers.get(&attachment);
        if current == provider.as_ref() {
            return Ok(());
        }

        match provider {
            Some(provider) => {
                self.providers.insert(attachment, provider);
                runtime.removes.insert(attachment);
                self.adds.insert(attachment);

                if let Some(draw_buffer) = attachment.to_draw_buffer() {
                    let draw_buffer = JsValue::from_f64(draw_buffer.gl_enum() as f64);
                    if !self.draw_buffers.includes(&draw_buffer, 0) {
                        self.draw_buffers.push(&draw_buffer);
                        self.draw_buffers.sort();
                    }
                }
            }
            None => {
                self.providers.remove(&attachment);
                runtime.removes.insert(attachment);

                if let Some(draw_buffer) = attachment.to_draw_buffer() {
                    let draw_buffer = draw_buffer.gl_enum() as u32;
                    self.draw_buffers = self
                        .draw_buffers
                        .filter(&mut |value, _, _| value.as_f64().unwrap() as u32 != draw_buffer);
                }
            }
        }

        Ok(())
    }
}

pub struct FramebufferBuilder {
    size_policy: SizePolicy,
    providers: HashMap<FramebufferAttachment, AttachmentProvider>,
    adds: HashSet<FramebufferAttachment>,
    draw_buffers: HashSet<OperatableBuffer>,
    renderbuffer_samples: Option<i32>,
}

impl FramebufferBuilder {
    pub fn new() -> Self {
        Self {
            size_policy: SizePolicy::FollowDrawingBuffer,
            providers: HashMap::new(),
            adds: HashSet::new(),
            draw_buffers: HashSet::new(),
            renderbuffer_samples: None,
        }
    }

    pub fn with_size_policy(mut self, size_policy: SizePolicy) -> Self {
        self.size_policy = size_policy;
        self
    }

    pub fn with_samples(mut self, samples: i32) -> Self {
        let samples = if samples == 0 { None } else { Some(samples) };
        self.renderbuffer_samples = samples;
        self
    }

    pub fn without_samples(mut self) -> Self {
        self.renderbuffer_samples = None;
        self
    }

    pub fn build(self, gl: WebGl2RenderingContext) -> Framebuffer {
        let draw_buffers = Array::from_iter(
            self.draw_buffers
                .into_iter()
                .map(|v| JsValue::from_f64(v.gl_enum() as f64)),
        );
        draw_buffers.sort();
        // takes the first one of draw buffers as default read buffer. uses NONE if no draw buffer.
        let read_buffer = draw_buffers
            .get(0)
            .as_f64()
            .map(|v| v as u32)
            .unwrap_or(WebGl2RenderingContext::NONE);

        Framebuffer {
            gl,
            size_policy: self.size_policy,
            providers: self.providers,
            read_buffer,
            draw_buffers,
            renderbuffer_samples: self.renderbuffer_samples,
            adds: self.adds,

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
                    self.draw_buffers.insert(draw_buffer);
                }
                self.adds.insert($attachment);
                self
            }

            pub fn $without_func(mut self) -> Self {
                self.providers.remove(&$attachment);
                if let Some(draw_buffer) = $attachment.to_draw_buffer() {
                    self.draw_buffers.remove(&draw_buffer);
                }
                self.adds.remove(&$attachment);
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
