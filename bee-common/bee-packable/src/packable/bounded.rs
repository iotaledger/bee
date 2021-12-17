// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types and utilities related to bounded integers.

use crate::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use core::fmt::{self, Display};

/// Trait that provides an interface for bounded types.
pub trait Bounded: TryFrom<usize> + Into<Self::Bounds> {
    /// The type used to define the bounds.
    type Bounds: PartialOrd + TryInto<Self> + TryInto<usize> + Default + Copy;
}

macro_rules! bounded {
    ($wrapper:ident, $error:ident, $try_error:ident, $ty:ident) => {
        #[doc = concat!("Error encountered when attempting to wrap a  [`", stringify!($ty),"`] that is not within the given bounds.")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $error<const MIN: $ty, const MAX: $ty>(pub $ty);

        impl<const MIN: $ty, const MAX: $ty> Display for $error<MIN, MAX> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "the integer `{}` is out of bounds (`{}..={}`)", self.0, MIN, MAX)
            }
        }

        #[allow(clippy::from_over_into)]
        impl<const MIN: $ty, const MAX: $ty> Into<$ty> for $error<MIN, MAX> {
            fn into(self) -> $ty {
                self.0
            }
        }

        #[doc = concat!("Error encountered when attempting to convert a [`usize`] into a [`", stringify!($wrapper),"`].")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $try_error<const MIN: $ty, const MAX: $ty>{
            #[doc = concat!("The `usize` could be converted into a [`", stringify!($ty),"`] but it is not within the given bounds.")]
            Invalid($ty),
            #[doc = concat!("The `usize` could not be converted into a [`", stringify!($ty),"`].")]
            Truncated(usize),
        }
        impl<const MIN: $ty, const MAX: $ty> Display for $try_error<MIN, MAX> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Truncated(len) => write!(f, "the integer `{}` was truncated while casting it into a `{}`", len, core::any::type_name::<$ty>()),
                Self::Invalid(err) => $error::<MIN, MAX>(*err).fmt(f),
            }
        }
}

        impl<const MIN: $ty, const MAX: $ty> From<$error<MIN, MAX>> for $try_error<MIN, MAX> {
            fn from(err: $error<MIN, MAX>) -> Self {
                Self::Invalid(err.0)
            }
        }

        #[doc = concat!("Wrapper type for a [`", stringify!($ty),"`], providing minimum and maximum value bounds.")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $wrapper<const MIN: $ty, const MAX: $ty>($ty);

        impl<const MIN: $ty, const MAX: $ty> Bounded for $wrapper<MIN, MAX> {
            type Bounds = $ty;
        }

        impl<const MIN: $ty, const MAX: $ty> $wrapper<MIN, MAX> {
            /// Minimum bounded value.
            pub const MIN: $ty = MIN;
            /// Maximum bounded value.
            pub const MAX: $ty = MAX;

            /// Creates a bounded integer without checking whether the value is in-bounds.
            ///
            /// # Safety
            /// The caller must guarantee that the created integer is actually in-bounds.
            pub unsafe fn new_unchecked(value: $ty) -> Self {
                debug_assert!((value >= MIN) && (value <= MAX));

                Self(value)
            }

            /// Returns the value as a primitive type.
            #[inline(always)]
            pub const fn get(self) -> $ty {
                self.0
            }
        }

        /// This implementation returns the closest bounded integer to zero.
        impl<const MIN: $ty, const MAX: $ty> Default for $wrapper<MIN, MAX> {
            fn default() -> Self {
                // SAFETY: this value is larger or equal than `MIN` and smaller or equal than `MAX` by construction.
                unsafe { Self::new_unchecked(0.min(MAX).max(MIN)) }
            }
        }

        // We cannot provide a [`From`] implementation because primitives are not in this crate.
        #[allow(clippy::from_over_into)]
        impl<const MIN: $ty, const MAX: $ty> Into<$ty> for $wrapper<MIN, MAX> {
            fn into(self) -> $ty {
                self.get()
            }
        }

        impl<const MIN: $ty, const MAX: $ty> TryFrom<$ty> for $wrapper<MIN, MAX> {
            type Error = $error<MIN, MAX>;

            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                if (MIN..=MAX).contains(&value) {
                    // SAFETY: We just checked that the value is in-bounds.
                    Ok(unsafe { Self::new_unchecked(value) })
                } else {
                    Err($error(value))
                }
            }
        }

        impl<const MIN: $ty, const MAX: $ty> TryFrom<usize> for $wrapper<MIN, MAX> {
            type Error = $try_error<MIN, MAX>;

            fn try_from(value: usize) -> Result<Self, Self::Error> {
                Ok(<$ty>::try_from(value)
                    .map_err(|_| $try_error::Truncated(value))?
                    .try_into()?)
            }
        }

        impl<const MIN: $ty, const MAX: $ty> Packable for $wrapper<MIN, MAX> {
            type UnpackError = $error<MIN, MAX>;

            #[inline(always)]
            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                self.0.pack(packer)
            }


            fn unpack<U: Unpacker, const VERIFY: bool>(
                unpacker: &mut U,
            ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
                let value = $ty::unpack::<_, VERIFY>(unpacker).infallible()?;

                if VERIFY && !(MIN..=MAX).contains(&value) {
                    return Err(UnpackError::Packable($error(value)));
                }

                Ok(Self(value))
            }
        }
    };
}

bounded!(BoundedU8, InvalidBoundedU8, TryIntoBoundedU8Error, u8);
bounded!(BoundedU16, InvalidBoundedU16, TryIntoBoundedU16Error, u16);
bounded!(BoundedU32, InvalidBoundedU32, TryIntoBoundedU32Error, u32);
bounded!(BoundedU64, InvalidBoundedU64, TryIntoBoundedU64Error, u64);

impl Bounded for u8 {
    type Bounds = Self;
}

impl Bounded for u16 {
    type Bounds = Self;
}

impl Bounded for u32 {
    type Bounds = Self;
}

impl Bounded for u64 {
    type Bounds = Self;
}
