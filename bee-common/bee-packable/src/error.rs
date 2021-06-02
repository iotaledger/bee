// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Errors related to packable operations.

use core::convert::Infallible;

/// Error type raised when `Packable::unpack` fails.
#[derive(Debug)]
pub enum UnpackError<T, U> {
    /// Semantic error. Typically this is `Packable::Error`.
    Packable(T),
    /// Error produced by the unpacker. Typically this is `Unpacker::Error`.
    Unpacker(U),
}

impl<T, U> UnpackError<T, U> {
    /// Map the `Packable` variant of this enum.
    pub fn map<V, F: Fn(T) -> V>(self, f: F) -> UnpackError<V, U> {
        match self {
            Self::Packable(err) => UnpackError::Packable(f(err)),
            Self::Unpacker(err) => UnpackError::Unpacker(err),
        }
    }

    /// Coerce the value by calling `.into()` for the `Packable` variant.
    pub fn coerce<V: From<T>>(self) -> UnpackError<V, U> {
        self.map(|x| x.into())
    }
}

impl<T, U> From<U> for UnpackError<T, U> {
    fn from(err: U) -> Self {
        Self::Unpacker(err)
    }
}

impl<U> UnpackError<Infallible, U> {
    /// Coerce the value if the `Packable` variant is `Infallible`.
    pub fn infallible<E>(self) -> UnpackError<E, U> {
        match self {
            Self::Packable(err) => match err {},
            Self::Unpacker(err) => UnpackError::Unpacker(err),
        }
    }
}

/// Error type raised when an unknown tag is found while unpacking.
#[derive(Debug)]
pub struct UnknownTagError<T>(pub T);

impl<T> From<Infallible> for UnknownTagError<T> {
    fn from(err: Infallible) -> Self {
        match err {}
    }
}
