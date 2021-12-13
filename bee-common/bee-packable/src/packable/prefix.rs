// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Types and utilities used to pack and unpack dynamic sequences of values with restricted length prefixes.

extern crate alloc;

use crate::{
    error::{UnpackError, UnpackErrorExt},
    packable::bounded::Bounded,
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

use alloc::{boxed::Box, vec::Vec};
use core::{
    convert::Infallible,
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, Range},
};

/// Semantic error raised while unpacking a dynamically-sized sequences that use a type different than `usize` for their
/// length-prefix.
#[derive(Debug)]
pub enum UnpackPrefixError<T, E> {
    /// Semantic error raised while unpacking an element of the sequence.
    /// Typically this is [`Packable::UnpackError`](crate::Packable::UnpackError).
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
    /// Returns the contained [`Packable`](UnpackPrefixError::Packable) value or computes it from a closure.
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

/// Wrapper type for [`Vec<T>`] with a length prefix.
/// The [`Vec<T>`]'s prefix bounds are provided by `B`, where `B` is a [`Bounded`] type.
/// The prefix type is the `Bounds` type associated with `B`.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct VecPrefix<T, B: Bounded> {
    inner: Vec<T>,
    bounded: PhantomData<B>,
}

impl<T, B: Bounded> Default for VecPrefix<T, B> {
    fn default() -> Self {
        Self {
            inner: Vec::new(),
            bounded: PhantomData,
        }
    }
}

impl<T, B: Bounded> Deref for VecPrefix<T, B> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// We cannot provide a [`From`] implementation because [`Vec`] is not from this crate.
#[allow(clippy::from_over_into)]
impl<T, B: Bounded> Into<Vec<T>> for VecPrefix<T, B> {
    fn into(self) -> Vec<T> {
        self.inner
    }
}

impl<T, B> TryFrom<Vec<T>> for VecPrefix<T, B>
where
    B: Bounded,
{
    type Error = <B as TryFrom<usize>>::Error;

    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        B::try_from(vec.len())?;

        Ok(Self {
            inner: vec,
            bounded: PhantomData,
        })
    }
}

impl<T, B> Packable for VecPrefix<T, B>
where
    T: Packable,
    B: Bounded + Packable,
    <B::Bounds as TryInto<B>>::Error: Debug,
    <B as TryFrom<usize>>::Error: Debug,
    Range<B::Bounds>: Iterator<Item = B::Bounds>,
{
    type UnpackError = UnpackPrefixError<T::UnpackError, B::UnpackError>;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        // The length of any dynamically-sized sequence must be prefixed.
        // This unwrap is fine, since we have already validated the length in `try_from`.
        B::try_from(self.len()).unwrap().pack(packer)?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        // The length of any dynamically-sized sequence must be prefixed.
        let len = B::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(UnpackPrefixError::Prefix)?
            .into();

        let mut inner = Vec::with_capacity(len.try_into().unwrap_or(0));

        for _ in Default::default()..len {
            let item = T::unpack::<_, VERIFY>(unpacker).coerce()?;
            inner.push(item);
        }

        Ok(VecPrefix {
            inner,
            bounded: PhantomData,
        })
    }
}

/// Wrapper type for `Box<[T]>` with a length prefix.
/// The boxed slice's prefix bounds are provided by `B`, where `B` is a [`Bounded`] type.
/// The prefix type is the `Bounds` type associated with `B`.
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
pub struct BoxedSlicePrefix<T, B: Bounded> {
    inner: Box<[T]>,
    bounded: PhantomData<B>,
}

impl<T, B: Bounded> Default for BoxedSlicePrefix<T, B> {
    fn default() -> Self {
        Self {
            inner: Box::new([]),
            bounded: PhantomData,
        }
    }
}

impl<T, B: Bounded> Deref for BoxedSlicePrefix<T, B> {
    type Target = Box<[T]>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// We cannot provide a [`From`] implementation because [`Vec`] is not from this crate.
#[allow(clippy::from_over_into)]
impl<T, B: Bounded> Into<Box<[T]>> for BoxedSlicePrefix<T, B> {
    fn into(self) -> Box<[T]> {
        self.inner
    }
}

impl<T, B> TryFrom<Box<[T]>> for BoxedSlicePrefix<T, B>
where
    B: Bounded,
{
    type Error = <B as TryFrom<usize>>::Error;

    fn try_from(boxed_slice: Box<[T]>) -> Result<Self, Self::Error> {
        B::try_from(boxed_slice.len())?;

        Ok(Self {
            inner: boxed_slice,
            bounded: PhantomData,
        })
    }
}

impl<T, B> Packable for BoxedSlicePrefix<T, B>
where
    T: Packable,
    B: Bounded + Packable,
    <B::Bounds as TryInto<B>>::Error: Debug,
    <B as TryFrom<usize>>::Error: Debug,
    Range<B::Bounds>: Iterator<Item = B::Bounds>,
{
    type UnpackError = <VecPrefix<T, B> as Packable>::UnpackError;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        // The length of any dynamically-sized sequence must be prefixed.
        // This unwrap is fine, since we have already validated the length in `try_from`.
        B::try_from(self.len()).unwrap().pack(packer)?;

        for item in self.iter() {
            item.pack(packer)?;
        }

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let vec: Vec<T> = VecPrefix::<T, B>::unpack::<_, VERIFY>(unpacker)?.into();

        Ok(Self {
            inner: vec.into_boxed_slice(),
            bounded: PhantomData,
        })
    }
}
