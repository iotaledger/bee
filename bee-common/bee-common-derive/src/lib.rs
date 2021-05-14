// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use proc_macro::{self, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Type};

mod packable;

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
    // Check if the type has an `error` attribute.
    let error_type = packable::parse_attr::<Type>("error", &attrs).map(ToTokens::into_token_stream);

    match data {
        Data::Struct(data) => {
            // Use `Infallible` if there was no error attribute.
            let error_type = error_type.unwrap_or_else(|| quote!(core::convert::Infallible));
            // Generate the implementation for the struct.
            let (pack, packed_len, unpack) = packable::gen_bodies_for_struct(data.fields);
            packable::gen_impl(&ident, &generics, error_type, pack, unpack, packed_len).into()
        }
        Data::Enum(data) => {
            // Verify that the enum has a `"tag_ty"` attribute for the type of the tag.
            let tag_ty = packable::parse_attr::<Type>("tag_ty", &attrs).unwrap_or_else(|| {
                abort!(
                    ident.span(),
                    "Enums that derive `Packable` require a `#[packable(tag_ty = ...)]` attribute."
                )
            });
            // Use `UnknownTagError` if there was no error attribute.
            let error_type = error_type.unwrap_or_else(|| quote!(bee_common::packable::UnknownTagError<#tag_ty>));
            // Generate the implementation for the enum.
            let (pack, packed_len, unpack) = packable::gen_bodies_for_enum(&data.variants, tag_ty);
            packable::gen_impl(&ident, &generics, error_type, pack, unpack, packed_len).into()
        }
        // Unions are not supported.
        Data::Union(..) => abort!(ident.span(), "Unions cannot derive `Packable`."),
    }
}
