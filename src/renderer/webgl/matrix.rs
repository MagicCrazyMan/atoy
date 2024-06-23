use gl_matrix4rust::{mat2::Mat2, mat3::Mat3, mat4::Mat4, vec2::Vec2, vec3::Vec3, vec4::Vec4};

pub trait GlF32<const N: usize> {
    fn to_f32_array(&self) -> [f32; N];
}

pub trait GlI32<const N: usize> {
    fn to_i32_array(&self) -> [i32; N];
}

pub trait GlU32<const N: usize> {
    fn to_u32_array(&self) -> [u32; N];
}

pub trait ToF32 {
    fn to_f32(self) -> f32;
}

pub trait ToI32 {
    fn to_i32(self) -> i32;
}

pub trait ToU32 {
    fn to_u32(self) -> u32;
}

macro_rules! impl_integers {
    ($($i: tt),+) => {
        $(
            impl ToU32 for $i {
                fn to_u32(self) -> u32 {
                    self as u32
                }
            }

            impl ToI32 for $i {
                fn to_i32(self) -> i32 {
                    self as i32
                }
            }

            impl ToF32 for $i {
                fn to_f32(self) -> f32 {
                    self as f32
                }
            }
        )+
    };
}

macro_rules! impl_decimals {
    ($($f: tt),+) => {
        $(
            impl ToF32 for $f {
                fn to_f32(self) -> f32 {
                    self as f32
                }
            }
        )+
    };
}

impl_integers!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);
impl_decimals!(f32, f64);

impl<T: Copy + ToF32> GlF32<2> for Vec2<T> {
    fn to_f32_array(&self) -> [f32; 2] {
        [self.0.to_f32(), self.1.to_f32()]
    }
}

impl<T: Copy + ToF32> GlF32<3> for Vec3<T> {
    fn to_f32_array(&self) -> [f32; 3] {
        [self.0.to_f32(), self.1.to_f32(), self.2.to_f32()]
    }
}

impl<T: Copy + ToF32> GlF32<4> for Vec4<T> {
    fn to_f32_array(&self) -> [f32; 4] {
        [
            self.0.to_f32(),
            self.1.to_f32(),
            self.2.to_f32(),
            self.3.to_f32(),
        ]
    }
}

impl<T: Copy + ToF32> GlF32<4> for Mat2<T> {
    fn to_f32_array(&self) -> [f32; 4] {
        [
            self.0.to_f32(),
            self.1.to_f32(),
            self.2.to_f32(),
            self.3.to_f32(),
        ]
    }
}

impl<T: Copy + ToF32> GlF32<9> for Mat3<T> {
    fn to_f32_array(&self) -> [f32; 9] {
        [
            self.0.to_f32(),
            self.1.to_f32(),
            self.2.to_f32(),
            self.3.to_f32(),
            self.4.to_f32(),
            self.5.to_f32(),
            self.6.to_f32(),
            self.7.to_f32(),
            self.8.to_f32(),
        ]
    }
}

impl<T: Copy + ToF32> GlF32<16> for Mat4<T> {
    fn to_f32_array(&self) -> [f32; 16] {
        [
            self.0.to_f32(),
            self.1.to_f32(),
            self.2.to_f32(),
            self.3.to_f32(),
            self.4.to_f32(),
            self.5.to_f32(),
            self.6.to_f32(),
            self.7.to_f32(),
            self.8.to_f32(),
            self.9.to_f32(),
            self.10.to_f32(),
            self.11.to_f32(),
            self.12.to_f32(),
            self.13.to_f32(),
            self.14.to_f32(),
            self.15.to_f32(),
        ]
    }
}

impl<T: Copy + ToI32> GlI32<2> for Vec2<T> {
    fn to_i32_array(&self) -> [i32; 2] {
        [self.0.to_i32(), self.1.to_i32()]
    }
}

impl<T: Copy + ToI32> GlI32<3> for Vec3<T> {
    fn to_i32_array(&self) -> [i32; 3] {
        [self.0.to_i32(), self.1.to_i32(), self.2.to_i32()]
    }
}

impl<T: Copy + ToI32> GlI32<4> for Vec4<T> {
    fn to_i32_array(&self) -> [i32; 4] {
        [
            self.0.to_i32(),
            self.1.to_i32(),
            self.2.to_i32(),
            self.3.to_i32(),
        ]
    }
}

impl<T: Copy + ToI32> GlI32<4> for Mat2<T> {
    fn to_i32_array(&self) -> [i32; 4] {
        [
            self.0.to_i32(),
            self.1.to_i32(),
            self.2.to_i32(),
            self.3.to_i32(),
        ]
    }
}

impl<T: Copy + ToI32> GlI32<9> for Mat3<T> {
    fn to_i32_array(&self) -> [i32; 9] {
        [
            self.0.to_i32(),
            self.1.to_i32(),
            self.2.to_i32(),
            self.3.to_i32(),
            self.4.to_i32(),
            self.5.to_i32(),
            self.6.to_i32(),
            self.7.to_i32(),
            self.8.to_i32(),
        ]
    }
}

impl<T: Copy + ToI32> GlI32<16> for Mat4<T> {
    fn to_i32_array(&self) -> [i32; 16] {
        [
            self.0.to_i32(),
            self.1.to_i32(),
            self.2.to_i32(),
            self.3.to_i32(),
            self.4.to_i32(),
            self.5.to_i32(),
            self.6.to_i32(),
            self.7.to_i32(),
            self.8.to_i32(),
            self.9.to_i32(),
            self.10.to_i32(),
            self.11.to_i32(),
            self.12.to_i32(),
            self.13.to_i32(),
            self.14.to_i32(),
            self.15.to_i32(),
        ]
    }
}

impl<T: Copy + ToU32> GlU32<2> for Vec2<T> {
    fn to_u32_array(&self) -> [u32; 2] {
        [self.0.to_u32(), self.1.to_u32()]
    }
}

impl<T: Copy + ToU32> GlU32<3> for Vec3<T> {
    fn to_u32_array(&self) -> [u32; 3] {
        [self.0.to_u32(), self.1.to_u32(), self.2.to_u32()]
    }
}

impl<T: Copy + ToU32> GlU32<4> for Vec4<T> {
    fn to_u32_array(&self) -> [u32; 4] {
        [
            self.0.to_u32(),
            self.1.to_u32(),
            self.2.to_u32(),
            self.3.to_u32(),
        ]
    }
}

impl<T: Copy + ToU32> GlU32<4> for Mat2<T> {
    fn to_u32_array(&self) -> [u32; 4] {
        [
            self.0.to_u32(),
            self.1.to_u32(),
            self.2.to_u32(),
            self.3.to_u32(),
        ]
    }
}

impl<T: Copy + ToU32> GlU32<9> for Mat3<T> {
    fn to_u32_array(&self) -> [u32; 9] {
        [
            self.0.to_u32(),
            self.1.to_u32(),
            self.2.to_u32(),
            self.3.to_u32(),
            self.4.to_u32(),
            self.5.to_u32(),
            self.6.to_u32(),
            self.7.to_u32(),
            self.8.to_u32(),
        ]
    }
}

impl<T: Copy + ToU32> GlU32<16> for Mat4<T> {
    fn to_u32_array(&self) -> [u32; 16] {
        [
            self.0.to_u32(),
            self.1.to_u32(),
            self.2.to_u32(),
            self.3.to_u32(),
            self.4.to_u32(),
            self.5.to_u32(),
            self.6.to_u32(),
            self.7.to_u32(),
            self.8.to_u32(),
            self.9.to_u32(),
            self.10.to_u32(),
            self.11.to_u32(),
            self.12.to_u32(),
            self.13.to_u32(),
            self.14.to_u32(),
            self.15.to_u32(),
        ]
    }
}

// pub trait GlMatrix<const f32Len: usize, const u8Len: usize> {
//     fn to_f32_array(&self) -> [f32; f32Len];

//     fn to_u8_array(&self) -> [u8; u8Len];

//     fn fill_array_buffer(&self, buffer: &ArrayBuffer);

//     fn fill_array_buffer_with_offset(&self, buffer: &ArrayBuffer, offset: usize);
// }

// impl<T> GlMatrix<2, 8> for Vec2<T> {
//     fn to_u8_array(&self) -> [u8; 8] {
//         todo!()
//     }

//     fn fill_array_buffer(&self, buffer: &ArrayBuffer) {
//         todo!()
//     }

//     fn fill_array_buffer_with_offset(&self, buffer: &ArrayBuffer, offset: usize) {
//         todo!()
//     }
// }
