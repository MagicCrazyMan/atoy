use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::render::webgl::{capabilities::Capabilities, conversion::ToGlEnum, error::Error, utils};

use super::{
    Runtime, Texture, TextureArray, TextureDescriptor, TextureInner,
    TextureInternalFormatUncompressed, TexturePlanar, TextureSource, TextureSourceUncompressed,
    TextureTarget, TextureUnit, TextureUpload,
};

/// Construction policies telling texture store how to create a texture.
pub enum ConstructionPolicy {
    /// Simple texture creation procedure.
    ///
    /// Under this policy, the size of the base texture source uses as the size of texture in level 0.
    /// And for a 2d array texture, a array length in level 0 is also required.
    ///
    /// The max level of the texture is applied as `floor(log2(max(width, height, array_length, 1)))`.
    /// After the base texture source uploaded, mipmaps are automatically generated then.
    ///
    /// Image data upload by calling [`Texture2DArray::tex_image`] and [`Texture2DArray::tex_sub_image`]
    /// are uploaded after mipmap generated.
    Simple {
        internal_format: TextureInternalFormatUncompressed,
        array_length: usize,
        base: TextureSourceUncompressed,
    },
    /// Preallocates a texture only without uploading any image data.
    ///
    /// - Required `internal_format` defines the internal format.
    /// - Required `width`, `height` and `array_length` defines the size of texture in level 0.
    /// - Optional `max_level` defines the max mipmap level following rules:
    ///     - If `max_level` is `None`, mipmaps are available and the max mipmap level is `floor(log2(max(width, height, array_length, 1)))`.
    ///     - If `max_level` is `0`, no mipmaps are allowed.
    ///     - If `max_level` is any other value, max mipmap level is `min(max_level, floor(log2(max(width, height, array_length, 1))))`.
    ///
    /// Developers could modify each mipmap level manually then.
    Preallocate {
        internal_format: TextureInternalFormatUncompressed,
        width: usize,
        height: usize,
        array_length: usize,
        max_level: Option<usize>,
    },
    /// Creates a texture with existing [`TextureUpload`] for each level.
    ///
    /// - Texture will first generate following the same procedure as [`ConstructionPolicy::Preallocate`].
    /// - Required `uploads` defines texture source for uploading in each level.
    WithSources {
        internal_format: TextureInternalFormatUncompressed,
        width: usize,
        height: usize,
        array_length: usize,
        max_level: Option<usize>,
        uploads: Vec<TextureUpload<TextureSourceUncompressed>>,
    },
    /// Creates a texture by providing all customizable parameters.
    ///
    /// - Texture will first generate following the same procedure as [`ConstructionPolicy::Preallocate`].
    /// - Required `uploads` defines data for uploading in each level, leaves an empty vector if no data need to upload currently.
    /// - Optional `max_level` defines the max mipmap level, takes `floor(log2(max(width, height, array_length, 1)))` if not provide.
    /// - Optional `mipmap_source` defines the texture source in level 0 for generating mipmaps automatically.
    /// Skips automatic mipmaps generation if not provide.
    /// - Optional `mipmap_base_level` defines the base level for generating mipmaps.
    /// - Optional `mipmap_max_level` defines the max level for generating mipmaps.
    ///
    /// If `mipmap_source` is specified, it will upload first and then generate mipmaps
    /// before uploading data in `uploads` or lately upload by [`Texture2DArray::tex_image`] and [`Texture2DArray::tex_sub_image`].
    Full {
        internal_format: TextureInternalFormatUncompressed,
        width: usize,
        height: usize,
        array_length: usize,
        uploads: Vec<TextureUpload<TextureSourceUncompressed>>,
        max_level: Option<usize>,
        mipmap_source: Option<TextureUpload<TextureSourceUncompressed>>,
        mipmap_base_level: Option<usize>,
        mipmap_max_level: Option<usize>,
    },
}

/// A container provides content for restoring a texture.
pub struct Restore {
    uploads: Vec<TextureUpload<TextureSourceUncompressed>>,
    mipmap_source: Option<TextureUpload<TextureSourceUncompressed>>,
    mipmap_base_level: Option<usize>,
    mipmap_max_level: Option<usize>,
}

/// Memory policies controlling how to manage memory of a texture.
pub enum MemoryPolicy {
    Unfree,
    Restorable(Box<dyn Fn() -> Restore>),
}

/// A WebGL 2d array texture workload.
///
/// Different from [`WebGl2RenderingContext::TEXTURE_2D_ARRAY`],
/// a depth value in [`WebGl2RenderingContext::TEXTURE_2D_ARRAY`] refers to the length of array, not a dimensional value.
pub struct Texture2DArray {
    width: usize,
    height: usize,
    array_length: usize,
    /// Max mipmap level clamped to max available level already if mipmap enabled.
    max_level: Option<usize>,
    internal_format: TextureInternalFormatUncompressed,
    memory_policy: MemoryPolicy,
    mipmap_base: Option<(
        TextureUpload<TextureSourceUncompressed>,
        Option<usize>,
        Option<usize>,
    )>,
    uploads: Vec<TextureUpload<TextureSourceUncompressed>>,

    runtime: Option<Box<Runtime>>,
}

impl Drop for Texture2DArray {
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

impl Texture2DArray {
    /// Returns [`TextureInternalFormatUncompressed`].
    pub fn internal_format(&self) -> TextureInternalFormatUncompressed {
        self.internal_format
    }

    /// Returns [`MemoryPolicy`].
    pub fn memory_policy(&self) -> &MemoryPolicy {
        &self.memory_policy
    }

    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(
        &mut self,
        source: TextureSourceUncompressed,
        level: usize,
        array_length: usize,
    ) -> Result<(), Error> {
        self.uploads
            .push(TextureUpload::<TextureSourceUncompressed>::with_params_uncompressed(
                source,
                level,
                array_length,
                None,
                None,
                None,
                None,
                None,
            ));
        Ok(())
    }

    /// Uploads a sub data from a texture source to specified level of this texture.
    pub fn tex_sub_image(
        &mut self,
        source: TextureSourceUncompressed,
        level: usize,
        array_length: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) -> Result<(), Error> {
        self.uploads
            .push(TextureUpload::<TextureSourceUncompressed>::with_params_uncompressed(
                source,
                level,
                array_length,
                Some(width),
                Some(height),
                Some(x_offset),
                Some(y_offset),
                Some(z_offset),
            ));
        Ok(())
    }
}

impl Texture for Texture2DArray {
    fn target(&self) -> TextureTarget {
        TextureTarget::TEXTURE_2D_ARRAY
    }

    fn max_available_mipmap_level(&self) -> usize {
        <Self as TexturePlanar>::max_available_mipmap_level(self.width, self.height)
    }

    fn max_level(&self) -> Option<usize> {
        self.max_level
    }

    fn bytes_length(&self) -> usize {
        let mut used_memory = 0;
        for level in 0..=self.max_level().unwrap_or(0) {
            let width = self.width_of_level(level).unwrap();
            let height = self.height_of_level(level).unwrap();
            used_memory += self.internal_format.bytes_length(width, height) * self.array_length;
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

        Some(self.internal_format.bytes_length(width, height) * self.array_length)
    }
}

impl TexturePlanar for Texture2DArray {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl TextureArray for Texture2DArray {
    fn array_length(&self) -> usize {
        self.array_length
    }
}

impl TextureInner for Texture2DArray {
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
        capabilities.verify_internal_format(self.internal_format)?;
        Ok(())
    }

    fn create(
        &self,
        gl: &WebGl2RenderingContext,
        unit: TextureUnit,
    ) -> Result<WebGlTexture, Error> {
        let texture = gl.create_texture().ok_or(Error::CreateTextureFailure)?;
        let bound = utils::texture_binding_2d_array(gl);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D_ARRAY, Some(&texture));
        gl.tex_storage_3d(
            WebGl2RenderingContext::TEXTURE_2D_ARRAY,
            (self.max_level.unwrap_or(0) + 1) as i32,
            self.internal_format.gl_enum(),
            self.width as i32,
            self.height as i32,
            self.array_length as i32,
        );
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D_ARRAY, bound.as_ref());
        Ok(texture)
    }

    fn upload(&mut self, unit: TextureUnit) -> Result<(), Error> {
        if self.mipmap_base.is_none() && self.uploads.is_empty() {
            return Ok(());
        }

        let runtime = self.runtime.as_deref().unwrap();

        let bound_texture = utils::texture_binding_2d_array(&runtime.gl);
        let bound_unit = utils::active_texture_unit(&runtime.gl);

        runtime.gl.active_texture(unit.gl_enum());
        runtime.gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D_ARRAY,
            Some(&runtime.texture),
        );

        // uploads mipmap base source and generates mipmap first if automatic mipmap is enabled
        if let Some((source, base_level, max_level)) = self.mipmap_base.take() {
            let bound_base_level = match base_level {
                Some(base_level) => {
                    let bound =
                        utils::texture_base_level(&runtime.gl, TextureTarget::TEXTURE_2D_ARRAY);
                    runtime.gl.tex_parameteri(
                        WebGl2RenderingContext::TEXTURE_2D_ARRAY,
                        WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                        base_level as i32,
                    );
                    bound
                }
                None => None,
            };
            let bound_max_level = match max_level {
                Some(max_level) => {
                    let bound =
                        utils::texture_max_level(&runtime.gl, TextureTarget::TEXTURE_2D_ARRAY);
                    runtime.gl.tex_parameteri(
                        WebGl2RenderingContext::TEXTURE_2D_ARRAY,
                        WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                        max_level as i32,
                    );
                    bound
                }
                None => None,
            };

            source.tex_sub_image_3d(&runtime.gl, TextureTarget::TEXTURE_2D_ARRAY)?;
            runtime
                .gl
                .generate_mipmap(WebGl2RenderingContext::TEXTURE_2D_ARRAY);

            if let Some(bound_base_level) = bound_base_level {
                runtime.gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D_ARRAY,
                    WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                    bound_base_level as i32,
                );
            }
            if let Some(bound_max_level) = bound_max_level {
                runtime.gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D_ARRAY,
                    WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                    bound_max_level as i32,
                );
            }
        }

        // then uploading all regular sources
        for upload in self.uploads.drain(..) {
            // abilities.verify_texture_size(source.width(), source.height())?;
            upload.tex_sub_image_3d(&runtime.gl, TextureTarget::TEXTURE_2D_ARRAY)?;
        }

        runtime.gl.active_texture(bound_unit);
        runtime.gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_2D_ARRAY,
            bound_texture.as_ref(),
        );

        Ok(())
    }

    fn free(&mut self) -> bool {
        match &mut self.memory_policy {
            MemoryPolicy::Unfree => false,
            MemoryPolicy::Restorable(restore) => {
                let Restore {
                    uploads,
                    mipmap_source,
                    mipmap_base_level,
                    mipmap_max_level,
                } = restore.as_mut()();
                self.uploads.extend(uploads);
                if let Some(mipmap_source) = mipmap_source {
                    self.mipmap_base = Some((mipmap_source, mipmap_base_level, mipmap_max_level));
                }
                true
            }
        }
    }
}

impl TextureDescriptor<Texture2DArray> {
    /// Constructs a new texture descriptor with [`Texture2DArray`] from a [`ConstructionPolicy`] and [`MemoryPolicy`].
    pub fn new(mut construction_policy: ConstructionPolicy, memory_policy: MemoryPolicy) -> Self {
        let texture = match construction_policy {
            ConstructionPolicy::Simple {
                internal_format,
                array_length,
                base,
            } => {
                let width = base.width();
                let height = base.height();
                Texture2DArray {
                    width,
                    height,
                    array_length,
                    max_level: Some(
                        <Texture2DArray as TexturePlanar>::max_available_mipmap_level(
                            width, height,
                        ),
                    ),
                    internal_format,
                    memory_policy,
                    mipmap_base: Some((TextureUpload::new_3d(base, 0, 0), None, None)),
                    uploads: Vec::new(),
                    runtime: None,
                }
            }
            _ => {
                let (internal_format, width, height, array_length, max_level) =
                    match construction_policy {
                        ConstructionPolicy::Preallocate {
                            internal_format,
                            width,
                            height,
                            array_length,
                            max_level,
                        }
                        | ConstructionPolicy::WithSources {
                            internal_format,
                            width,
                            height,
                            array_length,
                            max_level,
                            ..
                        }
                        | ConstructionPolicy::Full {
                            internal_format,
                            width,
                            height,
                            array_length,
                            max_level,
                            ..
                        } => {
                            let max_level = match max_level {
                                Some(max_level) => {
                                    if max_level == 0 {
                                        None
                                    } else {
                                        Some(
                                            (max_level)
                                                .min(<Texture2DArray as TexturePlanar>::max_available_mipmap_level(width, height)),
                                        )
                                    }
                                }
                                None => Some(
                                    <Texture2DArray as TexturePlanar>::max_available_mipmap_level(
                                        width, height,
                                    ),
                                ),
                            };
                            (internal_format, width, height, array_length, max_level)
                        }
                        _ => unreachable!(),
                    };
                let mipmap_base = match construction_policy {
                    ConstructionPolicy::Preallocate { .. }
                    | ConstructionPolicy::WithSources { .. } => None,
                    ConstructionPolicy::Full {
                        mipmap_base_level,
                        mipmap_max_level,
                        ref mut mipmap_source,
                        ..
                    } => match mipmap_source.take() {
                        Some(mipmap_source) => {
                            Some((mipmap_source, mipmap_base_level, mipmap_max_level))
                        }
                        None => None,
                    },
                    _ => unreachable!(),
                };
                let uploads = match construction_policy {
                    ConstructionPolicy::Preallocate { .. } => Vec::new(),
                    ConstructionPolicy::WithSources { uploads, .. }
                    | ConstructionPolicy::Full { uploads, .. } => uploads,
                    _ => unreachable!(),
                };

                Texture2DArray {
                    width,
                    height,
                    array_length,
                    max_level,
                    internal_format,
                    memory_policy,
                    mipmap_base,
                    uploads,
                    runtime: None,
                }
            }
        };

        Self(Rc::new(RefCell::new(texture)))
    }
}
