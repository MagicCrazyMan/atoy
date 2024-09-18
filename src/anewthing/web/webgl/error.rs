use std::fmt::Display;

use super::program::WebGlShaderType;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    SnippetNotFound(String),
    CreateShaderFailure(WebGlShaderType),
    CompileShaderFailure(Option<String>),
    CreateProgramFailure,
    ProgramNotFound,
    NoUsingProgram,
    AttributeLocationNotFound(String),
    UniformLocationNotFound(String),
    UniformBlockLocationNotFound(String),
    LinkProgramFailure(Option<String>),
    CreateBufferFailure,
    BufferManagedByOtherManager,
    BufferDataUnsupported,
    CreateFenceSyncFailure,
    ClientWaitFailure(Option<String>),
    CreateSamplerFailure,
    CreateTextureFailure,
    TextureManagedByOtherManager,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}
