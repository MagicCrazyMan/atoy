use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::render::webgl::{capabilities::Capabilities, conversion::ToGlEnum, error::Error, utils};

use super::{
    Runtime, Texture, TextureDescriptor, TextureInner, TextureInternalFormatUncompressed,
    TexturePlanar, TextureSource, TextureSourceUncompressed, TextureTarget, TextureUnit,
    TextureUpload,
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
    /// After the base texture source uploaded, mipmaps are automatically generated then.
    ///
    /// Image data upload by calling [`TextureCubeMap::tex_image`] and [`TextureCubeMap::tex_sub_image`]
    /// are uploaded after mipmap generated.
    Simple {
        internal_format: TextureInternalFormatUncompressed,
        positive_x: TextureSourceUncompressed,
        negative_x: TextureSourceUncompressed,
        positive_y: TextureSourceUncompressed,
        negative_y: TextureSourceUncompressed,
        positive_z: TextureSourceUncompressed,
        negative_z: TextureSourceUncompressed,
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
        internal_format: TextureInternalFormatUncompressed,
        width: usize,
        height: usize,
        max_level: Option<usize>,
    },
    /// Creates a texture with existing [`TextureUpload`] for each level and each cube map face.
    ///
    /// - Texture will first generate following the same procedure as [`ConstructionPolicy::Preallocate`].
    /// - Required `positive_x`, `negative_x`, `positive_y`, `negative_y`, `positive_z` and `negative_z`
    /// defines data for uploading in each level and each cube map face.
    WithSources {
        internal_format: TextureInternalFormatUncompressed,
        width: usize,
        height: usize,
        max_level: Option<usize>,
        positive_x: Vec<TextureUpload<TextureSourceUncompressed>>,
        negative_x: Vec<TextureUpload<TextureSourceUncompressed>>,
        positive_y: Vec<TextureUpload<TextureSourceUncompressed>>,
        negative_y: Vec<TextureUpload<TextureSourceUncompressed>>,
        positive_z: Vec<TextureUpload<TextureSourceUncompressed>>,
        negative_z: Vec<TextureUpload<TextureSourceUncompressed>>,
    },
    /// Creates a texture by providing all customizable parameters.
    ///
    /// - Texture will first generate following the same procedure as [`ConstructionPolicy::Preallocate`].
    /// - Required `positive_x`, `negative_x`, `positive_y`, `negative_y`, `positive_z` and `negative_z`
    /// defines data for uploading in each level and each cube map face, leaves empty vectors if no data need to upload currently.
    /// - Optional `max_level` defines the max mipmap level, takes `floor(log2(max(width, height, 1)))` if not provide.
    /// - Optional `mipmap_source` defines the texture sources of each cube map face in level 0 for generating mipmaps automatically.
    /// `positive_x`,`negative_x`, `positive_y`, `negative_y`, `positive_z` and `negative_z` index from `0` to `5` respectively.
    /// Skips automatic mipmaps generation if not provide.
    /// - Optional `mipmap_base_level` defines the base level for generating mipmaps.
    /// - Optional `mipmap_max_level` defines the max level for generating mipmaps.
    ///
    /// If `mipmap_source` is specified, it will upload first and then generate mipmaps
    /// before uploading data in `uploads` or lately upload by [`TextureCubeMap::tex_image`] and [`TextureCubeMap::tex_sub_image`].
    Full {
        internal_format: TextureInternalFormatUncompressed,
        width: usize,
        height: usize,
        max_level: Option<usize>,

        positive_x: Vec<TextureUpload<TextureSourceUncompressed>>,
        negative_x: Vec<TextureUpload<TextureSourceUncompressed>>,
        positive_y: Vec<TextureUpload<TextureSourceUncompressed>>,
        negative_y: Vec<TextureUpload<TextureSourceUncompressed>>,
        positive_z: Vec<TextureUpload<TextureSourceUncompressed>>,
        negative_z: Vec<TextureUpload<TextureSourceUncompressed>>,

        mipmap_source: Option<[TextureUpload<TextureSourceUncompressed>; 6]>,
        mipmap_base_level: Option<usize>,
        mipmap_max_level: Option<usize>,
    },
}

/// A container provides content for restoring a texture.
pub struct Restore {
    positive_x: Vec<TextureUpload<TextureSourceUncompressed>>,
    negative_x: Vec<TextureUpload<TextureSourceUncompressed>>,
    positive_y: Vec<TextureUpload<TextureSourceUncompressed>>,
    negative_y: Vec<TextureUpload<TextureSourceUncompressed>>,
    positive_z: Vec<TextureUpload<TextureSourceUncompressed>>,
    negative_z: Vec<TextureUpload<TextureSourceUncompressed>>,

    mipmap_sources: Option<[TextureUpload<TextureSourceUncompressed>; 6]>,
    mipmap_base_level: Option<usize>,
    mipmap_max_level: Option<usize>,
}

/// Memory policies controlling how to manage memory of a texture.
pub enum MemoryPolicy {
    Unfree,
    Restorable(Box<dyn Fn() -> Restore>),
}

/// A WebGL cube map texture workload.
pub struct TextureCubeMap {
    width: usize,
    height: usize,
    /// Max mipmap level clamped to max available level already if mipmap enabled.
    max_level: Option<usize>,
    internal_format: TextureInternalFormatUncompressed,
    memory_policy: MemoryPolicy,

    mipmap_base: Option<(
        [TextureUpload<TextureSourceUncompressed>; 6],
        Option<usize>,
        Option<usize>,
    )>,

    faces: [Vec<TextureUpload<TextureSourceUncompressed>>; 6],

    runtime: Option<Box<Runtime>>,
}

impl Drop for TextureCubeMap {
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

impl TextureCubeMap {
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
        face: CubeMapFace,
        source: TextureSourceUncompressed,
        level: usize,
    ) -> Result<(), Error> {
        self.faces[face as usize].push(TextureUpload::<TextureSourceUncompressed>::with_params_2d(
            source, level, None, None, None, None,
        ));
        Ok(())
    }

    /// Uploads a sub data from a texture source to specified level of this texture.
    pub fn tex_sub_image(
        &mut self,
        face: CubeMapFace,
        source: TextureSourceUncompressed,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        self.faces[face as usize].push(TextureUpload::<TextureSourceUncompressed>::with_params_2d(
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

impl Texture for TextureCubeMap {
    fn target(&self) -> TextureTarget {
        TextureTarget::TEXTURE_CUBE_MAP
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
            used_memory += self.internal_format.bytes_length(width, height) * 6;
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

        Some(self.internal_format.bytes_length(width, height) * 6)
    }
}

impl TexturePlanar for TextureCubeMap {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl TextureInner for TextureCubeMap {
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

    fn upload(&mut self, unit: TextureUnit) -> Result<(), Error> {
        if self.mipmap_base.is_none()
            && self.faces.iter().map(|face| face.len()).sum::<usize>() == 0
        {
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

        // uploads mipmap base source and generates mipmap first if automatic mipmap is enabled
        if let Some((mipmap_sources, base_level, max_level)) = self.mipmap_base.take() {
            let bound_base_level = match base_level {
                Some(base_level) => {
                    let bound =
                        utils::texture_base_level(&runtime.gl, TextureTarget::TEXTURE_CUBE_MAP);
                    runtime.gl.tex_parameteri(
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP,
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
                        utils::texture_max_level(&runtime.gl, TextureTarget::TEXTURE_CUBE_MAP);
                    runtime.gl.tex_parameteri(
                        WebGl2RenderingContext::TEXTURE_CUBE_MAP,
                        WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                        max_level as i32,
                    );
                    bound
                }
                None => None,
            };

            for source in mipmap_sources {
                source.tex_sub_image_2d(&runtime.gl, TextureTarget::TEXTURE_CUBE_MAP)?;
            }
            runtime
                .gl
                .generate_mipmap(WebGl2RenderingContext::TEXTURE_CUBE_MAP);

            if let Some(bound_base_level) = bound_base_level {
                runtime.gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_CUBE_MAP,
                    WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                    bound_base_level as i32,
                );
            }
            if let Some(bound_max_level) = bound_max_level {
                runtime.gl.tex_parameteri(
                    WebGl2RenderingContext::TEXTURE_CUBE_MAP,
                    WebGl2RenderingContext::TEXTURE_BASE_LEVEL,
                    bound_max_level as i32,
                );
            }
        }

        // then uploading all regular sources
        for face in self.faces.iter_mut() {
            for source in face.drain(..) {
                // abilities.verify_texture_size(source.width(), source.height())?;
                source.tex_sub_image_2d(&runtime.gl, TextureTarget::TEXTURE_CUBE_MAP)?;
            }
        }

        runtime.gl.active_texture(bound_unit);
        runtime.gl.bind_texture(
            WebGl2RenderingContext::TEXTURE_CUBE_MAP,
            bound_texture.as_ref(),
        );

        Ok(())
    }

    fn free(&mut self) -> bool {
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

                    mipmap_sources,
                    mipmap_base_level,
                    mipmap_max_level,
                } = restore.as_mut()();
                self.faces[0].extend(positive_x);
                self.faces[1].extend(negative_x);
                self.faces[2].extend(positive_y);
                self.faces[3].extend(negative_y);
                self.faces[4].extend(positive_z);
                self.faces[5].extend(negative_z);
                if let Some(mipmap_sources) = mipmap_sources {
                    self.mipmap_base = Some((mipmap_sources, mipmap_base_level, mipmap_max_level));
                }
                true
            }
        }
    }
}

impl TextureDescriptor<TextureCubeMap> {
    /// Constructs a new texture descriptor with [`TextureCubeMap`] from a [`ConstructionPolicy`] and [`MemoryPolicy`].
    pub fn new(mut construction_policy: ConstructionPolicy, memory_policy: MemoryPolicy) -> Self {
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
                TextureCubeMap {
                    width,
                    height,
                    max_level: Some(
                        <TextureCubeMap as TexturePlanar>::max_available_mipmap_level(
                            width, height,
                        ),
                    ),
                    internal_format,
                    memory_policy,

                    mipmap_base: Some((
                        [
                            TextureUpload::new_2d(positive_x, 0),
                            TextureUpload::new_2d(negative_x, 0),
                            TextureUpload::new_2d(positive_y, 0),
                            TextureUpload::new_2d(negative_y, 0),
                            TextureUpload::new_2d(positive_z, 0),
                            TextureUpload::new_2d(negative_z, 0),
                        ],
                        None,
                        None,
                    )),

                    faces: [
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
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
                    | ConstructionPolicy::WithSources {
                        internal_format,
                        width,
                        height,
                        max_level,
                        ..
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
                                    Some((max_level).min(<TextureCubeMap as TexturePlanar>::max_available_mipmap_level(width, height)))
                                }
                            }
                            None => Some(
                                <TextureCubeMap as TexturePlanar>::max_available_mipmap_level(
                                    width, height,
                                ),
                            ),
                        };
                        (internal_format, width, height, max_level)
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
                let faces = match construction_policy {
                    ConstructionPolicy::Preallocate { .. } => [
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                        Vec::new(),
                    ],
                    ConstructionPolicy::WithSources {
                        positive_x,
                        negative_x,
                        positive_y,
                        negative_y,
                        positive_z,
                        negative_z,
                        ..
                    }
                    | ConstructionPolicy::Full {
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

                TextureCubeMap {
                    width,
                    height,
                    max_level,
                    internal_format,
                    memory_policy,
                    mipmap_base,
                    faces,
                    runtime: None,
                }
            }
        };

        Self(Rc::new(RefCell::new(texture)))
    }
}
