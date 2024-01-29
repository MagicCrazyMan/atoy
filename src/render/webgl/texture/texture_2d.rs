use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::render::webgl::{conversion::ToGlEnum, error::Error, utils};

use super::{
    max_available_mipmap_level, Runtime, TextureDescriptor, TextureSource, TextureTarget,
    TextureUncompressedInternalFormat, TextureUncompressedSource, TextureUpload,
};

/// construction policies telling texture store how to create a texture.
pub enum ConstructPolicy {
    /// Simple texture creation procedure.
    ///
    /// Under this policy, texture store may takes the size of the base texture source as the size of texture in level 0.
    /// The max level of the texture is applied as `floor(log2(max(width, height, 1)))`.
    /// After the base texture source uploaded, mipmaps are automatically generated then.
    ///
    /// [`TextureUpload`] upload by calling [`Texture2D::tex_image`] and [`Texture2D::tex_sub_image`]
    /// are uploaded after mipmap generated.
    Simple {
        internal_format: TextureUncompressedInternalFormat,
        base: TextureUncompressedSource,
    },
    /// Preallocates a texture only without uploading any image data.
    ///
    /// - Required `internal_format` defines the internal format.
    /// - Required `width` and `height` defines the size of texture in level 0.
    /// - Optional `max_level` defines the max mipmap level following rules:
    ///     - If `max_level` is `None`, mipmaps are available and the max mipmap level is `floor(log2(max(width, height, 1)))`.
    ///     - If `max_level` is `0`, no mipmaps are allowed.
    ///     - If `max_level` is any other value, max mipmap level is `min(max_level, floor(log2(max(width, height, 1))))`.
    ///
    /// Developers could modify each mipmap level manually then.
    Preallocate {
        internal_format: TextureUncompressedInternalFormat,
        width: usize,
        height: usize,
        max_level: Option<usize>,
    },
    /// Creates a texture with existing [`TextureUpload`] for each level.
    ///
    /// - Texture will first generate following the same procedure as [`ConstructPolicy::Preallocate`].
    /// - Required `uploads` defines texture source for uploading in each level.
    WithSources {
        internal_format: TextureUncompressedInternalFormat,
        width: usize,
        height: usize,
        max_level: Option<usize>,
        uploads: Vec<TextureUpload<TextureUncompressedSource>>,
    },
    /// Creates a texture by providing all customizable parameters.
    ///
    /// - Texture will first generate following the same procedure as [`ConstructPolicy::Preallocate`].
    /// - Required `uploads` defines data for uploading in each level, leaves an empty vector if no data need to upload currently.
    /// - Optional `max_level` defines the max mipmap level, takes `floor(log2(max(width, height, 1)))` if not provide.
    /// - Optional `mipmap_source` defines the texture source in level 0 for generating mipmaps automatically.
    /// Skips automatic mipmaps generation if not provide.
    /// - Optional `mipmap_base_level` defines the base level for generating mipmaps.
    /// - Optional `mipmap_max_level` defines the max level for generating mipmaps.
    ///
    /// *Automatic mipmaps generation is skips if compressed texture source is provided.*
    ///
    /// If `mipmap_source` is specified, it will upload first and then generate mipmaps
    /// before uploading data in `uploads` or lately upload by [`Texture2D::tex_image`] and [`Texture2D::tex_sub_image`].
    Full {
        internal_format: TextureUncompressedInternalFormat,
        width: usize,
        height: usize,
        uploads: Vec<TextureUpload<TextureUncompressedSource>>,
        max_level: Option<usize>,
        mipmap_source: Option<TextureUpload<TextureUncompressedSource>>,
        mipmap_base_level: Option<usize>,
        mipmap_max_level: Option<usize>,
    },
}

/// A WebGL 2d texture workload.
pub struct Texture2D {
    width: usize,
    height: usize,
    /// Max mipmap level clamped to max available level already if mipmap enabled.
    max_level: Option<usize>,
    internal_format: TextureUncompressedInternalFormat,
    // memory_policy: MemoryPolicy,
    mipmap_base: Option<(
        TextureUpload<TextureUncompressedSource>,
        Option<usize>,
        Option<usize>,
    )>,
    uploads: Vec<TextureUpload<TextureUncompressedSource>>,

    pub(super) runtime: Option<Box<Runtime<Texture2D>>>,
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            if let Some(runtime) = self.runtime.take() {
                (*runtime.descriptors).remove(&runtime.id);
                (*runtime.lru).remove(runtime.lru_node);
                (*runtime.used_memory) -= runtime.bytes_length;
                runtime.gl.delete_texture(Some(&runtime.texture));
            }
        }
    }
}

impl Texture2D {
    /// Returns texture base width in level 0.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns texture base height in level 0.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns [`TextureUncompressedInternalFormat`].
    pub fn internal_format(&self) -> TextureUncompressedInternalFormat {
        self.internal_format
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
        source: TextureUncompressedSource,
        level: usize,
    ) -> Result<(), Error> {
        // let width = source.width();
        // let height = source.height();
        // inner.verify_tex_image_level_size(level, width, height)?;

        self.uploads
            .push(TextureUpload::<TextureUncompressedSource>::with_params(
                source, level, None, None, None, None,
            ));
        Ok(())
    }

    /// Uploads a sub data from a texture source to specified level of this texture.
    pub fn tex_sub_image(
        &mut self,
        source: TextureUncompressedSource,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        // let width = source.width();
        // let height = source.height();
        // inner.verify_tex_sub_image_level_size(level, width, height, x_offset, y_offset)?;

        self.uploads
            .push(TextureUpload::<TextureUncompressedSource>::with_params(
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

    /// Uploads data in `subs` to WebGL.
    /// In this stage, [`Texture2D::runtime`] is created already, it's safe to unwrap it and use fields inside.
    pub(super) fn tex(&mut self) -> Result<(), Error> {
        if self.mipmap_base.is_none() && self.uploads.is_empty() {
            return Ok(());
        }

        let runtime = self.runtime.as_deref().unwrap();

        let bound = utils::texture_binding_2d(&runtime.gl);
        runtime
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&runtime.texture));

        // uploads mipmap base source and generates mipmap first if automatic mipmap is enabled
        if let Some((mipmap_base, base_level, max_level)) = self.mipmap_base.take() {
            let bound_base_level = match base_level {
                Some(base_level) => {
                    let bound = utils::texture_base_level(&runtime.gl, TextureTarget::TEXTURE_2D);
                    runtime.gl.tex_parameteri(
                        WebGl2RenderingContext::TEXTURE_2D,
                        WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                        base_level as i32,
                    );
                    bound
                }
                None => None,
            };
            let bound_max_level = match max_level {
                Some(max_level) => {
                    let bound = utils::texture_max_level(&runtime.gl, TextureTarget::TEXTURE_2D);
                    runtime.gl.tex_parameteri(
                        WebGl2RenderingContext::TEXTURE_2D,
                        WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                        max_level as i32,
                    );
                    bound
                }
                None => None,
            };

            mipmap_base.source.tex_sub_image_2d(
                &runtime.gl,
                TextureTarget::TEXTURE_2D,
                0,
                None,
                None,
                None,
                None,
            )?;
            runtime
                .gl
                .generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

            if let Some(bound_base_level) = bound_base_level {
                runtime.gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                    bound_base_level as i32,
                );
            }
            if let Some(bound_max_level) = bound_max_level {
                runtime.gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_2D,
                    WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                    bound_max_level as i32,
                );
            }
        }

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
}

impl TextureDescriptor<Texture2D> {
    pub fn new(mut construction_policy: ConstructPolicy) -> Self {
        let texture = match construction_policy {
            ConstructPolicy::Simple {
                internal_format,
                base,
            } => {
                let width = base.width();
                let height = base.height();
                Texture2D {
                    width,
                    height,
                    max_level: Some(max_available_mipmap_level(width, height)),
                    internal_format,
                    mipmap_base: Some((TextureUpload::new(base, 0), None, None)),
                    uploads: Vec::new(),
                    runtime: None,
                }
            }
            _ => {
                let (internal_format, width, height, max_level) = match construction_policy {
                    ConstructPolicy::Preallocate {
                        internal_format,
                        width,
                        height,
                        max_level,
                    }
                    | ConstructPolicy::WithSources {
                        internal_format,
                        width,
                        height,
                        max_level,
                        ..
                    }
                    | ConstructPolicy::Full {
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
                let mipmap_base = match construction_policy {
                    ConstructPolicy::Preallocate { .. } | ConstructPolicy::WithSources { .. } => {
                        None
                    }
                    ConstructPolicy::Full {
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
                    ConstructPolicy::Preallocate { .. } => Vec::new(),
                    ConstructPolicy::WithSources { uploads, .. }
                    | ConstructPolicy::Full { uploads, .. } => uploads,
                    _ => unreachable!(),
                };

                Texture2D {
                    width,
                    height,
                    max_level,
                    internal_format,
                    mipmap_base,
                    uploads,
                    runtime: None,
                }
            }
        };

        Self(Rc::new(RefCell::new(texture)))
    }

    /// Returns [`Texture2D`] associated with this descriptor.
    pub fn texture(&self) -> Ref<'_, Texture2D> {
        self.0.borrow()
    }

    /// Returns mutable [`Texture2D`] associated with this descriptor.
    pub fn texture_mut(&self) -> RefMut<'_, Texture2D> {
        self.0.borrow_mut()
    }
}

pub enum MemoryPolicy {
    Unfree,
    Restorable,
}
