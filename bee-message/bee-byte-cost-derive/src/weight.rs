// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use syn::{spanned::Spanned, Attribute, Error, Meta, NestedMeta, Result};

pub(crate) enum Weight {
    Key,
    Data,
}

pub fn filter_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> + Clone {
    attrs.iter().filter(|attr| attr.path.is_ident("byte_cost"))
}

fn parse_byte_cost_attr(attr: &Attribute) -> Result<Vec<Weight>> {
    let input = attr.parse_meta()?;
    let mut tags = Vec::new();

    // Consider using parse_args_with instead
    match input {
        Meta::List(list) => {
            if list.nested.is_empty() {
                return Err(Error::new(
                    list.paren_token.span,
                    "field attribute requires one or more weight specifiers such as `key` or `data`",
                ));
            }

            for n in list.nested {
                match n {
                    NestedMeta::Meta(Meta::Path(p)) => {
                        if p.is_ident("key") {
                            tags.push(Weight::Key);
                        } else if p.is_ident("data") {
                            tags.push(Weight::Data);
                        } else {
                            return Err(Error::new(
                                p.span(),
                                format!("unknown weight identifier `{}`.", p.get_ident().unwrap()),
                            ));
                        }
                    }
                    other => {
                        return Err(Error::new(
                            other.span(),
                            "field attribute requires one or more weight specifiers such as `#[byte_cost(key)]` or `#[byte_cost(key,data)]`",
                        ));
                    }
                }
            }
        }
        other => {
            return Err(Error::new(
                other.span(),
                "field attribute requires one or more weight specifiers such as `#[byte_cost(key)]` or `#[byte_cost(key,data)]`",
            ));
        }
    }

    Ok(tags)
}

pub(crate) fn parse_byte_cost_attributes(attrs: &[Attribute]) -> Result<Vec<Weight>> {
    let byte_cost_attrs: Vec<&Attribute> = filter_attrs(attrs).collect();

    let tags = if byte_cost_attrs.is_empty() {
        Vec::new()
    } else if byte_cost_attrs.len() == 1 {
        parse_byte_cost_attr(byte_cost_attrs[0])?
    } else {
        return Err(Error::new(
            // Safety: We just checked if there are multiple elements.
            byte_cost_attrs[0].span(),
            "there can be at most one `#[byte_cost(...)]` attribute per field",
        ));
    };

    Ok(tags)
}
