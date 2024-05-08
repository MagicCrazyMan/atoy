use wasm_bindgen::JsValue;

use super::{
    buffer::BufferTarget,
    framebuffer::FramebufferTarget,
    texture::{TextureTarget, TextureUnit},
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
    ReadPixelsFailure,
    ClientWaitFailure,
    ClientWaitTimeout,
    CompileShaderFailure,
    CompileProgramFailure,
    ProgramOccupied,
    ProgramUnused,
    ProgramUsing,
    VertexArrayObjectOccupied,
    // NoSuchAttribute(AttributeBinding),
    // NoSuchUniform(UniformBinding),
    // NoSuchUniformBlock(UniformBlockBinding),
    BufferUnregistered,
    BufferTargetOccupied(BufferTarget),
    LoadBufferSourceFailure(Option<String>),
    // UniformBufferObjectMountPointOccupied(usize),
    RegisterBufferToMultipleRepositoryUnsupported,
    TextureUnregistered,
    TextureTargetOccupied(TextureUnit, TextureTarget),
    LoadTextureSourceFailure(Option<String>),
    // TextureInternalFormatMismatched,
    // TextureInternalFormatUnsupported(TextureInternalFormat),
    RegisterTextureToMultipleRepositoryUnsupported,
    FramebufferUnregistered,
    RegisterFramebufferToMultipleRepositoryUnsupported,
    // FramebufferAlreadyInitialized,
    FramebufferTargetOccupied(FramebufferTarget),
    // FramebufferUnboundAsRead,
    // FramebufferUnboundAsDraw,
    // CommonWebGLError(Option<String>),
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
