// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Error coercion utilities.

mod sealed {
    use crate::error::UnpackError;

    pub trait Sealed {}

    impl<T, U, V> Sealed for Result<T, UnpackError<U, V>> {}
}

use crate::error::UnpackError;

use core::convert::Infallible;

/// Trait used to convert `Result` values that use `UnpackError` as the `Err` variant.
pub trait UnpackCoerce<T, U, V>: sealed::Sealed + Sized {
    /// Maps the [`Packable`](UnpackError::Packable) variant if the result is an error.
    fn map_packable_err<W>(self, f: impl Fn(U) -> W) -> Result<T, UnpackError<W, V>>;

    /// Coerces the [`Packable`](UnpackError::Packable) variant value using [`Into`].
    fn coerce<W>(self) -> Result<T, UnpackError<W, V>>
    where
        U: Into<W>,
    {
        self.map_packable_err(U::into)
    }

    /// Coerces the [`Packable`](UnpackError::Packable) variant value to any type assuming the value is
    /// [`Infallible`].
    fn infallible<W>(self) -> Result<T, UnpackError<W, V>>
    where
        U: Into<Infallible>,
    {
        #[allow(unreachable_code)]
        self.map_packable_err(|err| match err.into() {})
    }
}

impl<T, U, V> UnpackCoerce<T, U, V> for Result<T, UnpackError<U, V>> {
    fn map_packable_err<W>(self, f: impl Fn(U) -> W) -> Result<T, UnpackError<W, V>> {
        self.map_err(|err| match err {
            UnpackError::Packable(err) => UnpackError::Packable(f(err)),
            UnpackError::Unpacker(err) => UnpackError::Unpacker(err),
        })
    }
}
