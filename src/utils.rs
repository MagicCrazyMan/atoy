use gl_matrix4rust::vec3::Vec3;
use web_sys::js_sys::{
    BigInt64Array, BigUint64Array, Float32Array, Float64Array, Int16Array, Int32Array, Int8Array,
    Uint16Array, Uint32Array, Uint8Array, Uint8ClampedArray,
};

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

macro_rules! slice_to_typed_array {
    ($(($n: ident, $s: tt, $b: tt, $dd: expr, $ss: expr)),+) => {
        $(
            #[doc = "Creates a JavaScript "]
            #[doc = $dd]
            #[doc = " from a "]
            #[doc = $ss]
            #[doc = " slice."]
            pub fn $n(slice: &$s) -> $b {
                let buffer = $b::new_with_length(slice.len() as u32);
                buffer.copy_from(slice);
                buffer
            }
        )+
    };
}

slice_to_typed_array! {
    (slice_to_int8_array, [i8], Int8Array, "[`Int8Array`]", r"`[u8]`"),
    (slice_to_uint8_array, [u8], Uint8Array, "[`Uint8Array`]", r"`[u8]`"),
    (slice_to_uint8_clamped_array, [u8], Uint8ClampedArray, "[`Uint8ClampedArray`]", r"`[u8]`"),
    (slice_to_int16_array, [i16], Int16Array, "[`Int16Array`]", r"`[i16]`"),
    (slice_to_uint16_array, [u16], Uint16Array, "[`Uint16Array`]", r"`[u16]`"),
    (slice_to_int32_array, [i32], Int32Array, "[`Int32Array`]", r"`[i32]`"),
    (slice_to_uint32_array, [u32], Uint32Array, "[`Uint32Array`]", r"`[u32]`"),
    (slice_to_float32_array, [f32], Float32Array, "[`Float32Array`]", r"`[f32]`"),
    (slice_to_float64_array, [f64], Float64Array, "[`Float64Array`]", r"`[f64]`"),
    (slice_to_big_int64_array, [i64], BigInt64Array, "[`BigInt64Array`]", r"`[i64]`"),
    (slice_to_big_uint64_array, [u64], BigUint64Array, "[`BigUint64Array`]", r"`[u64]`")
}

/// Calculates distance between a point and a plane.
/// A plane is defined by a point on plane and a normal.
/// Applies normalization to the normal before invoking this function,
/// this function does not normalize it again.
///
/// Positive & Negative: if point inside the space the normal points to,
/// returning a positive distance value, otherwise, returning a negative value.
/// If you wish to get the positive value always, use [`distance_point_and_plane_abs`].
#[inline]
pub fn distance_point_and_plane(p: &Vec3<f64>, pop: &Vec3<f64>, n: &Vec3<f64>) -> f64 {
    (*p - *pop).dot(n)
}

/// Absolution version of [`distance_point_and_plane`].
/// Sees [`distance_point_and_plane`] for more details.
#[inline]
pub fn distance_point_and_plane_abs(p: &Vec3<f64>, pop: &Vec3<f64>, n: &Vec3<f64>) -> f64 {
    distance_point_and_plane(p, pop, n).abs()
}

/// Formats bytes length to human readable string following rules below:
///
/// - For `N == 0 || N == 1`, uses unit `Byte`.
/// - For `N <= 10240`, uses unit `Bytes`.
/// - For `12034 < N <= 10485760`, uses unit `KiB` and no digit.
/// - For others, uses unit `MiB` and no digit.
#[inline]
pub fn format_byte_length(len: usize) -> String {
    if len == 0 || len == 1 {
        format!("{} Byte", len)
    } else if len <= 10240 {
        format!("{} Bytes", len)
    } else if len <= 10485760 {
        format!("{} KiB", len / 1024)
    } else {
        format!("{} MiB", len / 1024 / 1024)
    }
}

#[cfg(test)]
mod tests {
    use gl_matrix4rust::vec3::Vec3;

    use crate::utils::distance_point_and_plane;

    #[test]
    fn test_distance_point_and_plane() {
        let pop = Vec3::<f64>::new(0.0, 0.0, 0.0);
        let n = Vec3::<f64>::new(1.0, 1.0, 0.0).normalize();

        assert_eq!(
            7.071067811865475,
            distance_point_and_plane(&Vec3::<f64>::new(10.0, 0.0, 0.0), &pop, &n)
        );
        assert_eq!(
            -7.071067811865475,
            distance_point_and_plane(&Vec3::<f64>::new(-10.0, 0.0, 0.0), &pop, &n)
        );
        assert_eq!(
            14.14213562373095,
            distance_point_and_plane(&Vec3::<f64>::new(10.0, 10.0, 0.0), &pop, &n)
        );
    }
}
