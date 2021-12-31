// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::parse::{filter_attrs, parse_kv, skip_stream};

use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::{parse::ParseStream, parse2, Expr, Field, Ident, Index, Result, Type};

pub(crate) enum IdentOrIndex {
    Ident(Ident),
    Index(Index),
}

impl ToTokens for IdentOrIndex {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::Index(index) => index.to_tokens(tokens),
        }
    }
}

pub(crate) struct FieldInfo {
    pub(crate) unpack_error_with: Expr,
    pub(crate) verify_with: Expr,
    pub(crate) pattern_ident: IdentOrIndex,
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
}

impl FieldInfo {
    pub(crate) fn new(field: &Field, default_unpack_error_with: &Expr, index: usize) -> Result<Self> {
        let pattern_ident = match &field.ident {
            Some(ident) => IdentOrIndex::Ident(ident.clone()),
            None => IdentOrIndex::Index(Index {
                index: index as u32,
                span: Span::call_site(),
            }),
        };

        let ident = format_ident!("field_{}", index);

        let mut unpack_error_with_opt = None;
        let mut verify_with_opt = None;

        for attr in filter_attrs(&field.attrs) {
            if let Some(verify_with) = attr.parse_args_with(|stream: ParseStream| {
                let opt = parse_kv("verify_with", stream)?;
                if opt.is_none() {
                    skip_stream(stream)?;
                }
                Ok(opt)
            })? {
                verify_with_opt = Some(verify_with);
            }

            if let Some(unpack_error_with) = attr.parse_args_with(|stream: ParseStream| {
                let opt = parse_kv("unpack_error_with", stream)?;
                if opt.is_none() {
                    skip_stream(stream)?;
                }
                Ok(opt)
            })? {
                unpack_error_with_opt = Some(unpack_error_with);
            }
        }

        Ok(Self {
            unpack_error_with: unpack_error_with_opt.unwrap_or_else(|| default_unpack_error_with.clone()),
            verify_with: verify_with_opt
                .unwrap_or_else(|| parse2(quote!(|_| -> Result<(), Self::UnpackError> { Ok(()) })).unwrap()),
            ident,
            pattern_ident,
            ty: field.ty.clone(),
        })
    }
}
