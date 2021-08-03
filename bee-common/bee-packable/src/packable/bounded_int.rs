// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    coerce::CoerceInfallible,
    error::{PackError, UnpackError},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use core::convert::{Infallible, TryFrom, TryInto};

/// Trait that provides an interface for bounded integers.
pub trait Bounded {
    /// The type used to define the bounds.
    type Bounds;

    /// Minimum bounded value.
    const MIN: Self::Bounds;

    /// Maximum bounded value.
    const MAX: Self::Bounds;
}

macro_rules! bounded_int {
    ($ty:ident, $err:ident, $int:ident) => {
        #[doc = concat!("Error type encountered when attempting to wrap a [`", stringify!($int), "`] that is not within the given bounds.")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $err<const MIN: $int, const MAX: $int>(pub $int);

        impl<const MIN: $int, const MAX: $int> Bounded for $err<MIN, MAX> {
            type Bounds = $int;

            const MIN: $int = MIN;
            const MAX: $int = MAX;
        }

        #[allow(clippy::from_over_into)]
        impl<const MIN: $int, const MAX: $int> Into<$int> for $err<MIN, MAX> {
            fn into(self) -> $int {
                self.0
            }
        }

        #[doc = concat!("Wrapper type for a [`", stringify!($int), "`], providing minimum and maximum value bounds.")]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $ty<const MIN: $int, const MAX: $int>($int);

        impl<const MIN: $int, const MAX: $int> Bounded for $ty<MIN, MAX> {
            type Bounds = $int;

            const MIN: $int = MIN;
            const MAX: $int = MAX;
        }

        #[allow(clippy::from_over_into)]
        impl<const MIN: $int, const MAX: $int> Into<$int> for $ty<MIN, MAX> {
            fn into(self) -> $int {
                self.0
            }
        }

        impl<const MIN: $int, const MAX: $int> TryFrom<$int> for $ty<MIN, MAX> {
            type Error = $err<MIN, MAX>;

            fn try_from(value: $int) -> Result<Self, Self::Error> {
                if (MIN..MAX).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err($err(value))
                }
            }
        }

        impl<const MIN: $int, const MAX: $int> Packable for $ty<MIN, MAX> {
            type PackError = Infallible;
            type UnpackError = $err<MIN, MAX>;

            fn packed_len(&self) -> usize {
                self.0.packed_len()
            }

            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
                self.0.pack(packer)
            }

            fn unpack<U: Unpacker>(unpacker: &mut U) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
                $int::unpack(unpacker)
                    .infallible()?
                    .try_into()
                    .map_err(UnpackError::Packable)
            }
        }
    };
}

bounded_int!(BoundedU8, InvalidBoundedU8, u8);
bounded_int!(BoundedU16, InvalidBoundedU16, u16);
bounded_int!(BoundedU32, InvalidBoundedU32, u32);
bounded_int!(BoundedU64, InvalidBoundedU64, u64);
