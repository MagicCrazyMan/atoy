use web_sys::js_sys::{Uint8Array, Int8Array, Uint8ClampedArray, Int16Array, Uint16Array, Int32Array, Uint32Array, Float32Array, Float64Array, BigInt64Array, BigUint64Array};

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