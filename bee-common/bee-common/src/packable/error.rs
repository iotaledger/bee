// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use core::fmt::{Debug, Display};

use super::Packable;

/// A type that represents errors with the unpacking format as well as with the unpacking process itself.
pub trait UnpackError: Sized + Debug {
    /// Raised when there is general error when unpacking a type.
    fn custom<T: Display>(msg: T) -> Self;

    /// Raised when there is an unknown variant tag while unpacking an enum.
    fn unknown_variant<P: Packable>(tag: u64) -> Self {
        Self::custom(core::format_args!(
            "the tag `{}` is not valid for the enum `{}`",
            tag,
            core::any::type_name::<P>(),
        ))
    }
}
