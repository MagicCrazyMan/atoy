use std::fmt::Display;

use web_sys::DomException;

use super::program::WebGlShaderType;

#[derive(Debug, Clone)]
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
    TextureImageSourceError(DomException),
    CreateFramebufferFailure,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}
