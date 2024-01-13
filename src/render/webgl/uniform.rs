use std::borrow::Cow;

use super::{
    buffer::BufferDescriptor,
    conversion::{GLintptr, GLsizeiptr},
    texture::{TextureDescriptor, TextureParameter, TextureUnit},
};

/// Uniform Buffer Object mount point for `atoy_UniversalUniformsVert` and `atoy_UniversalUniformsFrag`.
pub static UBO_UNIVERSAL_UNIFORMS_BINDING: u32 = 0;
/// Uniform Buffer Object mount point for `atoy_Lights`.
pub static UBO_LIGHTS_BINDING: u32 = 1;
/// Uniform Buffer Object mount point for gaussian blur.
pub static UBO_GAUSSIAN_BLUR_BINDING: u32 = 2;

/// Uniform Buffer Object bytes length for `atoy_UniversalUniformsVert` and `atoy_UniversalUniformsFrag`.
pub static UBO_UNIVERSAL_UNIFORMS_BYTES_LENGTH: u32 = 16 + 16 + 64 + 64 + 64;
/// Uniform Buffer Object bytes length for `u_RenderTime`.
pub static UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_LENGTH: u32 = 4;
/// Uniform Buffer Object bytes length for `u_EnableLighting`.
pub static UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_LENGTH: u32 = 4;
/// Uniform Buffer Object bytes length for `u_CameraPosition`.
pub static UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_LENGTH: u32 = 12;
/// Uniform Buffer Object bytes length for `u_ViewMatrix`.
pub static UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_ProjMatrix`.
pub static UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_ViewProjMatrix`.
pub static UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_LENGTH: u32 = 64;

/// Uniform Buffer Object bytes offset for `u_RenderTime`.
pub static UBO_UNIVERSAL_UNIFORMS_RENDER_TIME_BYTES_OFFSET: u32 = 0;
/// Uniform Buffer Object bytes offset for `u_EnableLighting`.
pub static UBO_UNIVERSAL_UNIFORMS_ENABLE_LIGHTING_BYTES_OFFSET: u32 = 4;
/// Uniform Buffer Object bytes offset for `u_CameraPosition`.
pub static UBO_UNIVERSAL_UNIFORMS_CAMERA_POSITION_BYTES_OFFSET: u32 = 16;
/// Uniform Buffer Object bytes offset for `u_ViewMatrix`.
pub static UBO_UNIVERSAL_UNIFORMS_VIEW_MATRIX_BYTES_OFFSET: u32 = 32;
/// Uniform Buffer Object bytes offset for `u_ProjMatrix`.
pub static UBO_UNIVERSAL_UNIFORMS_PROJ_MATRIX_BYTES_OFFSET: u32 = 96;
/// Uniform Buffer Object bytes offset for `u_ViewProjMatrix`.
pub static UBO_UNIVERSAL_UNIFORMS_VIEW_PROJ_MATRIX_BYTES_OFFSET: u32 = 160;

/// Uniform Buffer Object bytes length for `atoy_Lights`.
pub static UBO_LIGHTS_BYTES_LENGTH: u32 = 16 + 16 + 64 * 12 + 64 * 12 + 80 * 12 + 112 * 12;
/// Uniform Buffer Object bytes length for `u_Attenuations`.
pub static UBO_LIGHTS_ATTENUATIONS_BYTES_LENGTH: u32 = 12;
/// Uniform Buffer Object bytes length for `u_AmbientLight`.
pub static UBO_LIGHTS_AMBIENT_LIGHT_BYTES_LENGTH: u32 = 16;
/// Uniform Buffer Object bytes length for `u_DirectionalLights`.
pub static UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_PointLights`.
pub static UBO_LIGHTS_POINT_LIGHTS_BYTES_LENGTH: u32 = 64;
/// Uniform Buffer Object bytes length for `u_SpotLights`.
pub static UBO_LIGHTS_SPOT_LIGHTS_BYTES_LENGTH: u32 = 80;
/// Uniform Buffer Object bytes length for `u_AreaLights`.
pub static UBO_LIGHTS_AREA_LIGHTS_BYTES_LENGTH: u32 = 112;

/// Uniform Buffer Object bytes offset for `u_Attenuations`.
pub static UBO_LIGHTS_ATTENUATIONS_BYTES_OFFSET: u32 = 0;
/// Uniform Buffer Object bytes offset for `u_AmbientLight`.
pub static UBO_LIGHTS_AMBIENT_LIGHT_BYTES_OFFSET: u32 = 16;
/// Uniform Buffer Object bytes offset for `u_DirectionalLights`.
pub static UBO_LIGHTS_DIRECTIONAL_LIGHTS_BYTES_OFFSET: u32 = 32;
/// Uniform Buffer Object bytes offset for `u_PointLights`.
pub static UBO_LIGHTS_POINT_LIGHTS_BYTES_OFFSET: u32 = 800;
/// Uniform Buffer Object bytes offset for `u_SpotLights`.
pub static UBO_LIGHTS_SPOT_LIGHTS_BYTES_OFFSET: u32 = 1568;
/// Uniform Buffer Object bytes offset for `u_AreaLights`.
pub static UBO_LIGHTS_AREA_LIGHTS_BYTES_OFFSET: u32 = 2528;

/// Available uniform values.
#[derive(Clone)]
pub enum UniformValue {
    Bool(bool),
    Float1(f32),
    Float2(f32, f32),
    Float3(f32, f32, f32),
    Float4(f32, f32, f32, f32),
    UnsignedInteger1(u32),
    UnsignedInteger2(u32, u32),
    UnsignedInteger3(u32, u32, u32),
    UnsignedInteger4(u32, u32, u32, u32),
    Integer1(i32),
    Integer2(i32, i32),
    Integer3(i32, i32, i32),
    Integer4(i32, i32, i32, i32),
    FloatVector1([f32; 1]),
    FloatVector2([f32; 2]),
    FloatVector3([f32; 3]),
    FloatVector4([f32; 4]),
    IntegerVector1([i32; 1]),
    IntegerVector2([i32; 2]),
    IntegerVector3([i32; 3]),
    IntegerVector4([i32; 4]),
    UnsignedIntegerVector1([u32; 1]),
    UnsignedIntegerVector2([u32; 2]),
    UnsignedIntegerVector3([u32; 3]),
    UnsignedIntegerVector4([u32; 4]),
    Matrix2 {
        data: [f32; 4],
        transpose: bool,
    },
    Matrix3 {
        data: [f32; 9],
        transpose: bool,
    },
    Matrix4 {
        data: [f32; 16],
        transpose: bool,
    },
    Texture {
        descriptor: TextureDescriptor,
        params: Vec<TextureParameter>,
        unit: TextureUnit,
    },
}

/// Available uniform block values.
#[derive(Clone)]
pub enum UniformBlockValue {
    BufferBase {
        descriptor: BufferDescriptor,
        binding: u32,
    },
    BufferRange {
        descriptor: BufferDescriptor,
        binding: u32,
        offset: GLintptr,
        size: GLsizeiptr,
    },
}

/// Uniform binding sources.
#[derive(Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UniformBinding {
    RenderTime,
    CanvasSize,
    DrawingBufferSize,
    ModelMatrix,
    NormalMatrix,
    ViewMatrix,
    ProjMatrix,
    ViewProjMatrix,
    CameraPosition,
    Transparency,
    FromGeometry(Cow<'static, str>),
    FromMaterial(Cow<'static, str>),
    FromEntity(Cow<'static, str>),
}

impl UniformBinding {
    /// Returns variable name.
    pub fn variable_name(&self) -> &str {
        match self {
            UniformBinding::RenderTime => "u_RenderTime",
            UniformBinding::CanvasSize => "u_CanvasSize",
            UniformBinding::DrawingBufferSize => "u_DrawingBufferSize",
            UniformBinding::ModelMatrix => "u_ModelMatrix",
            UniformBinding::NormalMatrix => "u_NormalMatrix",
            UniformBinding::ViewMatrix => "u_ViewMatrix",
            UniformBinding::ProjMatrix => "u_ProjMatrix",
            UniformBinding::ViewProjMatrix => "u_ViewProjMatrix",
            UniformBinding::CameraPosition => "u_CameraPosition",
            UniformBinding::Transparency => "u_Transparency",
            UniformBinding::FromGeometry(name)
            | UniformBinding::FromMaterial(name)
            | UniformBinding::FromEntity(name) => name,
        }
    }
}

/// Uniform block binding sources.
#[derive(Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UniformBlockBinding {
    FromGeometry(Cow<'static, str>),
    FromMaterial(Cow<'static, str>),
    FromEntity(Cow<'static, str>),
}

impl UniformBlockBinding {
    /// Returns uniform block interface name.
    pub fn block_name(&self) -> &str {
        match self {
            UniformBlockBinding::FromGeometry(name)
            | UniformBlockBinding::FromMaterial(name)
            | UniformBlockBinding::FromEntity(name) => name,
        }
    }
}
