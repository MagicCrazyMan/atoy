use std::{borrow::Cow, sync::OnceLock};

use regex::Regex;

use crate::value::Readonly;

use super::{
    buffer::Buffer,
    texture::{Texture, Texture2D, Texture2DArray, Texture3D, TextureCubeMap, TextureUnit},
};

/// Available uniform values.
pub enum UniformValue<'a> {
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
        texture: Readonly<'a, Texture<Texture2D>>,
        unit: TextureUnit,
    },
    Texture2DArray {
        texture: Readonly<'a, Texture<Texture2DArray>>,
        unit: TextureUnit,
    },
    Texture3D {
        texture: Readonly<'a, Texture<Texture3D>>,
        unit: TextureUnit,
    },
    TextureCubeMap {
        texture: Readonly<'a, Texture<TextureCubeMap>>,
        unit: TextureUnit,
    },
}

/// Available uniform block values.
pub enum UniformBlockValue<'a> {
    BufferBase {
        buffer: Readonly<'a, Buffer>,
        mount_point: u32,
    },
    BufferRange {
        buffer: Readonly<'a, Buffer>,
        mount_point: u32,
        offset: i32,
        size: i32,
    },
}

/// Available internal and custom uniform bindings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    FromEntity(Cow<'static, str>),
    FromGeometry(Cow<'static, str>),
    FromMaterial(Cow<'static, str>),
    Custom(Cow<'static, str>),
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
            UniformBinding::FromEntity(name)
            | UniformBinding::FromGeometry(name)
            | UniformBinding::FromMaterial(name)
            | UniformBinding::Custom(name) => &name,
        }
    }
}

/// Regular expression to find where to get value for a uniform.
const GLSL_UNIFORM_REGEX_STRING: &'static str = "u_(.*)_(.*)";

static GLSL_UNIFORM_REGEX: OnceLock<Regex> = OnceLock::new();

impl<T> From<T> for UniformBinding
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let value = value.as_ref();
        match value {
            "u_RenderTime" => UniformBinding::RenderTime,
            "u_CanvasSize" => UniformBinding::CanvasSize,
            "u_DrawingBufferSize" => UniformBinding::DrawingBufferSize,
            "u_ModelMatrix" => UniformBinding::ModelMatrix,
            "u_NormalMatrix" => UniformBinding::NormalMatrix,
            "u_ViewMatrix" => UniformBinding::ViewMatrix,
            "u_ProjMatrix" => UniformBinding::ProjMatrix,
            "u_ViewProjMatrix" => UniformBinding::ViewProjMatrix,
            "u_CameraPosition" => UniformBinding::CameraPosition,
            _ => {
                let regex = GLSL_UNIFORM_REGEX
                    .get_or_init(|| Regex::new(GLSL_UNIFORM_REGEX_STRING).unwrap());

                let name = Cow::Owned(value.to_string());

                // when regular expression capture nothing, fallback to FromMaterial
                let Some(captures) = regex.captures(value) else {
                    return UniformBinding::Custom(name);
                };
                let Some(c1) = captures.get(1) else {
                    return UniformBinding::Custom(name);
                };

                match c1.as_str() {
                    "Entity" => UniformBinding::FromEntity(name),
                    "Geometry" => UniformBinding::FromGeometry(name),
                    "Material" => UniformBinding::FromMaterial(name),
                    _ => UniformBinding::Custom(name),
                }
            }
        }
    }
}

/// Available custom uniform block bindings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum UniformBlockBinding {
    FromEntity(Cow<'static, str>),
    FromGeometry(Cow<'static, str>),
    FromMaterial(Cow<'static, str>),
    Custom(Cow<'static, str>),
}

impl UniformBlockBinding {
    /// Returns variable name.
    pub fn variable_name(&self) -> &str {
        match self {
            UniformBlockBinding::FromEntity(name)
            | UniformBlockBinding::FromGeometry(name)
            | UniformBlockBinding::FromMaterial(name)
            | UniformBlockBinding::Custom(name) => &name,
        }
    }
}

/// Regular expression to find where to get value for a uniform block.
const GLSL_UNIFORM_BLOCK_REGEX_STRING: &'static str = "ub_(.*)_(.*)";

static GLSL_UNIFORM_BLOCK_REGEX: OnceLock<Regex> = OnceLock::new();

impl<T> From<T> for UniformBlockBinding
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let value = value.as_ref();
        match value {
            _ => {
                let regex = GLSL_UNIFORM_BLOCK_REGEX
                    .get_or_init(|| Regex::new(GLSL_UNIFORM_BLOCK_REGEX_STRING).unwrap());

                let name = Cow::Owned(value.to_string());

                // when regular expression capture nothing, fallback to FromMaterial
                let Some(captures) = regex.captures(value) else {
                    return UniformBlockBinding::Custom(name);
                };
                let Some(c1) = captures.get(1) else {
                    return UniformBlockBinding::Custom(name);
                };

                match c1.as_str() {
                    "Entity" => UniformBlockBinding::FromEntity(name),
                    "Geometry" => UniformBlockBinding::FromGeometry(name),
                    "Material" => UniformBlockBinding::FromMaterial(name),
                    _ => UniformBlockBinding::Custom(name),
                }
            }
        }
    }
}
