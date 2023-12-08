use super::texture::{TextureDescriptor, TextureParameter, TextureUnit};

#[derive(Clone)]
pub enum UniformValue {
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
        texture_unit: TextureUnit,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UniformBinding {
    CanvasSize,
    ModelMatrix,
    NormalMatrix,
    ViewMatrix,
    ProjMatrix,
    ViewProjMatrix,
    ActiveCameraPosition,
    ActiveCameraCenter,
    FromGeometry(&'static str),
    FromMaterial(&'static str),
    FromEntity(&'static str),
}

impl UniformBinding {
    pub fn as_str(&self) -> &str {
        match self {
            UniformBinding::CanvasSize => "u_CanvasSize",
            UniformBinding::ModelMatrix => "u_ModelMatrix",
            UniformBinding::NormalMatrix => "u_NormalMatrix",
            UniformBinding::ViewMatrix => "u_ViewMatrix",
            UniformBinding::ProjMatrix => "u_ProjMatrix",
            UniformBinding::ViewProjMatrix => "u_ViewProjMatrix",
            UniformBinding::ActiveCameraPosition => "u_ActiveCameraPosition",
            UniformBinding::ActiveCameraCenter => "u_ActiveCameraDirection",
            UniformBinding::FromGeometry(name)
            | UniformBinding::FromMaterial(name)
            | UniformBinding::FromEntity(name) => name,
        }
    }
}
