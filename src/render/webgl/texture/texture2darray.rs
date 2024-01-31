use std::{cell::RefCell, rc::Rc};

use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::render::webgl::{capabilities::Capabilities, conversion::ToGlEnum, error::Error, utils};

use super::{
    NativeFormat, Runtime, SamplerParameter, Texture, TextureArray, TextureCompressedFormat,
    TextureDescriptor, TextureInternalFormat, TextureItem, TextureParameter, TexturePlanar,
    TextureSource, TextureSourceCompressed, TextureTarget, TextureUnit, UploadItem,
};

/// Memory policies controlling how to manage memory of a texture.
pub enum MemoryPolicy<F> {
    Unfree,
    Restorable(Box<dyn Fn(Builder<F>) -> Builder<F>>),
}

/// A WebGL 2d array texture workload.
///
/// Different from [`WebGl2RenderingContext::TEXTURE_3D`],
/// a depth value in [`WebGl2RenderingContext::TEXTURE_2D_ARRAY`] refers to the length of array, not a dimensional value.
pub struct Texture2DArray<F> {
    width: usize,
    height: usize,
    array_length: usize,
    max_level: usize,
    internal_format: F,
    memory_policy: MemoryPolicy<F>,
    sampler_params: Vec<SamplerParameter>,
    tex_params: Vec<TextureParameter>,

    mipmap_base: Option<(UploadItem, Option<usize>, Option<usize>)>,

    uploads: Vec<UploadItem>,

    runtime: Option<Box<Runtime>>,
}

#[allow(private_bounds)]
impl<F> Texture2DArray<F>
where
    F: NativeFormat,
{
    /// Returns internal format.
    pub fn internal_format(&self) -> F {
        self.internal_format
    }

    /// Returns [`MemoryPolicy`].
    pub fn memory_policy(&self) -> &MemoryPolicy<F> {
        &self.memory_policy
    }

    /// Sets sampler parameters.
    pub fn set_sampler_parameters<I>(&mut self, params: I)
    where
        I: IntoIterator<Item = SamplerParameter>,
    {
        self.sampler_params = params.into_iter().collect();
        if let Some(runtime) = self.runtime.as_deref_mut() {
            let sampler = runtime.sampler.take();
            runtime.gl.delete_sampler(sampler.as_ref());
        }
    }

    /// Sets texture parameters.
    pub fn set_texture_parameters<I>(&mut self, params: I)
    where
        I: IntoIterator<Item = TextureParameter>,
    {
        self.tex_params = params.into_iter().collect();
    }
}

impl Texture2DArray<TextureInternalFormat> {
    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(
        &mut self,
        source: TextureSource,
        level: usize,
        array_length: usize,
    ) -> Result<(), Error> {
        self.uploads.push(UploadItem::with_params_uncompressed(
            source,
            Some(level),
            Some(array_length),
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
        source: TextureSource,
        level: usize,
        array_length: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) -> Result<(), Error> {
        self.uploads.push(UploadItem::with_params_uncompressed(
            source,
            Some(level),
            Some(array_length),
            Some(width),
            Some(height),
            Some(x_offset),
            Some(y_offset),
            Some(z_offset),
        ));
        Ok(())
    }
}

impl Texture2DArray<TextureCompressedFormat> {
    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(
        &mut self,
        source: TextureSourceCompressed,
        level: usize,
        array_length: usize,
    ) -> Result<(), Error> {
        self.uploads.push(UploadItem::with_params_compressed(
            source,
            Some(level),
            Some(array_length),
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
        source: TextureSourceCompressed,
        level: usize,
        array_length: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) -> Result<(), Error> {
        self.uploads.push(UploadItem::with_params_compressed(
            source,
            Some(level),
            Some(array_length),
            Some(width),
            Some(height),
            Some(x_offset),
            Some(y_offset),
            Some(z_offset),
        ));
        Ok(())
    }
}

impl<F> Texture for Texture2DArray<F>
where
    F: NativeFormat,
{
    fn target(&self) -> TextureTarget {
        TextureTarget::TEXTURE_2D_ARRAY
    }

    fn sampler_parameters(&self) -> &[SamplerParameter] {
        &self.sampler_params
    }

    fn texture_parameters(&self) -> &[TextureParameter] {
        &self.tex_params
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

impl<F> TexturePlanar for Texture2DArray<F>
where
    F: NativeFormat,
{
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }
}

impl<F> TextureArray for Texture2DArray<F>
where
    F: NativeFormat,
{
    fn array_length(&self) -> usize {
        self.array_length
    }
}

impl<F> TextureItem for Texture2DArray<F>
where
    F: NativeFormat,
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

    fn create_texture(&self, gl: &WebGl2RenderingContext) -> Result<WebGlTexture, Error> {
        let texture = gl.create_texture().ok_or(Error::CreateTextureFailure)?;
        let bound = utils::texture_binding_2d_array(gl);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D_ARRAY, Some(&texture));
        gl.tex_storage_3d(
            WebGl2RenderingContext::TEXTURE_2D_ARRAY,
            (self.max_level + 1) as i32,
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
                    let bound = utils::texture_parameter_base_level(
                        &runtime.gl,
                        TextureTarget::TEXTURE_2D_ARRAY,
                    );
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
                    let bound = utils::texture_parameter_max_level(
                        &runtime.gl,
                        TextureTarget::TEXTURE_2D_ARRAY,
                    );
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
                let builder = Builder::new(
                    self.width,
                    self.height,
                    self.array_length,
                    self.internal_format,
                );
                let builder = restore(builder);
                let texture = builder.build();
                self.mipmap_base = texture.mipmap_base;
                self.sampler_params = texture.sampler_params;
                self.tex_params = texture.tex_params;
                self.uploads = texture.uploads;
                true
            }
        }
    }
}

impl<F> TextureDescriptor<Texture2DArray<F>> {
    /// Constructs a new texture descriptor with [`Texture2DArray`].
    pub fn new(texture: Texture2DArray<F>) -> Self {
        Self(Rc::new(RefCell::new(texture)))
    }
}

/// A builder to build a [`Texture2DArray`].
pub struct Builder<F> {
    internal_format: F,
    width: usize,
    height: usize,
    array_length: usize,
    max_level: usize,
    memory_policy: MemoryPolicy<F>,
    sampler_params: Vec<SamplerParameter>,
    tex_params: Vec<TextureParameter>,

    base_source: Option<UploadItem>,
    uploads: Vec<UploadItem>,

    mipmap: bool,
    mipmap_base_level: Option<usize>,
    mipmap_max_level: Option<usize>,
}

#[allow(private_bounds)]
impl<F> Builder<F>
where
    F: NativeFormat,
{
    /// Initializes a new builder with specified width, height and internal format.
    pub fn new(width: usize, height: usize, array_length: usize, internal_format: F) -> Self {
        Self {
            internal_format,
            width,
            height,
            array_length,
            max_level: <Texture2DArray<F> as TexturePlanar>::max_available_mipmap_level(
                width, height,
            ),
            memory_policy: MemoryPolicy::Unfree,
            sampler_params: Vec::new(),
            tex_params: Vec::new(),

            base_source: None,
            uploads: Vec::new(),

            mipmap: false,
            mipmap_base_level: None,
            mipmap_max_level: None,
        }
    }

    /// Sets max mipmap level. Max mipmap level is clamped to [`Texture2D::max_available_mipmap_level`].
    pub fn set_max_level(mut self, max_level: usize) -> Self {
        self.max_level = self.max_level.min(max_level);
        self
    }

    /// Sets [`SamplerParameter`]s.
    pub fn set_sampler_parameters<I>(mut self, params: I) -> Self
    where
        I: IntoIterator<Item = SamplerParameter>,
    {
        self.sampler_params = params.into_iter().collect();
        self
    }

    /// Sets [`TextureParameter`]s.
    pub fn set_texture_parameters<I>(mut self, params: I) -> Self
    where
        I: IntoIterator<Item = TextureParameter>,
    {
        self.tex_params = params.into_iter().collect();
        self
    }

    /// Sets memory policy. Default memory policy is [`MemoryPolicy::Unfree`].
    pub fn set_memory_policy(mut self, memory_policy: MemoryPolicy<F>) -> Self {
        self.memory_policy = memory_policy;
        self
    }

    /// Builds a [`Texture2DArray`].
    pub fn build(mut self) -> Texture2DArray<F> {
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

        Texture2DArray {
            width: self.width,
            height: self.height,
            array_length: self.array_length,
            max_level: self.max_level,
            internal_format: self.internal_format,
            memory_policy: self.memory_policy,
            sampler_params: self.sampler_params,
            tex_params: self.tex_params,
            mipmap_base,
            uploads,
            runtime: None,
        }
    }
}

impl Builder<TextureInternalFormat> {
    /// Initializes a new builder from an existing [`TextureSource`] and [`TextureInternalFormat`].
    /// Beside, a array length is also required.
    pub fn with_base_source(
        array_length: usize,
        base_source: TextureSource,
        internal_format: TextureInternalFormat,
    ) -> Self {
        let width = base_source.width();
        let height = base_source.height();
        Self {
            internal_format,
            width,
            height,
            array_length,
            max_level:
                <Texture2DArray<TextureInternalFormat> as TexturePlanar>::max_available_mipmap_level(
                    width, height,
                ),
            memory_policy: MemoryPolicy::Unfree,
            sampler_params: Vec::new(),
            tex_params: Vec::new(),

            base_source: Some(UploadItem::new_uncompressed(base_source)),
            uploads: Vec::new(),

            mipmap: false,
            mipmap_base_level: None,
            mipmap_max_level: None,
        }
    }

    /// Sets the source in level 0.
    pub fn set_base_source(mut self, base: TextureSource) -> Self {
        self.base_source = Some(UploadItem::new_uncompressed(base));
        self
    }

    /// Enable automatic mipmap generation.
    /// Available only when internal format is one kind of [`TextureInternalFormat`](super::TextureInternalFormat)
    /// and base source is set.
    ///
    /// Automatic Mipmaps Generation is never enable for [`TextureCompressedFormat`](super::TextureCompressedFormat).
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
    pub fn tex_image(mut self, source: TextureSource, level: usize) -> Self {
        self.uploads.push(UploadItem::with_params_uncompressed(
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
        source: TextureSource,
        level: usize,
        array_length: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) -> Self {
        self.uploads.push(UploadItem::with_params_uncompressed(
            source,
            Some(level),
            Some(array_length),
            Some(width),
            Some(height),
            Some(x_offset),
            Some(y_offset),
            Some(z_offset),
        ));
        self
    }
}

impl Builder<TextureCompressedFormat> {
    /// Initializes a new builder from an existing [`TextureSourceCompressed`] and [`TextureCompressedFormat`].
    /// Beside, a array length is also required.
    pub fn with_base_source(
        array_length: usize,
        base_source: TextureSourceCompressed,
        internal_format: TextureCompressedFormat,
    ) -> Self {
        let width = base_source.width();
        let height = base_source.height();
        Self {
            internal_format,
            width,
            height,
            array_length,
            max_level:
                <Texture2DArray<TextureInternalFormat> as TexturePlanar>::max_available_mipmap_level(
                    width, height,
                ),
            memory_policy: MemoryPolicy::Unfree,
            sampler_params: Vec::new(),
            tex_params: Vec::new(),

            base_source: Some(UploadItem::new_compressed(base_source)),
            uploads: Vec::new(),

            mipmap: false,
            mipmap_base_level: None,
            mipmap_max_level: None,
        }
    }

    /// Sets the source in level 0.
    pub fn set_base_source(mut self, base_source: TextureSourceCompressed) -> Self {
        self.base_source = Some(UploadItem::new_compressed(base_source));
        self
    }

    /// Uploads a new image to texture.
    pub fn tex_image(mut self, source: TextureSourceCompressed, level: usize) -> Self {
        self.uploads.push(UploadItem::with_params_compressed(
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
        depth: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
        z_offset: usize,
    ) -> Self {
        self.uploads.push(UploadItem::with_params_compressed(
            source,
            Some(level),
            Some(depth),
            Some(width),
            Some(height),
            Some(x_offset),
            Some(y_offset),
            Some(z_offset),
        ));
        self
    }
}
