#[derive(Debug, Clone)]
pub enum Error {
    MatrixError(gl_matrix4rust::error::Error),
    WebGl2RenderingContextNotFound,
    CreateProgramFailure,
    CreateVertexShaderFailure,
    CreateFragmentShaderFailure,
    CompileShaderFailure(Option<String>),
    CompileProgramFailure(Option<String>),
}

impl Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MatrixError(err) => err.fmt(f),
            Error::CreateVertexShaderFailure => todo!(),
            Error::CreateFragmentShaderFailure => todo!(),
            Error::CompileShaderFailure(_) => todo!(),
            Error::CompileProgramFailure(_) => todo!(),
            Error::CreateProgramFailure => todo!(),
            Error::WebGl2RenderingContextNotFound => todo!(),
            
            
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