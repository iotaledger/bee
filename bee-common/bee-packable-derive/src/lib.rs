// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! This crate provides the [`Packable`] derive macro.

#![deny(missing_docs)]

mod attribute;
mod fragments;
mod trait_impl;

use trait_impl::TraitImpl;

use proc_macro::{self, TokenStream};
use proc_macro_error::{abort, proc_macro_error};
use quote::ToTokens;
use syn::parse_macro_input;

/// The [`Packable` derive macro.
///
/// Please refer to the [`Packable`] spec for how to set this up.
/// <https://github.com/iotaledger/bee/blob/coordicide/docs/dev/specs/packable.md>
#[proc_macro_error]
#[proc_macro_derive(Packable, attributes(packable))]
pub fn packable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);

    match TraitImpl::new(input) {
        Ok(trait_impl) => trait_impl.into_token_stream().into(),
        Err(err) => abort!(err),
    }
}
