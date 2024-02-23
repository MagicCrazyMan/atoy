use super::{
    buffer::BufferDescriptor,
    texture::{
        texture2d::Texture2D, texture2darray::Texture2DArray, texture3d::Texture3D,
        texture_cubemap::TextureCubeMap, TextureDescriptor, TextureUnit,
    },
};

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
    Texture2D {
        descriptor: TextureDescriptor<Texture2D>,
        unit: TextureUnit,
    },
    Texture2DArray {
        descriptor: TextureDescriptor<Texture2DArray>,
        unit: TextureUnit,
    },
    Texture3D {
        descriptor: TextureDescriptor<Texture3D>,
        unit: TextureUnit,
    },
    TextureCubeMap {
        descriptor: TextureDescriptor<TextureCubeMap>,
        unit: TextureUnit,
    },
}

/// Available uniform block values.
pub enum UniformBlockValue {
    BufferBase {
        descriptor: BufferDescriptor,
        binding: u32,
    },
    BufferRange {
        descriptor: BufferDescriptor,
        binding: u32,
        offset: usize,
        size: usize,
    },
}

/// Uniform internal bindings.
#[derive(Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UniformInternalBinding {
    RenderTime,
    CanvasSize,
    DrawingBufferSize,
    ModelMatrix,
    NormalMatrix,
    ViewMatrix,
    ProjMatrix,
    ViewProjMatrix,
    CameraPosition,
}

impl UniformInternalBinding {
    /// Returns variable name.
    pub fn variable_name(&self) -> &str {
        match self {
            UniformInternalBinding::RenderTime => "u_RenderTime",
            UniformInternalBinding::CanvasSize => "u_CanvasSize",
            UniformInternalBinding::DrawingBufferSize => "u_DrawingBufferSize",
            UniformInternalBinding::ModelMatrix => "u_ModelMatrix",
            UniformInternalBinding::NormalMatrix => "u_NormalMatrix",
            UniformInternalBinding::ViewMatrix => "u_ViewMatrix",
            UniformInternalBinding::ProjMatrix => "u_ProjMatrix",
            UniformInternalBinding::ViewProjMatrix => "u_ViewProjMatrix",
            UniformInternalBinding::CameraPosition => "u_CameraPosition",
        }
    }

    /// Tries to find uniform internal binding from a variable name.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "u_RenderTime" => Some(UniformInternalBinding::RenderTime),
            "u_CanvasSize" => Some(UniformInternalBinding::CanvasSize),
            "u_DrawingBufferSize" => Some(UniformInternalBinding::DrawingBufferSize),
            "u_ModelMatrix" => Some(UniformInternalBinding::ModelMatrix),
            "u_NormalMatrix" => Some(UniformInternalBinding::NormalMatrix),
            "u_ViewMatrix" => Some(UniformInternalBinding::ViewMatrix),
            "u_ProjMatrix" => Some(UniformInternalBinding::ProjMatrix),
            "u_ViewProjMatrix" => Some(UniformInternalBinding::ViewProjMatrix),
            "u_CameraPosition" => Some(UniformInternalBinding::CameraPosition),
            _ => None,
        }
    }
}

// /// Uniform block binding sources.
// #[derive(Clone, PartialEq, Eq, Hash)]
// #[non_exhaustive]
// pub enum UniformBlockBinding {
//     FromGeometry(Cow<'static, str>),
//     FromMaterial(Cow<'static, str>),
//     FromEntity(Cow<'static, str>),
// }

// impl UniformBlockBinding {
//     /// Returns uniform block interface name.
//     pub fn block_name(&self) -> &str {
//         match self {
//             UniformBlockBinding::FromGeometry(name)
//             | UniformBlockBinding::FromMaterial(name)
//             | UniformBlockBinding::FromEntity(name) => name,
//         }
//     }
// }