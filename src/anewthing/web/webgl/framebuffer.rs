use proc::GlEnum;
use smallvec::SmallVec;
use web_sys::WebGl2RenderingContext;

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

enum DepthStencilAttachment {
    Depth,
    Stencil,
    DepthAndStencil,
}

pub struct WebGlFramebufferItem {
    size_policy: SizePolicy,
    color_attachments: SmallVec<[WebGlFramebufferAttachTarget; 4]>,
    
}

impl WebGlFramebufferItem {
    // pub fn new()
}
