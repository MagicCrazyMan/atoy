use uuid::Uuid;

use super::conversion::GLuint;

#[derive(Debug, Clone)]
pub enum Error {
    MatrixError(gl_matrix4rust::error::Error),
    WebGL2Unsupported,
    CanvasNotFound,
    MountElementNotFound,
    MountElementFailed,
    CreateCanvasFailed,
    CreateProgramFailed,
    GetUniformIndicesFailed,
    CreateBufferFailed,
    CreateFramebufferFailed,
    CreateRenderbufferFailed,
    CreateTextureFailed,
    CreateVertexShaderFailed,
    CreateFragmentShaderFailed,
    CompileShaderFailed(Option<String>),
    CompileProgramFailed(Option<String>),
    PickFailed(Option<String>),
    UniformBufferObjectBindingIndexAlreadyBound(GLuint),
    TexImageFailure(Option<String>),
    TextureStorageNotFount(Uuid),
    CommonWebGLError(Option<String>),
}

impl Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}

impl From<gl_matrix4rust::error::Error> for Error {
    fn from(err: gl_matrix4rust::error::Error) -> Self {
        Self::MatrixError(err)
    }
}
