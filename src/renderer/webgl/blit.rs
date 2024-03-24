use web_sys::WebGl2RenderingContext;

use super::{
    conversion::ToGlEnum,
    error::Error,
    framebuffer::{Framebuffer, FramebufferTarget, OperableBuffer},
};

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

pub struct Blit<'a> {
    gl: &'a WebGl2RenderingContext,
    read: &'a mut Framebuffer,
    read_buffer: Option<OperableBuffer>,
    draw: &'a mut Framebuffer,
    draw_buffers: Option<Vec<OperableBuffer>>,
    mask: BlitMask,
    filter: BlitFlilter,
    src_x0: Option<usize>,
    src_y0: Option<usize>,
    src_x1: Option<usize>,
    src_y1: Option<usize>,
    dst_x0: Option<usize>,
    dst_y0: Option<usize>,
    dst_x1: Option<usize>,
    dst_y1: Option<usize>,
}

impl<'a> Blit<'a> {
    pub fn new(
        gl: &'a WebGl2RenderingContext,
        read: &'a mut Framebuffer,
        draw: &'a mut Framebuffer,
        mask: BlitMask,
        filter: BlitFlilter,
    ) -> Self {
        Self {
            gl,
            read,
            read_buffer: None,
            draw,
            draw_buffers: None,
            mask,
            filter,
            src_x0: None,
            src_y0: None,
            src_x1: None,
            src_y1: None,
            dst_x0: None,
            dst_y0: None,
            dst_x1: None,
            dst_y1: None,
        }
    }

    pub fn with_buffers(
        gl: &'a WebGl2RenderingContext,
        read: &'a mut Framebuffer,
        read_buffer: OperableBuffer,
        draw: &'a mut Framebuffer,
        draw_buffers: Vec<OperableBuffer>,
        mask: BlitMask,
        filter: BlitFlilter,
    ) -> Self {
        Self {
            gl,
            read,
            read_buffer: Some(read_buffer),
            draw,
            draw_buffers: Some(draw_buffers),
            mask,
            filter,
            src_x0: None,
            src_y0: None,
            src_x1: None,
            src_y1: None,
            dst_x0: None,
            dst_y0: None,
            dst_x1: None,
            dst_y1: None,
        }
    }

    pub fn with_params(
        gl: &'a WebGl2RenderingContext,
        read: &'a mut Framebuffer,
        read_buffer: Option<OperableBuffer>,
        draw: &'a mut Framebuffer,
        draw_buffers: Option<Vec<OperableBuffer>>,
        mask: BlitMask,
        filter: BlitFlilter,
        src_x0: Option<usize>,
        src_y0: Option<usize>,
        src_x1: Option<usize>,
        src_y1: Option<usize>,
        dst_x0: Option<usize>,
        dst_y0: Option<usize>,
        dst_x1: Option<usize>,
        dst_y1: Option<usize>,
    ) -> Self {
        Self {
            gl,
            read,
            read_buffer,
            draw,
            draw_buffers,
            mask,
            filter,
            src_x0,
            src_y0,
            src_x1,
            src_y1,
            dst_x0,
            dst_y0,
            dst_x1,
            dst_y1,
        }
    }

    pub fn blit(&mut self) -> Result<(), Error> {
        self.read.bind(FramebufferTarget::READ_FRAMEBUFFER)?;
        if let Some(read_buffer) = self.read_buffer.as_ref() {
            self.read.set_read_buffer(*read_buffer)?;
        }

        self.draw.bind(FramebufferTarget::DRAW_FRAMEBUFFER)?;
        if let Some(draw_buffers) = self.draw_buffers.as_ref() {
            self.draw
                .set_draw_buffers(draw_buffers.iter().map(|b| *b))?;
        }

        let src_x0 = self.src_x0.unwrap_or(0) as i32;
        let src_y0 = self.src_y0.unwrap_or(0) as i32;
        let src_x1 = self.src_x1.or_else(|| self.read.width()).unwrap_or(0) as i32;
        let src_y1 = self.src_y1.or_else(|| self.read.height()).unwrap_or(0) as i32;
        let dst_x0 = self.dst_x0.unwrap_or(0) as i32;
        let dst_y0 = self.dst_y0.unwrap_or(0) as i32;
        let dst_x1 = self.dst_x1.or_else(|| self.draw.width()).unwrap_or(0) as i32;
        let dst_y1 = self.dst_y1.or_else(|| self.draw.height()).unwrap_or(0) as i32;
        let mask = self.mask.gl_enum();
        let filter = self.filter.gl_enum();
        self.gl.blit_framebuffer(
            src_x0, src_y0, src_x1, src_y1, dst_x0, dst_y0, dst_x1, dst_y1, mask, filter,
        );

        self.read.unbind(FramebufferTarget::READ_FRAMEBUFFER)?;
        self.draw.unbind(FramebufferTarget::DRAW_FRAMEBUFFER)?;

        Ok(())
    }
}
