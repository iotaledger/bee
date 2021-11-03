// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

extern crate alloc;

use crate::{
    coerce::*,
    error::{UnpackError, UnpackPrefixError},
    packer::Packer,
    unpacker::Unpacker,
    Bounded, BoundedU16, BoundedU32, BoundedU64, BoundedU8, InvalidBoundedU16, InvalidBoundedU32, InvalidBoundedU64,
    InvalidBoundedU8, Packable,
};

use alloc::vec::Vec;
use core::{
    convert::TryFrom,
    fmt::{self, Display},
    marker::PhantomData,
};

/// Semantic error raised when the prefix length cannot be unpacked.
#[derive(Debug)]
pub enum VecPrefixLengthError<E> {
    /// The prefix length was truncated.
    Truncated(usize),
    /// The prefix length is invalid.
    Invalid(E),
}

impl<E: Display> Display for VecPrefixLengthError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VecPrefixLengthError::Truncated(len) => write!(f, "length of `{}` was truncated", len),
            VecPrefixLengthError::Invalid(err) => err.fmt(f),
        }
    }
}

/// Wrapper type for [`Vec<T>`] with a length prefix.
/// The [`Vec<T>`]'s prefix bounds are provided by `B`, where `B` is a [`Bounded`] type.
/// The prefix type is the `Bounds` type associated with `B`.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct VecPrefix<T, B: Bounded> {
    inner: Vec<T>,
    bounded: PhantomData<B>,
}

impl<T, B: Bounded> Default for VecPrefix<T, B> {
    fn default() -> Self {
        Self {
            inner: Vec::new(),
            bounded: PhantomData,
        }
    }
}

macro_rules! impl_vec_prefix {
    ($ty:ident, $bounded:ident, $err:ident) => {
        impl<T, const MIN: $ty, const MAX: $ty> TryFrom<Vec<T>> for VecPrefix<T, $bounded<MIN, MAX>> {
            type Error = VecPrefixLengthError<$err<MIN, MAX>>;

            fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
                let len = vec.len();
                let _ = $bounded::<MIN, MAX>::try_from(
                    $ty::try_from(len).map_err(|_| VecPrefixLengthError::Truncated(len))?,
                )
                .map_err(VecPrefixLengthError::Invalid)?;

                Ok(Self {
                    inner: vec,
                    bounded: PhantomData,
                })
            }
        }

        impl<'a, T, const MIN: $ty, const MAX: $ty> TryFrom<&'a Vec<T>> for &'a VecPrefix<T, $bounded<MIN, MAX>> {
            type Error = VecPrefixLengthError<$err<MIN, MAX>>;

            fn try_from(vec: &Vec<T>) -> Result<Self, Self::Error> {
                let len = vec.len();
                let _ = $bounded::<MIN, MAX>::try_from(
                    $ty::try_from(len).map_err(|_| VecPrefixLengthError::Truncated(len))?,
                )
                .map_err(VecPrefixLengthError::Invalid)?;

                // SAFETY: `Vec<T>` and `VecPrefix<T, B>` have the same layout.
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
            type UnpackError = UnpackPrefixError<T::UnpackError, $err<MIN, MAX>>;

            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                // The length of any dynamically-sized sequence must be prefixed.
                // This unwrap is fine, since we have already validated the length in `try_from`.
                <$ty>::try_from(self.len()).unwrap().pack(packer)?;

                for item in self.iter() {
                    item.pack(packer)?;
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
                            UnpackError::Packable(UnpackPrefixError::InvalidPrefixLength(err))
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
