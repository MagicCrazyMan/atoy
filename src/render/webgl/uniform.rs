use super::texture::{TextureDescriptor, TextureParameter, TextureUnit};

#[derive(Clone)]
pub enum UniformValue {
    UnsignedInteger1(u32),
    UnsignedInteger2(u32, u32),
    UnsignedInteger3(u32, u32, u32),
    UnsignedInteger4(u32, u32, u32, u32),
    FloatVector1([f32; 1]),
    FloatVector2([f32; 2]),
    FloatVector3([f32; 3]),
    FloatVector4([f32; 4]),
    IntegerVector1([i32; 1]),
    IntegerVector2([i32; 1]),
    IntegerVector3([i32; 1]),
    IntegerVector4([i32; 1]),
    UnsignedIntegerVector1([u32; 1]),
    UnsignedIntegerVector2([u32; 1]),
    UnsignedIntegerVector3([u32; 1]),
    UnsignedIntegerVector4([u32; 1]),
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
    ModelMatrix,
    NormalMatrix,
    ViewMatrix,
    ProjMatrix,
    ModelViewMatrix,
    ModelViewProjMatrix,
    ViewProjMatrix,
    ActiveCameraPosition,
    ActiveCameraDirection,
    FromGeometry(&'static str),
    FromMaterial(&'static str),
    FromEntity(&'static str),
}

impl UniformBinding {
    pub fn as_str(&self) -> &str {
        match self {
            UniformBinding::ModelMatrix => "u_ModelMatrix",
            UniformBinding::NormalMatrix => "u_NormalMatrix",
            UniformBinding::ViewMatrix => "u_ViewMatrix",
            UniformBinding::ProjMatrix => "u_ProjMatrix",
            UniformBinding::ModelViewMatrix => "u_ModelViewMatrix",
            UniformBinding::ViewProjMatrix => "u_ViewProjMatrix",
            UniformBinding::ModelViewProjMatrix => "u_ModelViewProjMatrix",
            UniformBinding::ActiveCameraPosition => "u_ActiveCameraPosition",
            UniformBinding::ActiveCameraDirection => "u_ActiveCameraDirection",
            UniformBinding::FromGeometry(name)
            | UniformBinding::FromMaterial(name)
            | UniformBinding::FromEntity(name) => name,
        }
    }
}
