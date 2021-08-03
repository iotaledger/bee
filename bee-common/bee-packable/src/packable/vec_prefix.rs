// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use crate::{
    coerce::{Coerce, CoerceInfallible},
    error::{PackError, PackPrefixError, UnpackError, UnpackPrefixError},
    packer::Packer,
    unpacker::Unpacker,
    Bounded, BoundedU16, BoundedU32, BoundedU64, BoundedU8, InvalidBoundedU16, InvalidBoundedU32, InvalidBoundedU64,
    InvalidBoundedU8, Packable,
};

use alloc::vec::Vec;
use core::{convert::TryFrom, marker::PhantomData};

/// Wrapper type for [`Vec<T>`] where the length prefix is of type `P`.
/// The [`Vec<T>`]'s maximum length is provided by `N`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct VecPrefix<T, B: Bounded> {
    inner: Vec<T>,
    bounded: PhantomData<B>,
}

macro_rules! impl_vec_prefix {
    ($ty:ident, $bounded:ident, $err:ident) => {
        impl<T, const MIN: $ty, const MAX: $ty> VecPrefix<T, $bounded<MIN, MAX>> {
            /// Creates a new empty [`VecPrefix<T, P>`] with a specified capacity.
            pub fn with_capacity(capacity: usize) -> Self {
                Self {
                    inner: Vec::with_capacity(capacity),
                    bounded: PhantomData,
                }
            }
        }

        impl<T, const MIN: $ty, const MAX: $ty> TryFrom<Vec<T>> for VecPrefix<T, $bounded<MIN, MAX>> {
            type Error = $err<MIN, MAX>;

            fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
                let _ = $bounded::<MIN, MAX>::try_from(vec.len() as $ty)?;

                Ok(Self {
                    inner: vec,
                    bounded: PhantomData,
                })
            }
        }

        impl<'a, T, const MIN: $ty, const MAX: $ty> TryFrom<&'a Vec<T>> for &'a VecPrefix<T, $bounded<MIN, MAX>> {
            type Error = $err<MIN, MAX>;

            fn try_from(vec: &Vec<T>) -> Result<Self, Self::Error> {
                let _ = $bounded::<MIN, MAX>::try_from(vec.len() as $ty)?;

                // SAFETY: `Vec<T>` and `VecPrefix<T, P, N>` have the same layout.
                Ok(unsafe { &*(vec as *const Vec<T> as *const VecPrefix<T, $bounded<MIN, MAX>>) })
            }
        }

        /// We cannot provide a [`From`] implementation because [`Vec`] is not from this crate.
        #[allow(clippy::from_over_into)]
        impl<T, const MIN: $ty, const MAX: $ty> Into<Vec<T>> for VecPrefix<T, $bounded<MIN, MAX>> {
            fn into(self) -> Vec<T> {
                self.inner
            }
        }

        impl<T, const MIN: $ty, const MAX: $ty> core::ops::Deref for VecPrefix<T, $bounded<MIN, MAX>> {
            type Target = Vec<T>;

            fn deref(&self) -> &Self::Target {
                &self.inner
            }
        }

        impl<T: Packable, const MIN: $ty, const MAX: $ty> Packable for VecPrefix<T, $bounded<MIN, MAX>> {
            type PackError = PackPrefixError<T::PackError>;
            type UnpackError = UnpackPrefixError<T::UnpackError>;

            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
                // The length of any dynamically-sized sequence must be prefixed.
                <$ty>::try_from(self.len()).unwrap().pack(packer).infallible()?;

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
                    .map_err(|err| match err {
                        UnpackError::Packable(err) => {
                            UnpackError::Packable(UnpackPrefixError::InvalidPrefixLength(err.0 as usize))
                        }
                        UnpackError::Unpacker(err) => UnpackError::Unpacker(err),
                    })?
                    .into();

                let mut inner = Vec::with_capacity(len as usize);

                for _ in 0..len {
                    let item = T::unpack(unpacker).coerce()?;
                    inner.push(item);
                }

                Ok(VecPrefix {
                    inner,
                    bounded: PhantomData,
                })
            }
        }
    };
}

impl_vec_prefix!(u8, BoundedU8, InvalidBoundedU8);
impl_vec_prefix!(u16, BoundedU16, InvalidBoundedU16);
impl_vec_prefix!(u32, BoundedU32, InvalidBoundedU32);
impl_vec_prefix!(u64, BoundedU64, InvalidBoundedU64);
