use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::render::webgl::{capabilities::Capabilities, conversion::ToGlEnum, error::Error, utils};

use super::{
    max_available_mipmap_level, Runtime, TextureCompressedFormat, TextureDescriptor, TextureSource,
    TextureSourceCompressed, TextureTarget, TextureUnit, TextureUpload,
};

/// Cube map faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(usize)]
pub enum CubeMapFace {
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

/// Construction policies telling texture store how to create a texture.
pub enum ConstructionPolicy {
    /// Simple texture creation procedure.
    ///
    /// Under this policy, the size of the positive x texture source uses as the size of texture in level 0,
    /// and size of each cube map face must be the same.
    ///
    /// The max level of the texture is applied as `floor(log2(max(width, height, 1)))`.
    Simple {
        internal_format: TextureCompressedFormat,
        positive_x: TextureSourceCompressed,
        negative_x: TextureSourceCompressed,
        positive_y: TextureSourceCompressed,
        negative_y: TextureSourceCompressed,
        positive_z: TextureSourceCompressed,
        negative_z: TextureSourceCompressed,
    },
    /// Preallocates a texture only without uploading any image data.
    ///
    /// - Required `internal_format` defines the compressed format.
    /// - Required `width` and `height` defines the size of texture in level 0.
    /// - Optional `max_level` defines the max mipmap level following rules:
    ///     - If `max_level` is `None`, mipmaps are available and the max mipmap level is `floor(log2(max(width, height, 1)))`.
    ///     - If `max_level` is `0`, no mipmaps are allowed.
    ///     - If `max_level` is any other value, max mipmap level is `min(max_level, floor(log2(max(width, height, 1))))`.
    ///
    /// Developers could modify each mipmap level manually then.
    Preallocate {
        internal_format: TextureCompressedFormat,
        width: usize,
        height: usize,
        max_level: Option<usize>,
    },
    /// Creates a texture by providing all customizable parameters.
    ///
    /// - Texture will first generate following the same procedure as [`ConstructionPolicy::Preallocate`].
    /// - Required `positive_x`, `negative_x`, `positive_y`, `negative_y`, `positive_z` and `negative_z`
    /// defines data for uploading in each level and each cube map face, leaves empty vectors if no data need to upload currently.
    /// - Optional `max_level` defines the max mipmap level, takes `floor(log2(max(width, height, 1)))` if not provide.
    Full {
        internal_format: TextureCompressedFormat,
        width: usize,
        height: usize,
        max_level: Option<usize>,

        positive_x: Vec<TextureUpload<TextureSourceCompressed>>,
        negative_x: Vec<TextureUpload<TextureSourceCompressed>>,
        positive_y: Vec<TextureUpload<TextureSourceCompressed>>,
        negative_y: Vec<TextureUpload<TextureSourceCompressed>>,
        positive_z: Vec<TextureUpload<TextureSourceCompressed>>,
        negative_z: Vec<TextureUpload<TextureSourceCompressed>>,
    },
}

/// A container provides content for restoring a texture.
pub struct Restore {
    positive_x: Vec<TextureUpload<TextureSourceCompressed>>,
    negative_x: Vec<TextureUpload<TextureSourceCompressed>>,
    positive_y: Vec<TextureUpload<TextureSourceCompressed>>,
    negative_y: Vec<TextureUpload<TextureSourceCompressed>>,
    positive_z: Vec<TextureUpload<TextureSourceCompressed>>,
    negative_z: Vec<TextureUpload<TextureSourceCompressed>>,
}

/// Memory policies controlling how to manage memory of a texture.
pub enum MemoryPolicy {
    Unfree,
    Restorable(Box<dyn Fn() -> Restore>),
}

/// A WebGL cube map texture workload.
pub struct TextureCubeMapCompressed {
    width: usize,
    height: usize,
    max_level: Option<usize>,
    internal_format: TextureCompressedFormat,
    memory_policy: MemoryPolicy,

    faces: [Vec<TextureUpload<TextureSourceCompressed>>; 6],

    pub(super) runtime: Option<Box<Runtime>>,
}

impl Drop for TextureCubeMapCompressed {
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

impl TextureCubeMapCompressed {
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
            used_memory += self.internal_format.bytes_length(width, height) * 6;
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

        Some(self.internal_format.bytes_length(width, height) * 6)
    }

    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(
        &mut self,
        face: CubeMapFace,
        source: TextureSourceCompressed,
        level: usize,
    ) -> Result<(), Error> {
        self.faces[face as usize].push(TextureUpload::<TextureSourceCompressed>::with_params_2d(
            source, level, None, None, None, None,
        ));
        Ok(())
    }

    /// Uploads a sub data from a texture source to specified level of this texture.
    pub fn tex_sub_image(
        &mut self,
        face: CubeMapFace,
        source: TextureSourceCompressed,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        self.faces[face as usize].push(TextureUpload::<TextureSourceCompressed>::with_params_2d(
            source,
            level,
            Some(width),
            Some(height),
            Some(x_offset),
            Some(y_offset),
        ));
        Ok(())
    }

    /// Creates [`WebGlTexture`] for texture cube map.
    pub(super) fn create_texture(
        &self,
        gl: &WebGl2RenderingContext,
        capabilities: &Capabilities,
    ) -> Result<WebGlTexture, Error> {
        capabilities.verify_compressed_format(self.internal_format)?;

        let texture = gl.create_texture().ok_or(Error::CreateTextureFailure)?;
        let bound = utils::texture_binding_cube_map(gl);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_CUBE_MAP, Some(&texture));
        gl.tex_storage_2d(
            WebGl2RenderingContext::TEXTURE_CUBE_MAP,
            (self.max_level.unwrap_or(0) + 1) as i32,
            self.internal_format.gl_enum(),
            self.width as i32,
            self.height as i32,
        );
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_CUBE_MAP, bound.as_ref());
        Ok(texture)
    }

    /// Uploads data to WebGL.
    /// In this stage, [`TextureCubeMapCompressed::runtime`] is created already, it's safe to unwrap it and use fields inside.
    pub(super) fn tex(&mut self, unit: TextureUnit) -> Result<(), Error> {
        if self.faces.iter().map(|face| face.len()).sum::<usize>() == 0 {
            return Ok(());
        }

        let runtime = self.runtime.as_deref().unwrap();

        let bound_texture = utils::texture_binding_cube_map(&runtime.gl);
        let bound_unit = utils::active_texture_unit(&runtime.gl);

        runtime.gl.active_texture(unit.gl_enum());
        runtime.gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_CUBE_MAP,
            Some(&runtime.texture),
        );

        // then uploading all regular sources
        for face in self.faces.iter_mut() {
            for source in face.drain(..) {
                // abilities.verify_texture_size(source.width(), source.height())?;
                source.tex_sub_image_2d(&runtime.gl, TextureTarget::TEXTURE_CUBE_MAP)?;
            }
        }

        runtime.gl.active_texture(bound_unit);
        runtime
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_CUBE_MAP, bound_texture.as_ref());

        Ok(())
    }

    /// Applies memory free behavior.
    /// Returns `true` if this texture is released.
    pub(super) fn free(&mut self) -> bool {
        match &mut self.memory_policy {
            MemoryPolicy::Unfree => false,
            MemoryPolicy::Restorable(restore) => {
                let Restore {
                    positive_x,
                    negative_x,
                    positive_y,
                    negative_y,
                    positive_z,
                    negative_z,
                } = restore.as_mut()();
                self.faces[0].extend(positive_x);
                self.faces[1].extend(negative_x);
                self.faces[2].extend(positive_y);
                self.faces[3].extend(negative_y);
                self.faces[4].extend(positive_z);
                self.faces[5].extend(negative_z);

                true
            }
        }
    }
}

impl TextureDescriptor<TextureCubeMapCompressed> {
    /// Constructs a new texture descriptor with [`TextureCubeMapCompressed`] from a [`ConstructionPolicy`] and [`MemoryPolicy`].
    pub fn new(construction_policy: ConstructionPolicy, memory_policy: MemoryPolicy) -> Self {
        let texture = match construction_policy {
            ConstructionPolicy::Simple {
                internal_format,
                positive_x,
                negative_x,
                positive_y,
                negative_y,
                positive_z,
                negative_z,
            } => {
                let width = positive_x.width();
                let height = positive_x.height();
                TextureCubeMapCompressed {
                    width,
                    height,
                    max_level: Some(max_available_mipmap_level(width, height)),
                    internal_format,
                    memory_policy,

                    faces: [
                        vec![TextureUpload::new_2d(positive_x, 0)],
                        vec![TextureUpload::new_2d(negative_x, 0)],
                        vec![TextureUpload::new_2d(positive_y, 0)],
                        vec![TextureUpload::new_2d(negative_y, 0)],
                        vec![TextureUpload::new_2d(positive_z, 0)],
                        vec![TextureUpload::new_2d(negative_z, 0)],
                    ],

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
                let faces = match construction_policy {
                    ConstructionPolicy::Preallocate { .. } => [
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                    ],
                    ConstructionPolicy::Full {
                        positive_x,
                        negative_x,
                        positive_y,
                        negative_y,
                        positive_z,
                        negative_z,
                        ..
                    } => [
                        positive_x, negative_x, positive_y, negative_y, positive_z, negative_z,
                    ],
                    _ => unreachable!(),
                };

                TextureCubeMapCompressed {
                    width,
                    height,
                    max_level,
                    internal_format,
                    memory_policy,
                    faces,
                    runtime: None,
                }
            }
        };

        Self(Rc::new(RefCell::new(texture)))
    }

    /// Returns [`TextureCubeMapCompressed`] associated with this descriptor.
    pub fn texture(&self) -> Ref<'_, TextureCubeMapCompressed> {
        self.0.borrow()
    }

    /// Returns mutable [`TextureCubeMapCompressed`] associated with this descriptor.
    pub fn texture_mut(&self) -> RefMut<'_, TextureCubeMapCompressed> {
        self.0.borrow_mut()
    }
}
