use std::sync::Arc;

/// Represents the messages in this system.
#[allow(missing_docs)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Effect
{
    Empty,
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Bool(bool),
    Char(char),
    String(Arc<String>),
    Bytes(Arc<Vec<u8>>)
}

macro_rules! impl_from_primitive {
    ($type:ty, $variant:ident) => {
        impl From<$type> for Effect
        {
            fn from(p: $type) -> Self
            {
                Effect::$variant(p)
            }
        }
    };
}

impl_from_primitive!(u8, U8);
impl_from_primitive!(u16, U16);
impl_from_primitive!(u32, U32);
impl_from_primitive!(u64, U64);
impl_from_primitive!(i8, I8);
impl_from_primitive!(i16, I16);
impl_from_primitive!(i32, I32);
impl_from_primitive!(i64, I64);
impl_from_primitive!(bool, Bool);
impl_from_primitive!(char, Char);

macro_rules! from_unsized {
    ($type:ty, $variant:ident) => {
        impl From<$type> for Effect
        {
            fn from(a: $type) -> Self
            {
                Effect::$variant(Arc::new(a))
            }
        }
    };
}

from_unsized!(String, String);
from_unsized!(Vec<u8>, Bytes);

impl From<&str> for Effect
{
    fn from(s: &str) -> Self
    {
        Effect::String(Arc::new(String::from(s)))
    }
}
