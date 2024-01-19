#[derive(Debug, Clone)]
pub enum Error {
    CreateCanvasFailure,
    CanvasResizeObserverFailure(Option<String>),
    MountElementFailure,
    AddEventCallbackFailure(&'static str, Option<String>),
    NoSuchEntity,
    NoSuchGroup,
    WebGLRenderError(crate::render::webgl::error::Error)
}

impl Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl std::error::Error for Error {}

impl From<crate::render::webgl::error::Error> for Error {
    fn from(value: crate::render::webgl::error::Error) -> Self {
        Self::WebGLRenderError(value)
    }
}

impl Into<wasm_bindgen::JsValue> for Error {
    fn into(self) -> wasm_bindgen::JsValue {
        todo!()
    }
}
