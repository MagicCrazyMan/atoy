use std::{
    any::Any,
    borrow::Cow,
    cell::RefCell,
    rc::{Rc, Weak},
};

use log::warn;

use crate::{
    error::Error,
    loader::{Loader, LoaderStatus},
    notify::{Notifiee, Notifier},
    readonly::Readonly,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::ProgramSource,
        shader::Define,
        state::FrameState,
        texture::{texture2d::Texture2D, TextureDescriptor, TextureUnit},
        uniform::{UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue},
    },
};

use super::{StandardMaterial, StandardMaterialSource, Transparency};

pub struct TextureMaterial {
    transparency: Transparency,

    albedo_loader: Rc<RefCell<dyn Loader<Texture2D, Failure = Error>>>,
    albedo: Rc<RefCell<Option<UniformValue>>>,

    normal_loader: Option<Rc<RefCell<dyn Loader<Texture2D, Failure = Error>>>>,
    normal: Rc<RefCell<Option<UniformValue>>>,

    notifier: Rc<RefCell<Notifier<()>>>,
}

impl TextureMaterial {
    pub fn new<A>(albedo: A, transparency: Transparency) -> Self
    where
        A: Loader<Texture2D, Failure = Error> + 'static,
    {
        Self {
            transparency,

            albedo_loader: Rc::new(RefCell::new(albedo)),
            albedo: Rc::new(RefCell::new(None)),

            normal_loader: None,
            normal: Rc::new(RefCell::new(None)),

            notifier: Rc::new(RefCell::new(Notifier::new())),
        }
    }

    fn prepare_albedo(&self) {
        let mut loader = self.albedo_loader.borrow_mut();
        if LoaderStatus::Unload == loader.status() {
            loader.load();
            loader.notifier().borrow_mut().register(WaitLoader {
                unit: TextureUnit::TEXTURE0,
                loader: Rc::downgrade(&self.albedo_loader),
                texture_uniform: Rc::downgrade(&self.albedo),
                notifier: Rc::downgrade(&self.notifier),
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
                texture_uniform: Rc::downgrade(&self.normal),
                notifier: Rc::downgrade(&self.notifier),
            });
        }
    }
}

impl StandardMaterial for TextureMaterial {
    fn ready(&self) -> bool {
        match self.has_normal_map() {
            true => self.albedo.borrow().is_some() && self.normal.borrow().is_some(),
            false => self.albedo.borrow().is_some(),
        }
    }

    fn prepare(&mut self, _: &mut FrameState) {
        match self.has_normal_map() {
            true => {
                self.prepare_albedo();
                self.prepare_normal();
            }
            false => self.prepare_albedo(),
        }
    }

    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        match self.has_normal_map() {
            true => 
            &[
                AttributeBinding::GeometryPosition,
                AttributeBinding::GeometryNormal,
                AttributeBinding::GeometryTangent,
                AttributeBinding::GeometryBitangent,
                AttributeBinding::GeometryTextureCoordinate,
            ],
            false => 
            &[
                AttributeBinding::GeometryPosition,
                AttributeBinding::GeometryNormal,
                AttributeBinding::GeometryTextureCoordinate,
            ],
        }
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        match self.has_normal_map() {
            true => &[
                UniformBinding::ModelMatrix,
                UniformBinding::NormalMatrix,
                UniformBinding::FromMaterial(Cow::Borrowed("u_AlbedoMap")),
                UniformBinding::FromMaterial(Cow::Borrowed("u_NormalMap")),
                UniformBinding::FromMaterial(Cow::Borrowed("u_Transparency")),
                UniformBinding::FromMaterial(Cow::Borrowed("u_SpecularShininess")),
            ],
            false => &[
                UniformBinding::ModelMatrix,
                UniformBinding::NormalMatrix,
                UniformBinding::FromMaterial(Cow::Borrowed("u_AlbedoMap")),
                UniformBinding::FromMaterial(Cow::Borrowed("u_Transparency")),
                UniformBinding::FromMaterial(Cow::Borrowed("u_SpecularShininess")),
            ],
        }
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[]
    }

    fn attribute_value(&self, _: &str) -> Option<Readonly<'_, AttributeValue>> {
        None
    }

    fn uniform_value(&self, name: &str) -> Option<Readonly<'_, UniformValue>> {
        match name {
            "u_AlbedoMap" => Some(Readonly::Owned(self.albedo.borrow().clone()?)),
            "u_NormalMap" => match self.has_normal_map() {
                true => Some(Readonly::Owned(self.normal.borrow().clone()?)),
                false => None,
            },
            "u_Transparency" => Some(Readonly::Owned(UniformValue::Float1(
                self.transparency.alpha(),
            ))),
            "u_SpecularShininess" => Some(Readonly::Owned(UniformValue::Float1(128.0))),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str) -> Option<Readonly<'_, UniformBlockValue>> {
        None
    }

    fn notifier(&self) -> &Rc<RefCell<Notifier<()>>> {
        &self.notifier
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_standard_material_source(&self) -> &dyn StandardMaterialSource {
        self
    }

    fn as_program_source(&self) -> &dyn ProgramSource {
        self
    }
}

impl StandardMaterialSource for TextureMaterial {
    fn name(&self) -> Cow<'static, str> {
        match self.has_normal_map() {
            true => {
                Cow::Borrowed("TextureMaterial_NormalMap")
            }
            false => Cow::Borrowed("TextureMaterial"),
        }
    }

    fn vertex_process(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn fragment_process(&self) -> Cow<'static, str> {
        Cow::Borrowed(include_str!("./shaders/texture_build_material.glsl"))
    }

    fn vertex_defines(&self) -> Vec<Define> {
        match self.has_normal_map() {
            true => vec![Define::WithoutValue(Cow::Borrowed("NORMAL_MAP"))],
            false => vec![],
        }
    }

    fn fragment_defines(&self) -> Vec<Define> {
        match self.has_normal_map() {
            true => vec![Define::WithoutValue(Cow::Borrowed("NORMAL_MAP"))],
            false => vec![],
        }
    }
}

struct WaitLoader {
    unit: TextureUnit,
    loader: Weak<RefCell<dyn Loader<Texture2D, Failure = Error>>>,
    texture_uniform: Weak<RefCell<Option<UniformValue>>>,
    notifier: Weak<RefCell<Notifier<()>>>,
}

impl Notifiee<LoaderStatus> for WaitLoader {
    fn notify(&mut self, status: &LoaderStatus) {
        let Some(notifier) = self.notifier.upgrade() else {
            return;
        };

        match status {
            LoaderStatus::Unload => unreachable!(),
            LoaderStatus::Loading => {}
            LoaderStatus::Loaded => {
                let (Some(loader), Some(uniform)) =
                    (self.loader.upgrade(), self.texture_uniform.upgrade())
                else {
                    return;
                };

                let texture = loader.borrow().loaded().unwrap();
                *uniform.borrow_mut() = Some(UniformValue::Texture2D {
                    descriptor: TextureDescriptor::new(texture),
                    unit: self.unit,
                });
                notifier.borrow_mut().notify(&())
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
    albedo_loader: Rc<RefCell<dyn Loader<Texture2D, Failure = Error>>>,
    normal_loader: Option<Rc<RefCell<dyn Loader<Texture2D, Failure = Error>>>>,
}

impl Builder {
    /// Constructs a new texture material builder with an albedo map in required.
    /// By default, the transparency is set to [`Transparency::Opaque`].
    pub fn new<A>(albedo: A) -> Self
    where
        A: Loader<Texture2D, Failure = Error> + 'static,
    {
        Self {
            transparency: Transparency::Opaque,
            albedo_loader: Rc::new(RefCell::new(albedo)),
            normal_loader: None,
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
        N: Loader<Texture2D, Failure = Error> + 'static,
    {
        self.normal_loader = Some(Rc::new(RefCell::new(normal)));
        self
    }

    pub fn build(self) -> TextureMaterial {
        TextureMaterial {
            transparency: self.transparency,
            albedo_loader: self.albedo_loader,
            albedo: Rc::new(RefCell::new(None)),
            normal_loader: self.normal_loader,
            normal: Rc::new(RefCell::new(None)),
            notifier: Rc::new(RefCell::new(Notifier::new())),
        }
    }
}
