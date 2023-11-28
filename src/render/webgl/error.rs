use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Error {
    MatrixError(gl_matrix4rust::error::Error),
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
    CommonWebGLError(Option<String>)
}

impl Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MatrixError(err) => err.fmt(f),
            Error::CanvasNotFound => todo!(),
            Error::MountElementNotFound => todo!(),
            Error::CreateCanvasFailure => todo!(),
            Error::CreateProgramFailure => todo!(),
            Error::CreateBufferFailure => todo!(),
            Error::CreateFramebufferFailure => todo!(),
            Error::CreateRenderbufferFailure => todo!(),
            Error::CreateTextureFailure => todo!(),
            Error::CreateVertexShaderFailure => todo!(),
            Error::CreateFragmentShaderFailure => todo!(),
            Error::CompileShaderFailure(_) => todo!(),
            Error::CompileProgramFailure(_) => todo!(),
            Error::PickFailure(_) => todo!(),
            Error::WenGL2Unsupported => todo!(),
            Error::BufferStorageNotFound(_) => todo!(),
            Error::BufferUnexpectedDropped => f.write_str("buffer descriptor unexpected dropped"),
            Error::TexImageFailure(_) => todo!(),
            Error::TextureStorageNotFount(_) => todo!(),
            Error::CommonWebGLError(_) => todo!(),
        }
    }
}

impl std::error::Error for Error {}

impl From<gl_matrix4rust::error::Error> for Error {
    fn from(err: gl_matrix4rust::error::Error) -> Self {
        Self::MatrixError(err)
    }
}

impl Into<wasm_bindgen::JsValue> for Error {
    fn into(self) -> wasm_bindgen::JsValue {
        todo!()
    }
}
