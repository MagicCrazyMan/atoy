use std::{cell::RefCell, rc::Rc};

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::render::webgl::{capabilities::Capabilities, conversion::ToGlEnum, error::Error, utils};

use super::{
    Runtime, Texture, TextureDescriptor, TextureInner, TextureInternalFormat,
    TextureInternalFormatCompressed, TextureInternalFormatUncompressed, TexturePlanar,
    TextureSourceCompressed, TextureSourceUncompressed, TextureTarget, TextureUnit, TextureUpload,
};

/// Memory policies controlling how to manage memory of a texture.
#[allow(private_bounds)]
pub enum MemoryPolicy<F>
where
    F: TextureInternalFormat,
{
    Unfree,
    Restorable(Box<dyn Fn(Builder<F>) -> Builder<F>>),
}

/// A WebGL 2d texture workload.
#[allow(private_bounds)]
pub struct Texture2D<F>
where
    F: TextureInternalFormat,
{
    width: usize,
    height: usize,
    max_level: usize,
    internal_format: F,
    memory_policy: MemoryPolicy<F>,

    mipmap_base: Option<(TextureUpload, Option<usize>, Option<usize>)>,

    uploads: Vec<TextureUpload>,

    runtime: Option<Box<Runtime>>,
}

#[allow(private_bounds)]
impl<F> Texture2D<F>
where
    F: TextureInternalFormat,
{
    /// Returns [`TextureInternalFormat`].
    pub fn internal_format(&self) -> F {
        self.internal_format
    }

    /// Returns [`MemoryPolicy`].
    pub fn memory_policy(&self) -> &MemoryPolicy<F> {
        &self.memory_policy
    }
}

impl Texture2D<TextureInternalFormatUncompressed> {
    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(&mut self, source: TextureSourceUncompressed) -> Result<(), Error> {
        self.uploads.push(TextureUpload::with_params_uncompressed(
            source, None, None, None, None, None, None, None,
        ));
        Ok(())
    }

    /// Uploads a sub data from a texture source to specified level of this texture.
    pub fn tex_sub_image(
        &mut self,
        source: TextureSourceUncompressed,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        self.uploads.push(TextureUpload::with_params_uncompressed(
            source,
            Some(level),
            None,
            Some(width),
            Some(height),
            Some(x_offset),
            Some(y_offset),
            None,
        ));
        Ok(())
    }
}

impl Texture2D<TextureInternalFormatCompressed> {
    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(&mut self, source: TextureSourceCompressed) -> Result<(), Error> {
        self.uploads.push(TextureUpload::with_params_compressed(
            source, None, None, None, None, None, None, None,
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
        self.uploads.push(TextureUpload::with_params_compressed(
            source,
            Some(level),
            None,
            Some(width),
            Some(height),
            Some(x_offset),
            Some(y_offset),
            None,
        ));
        Ok(())
    }
}

impl<F> Texture for Texture2D<F>
where
    F: TextureInternalFormat,
{
    fn target(&self) -> TextureTarget {
        TextureTarget::TEXTURE_2D
    }

    fn max_available_mipmap_level(&self) -> usize {
        <Self as TexturePlanar>::max_available_mipmap_level(self.width, self.height)
    }

    fn max_level(&self) -> usize {
        self.max_level
    }

    fn bytes_length(&self) -> usize {
        let mut used_memory = 0;
        for level in 0..=self.max_level() {
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

impl<F> TexturePlanar for Texture2D<F>
where
    F: TextureInternalFormat,
{
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl<F> TextureInner for Texture2D<F>
where
    F: TextureInternalFormat,
{
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
        self.internal_format.capabilities(capabilities)?;
        Ok(())
    }

    fn create(&self, gl: &WebGl2RenderingContext, _: TextureUnit) -> Result<WebGlTexture, Error> {
        let texture = gl.create_texture().ok_or(Error::CreateTextureFailure)?;
        let bound = utils::texture_binding_2d(gl);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        gl.tex_storage_2d(
            WebGl2RenderingContext::TEXTURE_2D,
            (self.max_level + 1) as i32,
            self.internal_format.gl_enum(),
            self.width as i32,
            self.height as i32,
        );
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, bound.as_ref());
        Ok(texture)
    }

    fn upload(&mut self, unit: TextureUnit) -> Result<(), Error> {
        if self.mipmap_base.is_none() && self.uploads.is_empty() {
            return Ok(());
        }

        let runtime = self.runtime.as_deref().unwrap();

        let bound_texture = utils::texture_binding_2d(&runtime.gl);
        let bound_unit = utils::active_texture_unit(&runtime.gl);

        runtime.gl.active_texture(unit.gl_enum());
        runtime
            .gl
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&runtime.texture));

        // uploads mipmap base source and generates mipmap first if automatic mipmap is enabled
        if let Some((source, base_level, max_level)) = self.mipmap_base.take() {
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

            source.tex_sub_image_2d(&runtime.gl, TextureTarget::TEXTURE_2D)?;
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
                let builder = Builder::new(self.width, self.height, self.internal_format);
                let builder = restore(builder);
                let texture = builder.build();
                self.mipmap_base = texture.mipmap_base;
                self.uploads = texture.uploads;
                true
            }
        }
    }
}

#[allow(private_bounds)]
impl<F> TextureDescriptor<Texture2D<F>>
where
    F: TextureInternalFormat,
{
    /// Constructs a new texture descriptor with [`Texture2D`].
    pub fn new(texture: Texture2D<F>) -> Self {
        Self(Rc::new(RefCell::new(texture)))
    }
}

/// A builder to build a [`Texture2D`].
#[allow(private_bounds)]
pub struct Builder<F>
where
    F: TextureInternalFormat,
{
    internal_format: F,
    width: usize,
    height: usize,
    max_level: usize,
    memory_policy: MemoryPolicy<F>,

    base_source: Option<TextureUpload>,
    uploads: Vec<TextureUpload>,

    mipmap: bool,
    mipmap_base_level: Option<usize>,
    mipmap_max_level: Option<usize>,
}

#[allow(private_bounds)]
impl<F> Builder<F>
where
    F: TextureInternalFormat,
{
    /// Initializes a new builder with specified width, height and internal format.
    pub fn new(width: usize, height: usize, internal_format: F) -> Self {
        Self {
            internal_format,
            width,
            height,
            max_level: <Texture2D<F> as TexturePlanar>::max_available_mipmap_level(width, height),
            memory_policy: MemoryPolicy::Unfree,

            base_source: None,
            uploads: Vec::new(),

            mipmap: false,
            mipmap_base_level: None,
            mipmap_max_level: None,
        }
    }

    pub fn set_max_level(mut self, max_level: usize) -> Self {
        self.max_level = self.max_level.min(max_level);
        self
    }

    /// Sets memory policy. Default memory policy is [`MemoryPolicy::Unfree`].
    pub fn set_memory_policy(mut self, memory_policy: MemoryPolicy<F>) -> Self {
        self.memory_policy = memory_policy;
        self
    }

    /// Builds a [`Texture2D`].
    pub fn build(mut self) -> Texture2D<F> {
        let (mipmap_base, uploads) = match self.base_source {
            Some(base) => {
                if !self.mipmap {
                    self.uploads.insert(0, base);
                    (None, self.uploads)
                } else {
                    (
                        Some((base, self.mipmap_base_level, self.mipmap_max_level)),
                        self.uploads,
                    )
                }
            }
            None => (None, self.uploads),
        };

        Texture2D {
            width: self.width,
            height: self.height,
            max_level: self.max_level,
            internal_format: self.internal_format,
            memory_policy: self.memory_policy,
            mipmap_base,
            uploads,
            runtime: None,
        }
    }
}

impl Builder<TextureInternalFormatUncompressed> {
    /// Initializes a new builder from an existing [`TextureUpload`] and internal format.
    pub fn with_base_source(
        base_source: TextureSourceUncompressed,
        internal_format: TextureInternalFormatUncompressed,
    ) -> Self {
        let width = base_source.width();
        let height = base_source.height();
        Self {
            internal_format,
            width,
            height,
            max_level: <Texture2D<TextureInternalFormatUncompressed> as TexturePlanar>::max_available_mipmap_level(width, height),
            memory_policy: MemoryPolicy::Unfree,

            base_source: Some(TextureUpload::new_uncompressed(base_source)),
            uploads: Vec::new(),

            mipmap: false,
            mipmap_base_level: None,
            mipmap_max_level: None,
        }
    }

    /// Sets the source in level 0.
    pub fn set_base_source(mut self, base: TextureSourceUncompressed) -> Self {
        self.base_source = Some(TextureUpload::new_uncompressed(base));
        self
    }

    /// Enable automatic mipmap generation.
    /// Available only when internal format is one kind of [`TextureInternalFormatUncompressed`](super::TextureInternalFormatUncompressed)
    /// and base source is set.
    ///
    /// Automatic Mipmaps Generation is never enable for [`TextureInternalFormatCompressed`](super::TextureInternalFormatCompressed).
    pub fn generate_mipmap(mut self) -> Self {
        self.mipmap = true;
        self
    }

    /// Sets automatic mipmap generation base level.
    /// Available only when automatic mipmap generation enabled.
    pub fn set_mipmap_base_level(mut self, base_level: usize) -> Self {
        let mut mipmap_base_level = self.max_level.min(base_level);
        if let Some(mipmap_max_level) = self.mipmap_max_level {
            mipmap_base_level = mipmap_base_level.min(mipmap_max_level);
        }

        self.mipmap_base_level = Some(mipmap_base_level);
        self
    }

    /// Sets automatic mipmap generation max level.
    /// Available only when automatic mipmap generation enabled.
    pub fn set_mipmap_max_level(mut self, max_level: usize) -> Self {
        let mut mipmap_max_level = self.max_level.min(max_level);
        if let Some(mipmap_base_level) = self.mipmap_base_level {
            mipmap_max_level = mipmap_base_level.max(mipmap_max_level);
        }

        self.mipmap_max_level = Some(mipmap_max_level);
        self
    }

    /// Uploads a new source to texture.
    pub fn tex_image(mut self, source: TextureSourceUncompressed, level: usize) -> Self {
        self.uploads.push(TextureUpload::with_params_uncompressed(
            source,
            Some(level),
            None,
            None,
            None,
            None,
            None,
            None,
        ));
        self
    }

    /// Uploads a new source for a sub-rectangle of the texture.
    pub fn tex_sub_image(
        mut self,
        source: TextureSourceUncompressed,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Self {
        self.uploads.push(TextureUpload::with_params_uncompressed(
            source,
            Some(level),
            None,
            Some(width),
            Some(height),
            Some(x_offset),
            Some(y_offset),
            None,
        ));
        self
    }
}

impl Builder<TextureInternalFormatCompressed> {
    /// Initializes a new builder from an existing [`TextureUpload`] and internal format.
    pub fn with_base_source(
        base_source: TextureSourceCompressed,
        internal_format: TextureInternalFormatCompressed,
    ) -> Self {
        let width = base_source.width();
        let height = base_source.height();
        Self {
            internal_format,
            width,
            height,
            max_level: <Texture2D<TextureInternalFormatCompressed> as TexturePlanar>::max_available_mipmap_level(width, height),
            memory_policy: MemoryPolicy::Unfree,

            base_source: Some(TextureUpload::new_compressed(base_source)),
            uploads: Vec::new(),

            mipmap: false,
            mipmap_base_level: None,
            mipmap_max_level: None,
        }
    }

    /// Sets the source in level 0.
    pub fn set_base_source(mut self, base_source: TextureSourceCompressed) -> Self {
        self.base_source = Some(TextureUpload::new_compressed(base_source));
        self
    }

    /// Uploads a new image to texture.
    pub fn tex_image(mut self, source: TextureSourceCompressed, level: usize) -> Self {
        self.uploads.push(TextureUpload::with_params_compressed(
            source,
            Some(level),
            None,
            None,
            None,
            None,
            None,
            None,
        ));
        self
    }

    /// Uploads a new image for a sub-rectangle of the texture.
    pub fn tex_sub_image(
        mut self,
        source: TextureSourceCompressed,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Self {
        self.uploads.push(TextureUpload::with_params_compressed(
            source,
            Some(level),
            None,
            Some(width),
            Some(height),
            Some(x_offset),
            Some(y_offset),
            None,
        ));
        self
    }
}
