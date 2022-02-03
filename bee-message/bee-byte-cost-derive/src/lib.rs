// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod builder;
mod enum_info;
mod field_info;
mod trait_impl;
mod weight;
use trait_impl::process;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use syn::parse_macro_input;

#[proc_macro_error]
#[proc_macro_derive(ByteCost, attributes(byte_cost))]
pub fn byte_cost_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);

    match process(input) {
        Ok(token_stream) => token_stream.into(),
        Err(err) => abort!(err),
    }
}
