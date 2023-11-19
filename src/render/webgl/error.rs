#[derive(Debug, Clone)]
pub enum Error {
    MatrixError(gl_matrix4rust::error::Error),
    ProgramCreateFailure,
    VertexShaderCreateFailure,
    FragmentShaderCreateFailure,
    ShaderCompileFailure(Option<String>),
    ProgramCompileFailure(Option<String>),
}

impl Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MatrixError(err) => err.fmt(f),
            Error::VertexShaderCreateFailure => todo!(),
            Error::FragmentShaderCreateFailure => todo!(),
            Error::ShaderCompileFailure(_) => todo!(),
            Error::ProgramCompileFailure(_) => todo!(),
            Error::ProgramCreateFailure => todo!(),
            
            
        }
    }
}

impl std::error::Error for Error {}

impl From<gl_matrix4rust::error::Error> for Error {
    fn from(err: gl_matrix4rust::error::Error) -> Self {
        Self::MatrixError(err)
    }
}
