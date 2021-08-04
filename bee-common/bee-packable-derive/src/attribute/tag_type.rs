// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::attribute::parse_key;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Ident, Path, Token, Type,
};

pub(crate) struct TagType {
    pub(crate) ty: Type,
    pub(crate) with_err: TokenStream,
}

impl TagType {
    pub(crate) fn new(attrs: &[Attribute], enum_name: &Ident) -> syn::Result<Self> {
        super::parse_attribute::<Self>("tag_type", attrs).unwrap_or_else(|| {
            Err(syn::Error::new(
                enum_name.span(),
                "Enums that derive `Packable` require a `#[packable(tag_type = ...)]` attribute.",
            ))
        })
    }
}

impl Parse for TagType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        parse_key("tag_type", input)?;
        let ty = input.parse::<Type>()?;

        if input.parse::<Token![,]>().is_ok() {
            parse_key("with_error", input)?;
            let with_err = input.parse::<Path>()?.to_token_stream();

            Ok(Self { ty, with_err })
        } else {
            Ok(Self {
                ty,
                with_err: quote!(bee_packable::error::UnknownTagError),
            })
        }
    }
}
