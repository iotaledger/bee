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

    let mut error_type = packable::parse_attr::<Type>("error", &attrs).map(ToTokens::into_token_stream);

    let (pack, packed_len, unpack) = match data {
        Data::Struct(data_struct) => {
            error_type.get_or_insert_with(|| quote!(core::convert::Infallible));

            packable::gen_struct_bodies(data_struct.fields)
        }
        Data::Enum(data_enum) => {
            let tag_ty = packable::parse_attr::<Type>("tag_ty", &attrs).unwrap_or_else(|| {
                abort!(
                    ident.span(),
                    "Enums that derive `Packable` require a `#[packable(tag_ty = ...)]` attribute."
                )
            });

            error_type.get_or_insert_with(|| quote!(bee_common::packable::UnknownTagError<#tag_ty>));

            packable::gen_enum_bodies(&data_enum.variants, tag_ty)
        }
        Data::Union(..) => abort!(ident.span(), "Unions cannot derive `Packable`."),
    };

    packable::gen_impl(&ident, &generics, error_type.unwrap(), pack, unpack, packed_len).into()
}
