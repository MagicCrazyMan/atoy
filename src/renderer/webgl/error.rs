use wasm_bindgen::JsValue;

use super::{
    attribute::AttributeBinding,
    buffer::BufferTarget,
    framebuffer::FramebufferTarget,
    texture::{TextureInternalFormat, TextureTarget, TextureUnit},
    uniform::{UniformBinding, UniformBlockBinding},
};

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
    CreateVertexArrayObjectFailure,
    ExtensionUnsupported(&'static str),
    ReadPixelsFailure(Option<String>),
    ClientWaitFailure(Option<String>),
    CompileShaderFailure(Option<String>),
    CompileProgramFailure(Option<String>),
    ProgramOccupied,
    ProgramUnused,
    VertexArrayObjectOccupied,
    NoSuchAttribute(AttributeBinding),
    NoSuchUniform(UniformBinding),
    NoSuchUniformBlock(UniformBlockBinding),
    BufferUninitialized,
    BufferAlreadyInitialized,
    BufferTargetOccupied(BufferTarget),
    UniformBufferObjectMountPointOccupied(u32),
    RegisterBufferToMultipleStore,
    TextureUninitialized,
    TextureAlreadyInitialized,
    TextureTargetOccupied(TextureUnit, TextureTarget),
    TextureInternalFormatMismatched,
    TextureInternalFormatUnsupported(TextureInternalFormat),
    TextureUploadImageFailure(Option<String>),
    RegisterTextureToMultipleStore,
    TextureSizeOverflowed {
        max: (usize, usize),
        value: (usize, usize),
    },
    TextureUnitOverflowed {
        max: usize,
        value: usize,
    },
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
