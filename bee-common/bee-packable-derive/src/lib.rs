// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This crate provides the [`Packable`] derive macro.

#![warn(missing_docs)]

mod packable;

use proc_macro::{self, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Type};

/// The [`Packable` derive macro.
///
/// Please refer to the [`Packable`] spec for how to set this up.
/// <https://github.com/iotaledger/bee/blob/coordicide/docs/dev/specs/packable.md>
#[proc_macro_error]
#[proc_macro_derive(Packable, attributes(packable))]
pub fn packable(input: TokenStream) -> TokenStream {
    let DeriveInput {
        ident,
        data,
        attrs,
        generics,
        ..
    } = parse_macro_input!(input);
    // Parse a `pack_error` attribute if the input has one.
    let mut pack_error_type = match packable::parse_attr::<Type>("pack_error", &attrs) {
        Some(Ok(ty)) => Some(ty.into_token_stream()),
        Some(Err(span)) => abort!(span, "The `pack_error` attribute requires a type for its value."),
        None => None,
    };

    // Parse an `unpack_error` attribute if the input has one.
    let mut unpack_error_type = match packable::parse_attr::<Type>("unpack_error", &attrs) {
        Some(Ok(ty)) => Some(ty.into_token_stream()),
        Some(Err(span)) => abort!(span, "The `unpack_error` attribute requires a type for its value."),
        None => None,
    };

    match data {
        Data::Struct(data) => {
            // Generate the implementation for the struct.
            let (pack, packed_len, unpack) =
                packable::gen_bodies_for_struct(data.fields, &mut pack_error_type, &mut unpack_error_type);
            // Use `Infallible` if there was no pack_error attribute and the struct do not have
            // fields.
            let pack_error_type = pack_error_type.unwrap_or_else(|| quote!(core::convert::Infallible));
            // Use `Infallible` if there was no unpack_error attribute and the struct do not have
            // fields.
            let unpack_error_type = unpack_error_type.unwrap_or_else(|| quote!(core::convert::Infallible));

            packable::gen_impl(
                &ident,
                &generics,
                pack_error_type,
                unpack_error_type,
                pack,
                packed_len,
                unpack,
            )
            .into()
        }
        Data::Enum(data) => {
            // Verify that the enum has a `"tag_type"` attribute for the type of the tag.
            let tag_ty = match packable::parse_attr::<Type>("tag_type", &attrs) {
                Some(Ok(tag_ty)) => tag_ty,
                Some(Err(span)) => abort!(span, "The `tag_type` attribute requires a type for its value."),
                None => abort!(
                    ident.span(),
                    "Enums that derive `Packable` require a `#[packable(tag_type = ...)]` attribute."
                ),
            };
            // Use `UnknownTagError` if there was no unpack_error attribute. We override this first
            // because it is more reasonable to use `UnknownTagError` than the error provided by
            // any field.
            if unpack_error_type.is_none() {
                unpack_error_type = Some(quote!(bee_packable::error::UnknownTagError<#tag_ty>));
            }
            // Generate the implementation for the enum.
            let (pack, packed_len, unpack) = packable::gen_bodies_for_enum(
                &data.variants,
                tag_ty,
                &mut pack_error_type,
                // This reference will never be used inside `gen_bodies_for_enum`.
                &mut unpack_error_type,
            );
            // Use `Infallible` if there was no pack_error attribute and the variants do not have
            // fields.
            let pack_error_type = pack_error_type.unwrap_or_else(|| quote!(core::convert::Infallible));
            // This unwrap cannot fail because we set its value before.
            let unpack_error_type = unpack_error_type.unwrap();

            packable::gen_impl(
                &ident,
                &generics,
                pack_error_type,
                unpack_error_type,
                pack,
                packed_len,
                unpack,
            )
            .into()
        }
        // Unions are not supported.
        Data::Union(..) => abort!(ident.span(), "Unions cannot derive `Packable`."),
    }
}
