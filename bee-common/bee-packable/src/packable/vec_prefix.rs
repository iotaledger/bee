// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use crate::wrap::Wrap;
pub use crate::{
    error::{PackError, PrefixError, UnknownTagError, UnpackError},
    packer::{Packer, VecPacker},
    unpacker::{SliceUnpacker, UnexpectedEOF, Unpacker},
    Packable,
};

use alloc::vec::Vec;
use core::{convert::TryFrom, marker::PhantomData};

/// Wrapper type for `Vec<T>` where the length prefix is of type `P`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
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

/// We cannot provide a `From` implementation because `Vec` is not from this crate.
#[allow(clippy::from_over_into)]
impl<T, P> Into<Vec<T>> for VecPrefix<T, P> {
    fn into(self) -> Vec<T> {
        self.inner
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

macro_rules! impl_wrap_for_vec {
    ($ty:ty) => {
        impl<T> Wrap<VecPrefix<T, $ty>> for Vec<T> {
            fn wrap(&self) -> &VecPrefix<T, $ty> {
                unsafe { &*(self as *const Vec<T> as *const VecPrefix<T, $ty>) }
            }
        }
    };
}

impl_wrap_for_vec!(u8);
impl_wrap_for_vec!(u16);
impl_wrap_for_vec!(u32);
impl_wrap_for_vec!(u64);
#[cfg(has_u128)]
impl_wrap_for_vec!(u128);

macro_rules! impl_packable_for_vec_prefix {
    ($ty:ty) => {
        impl<T: Packable> Packable for VecPrefix<T, $ty> {
            type PackError = PrefixError<T::PackError, <$ty as TryFrom<usize>>::Error>;
            type UnpackError = PrefixError<T::UnpackError, <usize as TryFrom<$ty>>::Error>;

            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
                // The length of any dynamically-sized sequence must be prefixed.
                <$ty>::try_from(self.len())
                    .map_err(|err| PackError::Packable(PrefixError::Prefix(err)))?
                    .pack(packer)
                    .map_err(PackError::infallible)?;

                for item in self.iter() {
                    item.pack(packer).map_err(PackError::coerce)?;
                }

                Ok(())
            }

            fn packed_len(&self) -> usize {
                (0 as $ty).packed_len() + self.iter().map(T::packed_len).sum::<usize>()
            }

            fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
                // The length of any dynamically-sized sequence must be prefixed.
                let len = <$ty>::unpack(unpacker).map_err(UnpackError::infallible)?;

                let mut vec = Self::with_capacity(
                    usize::try_from(len).map_err(|err| UnpackError::Packable(PrefixError::Prefix(err)))?,
                );

                for _ in 0..len {
                    let item = T::unpack(unpacker).map_err(UnpackError::coerce)?;
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
