use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Error {
    MatrixError(gl_matrix4rust::error::Error),
    WenGL2Unsupported,
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
    CompileShaderFailure(Option<String>),
    CompileProgramFailure(Option<String>),
    PickFailure(Option<String>),
    BufferStorageNotFound(Uuid),
    BufferUnexpectedDropped,
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
