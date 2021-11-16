// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This crate provides the [`Packable`] derive macro.

#![deny(missing_docs)]

mod enum_info;
mod field_info;
mod fragments;
mod parse;
mod record_info;
mod struct_info;
mod tag_type_info;
mod trait_impl;
mod unpack_error_info;
mod variant_info;

use trait_impl::TraitImpl;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::ToTokens;
use syn::parse_macro_input;

/// The [`Packable`] derive macro.
///
/// # Example
///
/// We will implement [`Packable`] for the `Maybe` type described in the example that can be found
/// in the [`Packable`] trait documentation.
/// ```rust
/// use bee_packable::Packable;
///
/// #[derive(Packable)]
/// #[packable(tag_type = u8)]
/// pub enum Maybe {
///     #[packable(tag = 0)]
///     Nothing,
///     #[packable(tag = 1)]
///     Just(i32),
/// }
/// ```
/// The code produced by this macro is equivalent to the one shown in the [`Packable`] example.
///
/// # Attributes
///
/// The derive implementation can be tweaked using `#[packable(...)]` attributes.
///
/// ## Tags for enums
///
/// A very common pattern when implementing [`Packable`] for enums consists in introducing a prefix
/// value to differentiate each variant of the enumeration when unpacking, this prefix value is
/// known as a `tag`. The type of the `tag` is specified with the `#[packable(tag_type = ...)]`
/// attribute and it can only be one of `u8`, `u16`, `u32` or `u64`. The `tag` value used for each
/// variant is specified with the `#[packable(tag = ...)]` attribute and can only contain integer
/// literal without any type prefixes (e.g. `42` is valid but `42u8` is not).
///
/// In the example above, the `tag` type is `u8`, the `Nothing` variant has a `tag` value of `0`
/// and the `Just` variant has a `tag` value of `1`. This means that the packed version of
/// `Maybe::Nothing` is `[0u8]` and the packed version of `Maybe::Just(7)` is `[1u8, 0u8, 0u8, 0u8,
/// 7u8]`.
///
/// The `tag_type` and `tag` attributes are mandatory for enums unless the enum has a
/// `#[repr(...)]` attribute in which case the `repr` type will be used as the `tag_type` and each
/// variant discriminant will be used as the `tag`. The `tag_type` and `tag` attributes take
/// precedence over the `repr` attribute.
///
/// ## The `UnpackError` associated type
///
/// The derive macro provides the optional attribute and `#[packable(unpack_error = ...)]` to
/// specify the `UnpackError` associated type. The macro also provides sensible defaults for cases
/// when the attribute is not used.
///
/// For structs, the default `UnpackError` type is the `UnpackError` of any of the fields or
/// `core::convert::Infallible` in case the struct has no fields.
///
/// For enums, the default `UnpackError` type is `UnknownTagError<T>` where `T` is the type
/// specified according to the `tag_type` or `repr` attributes.
///
/// Following the example above, `Maybe::UnpackError` is `UnknownTagError<u8>` because no
/// `unpack_error` attribute was specified.
///
/// ## Error conversion
///
/// The `unpack_error` attribute can also receive an optional additional argument using the `with`
/// identifier: `#[packable(unpack_error = ..., with = ...)]`. This `with` argument must be a Rust
/// expression and it is used to map the `UnpackError` produced while unpacking each one of the
/// fields of the type.
///
/// Sometimes it is required to map the `UnpackError` for each field individually. The
/// `#[packable(unpack_error_with = ...)]` attribute can be applied to each field for this purpose.
/// This attribute takes precedence over the `with` expression specified in the `unpack_error`
/// attribute.
///
/// The error produced when an invalid `tag` is found while unpacking an `enum` can also be
/// specified using the `with_error` optional argument for the `tag_type` attribute:
/// `#[packable(tag_type = ..., with_error = ...)]`. This argument must be a valid Rust expression.
#[proc_macro_error]
#[proc_macro_derive(Packable, attributes(packable))]
pub fn packable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);

    match TraitImpl::new(input) {
        Ok(trait_impl) => trait_impl.into_token_stream().into(),
        Err(err) => abort!(err),
    }
}
