// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::fmt::Display;

use super::Packable;

/// A type that represents errors with the unpacking format as well as with the unpacking process itself.
pub trait UnpackError {
    /// Raised when there is general error when unpacking a type.
    fn custom<T: Display>(msg: T) -> Self;
}

/// An error raised when there is an unknown variant ID while unpacking an enum.
#[derive(Debug)]
pub struct UnknownVariant {
    id: u64,
    type_name: &'static str,
}

impl Display for UnknownVariant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "the ID {} is not valid for the enum `{}`", self.id, self.type_name)
    }
}

impl UnknownVariant {
    /// Create an error with unknown variant ID `id` for the enum `P`.
    pub fn new<P: Packable>(id: u64) -> Self {
        Self {
            id,
            type_name: core::any::type_name::<P>(),
        }
    }
}
