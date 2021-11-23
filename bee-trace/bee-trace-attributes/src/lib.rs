// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Attribute macro for extending functions and futures with the `Observe` trait.

#![deny(missing_docs, warnings)]

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Attribute, Signature, Visibility,
};

/// Attribute macro for extending functions and futures with the `Observe` trait.
///
/// This instruments the function or future with a `tracing` span with the `bee::observe` target, so that
/// it can be filtered by subscribers. It also records the location of the calling code to the span as
/// fields. This is assigned to `loc.file`, `loc.line` and `loc.col` fields, similar to how `tokio` instruments
/// tasks internally.
///
/// As such, `tokio` tasks, any functions or futures instrumented with `tracing`, and any functions or futures
/// instrumented with the `Observe` trait or macro will be wrapped in spans that contain similarly structured
/// information for diagnostics. The only difference should be the span target and the span name (if
/// available).
///
/// # Examples
///
/// A future or function can be wrapped in a `tracing` span with the following:
/// ```rust
/// use bee_trace::observe;
///
/// #[observe]
/// pub async fn say_hello() {
///     println!("hello");
/// }
/// ```
///
/// This will generate a span equivalent to the following:
/// ```ignore
/// // Location of the function signature.
/// let location = std::panic::Location::caller();
///
/// tracing::trace_span!(
///     "bee::observe",
///     "observed",
///     observed.name = "say_hello",
///     loc.file = location.file(),
///     loc.line = location.line(),
///     loc.col = location.column(),
/// );
/// ```
///
/// The future or function will then run inside the context of the generated span:
/// ```ignore
/// let _guard = span.enter();
///
/// async move {
///     println!("hello");
/// }
/// .await;
/// ```
#[proc_macro_attribute]
pub fn observe(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let observe_impl = parse_macro_input!(input as ObserveImpl);
    observe_impl.gen_tokens().into()
}

#[derive(Debug)]
struct ObserveImpl {
    attributes: Vec<Attribute>,
    visibility: Visibility,
    signature: Signature,
    block: TokenStream,
}

impl Parse for ObserveImpl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let visibility = input.parse()?;
        let signature = input.parse()?;
        let block = input.parse()?;

        Ok(Self {
            attributes,
            visibility,
            signature,
            block,
        })
    }
}

impl ObserveImpl {
    fn gen_tokens(self) -> TokenStream {
        let ObserveImpl {
            attributes,
            visibility,
            signature,
            block,
        } = self;

        let fn_name = signature.ident.to_string();

        let block = match &signature.asyncness {
            Some(_) => Self::gen_async_block(fn_name, &block),
            None => Self::gen_block(fn_name, &block),
        };

        quote! {
            #(#attributes)*
            #visibility #signature
            {
                #block
            }
        }
    }

    fn gen_block(fn_name: String, block: &TokenStream) -> TokenStream {
        let span = quote! {
            {
                let location = std::panic::Location::caller();

                tracing::trace_span!(
                    target: "bee::observe",
                    "observed",
                    observed.name = #fn_name,
                    loc.file = location.file(),
                    loc.line = location.line(),
                    loc.col = location.column(),
                )
            }
        };

        quote_spanned! {
            block.span() => {
                let span = #span;
                let _guard = span.enter();
                #block
            }
        }
    }

    fn gen_async_block(fn_name: String, block: &TokenStream) -> TokenStream {
        let observed_future = quote_spanned! {
            block.span() => async move #block
        };

        quote! {
            bee_trace::Observe::observe(#observed_future, #fn_name).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{quote, ObserveImpl, TokenStream};

    #[test]
    fn observe_async() {
        let input: TokenStream = quote! {
            async fn test_observe_fn() {
                println!("observing this function");
            }
        };

        let observe_impl = syn::parse2::<ObserveImpl>(input);
        assert!(observe_impl.is_ok());

        let output_tokens = observe_impl.unwrap().gen_tokens();
        let expected_tokens: TokenStream = quote! {
            async fn test_observe_fn() {
                bee_trace::Observe::observe(
                    async move {
                        println!("observing this function");
                    },
                    "test_observe_fn"
                ).await
            }
        };

        assert_eq!(output_tokens.to_string(), expected_tokens.to_string());
    }

    #[test]
    fn observe() {
        let input: TokenStream = quote! {
            fn test_observe_fn() {
                println!("observing this function");
            }
        };

        let observe_impl = syn::parse2::<ObserveImpl>(input);
        assert!(observe_impl.is_ok());

        let output_tokens = observe_impl.unwrap().gen_tokens();
        let expected_tokens: TokenStream = quote! {
            fn test_observe_fn() {
                {
                    let span = {
                        let location = std::panic::Location::caller();

                        tracing::trace_span!(
                            target: "bee::observe",
                            "observed",
                            observed.name = "test_observe_fn",
                            loc.file = location.file(),
                            loc.line = location.line(),
                            loc.col = location.column(),
                        )
                    };

                    let _guard = span.enter();
                    {
                        println!("observing this function");
                    }
                }
            }
        };

        assert_eq!(output_tokens.to_string(), expected_tokens.to_string());
    }
}
