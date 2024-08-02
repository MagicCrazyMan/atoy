use std::any::TypeId;

use uuid::Uuid;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(not(feature = "string_key"), derive(Copy))]
pub enum Key {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    U128(u128),
    I128(i128),
    Usize(usize),
    Isize(isize),
    Uuid(Uuid),
    TypeId(TypeId),
    Str(&'static str),
    #[cfg(feature = "string_key")]
    String(String),
}
