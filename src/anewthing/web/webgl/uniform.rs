use nalgebra::{Matrix2, Matrix3, Matrix4, Vector1, Vector2, Vector3, Vector4};



/// Available uniform values.
pub enum WebGlUniformValue {
    Bool(bool),
    Texture(u32),
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
    FloatVector1(Vector1<f32>),
    FloatVector2(Vector2<f32>),
    FloatVector3(Vector3<f32>),
    FloatVector4(Vector4<f32>),
    IntegerVector1(Vector1<i32>),
    IntegerVector2(Vector2<i32>),
    IntegerVector3(Vector3<i32>),
    IntegerVector4(Vector4<i32>),
    UnsignedIntegerVector1(Vector1<u32>),
    UnsignedIntegerVector2(Vector2<u32>),
    UnsignedIntegerVector3(Vector3<u32>),
    UnsignedIntegerVector4(Vector4<u32>),
    Matrix2 { data: Matrix2<f32>, transpose: bool },
    Matrix3 { data: Matrix3<f32>, transpose: bool },
    Matrix4 { data: Matrix4<f32>, transpose: bool },
}
