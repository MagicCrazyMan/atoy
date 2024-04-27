use std::ops::Range;

use hashbrown::HashMap;
use web_sys::WebGl2RenderingContext;

use super::{
    buffer::{Buffer, BufferRepository, BufferTarget},
    conversion::ToGlEnum,
    error::Error,
    texture::TextureRepository,
};

pub struct Context {
    gl: WebGl2RenderingContext,
    buffer_repository: BufferRepository,
    texture_repository: TextureRepository,

    uniform_buffer_objects: HashMap<usize, Buffer>,
}

impl Context {
    pub fn new(gl: WebGl2RenderingContext) -> Self {
        Self {
            buffer_repository: BufferRepository::new(gl.clone()),
            texture_repository: TextureRepository::new(gl.clone()),
            uniform_buffer_objects: HashMap::new(),
            gl,
        }
    }

    pub fn mount_uniform_buffer_object(
        &mut self,
        buffer: Buffer,
        mount_point: usize,
        range: Option<Range<usize>>,
    ) -> Result<(), Error> {
        if let Some(buffer) = self.uniform_buffer_objects.get(&mount_point) {
            if buffer.id() == buffer.id() {
                buffer.flush()?;
                return Ok(());
            } else {
                return Err(Error::UniformBufferObjectMountPointOccupied(mount_point));
            }
        }

        self.buffer_repository.register(&buffer)?;
        let gl_buffer = buffer.gl_buffer().unwrap();
        self.gl
            .bind_buffer(BufferTarget::UniformBuffer.gl_enum(), Some(&gl_buffer));
        match range {
            Some(range) => {
                let offset = range.start as i32;
                let size = (range.end - range.start) as i32;
                self.gl.bind_buffer_range_with_i32_and_i32(
                    BufferTarget::UniformBuffer.gl_enum(),
                    mount_point as u32,
                    Some(&gl_buffer),
                    offset,
                    size,
                );
            }
            None => {
                self.gl.bind_buffer_base(
                    BufferTarget::UniformBuffer.gl_enum(),
                    mount_point as u32,
                    Some(&gl_buffer),
                );
            }
        };
        self.gl
            .bind_buffer(BufferTarget::UniformBuffer.gl_enum(), None);

        self.uniform_buffer_objects
            .insert(mount_point, buffer.clone());

        Ok(())
    }

    pub fn unmount_uniform_buffer_object(&mut self, mount_point: usize) -> Result<(), Error> {
        let Some(_) = self.uniform_buffer_objects.remove(&mount_point) else {
            return Ok(());
        };

        self.gl.bind_buffer_base(
            BufferTarget::UniformBuffer.gl_enum(),
            mount_point as u32,
            None,
        );

        Ok(())
    }
}
