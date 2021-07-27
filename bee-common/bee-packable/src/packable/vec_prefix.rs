// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use crate::{
    error::{PackError, PackPrefixError, UnpackError, UnpackPrefixError},
    packer::Packer,
    unpacker::Unpacker,
    wrap::Wrap,
    Packable,
};

use alloc::vec::Vec;
use core::{convert::TryFrom, marker::PhantomData};

/// Error encountered when attempting to convert a [`Vec<T>`] into a [`VecPrefix`], where
/// the length of the source vector exceeds the maximum length of the [`VecPrefix`].
#[derive(Debug, PartialEq, Eq)]
pub struct PrefixedFromVecError {
    /// Maximum length of the [`VecPrefix`].
    pub max_len: usize,
    /// Actual length of the source vector.
    pub actual_len: usize,
}

/// Wrapper type for [`Vec<T>`] where the length prefix is of type `P`.
/// The [`Vec<T>`]'s maximum length is provided by `N`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct VecPrefix<T, P, const N: usize> {
    inner: Vec<T>,
    marker: PhantomData<P>,
}

impl<T, P, const N: usize> VecPrefix<T, P, N> {
    /// Creates a new empty [`VecPrefix<T, P>`].
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Creates a new empty [`VecPrefix<T, P>`] with a specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Vec::with_capacity(capacity),
            marker: PhantomData,
        }
    }
}

impl<T, P, const N: usize> Default for VecPrefix<T, P, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, P, const N: usize> TryFrom<Vec<T>> for VecPrefix<T, P, N> {
    type Error = PrefixedFromVecError;

    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        if vec.len() > N {
            Err(PrefixedFromVecError {
                max_len: N,
                actual_len: vec.len(),
            })
        } else {
            Ok(Self {
                inner: vec,
                marker: PhantomData,
            })
        }
    }
}

/// We cannot provide a [`From`] implementation because [`Vec`] is not from this crate.
#[allow(clippy::from_over_into)]
impl<T, P, const N: usize> Into<Vec<T>> for VecPrefix<T, P, N> {
    fn into(self) -> Vec<T> {
        self.inner
    }
}

impl<T, P, const N: usize> core::ops::Deref for VecPrefix<T, P, N> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T, P, const N: usize> core::ops::DerefMut for VecPrefix<T, P, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

macro_rules! impl_wrap_for_vec {
    ($ty:ty) => {
        impl<T, const N: usize> Wrap<VecPrefix<T, $ty, N>> for Vec<T> {
            fn wrap(&self) -> &VecPrefix<T, $ty, N> {
                // SAFETY: [`VecPrefix`] has a transparent representation and it is composed of one
                // [`Vec`] field and an additional zero-sized type field. This means that [`VecPrefix`]
                // has the same layout as [`Vec`] and any valid [`Vec`] reference can be casted into a
                // valid [`VecPrefix`] reference.
                unsafe { &*(self as *const Vec<T> as *const VecPrefix<T, $ty, N>) }
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
        impl<T: Packable, const N: usize> Packable for VecPrefix<T, $ty, N> {
            type PackError = PackPrefixError<T::PackError, $ty>;
            type UnpackError = UnpackPrefixError<T::UnpackError, $ty>;

            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
                // The length of any dynamically-sized sequence must be prefixed.
                <$ty>::try_from(self.len())
                    .map_err(|err| PackError::Packable(PackPrefixError::Prefix(err)))?
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
                let len = usize::try_from(len).map_err(|err| UnpackError::Packable(UnpackPrefixError::Prefix(err)))?;

                if len > N {
                    return Err(UnpackError::Packable(UnpackPrefixError::InvalidPrefixLength(len)));
                }

                let mut vec = Self::with_capacity(len);

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
