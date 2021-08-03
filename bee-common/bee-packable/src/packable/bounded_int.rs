use crate::{
    coerce::CoerceInfallible,
    error::{PackError, UnpackError},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use core::convert::{Infallible, TryFrom, TryInto};

macro_rules! bounded_int {
    ($(#[$ty_doc:meta])* $ty:ident, $(#[$err_doc:meta])* $err:ident, $int:ident) => {
        $(#[$ty_doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $ty<const MIN: $int, const MAX: $int>($int);

        $(#[$err_doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $err<const MIN: $int, const MAX: $int>(pub $int);

        impl<const MIN: $int, const MAX: $int> $ty<MIN, MAX> {
            /// Minimum bounded value.
            pub const MIN: $int = MIN;

            /// Maximum bounded value.
            pub const MAX: $int = MAX;
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

bounded_int!(
    /// Wrapper type for a `u8`, providing minimum and maximum value bounds.
    BoundedU8,
    /// Error encountered when attempting to wrap a `u8` that is not within the given bounds.
    InvalidBoundedU8,
    u8
);

bounded_int!(
    /// Wrapper type for a `u16`, providing minimum and maximum value bounds.
    BoundedU16,
    /// Error encountered when attempting to wrap a `u16` that is not within the given bounds.
    InvalidBoundedU16,
    u16
);

bounded_int!(
    /// Wrapper type for a `u32`, providing minimum and maximum value bounds.
    BoundedU32,
    /// Error encountered when attempting to wrap a `u32` that is not within the given bounds.
    InvalidBoundedU32,
    u32
);

bounded_int!(
    /// Wrapper type for a `u64`, providing minimum and maximum value bounds.
    BoundedU64,
    /// Error encountered when attempting to wrap a `u64` that is not within the given bounds.
    InvalidBoundedU64,
    u64
);
