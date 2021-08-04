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
    type Bounds: PartialOrd;

    /// Minimum bounded value.
    const MIN: Self::Bounds;

    /// Maximum bounded value.
    const MAX: Self::Bounds;
}

macro_rules! bounded {
    ($(#[$wrapper_doc:meta])* $wrapper:ident, $(#[$error_doc:meta])* $error:ident, $ty:ident) => {
        $(#[$error_doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $error<const MIN: $ty, const MAX: $ty>(pub $ty);

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

        $(#[$wrapper_doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $wrapper<const MIN: $ty, const MAX: $ty>($ty);

        impl<const MIN: $ty, const MAX: $ty> Bounded for $wrapper<MIN, MAX> {
            type Bounds = $ty;

            const MIN: $ty = MIN;
            const MAX: $ty = MAX;
        }

        // We cannot provide a [`From`] implementation because integer primitives are not in this crate.
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
                    Ok(Self(value))
                } else {
                    Err($error(value))
                }
            }
        }

        impl<const MIN: $ty, const MAX: $ty> Packable for $wrapper<MIN, MAX> {
            type PackError = Infallible;
            type UnpackError = $error<MIN, MAX>;

            fn packed_len(&self) -> usize {
                self.0.packed_len()
            }

            fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), PackError<Self::PackError, P::Error>> {
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

// TODO: replace with #[doc = concat!(<...>)] in macro when CI rust versions are updated.

bounded!(
    /// Wrapper type for a `u8`, providing minimum and maximum value bounds.
    BoundedU8,
    /// Error encountered when attempting to wrap a `u8` that is not within the given bounds.
    InvalidBoundedU8,
    u8
);

bounded!(
    /// Wrapper type for a `u16`, providing minimum and maximum value bounds.
    BoundedU16,
    /// Error encountered when attempting to wrap a `u16` that is not within the given bounds.
    InvalidBoundedU16,
    u16
);

bounded!(
    /// Wrapper type for a `u32`, providing minimum and maximum value bounds.
    BoundedU32,
    /// Error encountered when attempting to wrap a `u32` that is not within the given bounds.
    InvalidBoundedU32,
    u32
);

bounded!(
    /// Wrapper type for a `u64`, providing minimum and maximum value bounds.
    BoundedU64,
    /// Error encountered when attempting to wrap a `u64` that is not within the given bounds.
    InvalidBoundedU64,
    u64
);
