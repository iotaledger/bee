// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use crate::{
    coerce::{Coerce, CoerceInfallible},
    error::{PackError, PackPrefixError, UnpackError, UnpackPrefixError},
    packer::Packer,
    unpacker::Unpacker,
    Bounded,
    BoundedU8,
    BoundedU16,
    BoundedU32,
    BoundedU64,
    Packable,
};

use alloc::vec::Vec;
use core::{convert::TryFrom, marker::PhantomData};

/// Error encountered when attempting to convert a [`Vec<T>`] into a [`VecPrefix`], where
/// the length of the source vector exceeds the maximum length of the [`VecPrefix`].
#[derive(Debug, PartialEq, Eq)]
pub struct PrefixedFromVecError {
    /// Minimum length of the [`VecPrefix`].
    pub min_len: usize,
    /// Maximum length of the [`VecPrefix`].
    pub max_len: usize,
    /// Actual length of the source vector.
    pub actual_len: usize,
}

/// Wrapper type for [`Vec<T>`] where the length prefix is of type `P`.
/// The [`Vec<T>`]'s maximum length is provided by `N`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct VecPrefix<T, U, B: Bounded<U>> {
    inner: Vec<T>,
    bounded: PhantomData<B>,
    bounded_ty: PhantomData<U>,
}

macro_rules! impl_vec_prefix {
    ($ty:ident, $bounded:ident, $err:ident) => {
        impl<T, const MIN: $ty, const MAX: $ty> VecPrefix<T, $ty, $bounded<MIN, MAX>> {
            /// Creates a new empty [`VecPrefix<T, P>`] with a specified capacity.
            pub fn with_capacity(capacity: usize) -> Self {
                Self {
                    inner: Vec::with_capacity(capacity),
                    bounded: PhantomData,
                    bounded_ty: PhantomData,
                }
            }
        }
        
        impl<T, const MIN: $ty, const MAX: $ty> TryFrom<Vec<T>> for VecPrefix<T, $ty, $bounded<MIN, MAX>> {
            type Error = PrefixedFromVecError;
        
            fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
                if vec.len() > MAX as usize || vec.len() < MIN as usize {
                    Err(PrefixedFromVecError {
                        min_len: MIN as usize,
                        max_len: MAX as usize,
                        actual_len: vec.len(),
                    })
                } else {
                    Ok(Self {
                        inner: vec,
                        bounded: PhantomData,
                        bounded_ty: PhantomData,
                    })
                }
            }
        }
        
        impl<'a, T, const MIN: $ty, const MAX: $ty> TryFrom<&'a Vec<T>> for &'a VecPrefix<T, $ty, $bounded<MIN, MAX>> {
            type Error = PrefixedFromVecError;
        
            fn try_from(vec: &Vec<T>) -> Result<Self, Self::Error> {
                if vec.len() > MAX as usize || vec.len() < MIN as usize {
                    Err(PrefixedFromVecError {
                        min_len: MIN as usize,
                        max_len: MAX as usize,
                        actual_len: vec.len(),
                    })
                } else {
                    // SAFETY: `Vec<T>` and `VecPrefix<T, P, N>` have the same layout.
                    Ok(unsafe { &*(vec as *const Vec<T> as *const VecPrefix<T, $ty, $bounded<MIN, MAX>>) })
                }
            }
        }
        
        /// We cannot provide a [`From`] implementation because [`Vec`] is not from this crate.
        #[allow(clippy::from_over_into)]
        impl<T, const MIN: $ty, const MAX: $ty> Into<Vec<T>> for VecPrefix<T, $ty, $bounded<MIN, MAX>> {
            fn into(self) -> Vec<T> {
                self.inner
            }
        }
        
        impl<T, const MIN: $ty, const MAX: $ty> core::ops::Deref for VecPrefix<T, $ty, $bounded<MIN, MAX>> {
            type Target = Vec<T>;
        
            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl<T: Packable, const MIN: $ty, const MAX: $ty> Packable for VecPrefix<T, $ty, $bounded<MIN, MAX>> {
            type PackError = PackPrefixError<T::PackError>;
            type UnpackError = UnpackPrefixError<T::UnpackError>;

            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
                // The length of any dynamically-sized sequence must be prefixed.
                <$ty>::try_from(self.len())
                    .unwrap()
                    .pack(packer)
                    .infallible()?;

                for item in self.iter() {
                    item.pack(packer).coerce()?;
                }

                Ok(())
            }

            fn packed_len(&self) -> usize {
                (0 as $ty).packed_len() + self.iter().map(T::packed_len).sum::<usize>()
            }

            fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
                // The length of any dynamically-sized sequence must be prefixed.
                let len = <$bounded<MIN, MAX>>::unpack(unpacker)
                    .map_err(|err| {
                        match err {
                            UnpackError::Packable(err) => UnpackError::Packable(UnpackPrefixError::InvalidPrefixLength(err.0 as usize)),
                            UnpackError::Unpacker(err) => UnpackError::Unpacker(err),
                        }
                    })?
                    .value();

                let mut inner = Vec::with_capacity(len as usize);

                for _ in 0..len {
                    let item = T::unpack(unpacker).coerce()?;
                    inner.push(item);
                }

                Ok(VecPrefix {
                    inner,
                    bounded: PhantomData,
                    bounded_ty: PhantomData,
                })
            }
        }
    };
}

impl_vec_prefix!(u8, BoundedU8, InvalidBoundedU8);
impl_vec_prefix!(u16, BoundedU16, InvalidBoundedU16);
impl_vec_prefix!(u32, BoundedU32, InvalidBoundedU32);
impl_vec_prefix!(u64, BoundedU64, InvalidBoundedU64);
