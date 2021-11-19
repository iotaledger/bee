// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types and utilities related to bounded integers.

use crate::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::TryIntoPrefixError,
    unpacker::Unpacker,
    Packable,
};

use core::{
    fmt::{self, Display},
    ops::Range,
};

/// Trait that provides an interface for bounded types.
pub trait Bounded: TryFrom<usize> + Into<Self::Bounds> {
    /// The type used to define the bounds.
    type Bounds: PartialOrd + TryInto<Self> + TryInto<usize> + Default + Copy;
    /// The type used to define ranges over bounds.
    type Range: Iterator<Item = Self::Bounds>;

    /// Minimum bounded value.
    const MIN: Self::Bounds;
    /// Maximum bounded value.
    const MAX: Self::Bounds;

    /// Returns a range iterator over elements of type [`Bounds`](Self::Bounds).
    fn range(start: Self::Bounds, end: Self::Bounds) -> Self::Range;
}

macro_rules! bounded {
    ($wrapper:ident, $error:ident, $ty:ident) => {
        #[doc = concat!("Error encountered when attempting to wrap a  `", stringify!($ty),"` that is not within the given bounds.")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $error<const MIN: $ty, const MAX: $ty>(pub $ty);

        impl<const MIN: $ty, const MAX: $ty> Display for $error<MIN, MAX> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "integer {} is out of bounds", self.0)
            }
        }

        #[allow(clippy::from_over_into)]
        impl<const MIN: $ty, const MAX: $ty> Into<$ty> for $error<MIN, MAX> {
            fn into(self) -> $ty {
                self.0
            }
        }

        #[doc = concat!("Wrapper type for a `", stringify!($ty),"`, providing minimum and maximum value bounds.")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $wrapper<const MIN: $ty, const MAX: $ty>($ty);

        impl<const MIN: $ty, const MAX: $ty> Bounded for $wrapper<MIN, MAX> {
            type Bounds = $ty;
            type Range = Range<$ty>;

            const MIN: $ty = MIN;
            const MAX: $ty = MAX;

            fn range(start: Self::Bounds, end: Self::Bounds) -> Self::Range {
                start..end
            }
        }

        impl<const MIN: $ty, const MAX: $ty> $wrapper<MIN, MAX> {
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
            type Error = TryIntoPrefixError<$error<MIN, MAX>>;

            fn try_from(value: usize) -> Result<Self, Self::Error> {
                <$ty>::try_from(value)
                    .map_err(|_| TryIntoPrefixError::Truncated(value))?
                    .try_into()
                    .map_err(TryIntoPrefixError::Invalid)
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

bounded!(BoundedU8, InvalidBoundedU8, u8);
bounded!(BoundedU16, InvalidBoundedU16, u16);
bounded!(BoundedU32, InvalidBoundedU32, u32);
bounded!(BoundedU64, InvalidBoundedU64, u64);

impl Bounded for u8 {
    type Bounds = Self;
    type Range = Range<Self>;

    const MIN: Self::Bounds = u8::MIN;
    const MAX: Self::Bounds = u8::MAX;

    fn range(start: Self::Bounds, end: Self::Bounds) -> Self::Range {
        start..end
    }
}

impl Bounded for u16 {
    type Bounds = Self;
    type Range = Range<Self>;

    const MIN: Self::Bounds = u16::MIN;
    const MAX: Self::Bounds = u16::MAX;

    fn range(start: Self::Bounds, end: Self::Bounds) -> Self::Range {
        start..end
    }
}

impl Bounded for u32 {
    type Bounds = Self;
    type Range = Range<Self>;

    const MIN: Self::Bounds = u32::MIN;
    const MAX: Self::Bounds = u32::MAX;

    fn range(start: Self::Bounds, end: Self::Bounds) -> Self::Range {
        start..end
    }
}

impl Bounded for u64 {
    type Bounds = Self;
    type Range = Range<Self>;

    const MIN: Self::Bounds = u64::MIN;
    const MAX: Self::Bounds = u64::MAX;

    fn range(start: Self::Bounds, end: Self::Bounds) -> Self::Range {
        start..end
    }
}
