use std::cell::{Ref, RefMut};

use super::{max_mipmap_level, Runtime, TextureDescriptor, TextureInternalFormat, TextureSource};

pub enum MipmapPolicy {
    /// Applies automatic mipmaps generation when creating textures.
    /// For creating a texture and generating mipmaps, a texture source in level 0 is required.
    /// Size from texture source is used as the size in level 0,
    /// but optional `width` and `height` are also available for overriding it.
    ///
    /// For defining mipmap levels, follows rules below:
    ///     - If `levels` is `None` or is `0`, max mipmap level is `floor(log2(max(width, height, 1)))`.
    ///     - If `levels` is any other value, max mipmap level is `min(max_level, floor(log2(max(width, height, 1))))`.
    ///
    /// For defining a mipmap generation range, follows rules below:
    ///     - If `generation_base_level` and `generation_max_level` are both `None`,
    ///     generate mimaps for all levels until stop.
    ///     - If `generation_base_level` has value, then the base level refers to `base_level = max(min(base_level, levels), 0)`
    ///     and range from `[base_level, levels]`.
    ///     - If `generation_max_level` has value, then the max level refers to `max_level = max(min(max_level, levels), 0)`
    ///     and range from `[0, max_level]`.
    ///     - If `generation_base_level` and `generation_max_level` have values both, range from the union set of range set upper.
    ///
    /// Developers could modify each mipmap level manually after
    /// generating mipmap by texing sub texture sources.
    AutoMipmap {
        width: Option<usize>,
        height: Option<usize>,
        levels: Option<usize>,
        generation_base_level: Option<usize>,
        generation_max_level: Option<usize>,
        base: TextureSource,
    },
    /// Creates texture only without generating mipmaps automatically.
    /// For creating a texture, `width` and `height` in level 0 are required.
    ///
    /// For defining mipmap levels, follows rules below:
    ///     - If `levels` is `None`, mipmaps are available and the max mipmap level is `floor(log2(max(width, height, 1)))`.
    ///     - If `levels` is `0`, no mipmaps are allowed.
    ///     - If `levels` is any other value, max mipmap level is `min(max_level, floor(log2(max(width, height, 1))))`.
    ///
    /// Developers could modify each mipmap level manually by texing sub texture sources as well.
    ManualMipmap {
        width: usize,
        height: usize,
        levels: Option<usize>,
    },
}

/// A WebGL 2d texture workload.
///
/// Texture is created in Immutable Storage using `texStorage2D`
/// and then image data are uploaded by `texSubImage2D`.
/// Once the texture is created, the memory layout is no more alterable,
/// meaning that `texImage2D` and `texStorage2D` are no longer work.
/// But developer could still modify image data using `texSubImage2D`.
/// You have to create a new texture if you want to allocate a new texture with different layout.
pub struct Texture2D {
    width: usize,
    height: usize,
    levels: usize,
    internal_format: TextureInternalFormat,
    mipmap_policy: MipmapPolicy,
    // memory_policy: MemoryPolicy,
    subs: Vec<(TextureSource, Option<usize>, Option<usize>)>,

    runtime: Option<Box<Runtime<Texture2D>>>,
}

impl Texture2D {
    /// Returns max mipmap level.
    /// Returning `None` means mipmap is disabled,
    /// while returning `0` means texture size reaches the maximum level already, but not disabled.
    fn max_mipmap_level(&self) -> Option<usize> {
        match &self.mipmap_policy {
            MipmapPolicy::AutoMipmap { base, .. } => {
                Some(max_mipmap_level(self.width, self.height))
            }
            MipmapPolicy::ManualMipmap {
                levels: max_level, ..
            } => match max_level {
                Some(max_level) => {
                    if *max_level == 0 {
                        None
                    } else {
                        Some((*max_level).min(max_mipmap_level(self.width, self.height)))
                    }
                }
                None => Some(max_mipmap_level(self.width, self.height)),
            },
        }
    }

    /// Returns width of a mipmap level.
    /// Returns base width in level 0.
    fn width_of_level(&self, level: usize) -> Option<usize> {
        if level == 0 {
            return Some(self.width);
        }
        let Some(max_level) = self.max_mipmap_level() else {
            return None;
        };
        if level > max_level {
            return None;
        }

        Some((self.width >> level).max(1))
    }

    /// Returns height of a mipmap level.
    /// Returns base height in level 0.
    fn height_of_level(&self, level: usize) -> Option<usize> {
        if level == 0 {
            return Some(self.height);
        }
        let Some(max_level) = self.max_mipmap_level() else {
            return None;
        };
        if level > max_level {
            return None;
        }

        Some((self.height >> level).max(1))
    }

    /// Returns bytes length of a mipmap level.
    fn bytes_length(&self) -> usize {
        // estimates used memory of all levels
        let mut used_memory = 0;
        for level in 0..=self.max_mipmap_level().unwrap_or(0) {
            let width = self.width_of_level(level).unwrap();
            let height = self.height_of_level(level).unwrap();
            used_memory += self.internal_format.bytes_length(width, height);
        }
        used_memory
    }

    fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
        let Some(width) = self.width_of_level(level) else {
            return None;
        };
        let Some(height) = self.height_of_level(level) else {
            return None;
        };

        Some(self.internal_format.bytes_length(width, height))
    }
}

impl TextureDescriptor<Texture2D> {
    pub fn texture(&self) -> Ref<'_, Texture2D> {
        self.0.borrow()
    }

    pub fn texture_mut(&self) -> RefMut<'_, Texture2D> {
        self.0.borrow_mut()
    }

    pub fn internal_format(&self) -> TextureInternalFormat {
        self.0.borrow().internal_format
    }

    pub fn width(&self) -> usize {
        self.0.borrow().width
    }

    pub fn height(&self) -> usize {
        self.0.borrow().height
    }

    pub fn max_mipmap_level(&self) -> Option<usize> {
        self.0.borrow().max_mipmap_level()
    }

    pub fn width_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().width_of_level(level)
    }

    pub fn height_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().height_of_level(level)
    }

    pub fn bytes_length(&self) -> usize {
        self.0.borrow().bytes_length()
    }

    pub fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
        self.0.borrow().bytes_length_of_level(level)
    }
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

pub enum MemoryPolicy {
    Unfree,
    Restorable,
}
