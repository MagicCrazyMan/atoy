use std::collections::HashMap;

use crate::render::webgl::program::{
    AttributeBinding, AttributeValue, ShaderSource, UniformBinding, UniformValue,
};

pub mod solid_color;

pub trait WebGLMaterial {
    fn name(&self) -> &str;

    fn attribute_bindings(&self) -> &[AttributeBinding];

    fn uniform_bindings(&self) -> &[UniformBinding];

    fn attribute_values(&self) -> &HashMap<String, AttributeValue>;

    fn uniform_values(&self) -> &HashMap<String, UniformValue>;

    fn sources(&self) -> &[ShaderSource];
}
