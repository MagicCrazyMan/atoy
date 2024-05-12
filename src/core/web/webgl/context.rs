use web_sys::WebGl2RenderingContext;

use super::{buffer::BufferRegistry, framebuffer::FramebufferRegistry, texture::TextureRegistry};

#[derive(Debug)]
pub struct Context {
    gl: WebGl2RenderingContext,
    buffer_registry: BufferRegistry,
    texture_registry: TextureRegistry,
    framebuffer_registry: FramebufferRegistry,
    // uniform_buffer_objects: HashMap<usize, Buffer>,
}

impl Context {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        let buffer_registry = BufferRegistry::new(gl.clone());
        let texture_registry = TextureRegistry::new(gl.clone(), buffer_registry.clone());
        let framebuffer_registry = FramebufferRegistry::new(
            gl.clone(),
            buffer_registry.clone(),
            texture_registry.clone(),
        );
        Self {
            buffer_registry,
            texture_registry,
            framebuffer_registry,
            // uniform_buffer_objects: HashMap::new(),
            gl,
        }
    }

    pub fn gl(&self) -> &WebGl2RenderingContext {
        &self.gl
    }

    pub fn buffer_registry(&self) -> &BufferRegistry {
        &self.buffer_registry
    }

    pub fn texture_registry(&self) -> &TextureRegistry {
        &self.texture_registry
    }

    pub fn framebuffer_registry(&self) -> &FramebufferRegistry {
        &self.framebuffer_registry
    }

    // pub fn mount_uniform_buffer_object(
    //     &mut self,
    //     buffer: Buffer,
    //     mount_point: usize,
    //     range: Option<Range<usize>>,
    // ) -> Result<(), Error> {
    //     if let Some(buffer) = self.uniform_buffer_objects.get(&mount_point) {
    //         if buffer.id() == buffer.id() {
    //             return Ok(());
    //         } else {
    //             return Err(Error::UniformBufferObjectMountPointOccupied(mount_point));
    //         }
    //     }

    //     self.buffer_registry.register(&buffer)?;
    //     let gl_buffer = buffer.gl_buffer().unwrap();
    //     self.gl
    //         .bind_buffer(BufferTarget::UniformBuffer.to_gl_enum(), Some(&gl_buffer));
    //     match range {
    //         Some(range) => {
    //             let offset = range.start as i32;
    //             let size = (range.end - range.start) as i32;
    //             self.gl.bind_buffer_range_with_i32_and_i32(
    //                 BufferTarget::UniformBuffer.to_gl_enum(),
    //                 mount_point as u32,
    //                 Some(&gl_buffer),
    //                 offset,
    //                 size,
    //             );
    //         }
    //         None => {
    //             self.gl.bind_buffer_base(
    //                 BufferTarget::UniformBuffer.to_gl_enum(),
    //                 mount_point as u32,
    //                 Some(&gl_buffer),
    //             );
    //         }
    //     };
    //     self.gl
    //         .bind_buffer(BufferTarget::UniformBuffer.to_gl_enum(), None);

    //     self.uniform_buffer_objects
    //         .insert(mount_point, buffer.clone());

    //     Ok(())
    // }

    // pub fn unmount_uniform_buffer_object(&mut self, mount_point: usize) -> Result<(), Error> {
    //     let Some(_) = self.uniform_buffer_objects.remove(&mount_point) else {
    //         return Ok(());
    //     };

    //     self.gl.bind_buffer_base(
    //         BufferTarget::UniformBuffer.to_gl_enum(),
    //         mount_point as u32,
    //         None,
    //     );

    //     Ok(())
    // }
}
