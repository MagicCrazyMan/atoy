use std::borrow::Cow;

use gl_matrix4rust::{
    mat2::Mat2, mat3::Mat3, mat4::Mat4, vec2::Vec2, vec3::Vec3, vec4::Vec4, GLF32,
};

use crate::{material::Transparency, readonly::Readonly};

use super::{
    buffer::BufferDescriptor,
    texture::{
        texture2d::Texture2D, texture2darray::Texture2DArray, texture3d::Texture3D,
        texture_cubemap::TextureCubeMap, TextureDescriptor, TextureUnit,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum UniformValueType {
    Bool,
    Float1,
    Float2,
    Float3,
    Float4,
    UnsignedInteger1,
    UnsignedInteger2,
    UnsignedInteger3,
    UnsignedInteger4,
    Integer1,
    Integer2,
    Integer3,
    Integer4,
    FloatVector1,
    FloatVector2,
    FloatVector3,
    FloatVector4,
    IntegerVector1,
    IntegerVector2,
    IntegerVector3,
    IntegerVector4,
    UnsignedIntegerVector1,
    UnsignedIntegerVector2,
    UnsignedIntegerVector3,
    UnsignedIntegerVector4,
    Matrix2,
    Matrix3,
    Matrix4,
    Texture2D,
    Texture2DArray,
    Texture3D,
    TextureCubeMap,
}

pub trait UniformValue {
    fn uniform_type(&self) -> UniformValueType;

    fn bool(&self) -> bool {
        unimplemented!()
    }

    fn float1(&self) -> f32 {
        unimplemented!()
    }

    fn float2(&self) -> (f32, f32) {
        unimplemented!()
    }

    fn float3(&self) -> (f32, f32, f32) {
        unimplemented!()
    }

    fn float4(&self) -> (f32, f32, f32, f32) {
        unimplemented!()
    }

    fn unsigned_integer1(&self) -> u32 {
        unimplemented!()
    }

    fn unsigned_integer2(&self) -> (u32, u32) {
        unimplemented!()
    }

    fn unsigned_integer3(&self) -> (u32, u32, u32) {
        unimplemented!()
    }

    fn unsigned_integer4(&self) -> (u32, u32, u32, u32) {
        unimplemented!()
    }

    fn integer1(&self) -> i32 {
        unimplemented!()
    }

    fn integer2(&self) -> (i32, i32) {
        unimplemented!()
    }

    fn integer3(&self) -> (i32, i32, i32) {
        unimplemented!()
    }

    fn integer4(&self) -> (i32, i32, i32, i32) {
        unimplemented!()
    }

    fn float_vector1(&self) -> Readonly<'_, [f32; 1]> {
        unimplemented!()
    }

    fn float_vector2(&self) -> Readonly<'_, [f32; 2]> {
        unimplemented!()
    }

    fn float_vector3(&self) -> Readonly<'_, [f32; 3]> {
        unimplemented!()
    }

    fn float_vector4(&self) -> Readonly<'_, [f32; 4]> {
        unimplemented!()
    }

    fn integer_vector1(&self) -> Readonly<'_, [i32; 1]> {
        unimplemented!()
    }

    fn integer_vector2(&self) -> Readonly<'_, [i32; 2]> {
        unimplemented!()
    }

    fn integer_vector3(&self) -> Readonly<'_, [i32; 3]> {
        unimplemented!()
    }

    fn integer_vector4(&self) -> Readonly<'_, [i32; 4]> {
        unimplemented!()
    }

    fn unsigned_integer_vector1(&self) -> Readonly<'_, [u32; 1]> {
        unimplemented!()
    }

    fn unsigned_integer_vector2(&self) -> Readonly<'_, [u32; 2]> {
        unimplemented!()
    }

    fn unsigned_integer_vector3(&self) -> Readonly<'_, [u32; 3]> {
        unimplemented!()
    }

    fn unsigned_integer_vector4(&self) -> Readonly<'_, [u32; 4]> {
        unimplemented!()
    }

    fn matrix_transpose(&self) -> bool {
        unimplemented!()
    }

    fn matrix2(&self) -> Readonly<'_, [f32; 4]> {
        unimplemented!()
    }

    fn matrix3(&self) -> Readonly<'_, [f32; 9]> {
        unimplemented!()
    }

    fn matrix4(&self) -> Readonly<'_, [f32; 16]> {
        unimplemented!()
    }

    fn texture_unit(&self) -> TextureUnit {
        unimplemented!()
    }

    fn texture2d(&self) -> Readonly<'_, TextureDescriptor<Texture2D>> {
        unimplemented!()
    }

    fn texture2d_array(&self) -> Readonly<'_, TextureDescriptor<Texture2DArray>> {
        unimplemented!()
    }

    fn texture3d(&self) -> Readonly<'_, TextureDescriptor<Texture3D>> {
        unimplemented!()
    }

    fn texture_cube_map(&self) -> Readonly<'_, TextureDescriptor<TextureCubeMap>> {
        unimplemented!()
    }
}

impl UniformValue for bool {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Bool
    }

    fn bool(&self) -> bool {
        *self
    }
}

impl UniformValue for f32 {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Float1
    }

    fn float1(&self) -> f32 {
        *self
    }
}

impl UniformValue for (f32, f32) {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Float2
    }

    fn float2(&self) -> (f32, f32) {
        (self.0, self.1)
    }
}

impl UniformValue for (f32, f32, f32) {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Float3
    }

    fn float3(&self) -> (f32, f32, f32) {
        (self.0, self.1, self.2)
    }
}

impl UniformValue for (f32, f32, f32, f32) {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Float4
    }

    fn float4(&self) -> (f32, f32, f32, f32) {
        (self.0, self.1, self.2, self.3)
    }
}

impl UniformValue for u32 {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::UnsignedInteger1
    }

    fn unsigned_integer1(&self) -> u32 {
        *self
    }
}

impl UniformValue for (u32, u32) {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::UnsignedInteger2
    }

    fn unsigned_integer2(&self) -> (u32, u32) {
        (self.0, self.1)
    }
}

impl UniformValue for (u32, u32, u32) {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::UnsignedInteger3
    }

    fn unsigned_integer3(&self) -> (u32, u32, u32) {
        (self.0, self.1, self.2)
    }
}

impl UniformValue for (u32, u32, u32, u32) {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::UnsignedInteger4
    }

    fn unsigned_integer4(&self) -> (u32, u32, u32, u32) {
        (self.0, self.1, self.2, self.3)
    }
}

impl UniformValue for i32 {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Integer1
    }

    fn integer1(&self) -> i32 {
        *self
    }
}

impl UniformValue for (i32, i32) {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Integer2
    }

    fn integer2(&self) -> (i32, i32) {
        (self.0, self.1)
    }
}

impl UniformValue for (i32, i32, i32) {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Integer3
    }

    fn integer3(&self) -> (i32, i32, i32) {
        (self.0, self.1, self.2)
    }
}

impl UniformValue for (i32, i32, i32, i32) {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Integer4
    }

    fn integer4(&self) -> (i32, i32, i32, i32) {
        (self.0, self.1, self.2, self.3)
    }
}

impl UniformValue for [f32; 1] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::FloatVector1
    }

    fn float_vector1(&self) -> Readonly<'_, [f32; 1]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for [f32; 2] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::FloatVector2
    }

    fn float_vector2(&self) -> Readonly<'_, [f32; 2]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for [f32; 3] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::FloatVector3
    }

    fn float_vector3(&self) -> Readonly<'_, [f32; 3]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for [f32; 4] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::FloatVector4
    }

    fn float_vector4(&self) -> Readonly<'_, [f32; 4]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for [i32; 1] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::IntegerVector1
    }

    fn integer_vector1(&self) -> Readonly<'_, [i32; 1]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for [i32; 2] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::IntegerVector2
    }

    fn integer_vector2(&self) -> Readonly<'_, [i32; 2]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for [i32; 3] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::IntegerVector3
    }

    fn integer_vector3(&self) -> Readonly<'_, [i32; 3]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for [i32; 4] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::IntegerVector4
    }

    fn integer_vector4(&self) -> Readonly<'_, [i32; 4]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for [u32; 1] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::UnsignedIntegerVector1
    }

    fn unsigned_integer_vector1(&self) -> Readonly<'_, [u32; 1]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for [u32; 2] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::UnsignedIntegerVector2
    }

    fn unsigned_integer_vector2(&self) -> Readonly<'_, [u32; 2]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for [u32; 3] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::UnsignedIntegerVector3
    }

    fn unsigned_integer_vector3(&self) -> Readonly<'_, [u32; 3]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for [u32; 4] {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::UnsignedIntegerVector4
    }

    fn unsigned_integer_vector4(&self) -> Readonly<'_, [u32; 4]> {
        Readonly::Borrowed(self)
    }
}

impl UniformValue for Vec2<f32> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::FloatVector2
    }

    fn float_vector2(&self) -> Readonly<'_, [f32; 2]> {
        Readonly::Borrowed(self.raw())
    }
}

impl UniformValue for Vec2<f64> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::FloatVector2
    }

    fn float_vector2(&self) -> Readonly<'_, [f32; 2]> {
        Readonly::Owned(self.gl_f32())
    }
}

impl UniformValue for Vec3<f32> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::FloatVector3
    }

    fn float_vector3(&self) -> Readonly<'_, [f32; 3]> {
        Readonly::Borrowed(self.raw())
    }
}

impl UniformValue for Vec3<f64> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::FloatVector3
    }

    fn float_vector3(&self) -> Readonly<'_, [f32; 3]> {
        Readonly::Owned(self.gl_f32())
    }
}

impl UniformValue for Vec4<f32> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::FloatVector4
    }

    fn float_vector4(&self) -> Readonly<'_, [f32; 4]> {
        Readonly::Borrowed(self.raw())
    }
}

impl UniformValue for Vec4<f64> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::FloatVector4
    }

    fn float_vector4(&self) -> Readonly<'_, [f32; 4]> {
        Readonly::Owned(self.gl_f32())
    }
}

impl UniformValue for Mat2<f32> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Matrix2
    }

    fn matrix_transpose(&self) -> bool {
        false
    }

    fn matrix2(&self) -> Readonly<'_, [f32; 4]> {
        Readonly::Borrowed(self.raw())
    }
}

impl UniformValue for Mat2<f64> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Matrix2
    }

    fn matrix_transpose(&self) -> bool {
        false
    }

    fn matrix2(&self) -> Readonly<'_, [f32; 4]> {
        Readonly::Owned(self.gl_f32())
    }
}

impl UniformValue for Mat3<f32> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Matrix3
    }

    fn matrix_transpose(&self) -> bool {
        false
    }

    fn matrix3(&self) -> Readonly<'_, [f32; 9]> {
        Readonly::Borrowed(self.raw())
    }
}

impl UniformValue for Mat3<f64> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Matrix3
    }

    fn matrix_transpose(&self) -> bool {
        false
    }

    fn matrix3(&self) -> Readonly<'_, [f32; 9]> {
        Readonly::Owned(self.gl_f32())
    }
}

impl UniformValue for Mat4<f32> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Matrix4
    }

    fn matrix_transpose(&self) -> bool {
        false
    }

    fn matrix4(&self) -> Readonly<'_, [f32; 16]> {
        Readonly::Borrowed(self.raw())
    }
}

impl UniformValue for Mat4<f64> {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Matrix4
    }

    fn matrix_transpose(&self) -> bool {
        false
    }

    fn matrix4(&self) -> Readonly<'_, [f32; 16]> {
        Readonly::Owned(self.gl_f32())
    }
}

impl UniformValue for Transparency {
    fn uniform_type(&self) -> UniformValueType {
        UniformValueType::Float1
    }

    fn float1(&self) -> f32 {
        Readonly::Owned(self).alpha()
    }
}

pub trait AsUniformValue {
    fn as_uniform_value(&self) -> &dyn UniformValue;
}

impl<T> UniformValue for T
where
    T: AsUniformValue,
{
    fn uniform_type(&self) -> UniformValueType {
        self.as_uniform_value().uniform_type()
    }

    fn bool(&self) -> bool {
        self.as_uniform_value().bool()
    }

    fn float1(&self) -> f32 {
        self.as_uniform_value().float1()
    }

    fn float2(&self) -> (f32, f32) {
        self.as_uniform_value().float2()
    }

    fn float3(&self) -> (f32, f32, f32) {
        self.as_uniform_value().float3()
    }

    fn float4(&self) -> (f32, f32, f32, f32) {
        self.as_uniform_value().float4()
    }

    fn unsigned_integer1(&self) -> u32 {
        self.as_uniform_value().unsigned_integer1()
    }

    fn unsigned_integer2(&self) -> (u32, u32) {
        self.as_uniform_value().unsigned_integer2()
    }

    fn unsigned_integer3(&self) -> (u32, u32, u32) {
        self.as_uniform_value().unsigned_integer3()
    }

    fn unsigned_integer4(&self) -> (u32, u32, u32, u32) {
        self.as_uniform_value().unsigned_integer4()
    }

    fn integer1(&self) -> i32 {
        self.as_uniform_value().integer1()
    }

    fn integer2(&self) -> (i32, i32) {
        self.as_uniform_value().integer2()
    }

    fn integer3(&self) -> (i32, i32, i32) {
        self.as_uniform_value().integer3()
    }

    fn integer4(&self) -> (i32, i32, i32, i32) {
        self.as_uniform_value().integer4()
    }

    fn float_vector1(&self) -> Readonly<'_, [f32; 1]> {
        self.as_uniform_value().float_vector1()
    }

    fn float_vector2(&self) -> Readonly<'_, [f32; 2]> {
        self.as_uniform_value().float_vector2()
    }

    fn float_vector3(&self) -> Readonly<'_, [f32; 3]> {
        self.as_uniform_value().float_vector3()
    }

    fn float_vector4(&self) -> Readonly<'_, [f32; 4]> {
        self.as_uniform_value().float_vector4()
    }

    fn integer_vector1(&self) -> Readonly<'_, [i32; 1]> {
        self.as_uniform_value().integer_vector1()
    }

    fn integer_vector2(&self) -> Readonly<'_, [i32; 2]> {
        self.as_uniform_value().integer_vector2()
    }

    fn integer_vector3(&self) -> Readonly<'_, [i32; 3]> {
        self.as_uniform_value().integer_vector3()
    }

    fn integer_vector4(&self) -> Readonly<'_, [i32; 4]> {
        self.as_uniform_value().integer_vector4()
    }

    fn unsigned_integer_vector1(&self) -> Readonly<'_, [u32; 1]> {
        self.as_uniform_value().unsigned_integer_vector1()
    }

    fn unsigned_integer_vector2(&self) -> Readonly<'_, [u32; 2]> {
        self.as_uniform_value().unsigned_integer_vector2()
    }

    fn unsigned_integer_vector3(&self) -> Readonly<'_, [u32; 3]> {
        self.as_uniform_value().unsigned_integer_vector3()
    }

    fn unsigned_integer_vector4(&self) -> Readonly<'_, [u32; 4]> {
        self.as_uniform_value().unsigned_integer_vector4()
    }

    fn matrix_transpose(&self) -> bool {
        self.as_uniform_value().matrix_transpose()
    }

    fn matrix2(&self) -> Readonly<'_, [f32; 4]> {
        self.as_uniform_value().matrix2()
    }

    fn matrix3(&self) -> Readonly<'_, [f32; 9]> {
        self.as_uniform_value().matrix3()
    }

    fn matrix4(&self) -> Readonly<'_, [f32; 16]> {
        self.as_uniform_value().matrix4()
    }

    fn texture_unit(&self) -> TextureUnit {
        self.as_uniform_value().texture_unit()
    }

    fn texture2d(&self) -> Readonly<'_, TextureDescriptor<Texture2D>> {
        self.as_uniform_value().texture2d()
    }

    fn texture2d_array(&self) -> Readonly<'_, TextureDescriptor<Texture2DArray>> {
        self.as_uniform_value().texture2d_array()
    }

    fn texture3d(&self) -> Readonly<'_, TextureDescriptor<Texture3D>> {
        self.as_uniform_value().texture3d()
    }

    fn texture_cube_map(&self) -> Readonly<'_, TextureDescriptor<TextureCubeMap>> {
        self.as_uniform_value().texture_cube_map()
    }
}

impl<'a, T> AsUniformValue for Readonly<'a, T>
where
    T: UniformValue,
{
    fn as_uniform_value(&self) -> &dyn UniformValue {
        self.as_ref()
    }
}

impl<'a, T> AsUniformValue for Cow<'a, T>
where
    T: UniformValue + Clone,
{
    fn as_uniform_value(&self) -> &dyn UniformValue {
        self.as_ref()
    }
}

impl<T> AsUniformValue for T
where
    T: AsRef<dyn UniformValue>,
{
    fn as_uniform_value(&self) -> &dyn UniformValue {
        self.as_ref()
    }
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
