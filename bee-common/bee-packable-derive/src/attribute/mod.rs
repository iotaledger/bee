// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod tag;
mod tag_type;
mod unpack_error;
mod unpack_error_with;

pub(crate) use tag::Tag;
pub(crate) use tag_type::TagType;
pub(crate) use unpack_error::UnpackError;
pub(crate) use unpack_error_with::UnpackErrorWith;

use syn::{
    parse::{Parse, ParseStream},
    Attribute, Ident, Token,
};

fn parse_attribute<T: Parse>(key: &'static str, attrs: &[Attribute]) -> Option<syn::Result<T>> {
    find_attr(key, attrs).map(|attr| attr?.parse_args::<T>())
}

fn find_attr<'attr>(key: &'static str, attrs: &'attr [Attribute]) -> Option<syn::Result<&'attr Attribute>> {
    for attr in attrs {
        if attr.path.is_ident("packable") {
            if let Ok(found_key) = attr.parse_args_with(|input: ParseStream| {
                let ident = input.parse::<Ident>();
                if ident.is_ok() {
                    // Skip the rest of the `ParseStream` to avoid errors. Unwrapping will not
                    // panic because the `step` argument always returns `Ok`.
                    input
                        .step(|cursor| {
                            let mut rest = *cursor;
                            while let Some((_, next)) = rest.token_tree() {
                                rest = next;
                            }
                            Ok(((), rest))
                        })
                        .unwrap();
                }
                ident
            }) {
                if let Err(err) = known_ident(&found_key) {
                    return Some(Err(err));
                }

                if found_key == key {
                    return Some(Ok(attr));
                }
            }
        }
    }

    None
}

fn parse_key(key: &'static str, input: ParseStream) -> syn::Result<()> {
    let found_key = input.parse::<Ident>()?;

    known_ident(&found_key)?;

    if found_key == key {
        input.parse::<Token![=]>()?;
        Ok(())
    } else {
        Err(syn::Error::new(
            found_key.span(),
            format!("expected key `{}` found `{}`", key, found_key),
        ))
    }
}

fn known_ident(ident: &Ident) -> syn::Result<()> {
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
        Err(syn::Error::new(ident.span(), format!("unknown ident `{}`", ident)))
    }
}
