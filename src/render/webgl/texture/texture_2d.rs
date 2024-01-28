use std::cell::Ref;

use crate::render::webgl::buffer::BufferSource;

use super::{Runtime, TextureInternalFormat, TextureSource};

pub trait Restorer<T> {
    fn levels(&self) -> T;


}

pub enum MemoryPolicy<T> {
    Unfree,
    Restorable(Box<dyn Restorer<T>>),
}

struct QueueItem {
    source: BufferSource,
    level: usize,
    x_offset: usize,
    y_offset: usize,
}

pub struct Texture2D {
    width: usize,
    height: usize,
    internal_format: TextureInternalFormat,
    memory_policy: MemoryPolicy<TextureSource>,
    queue: Vec<QueueItem>,
    generate_mipmap: Option<TextureSource>,

    runtime: Option<Box<Runtime<Texture2D>>>,
}

// macro_rules! texture_2d {
//     ($(($name:ident, $f:ident, $r:ident, $s:ident))+) => {
//         $(
//             pub struct $name {
//                 width: usize,
//                 height: usize,
//                 internal_format: $f,
//                 memory_policy: MemoryPolicy<$r>,
//                 queue: Vec<($s, usize, usize, usize)>,
//             }

//             impl TextureDescriptorInner<$name> {
//                 fn max_mipmap_level(&self) -> usize {
//                     if !self.generate_mipmap {
//                         return 0;
//                     }

//                     (self.layout.width as f64)
//                         .max(self.layout.height as f64)
//                         .log2()
//                         .floor() as usize
//                 }

//                 fn width_of_level(&self, level: usize) -> Option<usize> {
//                     let max_level = self.max_mipmap_level();
//                     if level > max_level {
//                         return None;
//                     }

//                     Some((self.layout.width >> level).max(1))
//                 }

//                 fn height_of_level(&self, level: usize) -> Option<usize> {
//                     let max_level = self.max_mipmap_level();
//                     if level > max_level {
//                         return None;
//                     }

//                     Some((self.layout.height >> level).max(1))
//                 }

//                 fn bytes_length(&self) -> usize {
//                     // estimates used memory of all levels
//                     let mut used_memory = 0;
//                     for level in 0..=self.max_mipmap_level() {
//                         let width = self.width_of_level(level).unwrap();
//                         let height = self.height_of_level(level).unwrap();
//                         used_memory += self.layout.internal_format.bytes_length(width, height);
//                     }
//                     used_memory
//                 }

//                 fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
//                     let Some(width) = self.width_of_level(level) else {
//                         return None;
//                     };
//                     let Some(height) = self.height_of_level(level) else {
//                         return None;
//                     };

//                     Some(self.layout.internal_format.bytes_length(width, height))
//                 }

//                 fn verify_tex_image_level_size(
//                     &self,
//                     level: usize,
//                     width: usize,
//                     height: usize,
//                 ) -> Result<(), Error> {
//                     if self
//                         .width_of_level(level)
//                         .map(|w| w != width)
//                         .unwrap_or(true)
//                     {
//                         return Err(Error::TextureSizeMismatched);
//                     }
//                     if self
//                         .height_of_level(level)
//                         .map(|h| h != height)
//                         .unwrap_or(true)
//                     {
//                         return Err(Error::TextureSizeMismatched);
//                     }

//                     Ok(())
//                 }

//                 fn verify_tex_sub_image_level_size(
//                     &self,
//                     level: usize,
//                     width: usize,
//                     height: usize,
//                     x_offset: usize,
//                     y_offset: usize,
//                 ) -> Result<(), Error> {
//                     if self
//                         .width_of_level(level)
//                         .map(|w| width + x_offset > w)
//                         .unwrap_or(true)
//                     {
//                         return Err(Error::TextureSizeMismatched);
//                     }
//                     if self
//                         .height_of_level(level)
//                         .map(|h| height + y_offset > h)
//                         .unwrap_or(true)
//                     {
//                         return Err(Error::TextureSizeMismatched);
//                     }

//                     Ok(())
//                 }
//             }

//             impl TextureDescriptor<$name> {
//                 pub fn internal_format(&self) -> $f {
//                     self.0.borrow().layout.internal_format
//                 }

//                 pub fn memory_policy(&self) -> Ref<MemoryPolicy<$r>> {
//                     Ref::map(self.0.borrow(), |inner| &inner.layout.memory_policy)
//                 }

//                 pub fn width(&self) -> usize {
//                     self.0.borrow().layout.width
//                 }

//                 pub fn height(&self) -> usize {
//                     self.0.borrow().layout.height
//                 }

//                 pub fn max_mipmap_level(&self) -> usize {
//                     self.0.borrow().max_mipmap_level()
//                 }

//                 pub fn width_of_level(&self, level: usize) -> Option<usize> {
//                     self.0.borrow().width_of_level(level)
//                 }

//                 pub fn height_of_level(&self, level: usize) -> Option<usize> {
//                     self.0.borrow().height_of_level(level)
//                 }

//                 pub fn bytes_length(&self) -> usize {
//                     self.0.borrow().bytes_length()
//                 }

//                 pub fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
//                     self.0.borrow().bytes_length_of_level(level)
//                 }
//             }
//         )+
//     };
// }

// texture_2d! {
//     (Texture2D, TextureInternalFormat, Restorer, TextureSource)
//     (Texture2DCompressed, TextureCompressedFormat, RestorerCompressed, TextureSourceCompressed)
// }
