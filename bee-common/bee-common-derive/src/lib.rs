// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Derive macros for the bee-common crate.

#![warn(missing_docs)]
#![no_std]

use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derives an implementation of the trait `core::fmt::Debug` for a secret type that doesn't leak its internal secret.
/// Implements <https://github.com/iotaledger/bee-rfcs/blob/master/text/0042-secret-debug-display.md>.
/// Based on <https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs>.
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
        impl #impl_generics core::fmt::Debug for #name #ty_generics {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "<Omitted secret>")
            }
        }
    };

    expanded.into()
}

/// Derives an implementation of the trait `core::fmt::Display` for a secret type that doesn't leak its internal secret.
/// Implements <https://github.com/iotaledger/bee-rfcs/blob/master/text/0042-secret-debug-display.md>.
/// Based on <https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs>.
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
        impl #impl_generics core::fmt::Display for #name #ty_generics {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "<Omitted secret>")
            }
        }
    };

    expanded.into()
}

/// Derives an implementation of the trait `core::ops::Drop` for a secret type that calls `Zeroize::zeroize`.
/// Implements <https://github.com/iotaledger/bee-rfcs/blob/master/text/0044-secret-zeroize-drop.md>.
/// Based on <https://github.com/dtolnay/syn/blob/master/examples/heapsize/heapsize_derive/src/lib.rs>.
#[proc_macro_derive(SecretDrop)]
pub fn derive_secret_drop(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;
    // Get the different implementation elements from the input.
    let (impl_generics, ty_generics, _) = input.generics.split_for_impl();

    // The generated implementation.
    let expanded = quote! {
        impl #impl_generics core::ops::Drop for #name #ty_generics {
            fn drop(&mut self) {
                self.zeroize()
            }
        }
    };

    expanded.into()
}
