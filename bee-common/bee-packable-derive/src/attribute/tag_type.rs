// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::attribute::parse_key;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Path, Token, Type,
};

pub(crate) struct TagType {
    pub(crate) ty: Option<Type>,
    pub(crate) with_err: TokenStream,
}

impl TagType {
    pub(crate) fn new(attrs: &[Attribute]) -> syn::Result<Self> {
        super::parse_attribute::<Self>("tag_type", attrs).unwrap_or_else(|| {
            Ok(Self {
                ty: None,
                with_err: quote!(bee_packable::error::UnknownTagError),
            })
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

            Ok(Self { ty: Some(ty), with_err })
        } else {
            Ok(Self {
                ty: Some(ty),
                with_err: quote!(bee_packable::error::UnknownTagError),
            })
        }
    }
}
