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
    albedo_loader: Rc<RefCell<dyn Loader<Texture2D, Error = Error>>>,
    albedo: Rc<RefCell<Option<UniformValue>>>,

    notifier: Rc<RefCell<Notifier<()>>>,
}

impl TextureMaterial {
    pub fn from_loaders<A>(albedo: A, transparency: Transparency) -> Self
    where
        A: Loader<Texture2D, Error = Error> + 'static,
    {
        Self {
            transparency,
            albedo_loader: Rc::new(RefCell::new(albedo)),
            albedo: Rc::new(RefCell::new(None)),

            notifier: Rc::new(RefCell::new(Notifier::new())),
        }
    }

    fn prepare_albedo(&self) {
        let mut loader = self.albedo_loader.borrow_mut();
        if LoaderStatus::Unload == loader.status() {
            struct UpdateAlbedo {
                loader: Weak<RefCell<dyn Loader<Texture2D, Error = Error>>>,
                albedo: Weak<RefCell<Option<UniformValue>>>,
                notifier: Weak<RefCell<Notifier<()>>>,
            }

            impl Notifiee<LoaderStatus> for UpdateAlbedo {
                fn notify(&mut self, status: &LoaderStatus) {
                    let Some(notifier) = self.notifier.upgrade() else {
                        return;
                    };

                    match status {
                        LoaderStatus::Unload => unreachable!(),
                        LoaderStatus::Loading => {}
                        LoaderStatus::Loaded => {
                            let (Some(loader), Some(albedo)) =
                                (self.loader.upgrade(), self.albedo.upgrade())
                            else {
                                return;
                            };

                            let texture = loader.borrow().loaded().unwrap();
                            *albedo.borrow_mut() = Some(UniformValue::Texture2D {
                                descriptor: TextureDescriptor::new(texture),
                                unit: TextureUnit::TEXTURE0,
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

            loader.load();
            loader.notifier().borrow_mut().register(UpdateAlbedo {
                loader: Rc::downgrade(&self.albedo_loader),
                albedo: Rc::downgrade(&self.albedo),
                notifier: Rc::downgrade(&self.notifier),
            });
        }
    }
}

impl StandardMaterial for TextureMaterial {
    fn ready(&self) -> bool {
        self.albedo.borrow().is_some()
    }

    fn prepare(&mut self, _: &mut FrameState) {
        self.prepare_albedo();
    }

    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryNormal,
            AttributeBinding::GeometryTextureCoordinate,
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::NormalMatrix,
            UniformBinding::FromMaterial(Cow::Borrowed("u_DiffuseTexture")),
            UniformBinding::FromMaterial(Cow::Borrowed("u_Transparency")),
            UniformBinding::FromMaterial(Cow::Borrowed("u_SpecularShininess")),
        ]
    }

    fn uniform_block_bindings(&self) -> &[UniformBlockBinding] {
        &[]
    }

    fn attribute_value(&self, _: &str) -> Option<Readonly<'_, AttributeValue>> {
        None
    }

    fn uniform_value(&self, name: &str) -> Option<Readonly<'_, UniformValue>> {
        match name {
            "u_DiffuseTexture" => Some(Readonly::Owned(self.albedo.borrow().clone()?)),
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
        Cow::Borrowed("TextureMaterial")
    }

    fn vertex_process(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn fragment_process(&self) -> Cow<'static, str> {
        Cow::Borrowed(include_str!("./shaders/texture_build_material.glsl"))
    }

    fn vertex_defines(&self) -> Vec<Define> {
        vec![]
    }

    fn fragment_defines(&self) -> Vec<Define> {
        vec![]
    }
}
