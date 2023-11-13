use std::borrow::Cow;

use crate::render::webgl::program::{
    AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue,
};

pub mod solid_color;

pub trait WebGLMaterial {
    fn name(&self) -> &str;

    fn attribute_bindings(&self) -> &[AttributeBinding];

    fn uniform_bindings(&self) -> &[UniformBinding];

    fn sources(&self) -> &[ShaderSource];

    fn attribute_value(&self, name: &str) -> Option<&AttributeValue>;

    fn uniform_value(&self, name: &str) -> Option<&UniformValue>;
}
