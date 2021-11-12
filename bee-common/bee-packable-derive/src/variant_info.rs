// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    parse::{filter_attrs, parse_kv},
    record_info::RecordInfo,
};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse2, Error, Expr, ExprLit, ExprPath, Result, Variant,
};

#[derive(Debug, Clone)]
pub(crate) enum ExprTag {
    Lit(ExprLit),
    Path(ExprPath),
}

impl Parse for ExprTag {
    fn parse(input: ParseStream) -> Result<Self> {
        match ExprLit::parse(input) {
            Ok(lit) => Ok(Self::Lit(lit)),
            Err(_) => Ok(Self::Path(ExprPath::parse(input).map_err(|err| {
                Error::new(err.span(), "Tags for variants can only be literal or path expressions.")
            })?)),
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

pub(crate) struct VariantInfo {
    pub(crate) tag: ExprTag,
    pub(crate) inner: RecordInfo,
}

impl VariantInfo {
    pub(crate) fn new(variant: &Variant, enum_ident: &syn::Ident, default_unpack_error_with: &Expr) -> Result<Self> {
        let variant_ident = variant.ident.clone();

        for attr in filter_attrs(&variant.attrs) {
            if let Some(tag) = attr.parse_args_with(|stream: ParseStream| parse_kv("tag", stream))? {
                return Ok(Self {
                    tag,
                    inner: RecordInfo::new(
                        parse2(quote!(#enum_ident::#variant_ident))?,
                        &variant.fields,
                        default_unpack_error_with,
                    )?,
                });
            }
        }

        match &variant.discriminant {
            Some((_, tag)) => Ok(Self {
                tag: parse2(quote!(#tag))?,
                inner: RecordInfo::new(
                    parse2(quote!(#enum_ident::#variant_ident))?,
                    &variant.fields,
                    default_unpack_error_with,
                )?,
            }),
            None => Err(Error::new(
                variant_ident.span(),
                "All variants of an enum that derives `Packable` require a `#[packable(tag = ...)]` attribute or an explicitly set discriminant.",
            )),
        }
    }
}
