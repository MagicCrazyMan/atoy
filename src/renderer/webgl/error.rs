use wasm_bindgen::JsValue;

use super::framebuffer::FramebufferTarget;

#[derive(Debug, Clone)]
pub enum Error {
    WebGL2Unsupported,
    CreateProgramFailure,
    CreateBufferFailure,
    CreateFramebufferFailure,
    CreateRenderbufferFailure,
    CreateTextureFailure,
    CreateSamplerFailure,
    CreateVertexShaderFailure,
    CreateFragmentShaderFailure,
    CreateFenceSyncFailure,
    ExtensionUnsupported(&'static str),
    ReadPixelsFailure(Option<String>),
    ClientWaitFailure(Option<String>),
    CompileShaderFailure(Option<String>),
    CompileProgramFailure(Option<String>),
    NoSuchAttribute(String),
    NoSuchUniform(String),
    BufferUninitialized,
    BufferAlreadyInitialized,
    UniformBufferObjectIndexAlreadyBound(u32),
    TexImageFailure(Option<String>),
    TexCompressedImageFailure,
    TextureSizeMismatched,
    TextureSizeOverflowed {
        max: (usize, usize),
        value: (usize, usize),
    },
    TextureUnitOverflowed {
        max: usize,
        value: usize,
    },
    TextureSharingDisallowed,
    TextureSourceFunctionRecursionDisallowed,
    TextureSourceFunctionSizeMismatched,
    TextureSourceFunctionBytesLengthMismatched,
    TextureSourceFunctionFormatMismatched,
    FramebufferUninitialized,
    FramebufferUnbound,
    FramebufferBinding(FramebufferTarget),
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
