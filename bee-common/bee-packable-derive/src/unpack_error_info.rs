// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::parse::{parse_kv, parse_kv_after_comma, skip_stream};

use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2, Attribute, Error, Expr, Result,
};

pub(crate) struct UnpackErrorInfo {
    pub(crate) unpack_error: syn::Type,
    pub(crate) with: Expr,
}

struct Type(syn::Type);

impl Parse for Type {
    fn parse(input: ParseStream) -> Result<Self> {
        syn::Type::parse(input).map(Self).map_err(|err| {
            Error::new(
                err.span(),
                "The `unpack_error` attribute requires a type for its value.",
            )
        })
    }
}

impl UnpackErrorInfo {
    pub(crate) fn new<'a>(
        filtered_attrs: impl Iterator<Item = &'a Attribute>,
        default_unpack_error: impl FnOnce() -> Result<syn::Type>,
    ) -> Result<Self> {
        for attr in filtered_attrs {
            let opt_info =
                attr.parse_args_with(|stream: ParseStream| match parse_kv::<Type>("unpack_error", stream)? {
                    Some(Type(unpack_error)) => {
                        let with = match parse_kv_after_comma("with", stream)? {
                            Some(with) => with,
                            None => parse2(quote!(core::convert::identity))?,
                        };

                        Ok(Some(Self { unpack_error, with }))
                    }
                    None => {
                        skip_stream(stream)?;
                        Ok(None)
                    }
                })?;

            if let Some(info) = opt_info {
                return Ok(info);
            }
        }

        Ok(Self {
            unpack_error: default_unpack_error()?,
            with: parse2(quote!(core::convert::identity))?,
        })
    }
}
