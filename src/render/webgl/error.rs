use uuid::Uuid;
use wasm_bindgen::JsValue;

use super::conversion::GLuint;

#[derive(Debug, Clone)]
pub enum Error {
    WebGL2Unsupported,
    MountElementNotFound,
    MountElementFailure,
    CreateProgramFailure,
    GetUniformIndicesFailure,
    CreateBufferFailure,
    CreateFramebufferFailure,
    CreateRenderbufferFailure,
    CreateTextureFailure,
    CreateVertexShaderFailure,
    CreateFragmentShaderFailure,
    CreateFenceSyncFailure,
    ExtensionUnsupported(String),
    ReadPixelsFailure(Option<String>),
    ClientWaitFailure(Option<String>),
    CompileShaderFailure(Option<String>),
    CompileProgramFailure(Option<String>),
    NoSuchAttribute(String),
    NoSuchUniform(String),
    UniformBufferObjectIndexAlreadyBound(GLuint),
    TexImageFailure(Option<String>),
    TextureStorageNotFount(Uuid),
    CommonWebGLError(Option<String>),
    FramebufferUninitialized,
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
