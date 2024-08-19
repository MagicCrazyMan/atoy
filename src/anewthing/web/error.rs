use std::{borrow::Cow, fmt::Display};

use super::program::ShaderType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    SnippetNotFound(String),
    CreateShaderFailure(ShaderType),
    CompileShaderFailure(Option<String>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}
