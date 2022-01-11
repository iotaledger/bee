// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::record_info::RecordInfo;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

pub(crate) struct Fragments {
    // The pattern used to destructure the record.
    pub(crate) pattern: TokenStream,
    // An expression that packs the record.
    pub(crate) pack: TokenStream,
    // An expresion that unpacks the record.
    pub(crate) unpack: TokenStream,
}

impl Fragments {
    pub(crate) fn new(info: RecordInfo, crate_name: &Ident) -> Self {
        let RecordInfo {
            path,
            fields_unpack_error_with,
            fields_verify_with,
            fields_ident,
            fields_pattern_ident,
            fields_type,
        } = info;

        let fields_verification = fields_verify_with.into_iter().zip(fields_ident.iter()).map(|(verify_with, field_ident)| match verify_with {
            Some(verify_with) => quote!(#verify_with::<VERIFY>(&#field_ident).map_err(#crate_name::error::UnpackError::from_packable)?;),
            None => quote!(),
        });

        Self {
            pattern: quote!(#path { #(#fields_pattern_ident: #fields_ident),* }),
            pack: quote! {
                #(<#fields_type as #crate_name::Packable>::pack(#fields_ident, packer)?;) *
                Ok(())
            },
            unpack: quote! {
                #(
                    let #fields_ident = <#fields_type as #crate_name::Packable>::unpack::<_, VERIFY>(unpacker).map_packable_err(#fields_unpack_error_with).coerce()?;
                    #fields_verification
                )*

                Ok(#path {
                    #(#fields_pattern_ident: #fields_ident,)*
                })
            },
        }
    }
}
