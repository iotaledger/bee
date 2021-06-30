// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module to wrap `Packable` values.

/// A type whose values can be wrapped in values of type `W`.
///
/// In essence, `A: Wrap<B>` means that every value of type `&A` can be converted into a value of
/// type `&B` (via `Wrap::wrap`), also known as "wrapping", and that every `B` can be converted
/// into a value of type `A` (via `Into::into`), also known as "unwrapping".
pub trait Wrap<W: Into<Self>>: Sized {
    /// Wraps a reference.
    fn wrap(&self) -> &W;
}

/// `Wrap` is reflexive.
impl<T: Sized> Wrap<T> for T {
    fn wrap(&self) -> &Self {
        self
    }
}
