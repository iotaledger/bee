// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types and utilities related to packing and unpacking [`Option`] values.
use crate::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

/// Error type raised when a semantic error occurs while unpacking an option.
#[derive(Debug)]
pub enum UnpackOptionError<E> {
    /// The tag found while unpacking is not valid.
    UnknownTag(u8),
    /// A semantic error for the underlying type was raised.
    Inner(E),
}

impl<E> From<E> for UnpackOptionError<E> {
    fn from(err: E) -> Self {
        Self::Inner(err)
    }
}

/// Options are packed and unpacked using `0u8` as the prefix for `None` and `1u8` as the prefix for `Some`.
impl<T: Packable> Packable for Option<T> {
    type UnpackError = UnpackOptionError<T::UnpackError>;

    fn packed_len(&self) -> usize {
        0u8.packed_len()
            + match self {
                Some(item) => item.packed_len(),
                None => 0,
            }
    }

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        match self {
            None => 0u8.pack(packer),
            Some(item) => {
                1u8.pack(packer)?;
                item.pack(packer)
            }
        }
    }

    fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        match u8::unpack(unpacker).infallible()? {
            0 => Ok(None),
            1 => Ok(Some(T::unpack(unpacker).coerce()?)),
            n => Err(UnpackError::Packable(Self::UnpackError::UnknownTag(n))),
        }
    }
}
