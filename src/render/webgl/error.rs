use uuid::Uuid;
use wasm_bindgen::JsValue;

use super::conversion::GLuint;

#[derive(Debug, Clone)]
pub enum Error {
    WebGL2Unsupported,
    CanvasNotFound,
    CanvasResizeObserverFailed(Option<String>),
    AddEventCallbackFailed(&'static str, Option<String>),
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
    CreateFenceSyncFailed,
    ExtensionUnsupported(String),
    ReadPixelsFailed(Option<String>),
    ClientWaitFailed(Option<String>),
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
        log::error!("{:?}", self);
        todo!()
    }
}

impl std::error::Error for Error {}

impl Into<JsValue> for Error {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}
