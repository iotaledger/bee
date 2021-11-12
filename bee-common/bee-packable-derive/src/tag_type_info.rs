// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::parse::{parse_kv, parse_kv_after_comma, skip_stream};

use quote::{quote, ToTokens};
use syn::{parse::ParseStream, parse2, spanned::Spanned, Attribute, Error, Expr, Ident, Result, Type};

const VALID_TAG_TYPES: &[&str] = &["u8", "u16", "u32", "u64"];

pub(crate) struct TagTypeInfo {
    pub(crate) tag_type: Type,
    pub(crate) with_error: Expr,
}

impl TagTypeInfo {
    pub(crate) fn new<'a>(
        enum_ident: &Ident,
        filtered_attrs: impl Iterator<Item = &'a Attribute>,
        repr_type: &Option<Type>,
    ) -> Result<Self> {
        for attr in filtered_attrs {
            let opt_info = attr.parse_args_with(|stream: ParseStream| match parse_kv::<Type>("tag_type", stream)? {
                Some(tag_type) => {
                    if !VALID_TAG_TYPES.contains(&tag_type.to_token_stream().to_string().as_str()) {
                        return Err(Error::new(
                            tag_type.span(),
                            "Tags for enums can only be of type `u8`, `u16`, `u32` or `u64`.",
                        ));
                    }

                    let with_error = match parse_kv_after_comma("with_error", stream)? {
                        Some(with_error) => with_error,
                        None => {
                            skip_stream(stream)?;
                            parse2(quote!(bee_packable::error::UnknownTagError))?
                        }
                    };
                    Ok(Some(Self { tag_type, with_error }))
                }
                None => {
                    skip_stream(stream)?;
                    Ok(None)
                }
            });

            if let Some(info) = opt_info? {
                return Ok(info);
            }
        }

        match repr_type {
            Some(repr_type) => Ok(Self {
                tag_type: repr_type.clone(),
                with_error: parse2(quote!(bee_packable::error::UnknownTagError))?,
            }),
            None => Err(Error::new(
                enum_ident.span(),
                "Enums that derive `Packable` require a `#[packable(tag_type = ...)]` or `#[repr(...)]` attribute.",
            )),
        }
    }
}
