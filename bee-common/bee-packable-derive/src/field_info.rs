// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::parse::{filter_attrs, parse_kv};

use proc_macro2::Span;
use quote::{format_ident, ToTokens};
use syn::{parse::ParseStream, Expr, Field, Ident, Index, Result, Type};

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

        for attr in filter_attrs(&field.attrs) {
            if let Some(unpack_error_with) =
                attr.parse_args_with(|stream: ParseStream| parse_kv("unpack_error_with", stream))?
            {
                return Ok(Self {
                    unpack_error_with,
                    ident,
                    pattern_ident,
                    ty: field.ty.clone(),
                });
            }
        }
        Ok(Self {
            unpack_error_with: default_unpack_error_with.clone(),
            ident,
            pattern_ident,
            ty: field.ty.clone(),
        })
    }
}
