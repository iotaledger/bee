// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

//! Derive macros for the bee-signing crate.

#![warn(missing_docs)]

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derives an implementation of the trait Debug for a secret type that doesn't leak its internal secret.
/// Implements https://github.com/iotaledger/bee-rfcs/blob/master/text/0042-secret-debug-display.md.
/// Based on https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs.
#[proc_macro_derive(SecretDebug)]
pub fn derive_secret_debug(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    // Get the different implementation elements from the input.
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    // The generated implementation.
    let expanded = quote! {
        impl #impl_generics std::fmt::Debug for #name #ty_generics {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "<Omitted secret>")
            }
        }
    };

    expanded.into()
}

/// Derives an implementation of the trait Display for a secret type that doesn't leak its internal secret.
/// Implements https://github.com/iotaledger/bee-rfcs/blob/master/text/0042-secret-debug-display.md.
/// Based on https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs.
#[proc_macro_derive(SecretDisplay)]
pub fn derive_secret_display(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    // Get the different implementation elements from the input.
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    // The generated implementation.
    let expanded = quote! {
        impl #impl_generics std::fmt::Display for #name #ty_generics {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "<Omitted secret>")
            }
        }
    };

    expanded.into()
}
