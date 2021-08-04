// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::attribute::parse_key;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Path, Token, Type,
};

pub(crate) struct UnpackError {
    pub(crate) ty: TokenStream,
    pub(crate) with: TokenStream,
}

impl UnpackError {
    pub(crate) fn for_struct<'ty, F: Fn() -> Option<&'ty Type>>(
        attrs: &[Attribute],
        first_field_ty: F,
    ) -> syn::Result<Self> {
        match super::parse_attribute::<Self>("unpack_error", attrs) {
            Some(result) => result,
            None => {
                let ty = first_field_ty()
                    .map(|ty| quote!(<#ty as Packable>::UnpackError))
                    .unwrap_or_else(|| quote!(core::convert::Infallible));

                Ok(Self {
                    ty,
                    with: quote!(core::convert::identity),
                })
            }
        }
    }

    pub(crate) fn for_enum(attrs: &[Attribute], tag_ty: &Type) -> syn::Result<Self> {
        match super::parse_attribute::<Self>("unpack_error", attrs) {
            Some(result) => result,
            None => Ok(Self {
                ty: quote!(bee_packable::UnknownTagError::<#tag_ty>),
                with: quote!(core::convert::identity),
            }),
        }
    }
}

impl Parse for UnpackError {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        parse_key("unpack_error", input)?;
        let ty = input
            .parse::<Type>()
            .map_err(|err| {
                syn::Error::new(
                    err.span(),
                    "The `unpack_error` attribute key requires a type for its value.",
                )
            })?
            .to_token_stream();

        if input.parse::<Token![,]>().is_ok() {
            parse_key("with", input)?;
            let with = input.parse::<Path>()?.to_token_stream();

            Ok(Self { ty, with })
        } else {
            Ok(Self {
                ty,
                with: quote!(core::convert::identity),
            })
        }
    }
}
