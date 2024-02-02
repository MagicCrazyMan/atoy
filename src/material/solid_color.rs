use std::{any::Any, borrow::Cow, cell::RefCell, rc::Rc};

use gl_matrix4rust::{vec3::Vec3, GLF32};

use crate::{
    notify::Notifier,
    readonly::Readonly,
    render::webgl::{
        attribute::{AttributeBinding, AttributeValue},
        program::ProgramSource,
        shader::Define,
        state::FrameState,
        uniform::{UniformBinding, UniformBlockBinding, UniformBlockValue, UniformValue},
    },
};

use super::{StandardMaterial, StandardMaterialSource, Transparency};

/// A Phong Shading based solid color material,
/// with ambient, diffuse and specular light colors all to be the same one.
pub struct SolidColorMaterial {
    color: Vec3<f32>,
    specular_shininess: f32,
    transparency: Transparency,
    notifier: Rc<RefCell<Notifier<()>>>,
}

impl SolidColorMaterial {
    /// Constructs a solid color material with `(1.0, 0.0, 0.0, 1.0)``.
    pub fn new() -> Self {
        Self::with_color(Vec3::<f32>::new(1.0, 0.0, 0.0), 128.0, Transparency::Opaque)
    }

    /// Constructs a solid color material with specified color and transparency.
    pub fn with_color(
        color: Vec3<f32>,
        specular_shininess: f32,
        transparency: Transparency,
    ) -> Self {
        Self {
            color,
            specular_shininess,
            transparency,
            notifier: Rc::new(RefCell::new(Notifier::new())),
        }
    }

    /// Returns color.
    pub fn color(&self) -> Vec3<f32> {
        self.color
    }

    /// Sets color,
    pub fn set_color(&mut self, color: Vec3<f32>, transparency: Transparency) {
        self.color = color;
        self.transparency = transparency;
    }
}

impl StandardMaterial for SolidColorMaterial {
    fn ready(&self) -> bool {
        true
    }

    fn prepare(&mut self, _: &mut FrameState) {}

    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn attribute_bindings(&self) -> &[AttributeBinding] {
        &[
            AttributeBinding::GeometryPosition,
            AttributeBinding::GeometryNormal,
        ]
    }

    fn uniform_bindings(&self) -> &[UniformBinding] {
        &[
            UniformBinding::ModelMatrix,
            UniformBinding::NormalMatrix,
            UniformBinding::FromMaterial(Cow::Borrowed("u_Color")),
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
            "u_Color" => Some(Readonly::Owned(UniformValue::FloatVector3(
                self.color.gl_f32(),
            ))),
            "u_Transparency" => Some(Readonly::Owned(UniformValue::Float1(
                self.transparency.alpha(),
            ))),
            "u_SpecularShininess" => Some(Readonly::Owned(UniformValue::Float1(
                self.specular_shininess,
            ))),
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

impl StandardMaterialSource for SolidColorMaterial {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("SolidColorMaterial")
    }

    fn vertex_process(&self) -> Option<Cow<'static, str>> {
        None
    }

    fn fragment_process(&self) -> Cow<'static, str> {
        Cow::Borrowed(include_str!("./shaders/solid_color_build_material.glsl"))
    }

    fn vertex_defines(&self) -> Vec<Define> {
        vec![]
    }

    fn fragment_defines(&self) -> Vec<Define> {
        vec![]
    }
}
