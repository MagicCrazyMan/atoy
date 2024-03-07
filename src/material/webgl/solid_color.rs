use std::{any::Any, borrow::Cow};

use gl_matrix4rust::{vec3::Vec3, GLF32};

use crate::{
    clock::Tick,
    renderer::webgl::{
        attribute::AttributeValue,
        program::{CustomBinding, Define},
        state::FrameState,
        uniform::{UniformBlockValue, UniformValue},
    },
    value::Readonly,
};

use super::{StandardMaterial, Transparency};

/// A Phong Shading based solid color material,
/// with ambient, diffuse and specular light colors all to be the same one.
pub struct SolidColorMaterial {
    color: Vec3<f32>,
    specular_shininess: f32,
    transparency: Transparency,
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
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("SolidColorMaterial")
    }

    fn fragment_process(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("./shaders/solid_color_fragment_process.glsl"))
    }

    fn vertex_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn fragment_defines(&self) -> &[Define<'_>] {
        &[]
    }

    fn ready(&self) -> bool {
        true
    }

    fn prepare(&mut self, _: &mut FrameState) {}

    fn tick(&mut self, _: &Tick) -> bool {
        true
    }

    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue<'_>> {
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
            "u_SpecularShininess" => {
                Some(Readonly::Owned(UniformValue::Float1(self.specular_shininess)))
            }
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str) -> Option<Readonly<'_, UniformBlockValue>> {
        None
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
    }

    fn attribute_custom_bindings(&self) -> &[CustomBinding<'_>] {
        &[]
    }

    fn uniform_custom_bindings(&self) -> &[CustomBinding<'_>] {
        &[
            CustomBinding::FromMaterial(Cow::Borrowed("u_Color")),
            CustomBinding::FromMaterial(Cow::Borrowed("u_Transparency")),
            CustomBinding::FromMaterial(Cow::Borrowed("u_SpecularShininess")),
        ]
    }

    fn uniform_block_custom_bindings(&self) -> &[CustomBinding<'_>] {
        &[]
    }

    fn use_position_eye_space(&self) -> bool {
        false
    }

    fn use_normal(&self) -> bool {
        true
    }

    fn use_texture_coordinate(&self) -> bool {
        false
    }

    fn use_tbn(&self) -> bool {
        false
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
