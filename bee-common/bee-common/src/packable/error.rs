// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

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
}

impl<T, U> From<U> for UnpackError<T, U> {
    fn from(err: U) -> Self {
        Self::Unpacker(err)
    }
}

/// Error type used for semantic errors of enums.
///
/// It is recooemded to use this type as `Packable::Error` when implementing `Packable` for an
/// enum.
#[derive(Debug)]
pub enum UnpackEnumError<T> {
    /// The unpacked tag is invalid for this enum.
    UnknownTag(u64),
    /// Other semantic errors.
    Inner(T),
}

impl<T> From<T> for UnpackEnumError<T> {
    fn from(err: T) -> Self {
        Self::Inner(err)
    }
}
