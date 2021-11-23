// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    spanned::Spanned,
    Attribute, Signature, Visibility,
};

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
                bee_subscriber::Observe::observe(
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
