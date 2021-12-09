// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    builder::ByteCostBuilder,
    field_info::{fields_to_info, FieldInfo},
    weight::filter_attrs,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{Ident, Result, Variant};

pub(crate) struct VariantInfo {
    ident: Ident,
    fields: Vec<FieldInfo>,
}

impl VariantInfo {
    pub(crate) fn new(variant: &Variant) -> Result<Self> {
        if filter_attrs(&variant.attrs).next().is_some() {
            return Err(syn::Error::new(
                variant.ident.span(),
                "weight types cannot be applied to variants of an enum",
            ));
        }

        Ok(Self {
            fields: fields_to_info(&variant.fields)?,
            ident: variant.ident.clone(),
        })
    }
}

impl ToTokens for VariantInfo {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let variant_fields = (0..self.fields.len()).map(|n| format_ident!("field_{}", n));
        let variant_fields_2 = variant_fields.clone();
        let match_arm = quote! {
            Self::#ident(#(#variant_fields),*) => { #(bee_byte_cost::ByteCost::weighted_bytes(#variant_fields_2, config))+* },
        };
        match_arm.to_tokens(tokens);
    }
}

pub(crate) struct EnumInfo(Vec<VariantInfo>);

impl EnumInfo {
    pub(crate) fn new(variants: impl IntoIterator<Item = Variant>) -> Result<Self> {
        let variants = variants
            .into_iter()
            .map(|v| VariantInfo::new(&v))
            .collect::<Result<Vec<VariantInfo>>>();
        Ok(Self(variants?))
    }

    pub(crate) fn add_to(self, builder: &mut ByteCostBuilder) {
        let match_arms = self.0.iter();

        // TODO: Don't hard code the length of KIND;
        let match_tokens = quote! {
            + 1 + match self {
                #(#match_arms)*
            }
        };
        match_tokens.to_tokens(&mut builder.derived);
    }
}
