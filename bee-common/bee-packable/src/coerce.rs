// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Error coercion utilities.

use core::convert::Infallible;

use crate::error::{PackError, UnpackError};

/// This is `Coerce`.
pub trait Coerce<T> {
    /// This is `coerce`.
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
/// This is `CoerceInfallible`.
pub trait CoerceInfallible<T> {
    /// This is `infallible`.
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
