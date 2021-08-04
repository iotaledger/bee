// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Error coercion utilities.

mod sealed {
    use crate::error::{PackError, UnpackError};

    pub trait Sealed {}

    impl<T, U, V> Sealed for Result<T, UnpackError<U, V>> {}

    impl<T, U, V> Sealed for Result<T, PackError<U, V>> {}
}

use crate::error::{PackError, UnpackError};

use core::convert::Infallible;

/// Trait used to convert `Result` values that use `PackError` as the `Err` variant.
pub trait PackCoerce<T, U, V>: sealed::Sealed {
    /// Coerces the value to another result type.
    fn coerce<W>(self) -> Result<T, PackError<W, V>>
    where
        U: Into<W>;
}

impl<T, U, V> PackCoerce<T, U, V> for Result<T, PackError<U, V>> {
    fn coerce<W>(self) -> Result<T, PackError<W, V>>
    where
        U: Into<W>,
    {
        self.map_err(PackError::<U, V>::coerce::<W>)
    }
}

/// Trait used to convert `Result` values that use `UnpackError` as the `Err` variant.
pub trait UnpackCoerce<T, U, V>: sealed::Sealed {
    /// Coerces the value to another result type.
    fn coerce<W>(self) -> Result<T, UnpackError<W, V>>
    where
        U: Into<W>;
}

impl<T, U, V> UnpackCoerce<T, U, V> for Result<T, UnpackError<U, V>> {
    fn coerce<W>(self) -> Result<T, UnpackError<W, V>>
    where
        U: Into<W>,
    {
        self.map_err(UnpackError::<U, V>::coerce::<W>)
    }
}

/// Trait used to convert `Result` values that use `PackError<Infallible, _>`as the `Err` variant.
pub trait PackCoerceInfallible<T, V>: sealed::Sealed {
    /// Coerces the value to another result type.
    fn infallible<U>(self) -> Result<T, PackError<U, V>>;
}

impl<T, V> PackCoerceInfallible<T, V> for Result<T, PackError<Infallible, V>> {
    fn infallible<U>(self) -> Result<T, PackError<U, V>> {
        self.map_err(PackError::infallible)
    }
}

/// Trait used to convert `Result` values that use `UnpackError<Infallible, _>`as the `Err` variant.
pub trait UnpackCoerceInfallible<T, V>: sealed::Sealed {
    /// Coerces the value to another result type.
    fn infallible<U>(self) -> Result<T, UnpackError<U, V>>;
}

impl<T, V> UnpackCoerceInfallible<T, V> for Result<T, UnpackError<Infallible, V>> {
    fn infallible<U>(self) -> Result<T, UnpackError<U, V>> {
        self.map_err(UnpackError::infallible)
    }
}
