// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Errors related to packable operations.

use core::convert::Infallible;

mod sealed {
    use crate::error::UnpackError;

    pub trait Sealed {}

    impl<T, U, V> Sealed for Result<T, UnpackError<U, V>> {}
}

/// Trait providing utility methods for `Result` values that use `UnpackError` as the `Err`
/// variant.
pub trait UnpackErrorExt<T, U, V>: sealed::Sealed + Sized {
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

impl<T, U, V> UnpackErrorExt<T, U, V> for Result<T, UnpackError<U, V>> {
    fn map_packable_err<W>(self, f: impl Fn(U) -> W) -> Result<T, UnpackError<W, V>> {
        self.map_err(|err| match err {
            UnpackError::Packable(err) => UnpackError::Packable(f(err)),
            UnpackError::Unpacker(err) => UnpackError::Unpacker(err),
        })
    }
}
/// Error type raised when [`Packable::unpack`](crate::Packable) fails.
#[derive(Debug)]
pub enum UnpackError<T, U> {
    /// Semantic error. Typically this is [`Packable::UnpackError`](crate::Packable::UnpackError).
    Packable(T),
    /// Error produced by the unpacker. Typically this is [`Unpacker::Error`](crate::unpacker::Unpacker).
    Unpacker(U),
}

impl<T, U> UnpackError<T, U> {
    /// Wraps an error in the [`Packable`](UnpackError::Packable) variant.
    pub fn from_packable(err: impl Into<T>) -> Self {
        Self::Packable(err.into())
    }
}

impl<T, U> From<U> for UnpackError<T, U> {
    fn from(err: U) -> Self {
        Self::Unpacker(err)
    }
}

impl<U> UnpackError<Infallible, U> {
    /// Get the [`Packer`](UnpackError::Unpacker) variant if the [`Packable`](UnpackError::Packable) variant is
    /// [`Infallible`].
    pub fn into_unpacker(self) -> U {
        match self {
            Self::Packable(err) => match err {},
            Self::Unpacker(err) => err,
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

/// Semantic error raised while unpacking a dynamically-sized sequences that use a type different than `usize` for their
/// length-prefix.
#[derive(Debug)]
pub enum UnpackPrefixError<T, E> {
    /// Semantic error raised while unpacking an element of the sequence.
    /// Typically this is [`Packable::UnpackError`](crate::Packable).
    Packable(T),
    /// Semantic error raised when the length prefix cannot be unpacked.
    Prefix(E),
}

impl<E> UnpackPrefixError<Infallible, E> {
    /// Projects the value to the [`Prefix`](UnpackPrefixError::Prefix) variant.
    pub fn into_prefix(self) -> E {
        match self {
            Self::Packable(err) => match err {},
            Self::Prefix(err) => err,
        }
    }
}

impl<T, E> UnpackPrefixError<T, E> {
    /// Returns the contained [`Packable`](UnpackPrefixError::Packable) value or computes it from a
    /// closure.
    pub fn unwrap_packable_or_else<V: Into<T>>(self, f: impl FnOnce(E) -> V) -> T {
        match self {
            Self::Packable(err) => err,
            Self::Prefix(err) => f(err).into(),
        }
    }
}

impl<T, E> From<T> for UnpackPrefixError<T, E> {
    fn from(err: T) -> Self {
        Self::Packable(err)
    }
}

/// Error type to be raised when [`SliceUnpacker`](`crate::unpacker::SliceUnpacker`) does not have enough bytes to
/// unpack something or when [`SlicePacker`]('crate::packer::SlicePacker') does not have enough space to pack something.
#[derive(Debug)]
pub struct UnexpectedEOF {
    /// The required number of bytes.
    pub required: usize,
    /// The number of bytes the unpacker had or the number of bytes the packer can receive.
    pub had: usize,
}
