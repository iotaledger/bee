// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Macro that forwards a given `enum` to a wrapper `enum`, taking the data and moving it to the wrapper enum variant.
#[macro_export]
macro_rules! impl_wrapped_variant {
    ($dst:ty, $variant:path, $src:ty) => {
        impl From<$src> for $dst {
            fn from(src: $src) -> $dst {
                $variant(src)
            }
        }
    };
}

/// Macro with the same functionality as `impl_wrapped_variant`, but specifically forwards a `ValidationError` up the
/// `enum` variant chain.
#[macro_export]
macro_rules! impl_wrapped_validated {
    ($dst:ident, $variant:path, $src:ident) => {
        impl From<$src> for $dst {
            fn from(src: $src) -> $dst {
                match src {
                    $src::Validation(error) => $dst::Validation(error),
                    error => $variant(error),
                }
            }
        }
    };
}

/// Quickly implements [`From<Infallible>`] for a given type.
#[macro_export]
macro_rules! impl_from_infallible {
    ($type:ty) => {
        impl From<core::convert::Infallible> for $type {
            fn from(i: Infallible) -> $type {
                match i {}
            }
        }
    };
}
