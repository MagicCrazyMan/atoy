use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::render::webgl::{capabilities::Capabilities, conversion::ToGlEnum, error::Error, utils};

use super::{
    max_available_mipmap_level, Runtime, TextureCompressedFormat, TextureCompressedSource,
    TextureDescriptor, TextureSource, TextureTarget, TextureUpload,
};

/// Construction policies telling texture store how to create a texture.
pub enum ConstructionPolicy {
    /// Simple texture creation procedure.
    ///
    /// Under this policy, texture store may takes the size of the base texture source as the size of texture in level 0.
    /// The max level of the texture is applied as `floor(log2(max(width, height, 1)))`.
    Simple {
        internal_format: TextureCompressedFormat,
        base: TextureCompressedSource,
    },
    /// Preallocates a texture only without uploading any image data.
    ///
    /// - Required `internal_format` defines the internal format.
    /// - Required `width` and `height` defines the size of texture in level 0.
    /// - Optional `max_level` defines the max mipmap level following rules:
    ///     - If `max_level` is `None`, mipmaps are available and the max mipmap level is `floor(log2(max(width, height, 1)))`.
    ///     - If `max_level` is `0`, no mipmaps are allowed.
    ///     - If `max_level` is any other value, max mipmap level is `min(max_level, floor(log2(max(width, height, 1))))`.
    Preallocate {
        internal_format: TextureCompressedFormat,
        width: usize,
        height: usize,
        max_level: Option<usize>,
    },
    /// Creates a texture with existing [`TextureUpload`] for each level.
    ///
    /// - Texture will first generate following the same procedure as [`ConstructPolicy::Preallocate`].
    /// - Required `uploads` defines texture source for uploading in each level.
    Full {
        internal_format: TextureCompressedFormat,
        width: usize,
        height: usize,
        max_level: Option<usize>,
        uploads: Vec<TextureUpload<TextureCompressedSource>>,
    },
}

/// A container provides content for restoring a texture.
pub struct Restore {
    uploads: Vec<TextureUpload<TextureCompressedSource>>,
}

/// Memory policies controlling how to manage memory of a texture.
pub enum MemoryPolicy {
    Unfree,
    Restorable(Box<dyn Fn() -> Restore>),
}

/// A WebGL 2d texture in compressed internal format workload.
///
/// No automatic mipmaps generation available for a compressed format.
pub struct Texture2DCompressed {
    width: usize,
    height: usize,
    max_level: Option<usize>,
    internal_format: TextureCompressedFormat,
    memory_policy: MemoryPolicy,
    uploads: Vec<TextureUpload<TextureCompressedSource>>,

    pub(super) runtime: Option<Box<Runtime>>,
}

impl Drop for Texture2DCompressed {
    fn drop(&mut self) {
        unsafe {
            if let Some(runtime) = self.runtime.take() {
                (*runtime.textures).remove(&runtime.id);
                (*runtime.lru).remove(runtime.lru_node);
                (*runtime.used_memory) -= runtime.bytes_length;
                runtime.gl.delete_texture(Some(&runtime.texture));
            }
        }
    }
}

impl Texture2DCompressed {
    /// Returns texture base width in level 0.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns texture base height in level 0.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns [`TextureCompressedFormat`].
    pub fn internal_format(&self) -> TextureCompressedFormat {
        self.internal_format
    }

    /// Returns [`MemoryPolicy`].
    pub fn memory_policy(&self) -> &MemoryPolicy {
        &self.memory_policy
    }

    /// Returns max mipmap level.
    /// Returning `None` means mipmap is disabled,
    /// while returning `0` means texture size reaches the maximum level already, but not disabled.
    pub fn max_level(&self) -> Option<usize> {
        self.max_level
    }

    /// Returns width of a mipmap level.
    /// Returns texture base width in level 0.
    pub fn width_of_level(&self, level: usize) -> Option<usize> {
        if level == 0 {
            return Some(self.width);
        }
        let Some(max_level) = self.max_level() else {
            return None;
        };
        if level > max_level {
            return None;
        }

        Some((self.width >> level).max(1))
    }

    /// Returns height of a mipmap level.
    /// Returns texture base height in level 0.
    pub fn height_of_level(&self, level: usize) -> Option<usize> {
        if level == 0 {
            return Some(self.height);
        }
        let Some(max_level) = self.max_level() else {
            return None;
        };
        if level > max_level {
            return None;
        }

        Some((self.height >> level).max(1))
    }

    /// Returns bytes length the whole texture in all levels.
    pub fn bytes_length(&self) -> usize {
        // estimates used memory of all levels
        let mut used_memory = 0;
        for level in 0..=self.max_level().unwrap_or(0) {
            let width = self.width_of_level(level).unwrap();
            let height = self.height_of_level(level).unwrap();
            used_memory += self.internal_format.bytes_length(width, height);
        }
        used_memory
    }

    /// Returns bytes length of a mipmap level.
    pub fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
        let Some(width) = self.width_of_level(level) else {
            return None;
        };
        let Some(height) = self.height_of_level(level) else {
            return None;
        };

        Some(self.internal_format.bytes_length(width, height))
    }

    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(
        &mut self,
        source: TextureCompressedSource,
        level: usize,
    ) -> Result<(), Error> {
        self.uploads
            .push(TextureUpload::<TextureCompressedSource>::with_params(
                source, level, None, None, None, None,
            ));
        Ok(())
    }

    /// Uploads a sub data from a texture source to specified level of this texture.
    pub fn tex_sub_image(
        &mut self,
        source: TextureCompressedSource,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        self.uploads
            .push(TextureUpload::<TextureCompressedSource>::with_params(
                source,
                level,
                Some(width),
                Some(height),
                Some(x_offset),
                Some(y_offset),
            ));
        Ok(())
    }

    /// Creates [`WebGlTexture`] for texture 2d.
    pub(super) fn create_texture(
        &self,
        gl: &WebGl2RenderingContext,
        capabilities: &Capabilities,
    ) -> Result<WebGlTexture, Error> {
        capabilities.verify_compressed_format(self.internal_format)?;

        let texture = gl.create_texture().ok_or(Error::CreateTextureFailure)?;
        let bound = utils::texture_binding_2d(gl);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        gl.tex_storage_2d(
            WebGl2RenderingContext::TEXTURE_2D,
            (self.max_level.unwrap_or(0) + 1) as i32,
            self.internal_format.gl_enum(),
            self.width as i32,
            self.height as i32,
        );
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, bound.as_ref());
        Ok(texture)
    }

    /// Uploads data in `subs` to WebGL.
    /// In this stage, [`Texture2DCompressed::runtime`] is created already, it's safe to unwrap it and use fields inside.
    pub(super) fn tex(&mut self) -> Result<(), Error> {
        if self.uploads.is_empty() {
            return Ok(());
        }

        let runtime = self.runtime.as_deref().unwrap();

        let bound = utils::texture_binding_2d(&runtime.gl);
        runtime
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&runtime.texture));

        // then uploading all regular sources
        for TextureUpload {
            source,
            level,
            width,
            height,
            x_offset,
            y_offset,
        } in self.uploads.drain(..)
        {
            // abilities.verify_texture_size(source.width(), source.height())?;
            source.tex_sub_image_2d(
                &runtime.gl,
                TextureTarget::TEXTURE_2D,
                level,
                width,
                height,
                x_offset,
                y_offset,
            )?;
        }

        runtime
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, bound.as_ref());

        Ok(())
    }

    /// Applies memory free behavior.
    pub(super) fn free(&mut self) -> bool {
        match &mut self.memory_policy {
            MemoryPolicy::Unfree => false,
            MemoryPolicy::Restorable(restore) => {
                let restore = restore.as_mut()();
                self.uploads = restore.uploads;
                true
            }
        }
    }
}

impl TextureDescriptor<Texture2DCompressed> {
    /// Constructs a new texture descriptor with compressed texture 2d from a [`ConstructionPolicy`] and [`MemoryPolicy`].
    pub fn new(construction_policy: ConstructionPolicy, memory_policy: MemoryPolicy) -> Self {
        let texture = match construction_policy {
            ConstructionPolicy::Simple {
                internal_format,
                base,
            } => {
                let width = base.width();
                let height = base.height();
                Texture2DCompressed {
                    width,
                    height,
                    max_level: Some(max_available_mipmap_level(width, height)),
                    internal_format,
                    memory_policy,
                    uploads: vec![TextureUpload::<TextureCompressedSource>::new(base, 0)],
                    runtime: None,
                }
            }
            _ => {
                let (internal_format, width, height, max_level) = match construction_policy {
                    ConstructionPolicy::Preallocate {
                        internal_format,
                        width,
                        height,
                        max_level,
                    }
                    | ConstructionPolicy::Full {
                        internal_format,
                        width,
                        height,
                        max_level,
                        ..
                    } => {
                        let max_level = match max_level {
                            Some(max_level) => {
                                if max_level == 0 {
                                    None
                                } else {
                                    Some((max_level).min(max_available_mipmap_level(width, height)))
                                }
                            }
                            None => Some(max_available_mipmap_level(width, height)),
                        };
                        (internal_format, width, height, max_level)
                    }
                    _ => unreachable!(),
                };
                let uploads = match construction_policy {
                    ConstructionPolicy::Preallocate { .. } => Vec::new(),
                    ConstructionPolicy::Full { uploads, .. } => uploads,
                    _ => unreachable!(),
                };

                Texture2DCompressed {
                    width,
                    height,
                    max_level,
                    internal_format,
                    memory_policy,
                    uploads,
                    runtime: None,
                }
            }
        };

        Self(Rc::new(RefCell::new(texture)))
    }

    /// Returns [`Texture2DCompressed`] associated with this descriptor.
    pub fn texture(&self) -> Ref<'_, Texture2DCompressed> {
        self.0.borrow()
    }

    /// Returns mutable [`Texture2DCompressed`] associated with this descriptor.
    pub fn texture_mut(&self) -> RefMut<'_, Texture2DCompressed> {
        self.0.borrow_mut()
    }
}
