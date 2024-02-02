use wasm_bindgen::{JsCast, JsValue};

#[derive(Debug, Clone)]
pub enum Error {
    CreateCanvasFailure,
    CanvasResizeObserverFailure(Option<String>),
    MountElementFailure,
    AddEventCallbackFailure(&'static str, Option<String>),
    RemoveEventCallbackFailure(&'static str, Option<String>),
    PromiseAwaitFailure(Option<String>),
    NoSuchEntity,
    NoSuchGroup,
    InvalidDirectDrawSurface,
    WebGLRenderError(crate::render::webgl::error::Error),
    JsError(js_sys::Error),
    CommonError(Option<String>)
}

impl Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        log::error!("{:?}", self);
        todo!()
    }
}

impl std::error::Error for Error {}

impl From<crate::render::webgl::error::Error> for Error {
    fn from(value: crate::render::webgl::error::Error) -> Self {
        Self::WebGLRenderError(value)
    }
}


impl From<js_sys::Error> for Error {
    fn from(value: js_sys::Error) -> Self {
        Self::JsError(value)
    }
}

impl Into<wasm_bindgen::JsValue> for Error {
    fn into(self) -> wasm_bindgen::JsValue {
        wasm_bindgen::JsValue::from_str(&self.to_string())
    }
}

pub trait AsJsError {
    fn as_error(&self) -> Option<&js_sys::Error>;
}

impl AsJsError for JsValue {
    fn as_error(&self) -> Option<&js_sys::Error> {
        self.dyn_ref::<js_sys::Error>()
    }
}
