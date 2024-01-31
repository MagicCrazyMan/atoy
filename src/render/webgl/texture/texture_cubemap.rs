use std::{iter::FromIterator, marker::PhantomData};

use hashbrown::HashMap;
use web_sys::{WebGl2RenderingContext, WebGlTexture};

use crate::render::webgl::{capabilities::Capabilities, conversion::ToGlEnum, error::Error, utils};

use super::{
    NativeFormat, Runtime, SamplerParameter, Texture, TextureCompressedFormat,
    TextureInternalFormat, TextureItem, TextureParameter, TexturePlanar, TextureSource,
    TextureSourceCompressed, TextureTarget, TextureUploadTarget, UploadItem,
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

/// Memory policies controlling how to manage memory of a texture.
pub enum MemoryPolicy<F> {
    Unfree,
    Restorable(Box<dyn Fn(Builder<F>) -> Builder<F>>),
}

/// A WebGL cube map texture workload.
pub struct TextureCubeMap<F> {
    width: usize,
    height: usize,
    max_level: usize,
    internal_format: F,
    memory_policy: MemoryPolicy<F>,
    sampler_params: HashMap<u32, SamplerParameter>,
    tex_params: HashMap<u32, TextureParameter>,

    mipmap_base: Option<[UploadItem; 6]>,

    faces: [Vec<UploadItem>; 6],

    runtime: Option<Box<Runtime>>,
}

#[allow(private_bounds)]
impl<F> TextureCubeMap<F>
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
            let bound = utils::texture_binding_cube_map(&runtime.gl);
            runtime
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_CUBE_MAP, Some(&runtime.texture));
            param.tex_parameter(
                &runtime.gl,
                TextureTarget::TEXTURE_CUBE_MAP,
                &runtime.capabilities,
            )?;
            runtime
                .gl
                .bind_texture(WebGl2RenderingContext::TEXTURE_CUBE_MAP, bound.as_ref());
        }
        self.tex_params.insert(param.gl_enum(), param);

        Ok(())
    }
}

impl TextureCubeMap<TextureInternalFormat> {
    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(
        &mut self,
        face: CubeMapFace,
        source: TextureSource,
        level: usize,
    ) -> Result<(), Error> {
        self.faces[face as usize].push(UploadItem::with_params_uncompressed(
            source,
            Some(level),
            None,
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
        face: CubeMapFace,
        source: TextureSource,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        self.faces[face as usize].push(UploadItem::with_params_uncompressed(
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

impl TextureCubeMap<TextureCompressedFormat> {
    /// Uploads a new texture source cover a whole level of this texture.
    pub fn tex_image(
        &mut self,
        face: CubeMapFace,
        source: TextureSourceCompressed,
        level: usize,
    ) -> Result<(), Error> {
        self.faces[face as usize].push(UploadItem::with_params_compressed(
            source,
            Some(level),
            None,
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
        face: CubeMapFace,
        source: TextureSourceCompressed,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Result<(), Error> {
        self.faces[face as usize].push(UploadItem::with_params_compressed(
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

impl<F> Texture for TextureCubeMap<F>
where
    F: NativeFormat,
{
    fn target(&self) -> TextureTarget {
        TextureTarget::TEXTURE_CUBE_MAP
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

    fn max_level(&self) -> usize {
        self.max_level
    }

    fn bytes_length(&self) -> usize {
        let mut used_memory = 0;
        for level in 0..=self.max_level() {
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

impl<F> TexturePlanar for TextureCubeMap<F>
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

impl<F> TextureItem for TextureCubeMap<F>
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
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_CUBE_MAP, Some(&texture));
        gl.tex_storage_2d(
            WebGl2RenderingContext::TEXTURE_CUBE_MAP,
            (self.max_level + 1) as i32,
            self.internal_format.gl_enum(),
            self.width as i32,
            self.height as i32,
        );
        Ok(texture)
    }

    fn upload(&mut self, gl: &WebGl2RenderingContext) -> Result<(), Error> {
        if self.mipmap_base.is_none()
            && self.faces.iter().map(|face| face.len()).sum::<usize>() == 0
        {
            return Ok(());
        }

        // uploads mipmap base source and generates mipmap first if automatic mipmap is enabled
        if let Some(sources) = self.mipmap_base.take() {
            for (index, source) in sources.iter().enumerate() {
                source.tex_sub_image_2d(&gl, TextureUploadTarget::from_index(index))?;
            }
            gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_CUBE_MAP);
        }

        // then uploading all regular sources
        for (index, sources) in self.faces.iter_mut().enumerate() {
            for source in sources.drain(..) {
                // abilities.verify_texture_size(source.width(), source.height())?;
                source.tex_sub_image_2d(&gl, TextureUploadTarget::from_index(index))?;
            }
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
                self.faces = texture.faces;
                true
            }
        }
    }
}

/// Special trait for implements methods for builder.
pub struct Restore<F>(PhantomData<F>);

/// A builder to build a [`TextureCubeMap`].
pub struct Builder<F> {
    internal_format: F,
    width: usize,
    height: usize,
    max_level: usize,
    memory_policy: MemoryPolicy<F>,
    sampler_params: HashMap<u32, SamplerParameter>,
    tex_params: HashMap<u32, TextureParameter>,

    base_source: Option<[Option<UploadItem>; 6]>,
    faces: [Vec<UploadItem>; 6],

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
            max_level: <TextureCubeMap<F> as TexturePlanar>::max_available_mipmap_level(
                width, height,
            ),
            memory_policy: MemoryPolicy::Unfree,
            sampler_params: HashMap::new(),
            tex_params: HashMap::new(),

            base_source: None,
            faces: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],

            mipmap: false,
        }
    }

    /// Sets max mipmap level. Max mipmap level is clamped to [`TextureCubeMap::max_available_mipmap_level`].
    pub fn set_max_level(mut self, max_level: usize) -> Self {
        self.max_level = self.max_level.min(max_level);
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

    /// Builds a [`TextureCubeMap`].
    pub fn build(mut self) -> TextureCubeMap<F> {
        let (mipmap_base, faces) = match self.base_source {
            Some(mut bases) => {
                if !self.mipmap {
                    for index in 0..bases.len() {
                        self.faces[index].insert(0, bases[index].take().unwrap());
                    }
                    (None, self.faces)
                } else {
                    (
                        Some([
                            bases[0].take().unwrap(),
                            bases[1].take().unwrap(),
                            bases[2].take().unwrap(),
                            bases[3].take().unwrap(),
                            bases[4].take().unwrap(),
                            bases[5].take().unwrap(),
                        ]),
                        self.faces,
                    )
                }
            }
            None => (None, self.faces),
        };

        TextureCubeMap {
            width: self.width,
            height: self.height,
            max_level: self.max_level,
            internal_format: self.internal_format,
            memory_policy: self.memory_policy,
            sampler_params: self.sampler_params,
            tex_params: self.tex_params,
            mipmap_base,
            faces,
            runtime: None,
        }
    }
}

impl Builder<TextureInternalFormat> {
    /// Initializes a new builder from existing [`TextureSource`]s of each face and [`TextureInternalFormat`].
    pub fn with_base_source(
        positive_x: TextureSource,
        negative_x: TextureSource,
        positive_y: TextureSource,
        negative_y: TextureSource,
        positive_z: TextureSource,
        negative_z: TextureSource,
        internal_format: TextureInternalFormat,
    ) -> Self {
        let width = positive_x.width();
        let height = positive_x.height();
        Self {
            internal_format,
            width,
            height,
            max_level:
                <TextureCubeMap<TextureInternalFormat> as TexturePlanar>::max_available_mipmap_level(
                    width, height,
                ),
            memory_policy: MemoryPolicy::Unfree,
            sampler_params: HashMap::new(),
            tex_params: HashMap::new(),

            base_source: Some([
                Some(UploadItem::new_uncompressed(positive_x)),
                Some(UploadItem::new_uncompressed(negative_x)),
                Some(UploadItem::new_uncompressed(positive_y)),
                Some(UploadItem::new_uncompressed(negative_y)),
                Some(UploadItem::new_uncompressed(positive_z)),
                Some(UploadItem::new_uncompressed(negative_z)),
            ]),
            faces: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],

            mipmap: false,
        }
    }

    /// Sets the sources of each face in level 0.
    pub fn set_base_source(
        mut self,
        positive_x: TextureSource,
        negative_x: TextureSource,
        positive_y: TextureSource,
        negative_y: TextureSource,
        positive_z: TextureSource,
        negative_z: TextureSource,
    ) -> Self {
        self.base_source = Some([
            Some(UploadItem::new_uncompressed(positive_x)),
            Some(UploadItem::new_uncompressed(negative_x)),
            Some(UploadItem::new_uncompressed(positive_y)),
            Some(UploadItem::new_uncompressed(negative_y)),
            Some(UploadItem::new_uncompressed(positive_z)),
            Some(UploadItem::new_uncompressed(negative_z)),
        ]);
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

    /// Uploads a new source to a specified cube map face.
    pub fn tex_image(mut self, face: CubeMapFace, source: TextureSource, level: usize) -> Self {
        self.faces[face as usize].push(UploadItem::with_params_uncompressed(
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

    /// Uploads a new source for a sub-rectangle to a specified cube map face.
    pub fn tex_sub_image(
        mut self,
        face: CubeMapFace,
        source: TextureSource,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Self {
        self.faces[face as usize].push(UploadItem::with_params_uncompressed(
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
        positive_x: TextureSourceCompressed,
        negative_x: TextureSourceCompressed,
        positive_y: TextureSourceCompressed,
        negative_y: TextureSourceCompressed,
        positive_z: TextureSourceCompressed,
        negative_z: TextureSourceCompressed,
        internal_format: TextureCompressedFormat,
    ) -> Self {
        let width = positive_x.width();
        let height = positive_x.height();
        Self {
            internal_format,
            width,
            height,
            max_level:
                <TextureCubeMap<TextureCompressedFormat> as TexturePlanar>::max_available_mipmap_level(
                    width, height,
                ),
            memory_policy: MemoryPolicy::Unfree,
            sampler_params: HashMap::new(),
            tex_params: HashMap::new(),

            base_source: Some([
                Some(UploadItem::new_compressed(positive_x)),
                Some(UploadItem::new_compressed(negative_x)),
                Some(UploadItem::new_compressed(positive_y)),
                Some(UploadItem::new_compressed(negative_y)),
                Some(UploadItem::new_compressed(positive_z)),
                Some(UploadItem::new_compressed(negative_z)),
            ]),
            faces: [
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ],

            mipmap: false,
        }
    }

    /// Sets the source in level 0.
    pub fn set_base_source(
        mut self,
        positive_x: TextureSourceCompressed,
        negative_x: TextureSourceCompressed,
        positive_y: TextureSourceCompressed,
        negative_y: TextureSourceCompressed,
        positive_z: TextureSourceCompressed,
        negative_z: TextureSourceCompressed,
    ) -> Self {
        self.base_source = Some([
            Some(UploadItem::new_compressed(positive_x)),
            Some(UploadItem::new_compressed(negative_x)),
            Some(UploadItem::new_compressed(positive_y)),
            Some(UploadItem::new_compressed(negative_y)),
            Some(UploadItem::new_compressed(positive_z)),
            Some(UploadItem::new_compressed(negative_z)),
        ]);
        self
    }

    /// Uploads a new source to a specified cube map face.
    pub fn tex_image(
        mut self,
        face: CubeMapFace,
        source: TextureSourceCompressed,
        level: usize,
    ) -> Self {
        self.faces[face as usize].push(UploadItem::with_params_compressed(
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

    /// Uploads a new source for a sub-rectangle to a specified cube map face.
    pub fn tex_sub_image(
        mut self,
        face: CubeMapFace,
        source: TextureSourceCompressed,
        level: usize,
        width: usize,
        height: usize,
        x_offset: usize,
        y_offset: usize,
    ) -> Self {
        self.faces[face as usize].push(UploadItem::with_params_compressed(
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

impl TextureUploadTarget {
    fn from_index(index: usize) -> Self {
        match index {
            0 => TextureUploadTarget::TEXTURE_CUBE_MAP_POSITIVE_X,
            1 => TextureUploadTarget::TEXTURE_CUBE_MAP_NEGATIVE_X,
            2 => TextureUploadTarget::TEXTURE_CUBE_MAP_POSITIVE_Y,
            3 => TextureUploadTarget::TEXTURE_CUBE_MAP_NEGATIVE_Y,
            4 => TextureUploadTarget::TEXTURE_CUBE_MAP_POSITIVE_Z,
            5 => TextureUploadTarget::TEXTURE_CUBE_MAP_NEGATIVE_Z,
            _ => unreachable!(),
        }
    }
}
