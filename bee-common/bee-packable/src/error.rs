// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Errors related to packable operations.

use core::{
    convert::Infallible,
    fmt::{self, Display},
};

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
    pub(crate) fn coerce<V>(self) -> PackError<V, P>
    where
        T: Into<V>,
    {
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
    pub(crate) fn coerce<V>(self) -> UnpackError<V, U>
    where
        T: Into<V>,
    {
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

// We cannot provide a `From` implementation because `Infallible` is an extern type.
#[allow(clippy::from_over_into)]
impl Into<Infallible> for PackPrefixError<Infallible> {
    fn into(self) -> Infallible {
        self.0
    }
}

/// Semantic error raised while unpacking a dynamically-sized sequences that use a type different
/// than `usize` for their length-prefix.
#[derive(Debug)]
pub enum UnpackPrefixError<T, E> {
    /// Semantic error raised while unpacking an element of the sequence.
    /// Typically this is [`Packable::UnpackError`](crate::Packable).
    Packable(T),
    /// Semantic error raised when the length prefix cannot be unpacked.
    InvalidPrefixLength(E),
}

impl<T, E> From<T> for UnpackPrefixError<T, E> {
    fn from(err: T) -> Self {
        Self::Packable(err)
    }
}

/// Semantic error raised when the prefix length cannot be unpacked.
#[derive(Debug)]
pub enum VecPrefixLengthError<E> {
    /// The prefix length was truncated.
    Truncated(usize),
    /// The prefix length is invalid.
    Invalid(E),
}

impl<E: Display> Display for VecPrefixLengthError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VecPrefixLengthError::Truncated(len) => write!(f, "length of `{}` was truncated", len),
            VecPrefixLengthError::Invalid(err) => err.fmt(f),
        }
    }
}

/// Error type to be raised when [`SliceUnpacker`](`crate::unpacker::SliceUnpacker`) does not have
/// enough bytes to unpack something or when [`SlicePacker`]('crate::packer::SlicePacker') does not
/// have enough space to pack something.
#[derive(Debug)]
pub struct UnexpectedEOF {
    /// The required number of bytes.
    pub required: usize,
    /// The number of bytes the unpacker had or the number of bytes the packer can receive.
    pub had: usize,
}
