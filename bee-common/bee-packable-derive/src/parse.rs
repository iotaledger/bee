// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use syn::{
    parse::{Parse, ParseStream},
    Attribute, Error, Ident, Result, Token,
};

pub(crate) fn filter_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> + Clone {
    attrs.iter().filter(|attr| attr.path.is_ident("packable"))
}

pub(crate) fn skip_stream(stream: ParseStream) -> Result<()> {
    stream.step(|cursor| {
        let mut rest = *cursor;
        while let Some((_, next)) = rest.token_tree() {
            rest = next;
        }
        Ok(((), rest))
    })
}

pub(crate) fn parse_kv<T: Parse>(ident: &'static str, stream: ParseStream) -> Result<Option<T>> {
    let found_ident = stream.parse::<Ident>()?;
    validate_ident(&found_ident)?;

    if found_ident == ident {
        stream.parse::<Token![=]>()?;
        stream.parse::<T>().map(Some)
    } else {
        Ok(None)
    }
}

pub(crate) fn parse_kv_after_comma<T: Parse>(ident: &'static str, stream: ParseStream) -> Result<Option<T>> {
    if stream.is_empty() {
        return Ok(None);
    }

    stream.parse::<Token![,]>()?;
    parse_kv::<T>(ident, stream)?
        .ok_or_else(|| stream.error(format!("Expected `{}` identifier.", ident)))
        .map(Some)
}

fn validate_ident(ident: &Ident) -> Result<()> {
    const KNOWN_IDENTS: &[&str] = &[
        "unpack_error",
        "unpack_error_with",
        "tag_type",
        "tag",
        "with",
        "with_error",
    ];

    if KNOWN_IDENTS.iter().any(|known_ident| ident == known_ident) {
        Ok(())
    } else {
        Err(Error::new(ident.span(), format!("Unknown identifier `{}`.", ident)))
    }
}
