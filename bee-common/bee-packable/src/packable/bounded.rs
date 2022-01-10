// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types and utilities related to bounded integers.

use crate::Packable;

use core::{
    convert::Infallible,
    fmt::{self, Display},
};

/// Trait that provides an interface for bounded types.
pub trait Bounded: TryFrom<usize> + Into<Self::Bounds> {
    /// The type used to define the bounds.
    type Bounds: PartialOrd + TryInto<Self> + TryInto<usize> + Default + Copy;
}

macro_rules! bounded {
    ($wrapper:ident, $invalid_error:ident, $try_error:ident, $ty:ident) => {
        #[doc = concat!("Wrapper type for a [`", stringify!($ty),"`], providing minimum and maximum value bounds.")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Packable)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[packable(unpack_error = $invalid_error<MIN, MAX>)]
        pub struct $wrapper<const MIN: $ty, const MAX: $ty>(
            #[packable(verify_with = Self::verify)]
            $ty
        );

        impl<const MIN: $ty, const MAX: $ty> Bounded for $wrapper<MIN, MAX> {
            type Bounds = $ty;
        }

        impl<const MIN: $ty, const MAX: $ty> $wrapper<MIN, MAX> {
            /// Minimum bounded value.
            pub const MIN: $ty = MIN;
            /// Maximum bounded value.
            pub const MAX: $ty = MAX;

            /// Returns the value as a primitive type.
            #[inline(always)]
            pub const fn get(self) -> $ty {
                self.0
            }

            fn verify<const VERIFY: bool>(&value: &$ty) -> Result<(), $invalid_error<MIN, MAX>> {
                if VERIFY && !(MIN..=MAX).contains(&value) {
                    Err($invalid_error(value))
                } else {
                    Ok(())
                }
            }
        }

        /// This implementation returns the closest bounded integer to zero.
        impl<const MIN: $ty, const MAX: $ty> Default for $wrapper<MIN, MAX> {
            fn default() -> Self {
                Self(0.min(MAX).max(MIN))
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
            type Error = $invalid_error<MIN, MAX>;

            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                Self::verify::<true>(&value)?;
                Ok(Self(value))
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

        #[doc = concat!("Error encountered when attempting to wrap a  [`", stringify!($ty),"`] that is not within the given bounds.")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, )]
        pub struct $invalid_error<const MIN: $ty, const MAX: $ty>(pub $ty);

        impl<const MIN: $ty, const MAX: $ty> Display for $invalid_error<MIN, MAX> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "the integer `{}` is out of bounds (`{}..={}`)", self.0, MIN, MAX)
            }
        }

        #[allow(clippy::from_over_into)]
        impl<const MIN: $ty, const MAX: $ty> From<Infallible> for $invalid_error<MIN, MAX> {
            fn from(err: Infallible) -> Self {
                match err {}
            }
        }

        #[allow(clippy::from_over_into)]
        impl<const MIN: $ty, const MAX: $ty> Into<$ty> for $invalid_error<MIN, MAX> {
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
                Self::Invalid(err) => $invalid_error::<MIN, MAX>(*err).fmt(f),
            }
        }
}

        impl<const MIN: $ty, const MAX: $ty> From<$invalid_error<MIN, MAX>> for $try_error<MIN, MAX> {
            fn from(err: $invalid_error<MIN, MAX>) -> Self {
                Self::Invalid(err.0)
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
