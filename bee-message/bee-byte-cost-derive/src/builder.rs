// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Ident;

pub(crate) struct ByteCostBuilder {
    ident: Ident,
    pub key: TokenStream,
    pub data: TokenStream,
    pub derived: TokenStream,
}

impl ByteCostBuilder {
    pub(crate) fn new(ident: Ident) -> Self {
        Self {
            ident,
            key: quote! { 0 },
            data: quote! { 0 },
            derived: quote! { 0 },
        }
    }
}

impl ToTokens for ByteCostBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let key = &self.key;
        let data = &self.data;
        let derived = &self.derived;

        let output = quote! {
            impl ByteCost for #ident {
                fn weighted_bytes(&self, config: &bee_byte_cost::ByteCostConfig) -> u64 {
                    (#key) * config.weight_for_key +
                    (#data) * config.weight_for_data +
                    #derived // they have already been weighted
                }
            }
        };

        output.to_tokens(tokens);
    }
}
