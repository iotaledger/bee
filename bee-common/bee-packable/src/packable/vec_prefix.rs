// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

pub use crate::{
    error::{UnknownTagError, UnpackError},
    packer::{Packer, VecPacker},
    unpacker::{SliceUnpacker, UnexpectedEOF, Unpacker},
    Packable,
};

use alloc::vec::Vec;
use core::marker::PhantomData;

/// Wrapper type for `Vec<T>` where the length prefix is of type `P`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VecPrefix<T, P> {
    inner: Vec<T>,
    marker: PhantomData<P>,
}

impl<T, P> VecPrefix<T, P> {
    /// Creates a new empty `VecPrefix<T, P>`.
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Creates a new empty `VecPrefix<T, P>` with a specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            marker: PhantomData,
        }
    }
}

impl<T, P> Default for VecPrefix<T, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, P> From<Vec<T>> for VecPrefix<T, P> {
    fn from(vec: Vec<T>) -> Self {
        Self {
            inner: vec,
            marker: PhantomData,
        }
    }
}

impl<T, P> core::ops::Deref for VecPrefix<T, P> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, P> core::ops::DerefMut for VecPrefix<T, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

macro_rules! impl_packable_for_vec_prefix {
    ($ty:ty) => {
        impl<T: Packable> Packable for VecPrefix<T, $ty> {
            type Error = T::Error;

            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                // The length of any dynamically-sized sequence must be prefixed.
                (self.len() as $ty).pack(packer)?;

                for item in self.iter() {
                    item.pack(packer)?;
                }

                Ok(())
            }

            fn packed_len(&self) -> usize {
                (0 as $ty).packed_len() + self.iter().map(T::packed_len).sum::<usize>()
            }

            fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::Error, U::Error>> {
                // The length of any dynamically-sized sequence must be prefixed.
                let len = <$ty>::unpack(unpacker).map_err(UnpackError::infallible)?;

                let mut vec = Self::with_capacity(len as usize);

                for _ in 0..len {
                    let item = T::unpack(unpacker)?;
                    vec.push(item);
                }

                Ok(vec)
            }
        }
    };
}

impl_packable_for_vec_prefix!(u8);
impl_packable_for_vec_prefix!(u16);
impl_packable_for_vec_prefix!(u32);
impl_packable_for_vec_prefix!(u64);
#[cfg(has_u128)]
impl_packable_for_vec_prefix!(u128);
