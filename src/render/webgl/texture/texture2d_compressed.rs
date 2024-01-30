use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::render::webgl::{capabilities::Capabilities, conversion::ToGlEnum, error::Error, utils};

use super::{
    max_available_mipmap_level, Runtime, Texture, TextureCompressedFormat, TextureDescriptor,
    TextureInner, TexturePlanar, TextureSource, TextureSourceCompressed, TextureTarget,
    TextureUnit, TextureUpload,
};

/// Construction policies telling texture store how to create a texture.
pub enum ConstructionPolicy {
    /// Simple texture creation procedure.
    ///
    /// Under this policy, the size of the base texture source uses as the size of texture in level 0.
    /// The max level of the texture is applied as `floor(log2(max(width, height, 1)))`.
    Simple {
        internal_format: TextureCompressedFormat,
        base: TextureSourceCompressed,
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
    /// - Texture will first generate following the same procedure as [`ConstructionPolicy::Preallocate`].
    /// - Required `uploads` defines texture source for uploading in each level.
    Full {
        internal_format: TextureCompressedFormat,
        width: usize,
        height: usize,
        max_level: Option<usize>,
        uploads: Vec<TextureUpload<TextureSourceCompressed>>,
    },
}

/// A container provides content for restoring a texture.
pub struct Restore {
    uploads: Vec<TextureUpload<TextureSourceCompressed>>,
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
    uploads: Vec<TextureUpload<TextureSourceCompressed>>,

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
    /// Returns [`TextureCompressedFormat`].
    pub fn internal_format(&self) -> TextureCompressedFormat {
        self.internal_format
    }

    /// Returns [`MemoryPolicy`].
    pub fn memory_policy(&self) -> &MemoryPolicy {
        &self.memory_policy
    }

    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(
        &mut self,
        source: TextureSourceCompressed,
        level: usize,
    ) -> Result<(), Error> {
        self.uploads
            .push(TextureUpload::<TextureSourceCompressed>::with_params_2d(
                source, level, None, None, None, None,
            ));
        Ok(())
    }

    /// Uploads a sub data from a texture source to specified level of this texture.
    pub fn tex_sub_image(
        &mut self,
        source: TextureSourceCompressed,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        self.uploads
            .push(TextureUpload::<TextureSourceCompressed>::with_params_2d(
                source,
                level,
                Some(width),
                Some(height),
                Some(x_offset),
                Some(y_offset),
            ));
        Ok(())
    }
}

impl Texture for Texture2DCompressed {
    fn target(&self) -> TextureTarget {
        TextureTarget::TEXTURE_2D
    }

    fn max_level(&self) -> Option<usize> {
        self.max_level
    }

    fn bytes_length(&self) -> usize {
        let mut used_memory = 0;
        for level in 0..=self.max_level().unwrap_or(0) {
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

impl TexturePlanar for Texture2DCompressed {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl TextureInner for Texture2DCompressed {
    fn runtime(&self) -> Option<&Runtime> {
        self.runtime.as_deref()
    }

    fn runtime_mut(&mut self) -> Option<&mut Runtime> {
        self.runtime.as_deref_mut()
    }

    fn set_runtime(&mut self, runtime: Runtime) {
        self.runtime = Some(Box::new(runtime));
    }

    fn remove_runtime(&mut self) -> Option<Runtime> {
        self.runtime.take().map(|runtime| *runtime)
    }

    fn validate(&self, capabilities: &Capabilities) -> Result<(), Error> {
        capabilities.verify_compressed_format(self.internal_format)?;
        Ok(())
    }

    fn create(
        &self,
        gl: &WebGl2RenderingContext,
        unit: TextureUnit,
    ) -> Result<WebGlTexture, Error> {
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

    fn upload(&mut self, unit: TextureUnit) -> Result<(), Error> {
        if self.uploads.is_empty() {
            return Ok(());
        }

        let runtime = self.runtime.as_deref().unwrap();

        let bound_texture = utils::texture_binding_2d(&runtime.gl);
        let bound_unit = utils::active_texture_unit(&runtime.gl);

        runtime.gl.active_texture(unit.gl_enum());
        runtime
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&runtime.texture));

        // then uploading all regular sources
        for upload in self.uploads.drain(..) {
            // abilities.verify_texture_size(source.width(), source.height())?;
            upload.tex_sub_image_2d(&runtime.gl, TextureTarget::TEXTURE_2D)?;
        }

        runtime.gl.active_texture(bound_unit);
        runtime
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, bound_texture.as_ref());

        Ok(())
    }

    fn free(&mut self) -> bool {
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
    /// Constructs a new texture descriptor with [`Texture2DCompressed`] from a [`ConstructionPolicy`] and [`MemoryPolicy`].
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
                    uploads: vec![TextureUpload::new_2d(base, 0)],
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
}
