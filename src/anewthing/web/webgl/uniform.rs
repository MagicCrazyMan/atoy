use std::fmt::Debug;

use nalgebra::{
    Matrix2, Matrix2x3, Matrix2x4, Matrix3, Matrix3x2, Matrix3x4, Matrix4, Matrix4x2, Matrix4x3,
    Vector1, Vector2, Vector3, Vector4,
};

use super::buffer::WebGlBuffering;

/// Available uniform values.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WebGlUniformValue<'a> {
    Bool(bool),
    Texture(i32),
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
    FloatVector1(&'a Vector1<f32>),
    FloatVector2(&'a Vector2<f32>),
    FloatVector3(&'a Vector3<f32>),
    FloatVector4(&'a Vector4<f32>),
    IntegerVector1(&'a Vector1<i32>),
    IntegerVector2(&'a Vector2<i32>),
    IntegerVector3(&'a Vector3<i32>),
    IntegerVector4(&'a Vector4<i32>),
    UnsignedIntegerVector1(&'a Vector1<u32>),
    UnsignedIntegerVector2(&'a Vector2<u32>),
    UnsignedIntegerVector3(&'a Vector3<u32>),
    UnsignedIntegerVector4(&'a Vector4<u32>),
    Matrix2 {
        data: &'a Matrix2<f32>,
        transpose: bool,
    },
    Matrix3 {
        data: &'a Matrix3<f32>,
        transpose: bool,
    },
    Matrix4 {
        data: &'a Matrix4<f32>,
        transpose: bool,
    },
    Matrix3x2 {
        data: &'a Matrix3x2<f32>,
        transpose: bool,
    },
    Matrix4x2 {
        data: &'a Matrix4x2<f32>,
        transpose: bool,
    },
    Matrix2x3 {
        data: &'a Matrix2x3<f32>,
        transpose: bool,
    },
    Matrix4x3 {
        data: &'a Matrix4x3<f32>,
        transpose: bool,
    },
    Matrix2x4 {
        data: &'a Matrix2x4<f32>,
        transpose: bool,
    },
    Matrix3x4 {
        data: &'a Matrix3x4<f32>,
        transpose: bool,
    },
}

/// Uniform block value.
#[derive(Clone, Copy)]
pub enum WebGlUniformBlockValue<'a> {
    Base {
        buffer: &'a WebGlBuffering,
        mount_point: usize,
    },
    Range {
        buffer: &'a WebGlBuffering,
        mount_point: usize,
        byte_offset: usize,
        byte_length: Option<usize>,
    },
}

impl<'a> Debug for WebGlUniformBlockValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base { mount_point, .. } => f
                .debug_struct("Base")
                .field("mount_point", mount_point)
                .finish(),
            Self::Range {
                mount_point,
                byte_offset,
                byte_length,
                ..
            } => f
                .debug_struct("Range")
                .field("mount_point", mount_point)
                .field("byte_offset", byte_offset)
                .field("byte_length", byte_length)
                .finish(),
        }
    }
}
