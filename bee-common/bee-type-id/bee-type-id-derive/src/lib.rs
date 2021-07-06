// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A crate to derive unique identifiers for types.

#![warn(missing_docs)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use std::convert::TryInto;

/// The `TypeId` derive macro.
#[proc_macro_derive(TypeId)]
pub fn derive_type_id(input: TokenStream) -> TokenStream {
    let input_string = input.to_string();
    let DeriveInput { ident, generics, .. } = parse_macro_input!(input);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let mut hasher = blake3::Hasher::new();
    hasher.update(input_string.as_bytes());
    let result = hasher.finalize();
    let hash: [u8; 32] = result.as_bytes()[..32].try_into().unwrap();

    proc_macro::TokenStream::from(quote! {
        impl #impl_generics bee_type_id::TypeId for #ident #ty_generics #where_clause {
            const TYPE_ID: [u8; 32] = [#(#hash),*];
        }
    })
}
