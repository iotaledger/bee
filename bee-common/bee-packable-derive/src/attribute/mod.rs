// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod pack_error;
mod tag;
mod tag_type;
mod unpack_error;

use syn::{
    parse::{Parse, ParseStream},
    Attribute, Ident, Token,
};

pub(crate) use pack_error::PackError;
pub(crate) use tag::Tag;
pub(crate) use tag_type::TagType;
pub(crate) use unpack_error::UnpackError;

fn parse_attribute<T: Parse>(key: &'static str, attrs: &[Attribute]) -> Option<syn::Result<T>> {
    find_attr(key, attrs).map(|attr| attr.parse_args::<T>())
}

fn find_attr<'attr>(key: &'static str, attrs: &'attr [Attribute]) -> Option<&'attr Attribute> {
    for attr in attrs {
        if attr.path.is_ident("packable") {
            if let Ok(found_key) = attr.parse_args_with(|input: ParseStream| {
                let ident = input.parse::<Ident>();
                if ident.is_ok() {
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
                if found_key == key {
                    return Some(attr);
                }
            }
        }
    }

    None
}

fn parse_key(key: &'static str, input: ParseStream) -> syn::Result<()> {
    let found_key = input.parse::<Ident>()?;
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
