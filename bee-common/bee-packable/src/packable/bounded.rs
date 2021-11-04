// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{coerce::*, error::UnpackError, packer::Packer, unpacker::Unpacker, Packable};

use core::{
    convert::{TryFrom, TryInto},
    fmt::{self, Display},
};

/// Trait that provides an interface for bounded types.
pub trait Bounded {
    /// The type used to define the bounds.
    type Bounds: PartialOrd;

    /// Minimum bounded value.
    const MIN: Self::Bounds;
    /// Maximum bounded value.
    const MAX: Self::Bounds;
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

        impl<const MIN: $ty, const MAX: $ty> Bounded for $error<MIN, MAX> {
            type Bounds = $ty;

            const MIN: $ty = MIN;
            const MAX: $ty = MAX;
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

            const MIN: $ty = MIN;
            const MAX: $ty = MAX;
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
        }

        /// This implementation returns the closest bounded integer to zero.
        impl<const MIN: $ty, const MAX: $ty> Default for $wrapper<MIN, MAX> {
            fn default() -> Self {
                // SAFETY: this value is larger or equal than `MIN` and smaller or equal than
                // `MAX` by construction.
                unsafe { Self::new_unchecked(0.min(MAX).max(MIN)) }
            }
        }

        // We cannot provide a [`From`] implementation because primitives are not in this crate.
        #[allow(clippy::from_over_into)]
        impl<const MIN: $ty, const MAX: $ty> Into<$ty> for $wrapper<MIN, MAX> {
            fn into(self) -> $ty {
                self.0
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

        impl<const MIN: $ty, const MAX: $ty> Packable for $wrapper<MIN, MAX> {
            type UnpackError = $error<MIN, MAX>;

            fn packed_len(&self) -> usize {
                self.0.packed_len()
            }

            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
                self.0.pack(packer)
            }

            fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
                $ty::unpack(unpacker)
                    .infallible()?
                    .try_into()
                    .map_err(UnpackError::Packable)
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

    const MIN: Self::Bounds = u8::MIN;
    const MAX: Self::Bounds = u8::MAX;
}

impl Bounded for u16 {
    type Bounds = Self;

    const MIN: Self::Bounds = u16::MIN;
    const MAX: Self::Bounds = u16::MAX;
}

impl Bounded for u32 {
    type Bounds = Self;

    const MIN: Self::Bounds = u32::MIN;
    const MAX: Self::Bounds = u32::MAX;
}

impl Bounded for u64 {
    type Bounds = Self;

    const MIN: Self::Bounds = u64::MIN;
    const MAX: Self::Bounds = u64::MAX;
}
