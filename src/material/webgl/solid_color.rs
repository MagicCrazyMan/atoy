use std::{any::Any, borrow::Cow};

use gl_matrix4rust::vec3::Vec3;

use crate::{
    clock::Tick,
    message::{channel, Receiver, Sender},
    renderer::webgl::{
        attribute::AttributeValue,
        matrix::GlF32,
        program::Define,
        state::FrameState,
        uniform::{UniformBlockValue, UniformValue},
    },
};

use super::{MaterialMessage, StandardMaterial, Transparency};

/// A Phong Shading based solid color material,
/// with ambient, diffuse and specular light colors all to be the same one.
pub struct SolidColorMaterial {
    color: Vec3<f32>,
    specular_shininess: f32,
    transparency: Transparency,
    channel: (Sender<MaterialMessage>, Receiver<MaterialMessage>),
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
            channel: channel(),
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
        self.channel.0.send(MaterialMessage::Changed);
    }
}

impl StandardMaterial for SolidColorMaterial {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed("SolidColorMaterial")
    }

    fn fragment_process(&self) -> Cow<'_, str> {
        Cow::Borrowed(include_str!("./shaders/solid_color_fragment_process.glsl"))
    }

    fn vertex_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn fragment_defines(&self) -> Cow<'_, [Define<'_>]> {
        Cow::Borrowed(&[])
    }

    fn ready(&self) -> bool {
        true
    }

    fn prepare(&mut self, _: &mut FrameState) {}

    fn tick(&mut self, _: &Tick) {}

    fn changed(&self) -> Receiver<MaterialMessage> {
        self.channel.1.clone()
    }

    fn transparency(&self) -> Transparency {
        self.transparency
    }

    fn attribute_value(&self, _: &str) -> Option<AttributeValue<'_>> {
        None
    }

    fn uniform_value(&self, name: &str) -> Option<UniformValue<'_>> {
        match name {
            "u_Material_Color" => Some(UniformValue::FloatVector3(self.color.to_f32_array())),
            "u_Material_Transparency" => Some(UniformValue::Float1(self.transparency.alpha())),
            "u_Material_SpecularShininess" => Some(UniformValue::Float1(self.specular_shininess)),
            _ => None,
        }
    }

    fn uniform_block_value(&self, _: &str) -> Option<UniformBlockValue<'_>> {
        None
    }

    fn snippet(&self, _: &str) -> Option<Cow<'_, str>> {
        None
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
