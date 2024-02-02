use std::iter::FromIterator;

use hashbrown::HashMap;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::render::webgl::{capabilities::Capabilities, conversion::ToGlEnum, error::Error, utils};

use super::{
    NativeFormat, Runtime, SamplerParameter, Texture, TextureCompressedFormat,
    TextureInternalFormat, TextureItem, TextureParameter, TexturePlanar, TextureSource,
    TextureSourceCompressed, TextureTarget, TextureUploadTarget, UploadItem,
};

/// Memory policies controlling how to manage memory of a texture.
pub enum MemoryPolicy<F> {
    Unfree,
    Restorable(Box<dyn Fn(Builder<F>) -> Builder<F>>),
}

/// A WebGL 2d texture workload.
pub struct Texture2DBase<F> {
    width: usize,
    height: usize,
    max_mipmap_level: usize,
    internal_format: F,
    memory_policy: MemoryPolicy<F>,
    sampler_params: HashMap<u32, SamplerParameter>,
    tex_params: HashMap<u32, TextureParameter>,

    mipmap_base: Option<UploadItem>,

    uploads: Vec<UploadItem>,

    runtime: Option<Box<Runtime>>,
}

#[allow(private_bounds)]
impl<F> Texture2DBase<F>
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

    /// Sets sampler parameter.
    pub fn set_sampler_parameter(&mut self, param: SamplerParameter) {
        if let Some(runtime) = self.runtime.as_deref_mut() {
            param.sampler_parameter(&runtime.gl, &runtime.sampler);
        }
        self.sampler_params.insert(param.gl_enum(), param);
    }

    /// Sets texture parameter.
    pub fn set_texture_parameter(&mut self, param: TextureParameter) -> Result<(), Error> {
        if let Some(runtime) = self.runtime.as_deref_mut() {
            let bound = utils::texture_binding_2d(&runtime.gl);
            runtime
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&runtime.texture));
            param.tex_parameter(
                &runtime.gl,
                TextureTarget::TEXTURE_2D,
                &runtime.capabilities,
            )?;
            runtime
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, bound.as_ref());
        }
        self.tex_params.insert(param.gl_enum(), param);

        Ok(())
    }
}

impl Texture2DBase<TextureInternalFormat> {
    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(&mut self, source: TextureSource) -> Result<(), Error> {
        self.uploads.push(UploadItem::with_params_uncompressed(
            source, None, None, None, None, None, None, None,
        ));
        Ok(())
    }

    /// Uploads a sub data from a texture source to specified level of this texture.
    pub fn tex_sub_image(
        &mut self,
        source: TextureSource,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        self.uploads.push(UploadItem::with_params_uncompressed(
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

impl Texture2DBase<TextureCompressedFormat> {
    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(&mut self, source: TextureSourceCompressed) -> Result<(), Error> {
        self.uploads.push(UploadItem::with_params_compressed(
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
        self.uploads.push(UploadItem::with_params_compressed(
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

impl<F> Texture for Texture2DBase<F>
where
    F: NativeFormat,
{
    fn target(&self) -> TextureTarget {
        TextureTarget::TEXTURE_2D
    }

    fn sampler_parameters(&self) -> &HashMap<u32, SamplerParameter> {
        &self.sampler_params
    }

    fn texture_parameters(&self) -> &HashMap<u32, TextureParameter> {
        &self.tex_params
    }

    fn max_available_mipmap_level(&self) -> usize {
        <Self as TexturePlanar>::max_available_mipmap_level(self.width, self.height)
    }

    fn max_mipmap_level(&self) -> usize {
        self.max_mipmap_level
    }

    fn bytes_length(&self) -> usize {
        let mut used_memory = 0;
        for level in 0..=self.max_mipmap_level() {
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

impl<F> TexturePlanar for Texture2DBase<F>
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

impl<F> TextureItem for Texture2DBase<F>
where
    F: NativeFormat,
{
    fn runtime(&self) -> Option<&Runtime> {
        self.runtime.as_deref()
    }

    fn runtime_unchecked(&self) -> &Runtime {
        self.runtime.as_deref().unwrap()
    }

    fn runtime_mut(&mut self) -> Option<&mut Runtime> {
        self.runtime.as_deref_mut()
    }

    fn runtime_mut_unchecked(&mut self) -> &mut Runtime {
        self.runtime.as_deref_mut().unwrap()
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
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        gl.tex_storage_2d(
            WebGl2RenderingContext::TEXTURE_2D,
            (self.max_mipmap_level + 1) as i32,
            self.internal_format.gl_enum(),
            self.width as i32,
            self.height as i32,
        );
        Ok(texture)
    }

    fn upload(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        if self.mipmap_base.is_none() && self.uploads.is_empty() {
            return Ok(());
        }

        // uploads mipmap base source and generates mipmap first if automatic mipmap is enabled
        if let Some(source) = self.mipmap_base.take() {
            source.tex_sub_image_2d(&gl, TextureUploadTarget::TEXTURE_2D)?;
            gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
        }

        // then uploading all regular sources
        for upload in self.uploads.drain(..) {
            // abilities.verify_texture_size(source.width(), source.height())?;
            upload.tex_sub_image_2d(&gl, TextureUploadTarget::TEXTURE_2D)?;
        }

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
                self.sampler_params = texture.sampler_params;
                self.tex_params = texture.tex_params;
                self.uploads = texture.uploads;
                true
            }
        }
    }
}

/// A builder to build a [`Texture2DBase`].
pub struct Builder<F> {
    internal_format: F,
    width: usize,
    height: usize,
    max_mipmap_level: usize,
    memory_policy: MemoryPolicy<F>,
    sampler_params: HashMap<u32, SamplerParameter>,
    tex_params: HashMap<u32, TextureParameter>,

    base_source: Option<UploadItem>,
    uploads: Vec<UploadItem>,

    mipmap: bool,
}

#[allow(private_bounds)]
impl<F> Builder<F>
where
    F: NativeFormat,
{
    /// Initializes a new builder with specified width, height and internal format.
    pub fn new(width: usize, height: usize, internal_format: F) -> Self {
        Self {
            internal_format,
            width,
            height,
            max_mipmap_level: <Texture2DBase<F> as TexturePlanar>::max_available_mipmap_level(
                width, height,
            ),
            memory_policy: MemoryPolicy::Unfree,
            sampler_params: HashMap::new(),
            tex_params: HashMap::new(),

            base_source: None,
            uploads: Vec::new(),

            mipmap: false,
        }
    }

    /// Sets max mipmap level. Max mipmap level is clamped to [`Texture2DBase::max_available_mipmap_level`].
    ///
    /// When calling `texStorage2D` or `texStorage3D`, levels params equals to `max_mipmap_level + 1`.
    pub fn set_max_mipmap_level(mut self, max_mipmap_level: usize) -> Self {
        self.max_mipmap_level = self.max_mipmap_level.min(max_mipmap_level);
        self
    }

    /// Sets [`SamplerParameter`]s.
    pub fn set_sampler_parameters<I>(mut self, params: I) -> Self
    where
        I: IntoIterator<Item = SamplerParameter>,
    {
        self.sampler_params = HashMap::from_iter(params.into_iter().map(|p| (p.gl_enum(), p)));
        self
    }

    /// Sets [`TextureParameter`]s.
    pub fn set_texture_parameters<I>(mut self, params: I) -> Self
    where
        I: IntoIterator<Item = TextureParameter>,
    {
        self.tex_params = HashMap::from_iter(params.into_iter().map(|p| (p.gl_enum(), p)));
        self
    }

    /// Sets memory policy. Default memory policy is [`MemoryPolicy::Unfree`].
    pub fn set_memory_policy(mut self, memory_policy: MemoryPolicy<F>) -> Self {
        self.memory_policy = memory_policy;
        self
    }

    /// Builds a [`Texture2DBase`].
    pub fn build(mut self) -> Texture2DBase<F> {
        let (mipmap_base, uploads) = match self.base_source {
            Some(base) => {
                if !self.mipmap {
                    self.uploads.insert(0, base);
                    (None, self.uploads)
                } else {
                    (Some(base), self.uploads)
                }
            }
            None => (None, self.uploads),
        };

        Texture2DBase {
            width: self.width,
            height: self.height,
            max_mipmap_level: self.max_mipmap_level,
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
    pub fn with_base_source(
        base_source: TextureSource,
        internal_format: TextureInternalFormat,
    ) -> Self {
        let width = base_source.width();
        let height = base_source.height();
        Self {
            internal_format,
            width,
            height,
            max_mipmap_level:
                <Texture2DBase<TextureInternalFormat> as TexturePlanar>::max_available_mipmap_level(
                    width, height,
                ),
            memory_policy: MemoryPolicy::Unfree,
            sampler_params: HashMap::new(),
            tex_params: HashMap::new(),

            base_source: Some(UploadItem::new_uncompressed(base_source)),
            uploads: Vec::new(),

            mipmap: false,
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
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Self {
        self.uploads.push(UploadItem::with_params_uncompressed(
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

impl Builder<TextureCompressedFormat> {
    /// Initializes a new builder from an existing [`TextureSourceCompressed`] and [`TextureCompressedFormat`].
    pub fn with_base_source(
        base_source: TextureSourceCompressed,
        internal_format: TextureCompressedFormat,
    ) -> Self {
        let width = base_source.width();
        let height = base_source.height();
        Self {
            internal_format,
            width,
            height,
            max_mipmap_level:
                <Texture2DBase<TextureCompressedFormat> as TexturePlanar>::max_available_mipmap_level(
                    width, height,
                ),
            memory_policy: MemoryPolicy::Unfree,
            sampler_params: HashMap::new(),
            tex_params: HashMap::new(),

            base_source: Some(UploadItem::new_compressed(base_source)),
            uploads: Vec::new(),

            mipmap: false,
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
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Self {
        self.uploads.push(UploadItem::with_params_compressed(
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

/// A common texture 2d including [`TextureInternalFormat`] and [`TextureCompressedFormat`] formats.
pub enum Texture2D {
    Uncompressed(Texture2DBase<TextureInternalFormat>),
    Compressed(Texture2DBase<TextureCompressedFormat>),
}

impl Texture for Texture2D {
    fn target(&self) -> TextureTarget {
        match self {
            Texture2D::Uncompressed(t) => t.target(),
            Texture2D::Compressed(t) => t.target(),
        }
    }

    fn sampler_parameters(&self) -> &HashMap<u32, SamplerParameter> {
        match self {
            Texture2D::Uncompressed(t) => t.sampler_parameters(),
            Texture2D::Compressed(t) => t.sampler_parameters(),
        }
    }

    fn texture_parameters(&self) -> &HashMap<u32, TextureParameter> {
        match self {
            Texture2D::Uncompressed(t) => t.texture_parameters(),
            Texture2D::Compressed(t) => t.texture_parameters(),
        }
    }

    fn max_available_mipmap_level(&self) -> usize {
        match self {
            Texture2D::Uncompressed(t) => t.max_available_mipmap_level(),
            Texture2D::Compressed(t) => t.max_available_mipmap_level(),
        }
    }

    fn max_mipmap_level(&self) -> usize {
        match self {
            Texture2D::Uncompressed(t) => t.max_mipmap_level(),
            Texture2D::Compressed(t) => t.max_mipmap_level(),
        }
    }

    fn bytes_length(&self) -> usize {
        match self {
            Texture2D::Uncompressed(t) => t.bytes_length(),
            Texture2D::Compressed(t) => t.bytes_length(),
        }
    }

    fn bytes_length_of_level(&self, level: usize) -> Option<usize> {
        match self {
            Texture2D::Uncompressed(t) => t.bytes_length_of_level(level),
            Texture2D::Compressed(t) => t.bytes_length_of_level(level),
        }
    }
}

impl TextureItem for Texture2D {
    fn runtime(&self) -> Option<&Runtime> {
        match self {
            Texture2D::Uncompressed(t) => t.runtime(),
            Texture2D::Compressed(t) => t.runtime(),
        }
    }

    fn runtime_unchecked(&self) -> &Runtime {
        match self {
            Texture2D::Uncompressed(t) => t.runtime_unchecked(),
            Texture2D::Compressed(t) => t.runtime_unchecked(),
        }
    }

    fn runtime_mut(&mut self) -> Option<&mut Runtime> {
        match self {
            Texture2D::Uncompressed(t) => t.runtime_mut(),
            Texture2D::Compressed(t) => t.runtime_mut(),
        }
    }

    fn runtime_mut_unchecked(&mut self) -> &mut Runtime {
        match self {
            Texture2D::Uncompressed(t) => t.runtime_mut_unchecked(),
            Texture2D::Compressed(t) => t.runtime_mut_unchecked(),
        }
    }

    fn set_runtime(&mut self, runtime: Runtime) {
        match self {
            Texture2D::Uncompressed(t) => t.set_runtime(runtime),
            Texture2D::Compressed(t) => t.set_runtime(runtime),
        }
    }

    fn remove_runtime(&mut self) -> Option<Runtime> {
        match self {
            Texture2D::Uncompressed(t) => t.remove_runtime(),
            Texture2D::Compressed(t) => t.remove_runtime(),
        }
    }

    fn validate(&self, capabilities: &Capabilities) -> Result<(), Error> {
        match self {
            Texture2D::Uncompressed(t) => t.validate(capabilities),
            Texture2D::Compressed(t) => t.validate(capabilities),
        }
    }

    fn create_texture(&self, gl: &WebGl2RenderingContext) -> Result<WebGlTexture, Error> {
        match self {
            Texture2D::Uncompressed(t) => t.create_texture(gl),
            Texture2D::Compressed(t) => t.create_texture(gl),
        }
    }

    fn upload(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        match self {
            Texture2D::Uncompressed(t) => t.upload(gl),
            Texture2D::Compressed(t) => t.upload(gl),
        }
    }

    fn free(&mut self) -> bool {
        match self {
            Texture2D::Uncompressed(t) => t.free(),
            Texture2D::Compressed(t) => t.free(),
        }
    }
}
