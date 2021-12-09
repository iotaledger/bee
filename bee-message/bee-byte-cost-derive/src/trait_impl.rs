// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{builder::ByteCostBuilder, enum_info::EnumInfo, field_info::fields_to_info};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Data, DeriveInput, Result};

pub(crate) fn process(input: DeriveInput) -> Result<TokenStream> {
    let mut builder = ByteCostBuilder::new(input.ident.clone());

    match input.data {
        Data::Struct(s) => {
            for info in fields_to_info(&s.fields)? {
                info.add_to(&mut builder);
            }
        }
        Data::Enum(e) => {
            let enum_info = EnumInfo::new(e.variants)?;
            enum_info.add_to(&mut builder);
        }
        Data::Union(_) => {
            return Err(syn::Error::new(
                input.ident.span(),
                "the `ByteCost` trait cannot be derived for unions",
            ));
        }
    }

    Ok(builder.into_token_stream())
}
