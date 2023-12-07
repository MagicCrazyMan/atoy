use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Error {
    MatrixError(gl_matrix4rust::error::Error),
    PipelineError(crate::render::pp::error::Error),
    WenGL2Unsupported,
    CanvasNotFound,
    MountElementNotFound,
    CreateCanvasFailure,
    CreateProgramFailure,
    CreateBufferFailure,
    CreateFramebufferFailure,
    CreateRenderbufferFailure,
    CreateTextureFailure,
    CreateVertexShaderFailure,
    CreateFragmentShaderFailure,
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

impl From<crate::render::pp::error::Error> for Error {
    fn from(err: crate::render::pp::error::Error) -> Self {
        Self::PipelineError(err)
    }
}

impl Into<wasm_bindgen::JsValue> for Error {
    fn into(self) -> wasm_bindgen::JsValue {
        todo!()
    }
}
