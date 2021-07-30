// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module to wrap [`Packable`](crate::Packable) values.
use core::convert::Infallible;

/// A type whose values can be wrapped in values of type `W`.
///
/// In essence, `A: Wrap<B>` means that some values of type `&B` can be converted into a value of
/// type `&a` (via `Wrap::wrap`), also known as "wrapping", and that every `A` can be converted
/// into a value of type `B` (via [`Into::into`]), also known as "unwrapping".
pub trait Wrap<'a, W: 'a>: Sized + Into<W> {
    /// Error raised when it is not possible to convert a value to one of type `W`.
    type Error;
    /// Wraps a reference.
    fn wrap(value: &'a W) -> Result<&'a Self, Self::Error>;
}

/// `Wrap` is reflexive.
impl<'a, T: 'a + Sized> Wrap<'a, T> for T {
    type Error = Infallible;

    fn wrap(value: &'a T) -> Result<&'a Self, Self::Error> {
        Ok(value)
    }
}
