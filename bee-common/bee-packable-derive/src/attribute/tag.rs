// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::attribute::parse_key;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, ExprLit, ExprPath,
};

pub(crate) struct Tag {
    pub(crate) value: Option<ExprTag>,
}

impl Tag {
    pub(crate) fn new(attrs: &[Attribute]) -> syn::Result<Self> {
        super::parse_attribute::<Self>("tag", attrs).unwrap_or(Ok(Self { value: None }))
    }
}

impl Parse for Tag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        parse_key("tag", input)?;
        let value = input
            .parse::<ExprTag>()
            .map_err(|err| syn::Error::new(err.span(), "Tags for variants can only be literal or path expressions."))?;

        Ok(Self { value: Some(value) })
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ExprTag {
    Lit(ExprLit),
    Path(ExprPath),
}

impl Parse for ExprTag {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match ExprLit::parse(input) {
            Ok(lit) => Ok(Self::Lit(lit)),
            Err(_) => Ok(Self::Path(ExprPath::parse(input)?)),
        }
    }
}

impl ToTokens for ExprTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Lit(lit) => lit.to_tokens(tokens),
            Self::Path(path) => path.to_tokens(tokens),
        }
    }
}
