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

    let error_type = match packable::parse_attr::<Type>("error", &attrs) {
        Some(err) => match data {
            Data::Enum(_) => quote!(bee_common::packable::UnpackEnumError<#err>),
            _ => err.into_token_stream(),
        },
        None => abort!(
            ident.span(),
            "Types that derive `Packable` require a `#[packable(error = ...)]` attribute."
        ),
    };

    let (pack, unpack, packed_len) = match data {
        Data::Struct(data_struct) => packable::gen_struct_bodies(data_struct.fields),
        Data::Enum(data_enum) => match packable::parse_attr::<Type>("ty", &attrs) {
            Some(ty) => packable::gen_enum_bodies(data_enum.variants.iter(), ty),
            None => abort!(
                ident.span(),
                "Enums that derive `Packable` require a `#[packable(ty = ...)]` attribute."
            ),
        },
        Data::Union(..) => abort!(ident.span(), "Unions cannot derive `Packable`"),
    };

    packable::gen_impl(&ident, &generics, error_type, pack, unpack, packed_len).into()
}
