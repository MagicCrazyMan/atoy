use std::any::Any;

use super::{
    attribute::AttributeValue,
    uniform::{UniformBlockValue, UniformValue},
};

pub trait EntityComponent {
    fn attribute(&self, name: &str) -> Option<AttributeValue>;

    fn uniform(&self, name: &str) -> Option<UniformValue>;

    fn uniform_block(&self, name: &str) -> Option<UniformBlockValue>;

    fn property(&self, name: &str) -> Option<&dyn Any>;
}
