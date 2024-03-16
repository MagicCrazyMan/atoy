use std::{any::Any, borrow::Cow, cell::RefCell, rc::Rc};

use log::warn;

use crate::{
    clock::Tick,
    error::Error,
    loader::{Loader, LoaderStatus},
    notify::Notifiee,
    renderer::webgl::{
        attribute::AttributeValue,
        program::{CustomBinding, Define},
        state::FrameState,
        texture::{Texture, Texture2D, TextureUnit},
        uniform::{UniformBlockValue, UniformValue},
    },
    share::{Share, WeakShare},
    value::Readonly,
};

use super::{StandardMaterial, Transparency};

pub struct TextureMaterial {
    transparency: Transparency,

    albedo_loader: Share<dyn Loader<Texture<Texture2D>, Failure = Error>>,
    albedo: Share<Option<(Texture<Texture2D>, TextureUnit)>>,

    normal_loader: Option<Share<dyn Loader<Texture<Texture2D>, Failure = Error>>>,
    normal: Share<Option<(Texture<Texture2D>, TextureUnit)>>,

    parallax_loader: Option<Share<dyn Loader<Texture<Texture2D>, Failure = Error>>>,
    parallax: Share<Option<(Texture<Texture2D>, TextureUnit)>>,
}

impl TextureMaterial {
    pub fn new<A>(albedo: A, transparency: Transparency) -> Self
    where
        A: Loader<Texture<Texture2D>, Failure = Error> + 'static,
    {
        Self {
            transparency,

            albedo_loader: Rc::new(RefCell::new(albedo)),
            albedo: Rc::new(RefCell::new(None)),

            normal_loader: None,
            normal: Rc::new(RefCell::new(None)),

            parallax_loader: None,
            parallax: Rc::new(RefCell::new(None)),
        }
    }

    fn prepare_albedo(&self) {
        let mut loader = self.albedo_loader.borrow_mut();
        if LoaderStatus::Unload == loader.status() {
            loader.load();
            loader.notifier().borrow_mut().register(WaitLoader {
                unit: TextureUnit::TEXTURE0,
                loader: Rc::downgrade(&self.albedo_loader),
                target: Rc::downgrade(&self.albedo),
            });
        }
    }

    fn has_normal_map(&self) -> bool {
        self.normal_loader.is_some()
    }

    fn prepare_normal(&self) {
        let Some(normal_loader) = self.normal_loader.as_ref() else {
            return;
        };

        let mut loader = normal_loader.borrow_mut();
        if LoaderStatus::Unload == loader.status() {
            loader.load();
            loader.notifier().borrow_mut().register(WaitLoader {
                unit: TextureUnit::TEXTURE1,
                loader: Rc::downgrade(normal_loader),
                target: Rc::downgrade(&self.normal),
            });
        }
    }

    fn has_parallax_map(&self) -> bool {
        self.parallax_loader.is_some()
    }

    fn prepare_parallax(&self) {
        let Some(parallax_loader) = self.parallax_loader.as_ref() else {
            return;
        };

        let mut loader = parallax_loader.borrow_mut();
        if LoaderStatus::Unload == loader.status() {
            loader.load();
            loader.notifier().borrow_mut().register(WaitLoader {
                unit: TextureUnit::TEXTURE2,
                loader: Rc::downgrade(parallax_loader),
                target: Rc::downgrade(&self.parallax),
            });
        }
    }
}

impl StandardMaterial for TextureMaterial {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("TextureMaterial")
    }

    fn ready(&self) -> bool {
        if self.albedo.borrow().is_none() {
            return false;
        }

        match (self.has_normal_map(), self.has_parallax_map()) {
            (true, true) => self.parallax.borrow().is_some() && self.normal.borrow().is_some(),
            (true, false) => self.normal.borrow().is_some(),
            (false, true) => self.parallax.borrow().is_some(),
            (false, false) => true,
        }
    }

    fn prepare(&mut self, _: &mut FrameState) {
        self.prepare_albedo();
        if self.has_normal_map() {
            self.prepare_normal();
        }
        if self.has_parallax_map() {
            self.prepare_parallax();
        }
    }

    fn tick(&mut self, _: &Tick) -> bool {
        false
    }

    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn attribute_custom_bindings(&self) -> &[CustomBinding<'_>] {
        &[]
    }

    fn uniform_custom_bindings(&self) -> &[CustomBinding<'_>] {
        match (self.has_normal_map(), self.has_parallax_map()) {
            (true, true) => &[
                CustomBinding::FromMaterial(Cow::Borrowed("u_AlbedoMap")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_NormalMap")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_ParallaxMap")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_ParallaxHeightScale")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_Transparency")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_SpecularShininess")),
            ],
            (true, false) => &[
                CustomBinding::FromMaterial(Cow::Borrowed("u_AlbedoMap")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_NormalMap")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_Transparency")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_SpecularShininess")),
            ],
            (false, true) => &[
                CustomBinding::FromMaterial(Cow::Borrowed("u_AlbedoMap")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_ParallaxMap")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_ParallaxHeightScale")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_Transparency")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_SpecularShininess")),
            ],
            (false, false) => &[
                CustomBinding::FromMaterial(Cow::Borrowed("u_AlbedoMap")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_Transparency")),
                CustomBinding::FromMaterial(Cow::Borrowed("u_SpecularShininess")),
            ],
        }
    }

    fn uniform_block_custom_bindings(&self) -> &[CustomBinding<'_>] {
        &[]
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue<'_>> {
        None
    }

    fn uniform_value(&self, name: &str) -> Option<UniformValue<'_>> {
        match name {
            "u_AlbedoMap" => {
                let texture = self.albedo.borrow();
                match &*texture {
                    Some((texture, unit)) => Some(UniformValue::Texture2D {
                        texture: Readonly::Owned(texture.clone()),
                        unit: *unit,
                    }),
                    None => None,
                }
            }
            "u_NormalMap" => {
                let texture = self.normal.borrow();
                match &*texture {
                    Some((texture, unit)) => Some(UniformValue::Texture2D {
                        texture: Readonly::Owned(texture.clone()),
                        unit: *unit,
                    }),
                    None => None,
                }
            }
            "u_ParallaxHeightScale" => Some(UniformValue::Float1(0.1)),
            "u_ParallaxMap" => {
                let texture = self.parallax.borrow();
                match &*texture {
                    Some((texture, unit)) => Some(UniformValue::Texture2D {
                        texture: Readonly::Owned(texture.clone()),
                        unit: *unit,
                    }),
                    None => None,
                }
            }
            "u_Transparency" => Some(UniformValue::Float1(self.transparency.alpha())),
            "u_SpecularShininess" => Some(UniformValue::Float1(128.0)),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str) -> Option<UniformBlockValue<'_>> {
        None
    }

    fn fragment_process(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("./shaders/texture_fragment_process.glsl"))
    }

    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]> {
        let defines: &[Define<'_>] = match (self.has_normal_map(), self.has_parallax_map()) {
            (true, true) => &[
                Define::WithoutValue(Cow::Borrowed("USE_NORMAL_MAP")),
                Define::WithoutValue(Cow::Borrowed("USE_PARALLAX_MAP")),
            ],
            (true, false) => &[Define::WithoutValue(Cow::Borrowed("USE_NORMAL_MAP"))],
            (false, true) => &[Define::WithoutValue(Cow::Borrowed("USE_PARALLAX_MAP"))],
            (false, false) => &[],
        };
        Cow::Borrowed(defines)
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }

    fn use_position_eye_space(&self) -> bool {
        false
    }

    fn use_normal(&self) -> bool {
        self.has_normal_map() || self.has_parallax_map()
    }

    fn use_texture_coordinate(&self) -> bool {
        true
    }

    fn use_tbn(&self) -> bool {
        self.has_normal_map() || self.has_parallax_map()
    }

    fn use_calculated_bitangent(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

struct WaitLoader {
    unit: TextureUnit,
    loader: WeakShare<dyn Loader<Texture<Texture2D>, Failure = Error>>,
    target: WeakShare<Option<(Texture<Texture2D>, TextureUnit)>>,
}

impl Notifiee<LoaderStatus> for WaitLoader {
    fn notify(&mut self, status: &LoaderStatus) {
        match status {
            LoaderStatus::Unload => unreachable!(),
            LoaderStatus::Loading => {}
            LoaderStatus::Loaded => {
                let (Some(loader), Some(target)) = (self.loader.upgrade(), self.target.upgrade())
                else {
                    return;
                };

                let texture = loader.borrow().loaded().unwrap();
                *target.borrow_mut() = Some((texture, self.unit));
            }
            LoaderStatus::Errored => {
                let Some(loader) = self.loader.upgrade() else {
                    return;
                };

                let err = loader.borrow().loaded().err().unwrap();
                warn!("Failed to load albedo texture. {}", err)
            }
        }
    }
}

pub struct Builder {
    transparency: Transparency,
    albedo_loader: Share<dyn Loader<Texture<Texture2D>, Failure = Error>>,
    normal_loader: Option<Share<dyn Loader<Texture<Texture2D>, Failure = Error>>>,
    parallax_loader: Option<Share<dyn Loader<Texture<Texture2D>, Failure = Error>>>,
}

impl Builder {
    /// Constructs a new texture material builder with an albedo map in required.
    /// By default, the transparency is set to [`Transparency::Opaque`].
    pub fn new<A>(albedo: A) -> Self
    where
        A: Loader<Texture<Texture2D>, Failure = Error> + 'static,
    {
        Self {
            transparency: Transparency::Opaque,
            albedo_loader: Rc::new(RefCell::new(albedo)),
            normal_loader: None,
            parallax_loader: None,
        }
    }

    /// Sets transparency for the material.
    pub fn set_transparency(mut self, transparency: Transparency) -> Self {
        self.transparency = transparency;
        self
    }

    /// Sets normal map for the material.
    pub fn set_normal_map<N>(mut self, normal: N) -> Self
    where
        N: Loader<Texture<Texture2D>, Failure = Error> + 'static,
    {
        self.normal_loader = Some(Rc::new(RefCell::new(normal)));
        self
    }

    /// Sets parallax map for the material.
    pub fn set_parallax_loader<P>(mut self, parallax: P) -> Self
    where
        P: Loader<Texture<Texture2D>, Failure = Error> + 'static,
    {
        self.parallax_loader = Some(Rc::new(RefCell::new(parallax)));
        self
    }

    pub fn build(self) -> TextureMaterial {
        TextureMaterial {
            transparency: self.transparency,
            albedo_loader: self.albedo_loader,
            albedo: Rc::new(RefCell::new(None)),
            normal_loader: self.normal_loader,
            normal: Rc::new(RefCell::new(None)),
            parallax_loader: self.parallax_loader,
            parallax: Rc::new(RefCell::new(None)),
        }
    }
}
