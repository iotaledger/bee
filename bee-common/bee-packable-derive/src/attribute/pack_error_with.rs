// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::attribute::parse_key;

use syn::{
    parse::{Parse, ParseStream},
    Attribute, Expr,
};

pub(crate) struct PackErrorWith {
    pub(crate) with: Option<Expr>,
}

impl PackErrorWith {
    pub(crate) fn new(attrs: &[Attribute]) -> syn::Result<Self> {
        super::parse_attribute::<Self>("pack_error_with", attrs).unwrap_or(Ok(Self { with: None }))
    }
}

impl Parse for PackErrorWith {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        parse_key("pack_error_with", input)?;
        let with = input
            .parse::<Expr>()
            .map_err(|err| syn::Error::new(err.span(), "Expected an expression."))?;

        Ok(Self { with: Some(with) })
    }
}
