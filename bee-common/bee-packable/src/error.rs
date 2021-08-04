// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Errors related to packable operations.

use core::convert::Infallible;

/// Error type raised when [`Packable::pack`](crate::Packable) fails.
#[derive(Debug)]
pub enum PackError<T, P> {
    /// Semantic error. Typically this is [`Packable::PackError`](crate::Packable).
    Packable(T),
    /// Error produced by the packer. Typically this is [`Packer::Error`](crate::packer::Packer).
    Packer(P),
}

impl<T, P> PackError<T, P> {
    /// Map the [`Packable`](crate::Packable) variant of this enum.
    pub fn map<V, F: Fn(T) -> V>(self, f: F) -> PackError<V, P> {
        match self {
            Self::Packable(err) => PackError::Packable(f(err)),
            Self::Packer(err) => PackError::Packer(err),
        }
    }

    /// Coerce the value by calling `.into()` for the [`Packable`](crate::Packable) variant.
    pub(crate) fn coerce<V: From<T>>(self) -> PackError<V, P> {
        self.map(|x| x.into())
    }
}

impl<T, P> From<P> for PackError<T, P> {
    fn from(err: P) -> Self {
        Self::Packer(err)
    }
}

impl<P> PackError<Infallible, P> {
    /// Coerce the value if the [`Packable`](crate::Packable) variant is [`Infallible`].
    pub(crate) fn infallible<E>(self) -> PackError<E, P> {
        match self {
            Self::Packable(err) => match err {},
            Self::Packer(err) => PackError::Packer(err),
        }
    }
}

/// Error type raised when [`Packable::unpack`](crate::Packable) fails.
#[derive(Debug)]
pub enum UnpackError<T, U> {
    /// Semantic error. Typically this is [`Packable::UnpackError`](crate::Packable).
    Packable(T),
    /// Error produced by the unpacker. Typically this is [`Unpacker::Error`](crate::unpacker::Unpacker).
    Unpacker(U),
}

impl<T, U> UnpackError<T, U> {
    /// Maps the [`Packable`](crate::Packable) variant of this enum.
    pub fn map<V, F: Fn(T) -> V>(self, f: F) -> UnpackError<V, U> {
        match self {
            Self::Packable(err) => UnpackError::Packable(f(err)),
            Self::Unpacker(err) => UnpackError::Unpacker(err),
        }
    }

    /// Coerces the value by calling `.into()` for the [`Packable`](crate::Packable) variant.
    pub(crate) fn coerce<V: From<T>>(self) -> UnpackError<V, U> {
        self.map(|x| x.into())
    }
}

impl<T, U> From<U> for UnpackError<T, U> {
    fn from(err: U) -> Self {
        Self::Unpacker(err)
    }
}

impl<U> UnpackError<Infallible, U> {
    /// Coerces the value if the [`Packable`](crate::Packable) variant is [`Infallible`].
    pub(crate) fn infallible<E>(self) -> UnpackError<E, U> {
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

/// Semantic error raised while packing a dynamically-sized sequences that use a type different
/// than `usize` for their length-prefix.
#[derive(Debug)]
pub struct PackPrefixError<T>(pub T);

impl<T> From<T> for PackPrefixError<T> {
    fn from(err: T) -> Self {
        Self(err)
    }
}

/// Semantic error raised while unpacking a dynamically-sized sequences that use a type different
/// than `usize` for their length-prefix.
#[derive(Debug)]
pub enum UnpackPrefixError<T> {
    /// Semantic error raised while unpacking an element of the sequence.
    /// Typically this is [`Packable::UnpackError`](crate::Packable).
    Packable(T),
    /// Invalid prefix length (larger than maximum specified).
    InvalidPrefixLength(usize),
}

impl<T> From<T> for UnpackPrefixError<T> {
    fn from(err: T) -> Self {
        Self::Packable(err)
    }
}
