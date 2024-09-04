use std::{borrow::Cow, fmt::Display};

use super::program::WebGlShaderType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    SnippetNotFound(String),
    CreateShaderFailure(WebGlShaderType),
    CompileShaderFailure(Option<String>),
    CreateProgramFailure,
    ProgramNotFound,
    LinkProgramFailure(Option<String>),
    CreateBufferFailure,
    BufferManagedByOtherManager,
    CreateFenceSyncFailure,
    ClientWaitFailure(Option<String>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}
