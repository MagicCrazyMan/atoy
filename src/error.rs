#[derive(Debug, Clone)]
pub enum Error {
    ParseObjectFailure,
    WebGL2RenderError(crate::render::webgl::error::Error),
}

impl Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseObjectFailure => todo!(),
            Error::WebGL2RenderError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<crate::render::webgl::error::Error> for Error {
    fn from(err: crate::render::webgl::error::Error) -> Self {
        Self::WebGL2RenderError(err)
    }
}

impl Into<wasm_bindgen::JsValue> for Error {
    fn into(self) -> wasm_bindgen::JsValue {
        todo!()
    }
}
