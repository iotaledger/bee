// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    builder::ByteCostBuilder,
    weight::{parse_byte_cost_attributes, Weight},
};

use proc_macro2::TokenStream;
use quote::{quote_spanned, ToTokens};
use syn::{spanned::Spanned, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Index, Result};

pub(crate) enum IdentOrIndex {
    Ident(Ident),
    Index(Index),
}

impl ToTokens for IdentOrIndex {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Ident(ident) => ident.to_tokens(tokens),
            Self::Index(index) => index.to_tokens(tokens),
        }
    }
}

pub(crate) struct FieldInfo {
    pub(crate) tags: Vec<Weight>,
    pub(crate) pattern_ident: IdentOrIndex,
}

// TODO: Need better name
pub(crate) fn fields_to_info(fields: &Fields) -> Result<Vec<FieldInfo>> {
    match fields {
        Fields::Named(FieldsNamed { named, .. }) => named.into_iter().map(FieldInfo::new_named).collect(),
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => unnamed
            .into_iter()
            .enumerate()
            .map(|(index, field)| FieldInfo::new_unnamed(field, index))
            .collect(),
        unit @ Fields::Unit => Err(syn::Error::new(
            unit.span(),
            "the `ByteCost` trait cannot be derived for unit fields",
        )),
    }
}

impl ToTokens for FieldInfo {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pattern_ident = &self.pattern_ident;

        let token_stream = if self.tags.is_empty() {
            quote_spanned! {pattern_ident.span()=> + bee_byte_cost::ByteCost::weighted_bytes(&self.#pattern_ident, config) }
        } else {
            quote_spanned! {pattern_ident.span()=> + packable::PackableExt::packed_len(&self.#pattern_ident) as u64 }
        };

        token_stream.to_tokens(tokens)
    }
}

impl FieldInfo {
    pub(crate) fn new_named(field: &Field) -> Result<Self> {
        // Safety: We can unwrap here because we are looking at a named field.
        let pattern_ident = IdentOrIndex::Ident(field.ident.as_ref().unwrap().clone());
        Ok(Self {
            tags: parse_byte_cost_attributes(&field.attrs)?,
            pattern_ident,
        })
    }

    pub(crate) fn new_unnamed(field: &Field, index: usize) -> Result<Self> {
        let pattern_ident = IdentOrIndex::Index(Index {
            index: index as u32,
            span: field.span(),
        });
        Ok(Self {
            tags: parse_byte_cost_attributes(&field.attrs)?,
            pattern_ident,
        })
    }

    pub(crate) fn add_to(self, builder: &mut ByteCostBuilder) {
        let field = self.to_token_stream();
        if self.tags.is_empty() {
            field.to_tokens(&mut builder.derived);
        } else {
            for tag in self.tags.iter() {
                match tag {
                    Weight::Data => field.to_tokens(&mut builder.data),
                    Weight::Key => field.to_tokens(&mut builder.key),
                }
            }
        }
    }
}
