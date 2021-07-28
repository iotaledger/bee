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

/// Trait used to convert `Result` values that contain `PackError` and `UnpackError` as the `Err`
/// variant.
pub trait Coerce<T>: sealed::Sealed {
    /// Coerces the value to another result type.
    fn coerce(self) -> T;
}

impl<T, U, V, W: From<U>> Coerce<Result<T, UnpackError<W, V>>> for Result<T, UnpackError<U, V>> {
    fn coerce(self) -> Result<T, UnpackError<W, V>> {
        self.map_err(UnpackError::<U, V>::coerce::<W>)
    }
}

impl<T, U, V, W: From<U>> Coerce<Result<T, PackError<W, V>>> for Result<T, PackError<U, V>> {
    fn coerce(self) -> Result<T, PackError<W, V>> {
        self.map_err(PackError::<U, V>::coerce::<W>)
    }
}

/// Trait used to convert `Result` values that contain `PackError<Infallible, _>` and
/// `UnpackError<Infallible, _>` as the `Err` variant.
pub trait CoerceInfallible<T>: sealed::Sealed {
    /// Coerces the value to another result type.
    fn infallible(self) -> T;
}

impl<T, U, V> CoerceInfallible<Result<T, UnpackError<U, V>>> for Result<T, UnpackError<Infallible, V>> {
    fn infallible(self) -> Result<T, UnpackError<U, V>> {
        self.map_err(UnpackError::infallible)
    }
}

impl<T, U, V> CoerceInfallible<Result<T, PackError<U, V>>> for Result<T, PackError<Infallible, V>> {
    fn infallible(self) -> Result<T, PackError<U, V>> {
        self.map_err(PackError::infallible)
    }
}
