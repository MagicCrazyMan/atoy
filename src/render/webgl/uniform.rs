use std::borrow::Cow;

use super::{
    buffer::BufferDescriptor,
    texture::{
        texture2d::Texture2D, texture2darray::Texture2DArrayBase, texture3d::Texture3DBase,
        texture_cubemap::TextureCubeMapBase, TextureCompressedFormat, TextureDescriptor,
        TextureInternalFormat, TextureUnit,
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
    Texture3D {
        descriptor: TextureDescriptor<Texture3DBase<TextureInternalFormat>>,
        unit: TextureUnit,
    },
    Texture3DCompressed {
        descriptor: TextureDescriptor<Texture3DBase<TextureCompressedFormat>>,
        unit: TextureUnit,
    },
    Texture2DArray {
        descriptor: TextureDescriptor<Texture2DArrayBase<TextureInternalFormat>>,
        unit: TextureUnit,
    },
    Texture2DArrayCompressed {
        descriptor: TextureDescriptor<Texture2DArrayBase<TextureCompressedFormat>>,
        unit: TextureUnit,
    },
    TextureCubeMap {
        descriptor: TextureDescriptor<TextureCubeMapBase<TextureInternalFormat>>,
        unit: TextureUnit,
    },
    TextureCubeMapCompressed {
        descriptor: TextureDescriptor<TextureCubeMapBase<TextureCompressedFormat>>,
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
