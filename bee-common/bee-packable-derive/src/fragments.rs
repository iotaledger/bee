// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::record_info::RecordInfo;

use proc_macro2::TokenStream;
use quote::quote;

pub(crate) struct Fragments {
    // The pattern used to destructure the record.
    pub(crate) pattern: TokenStream,
    // An expression that packs the record.
    pub(crate) pack: TokenStream,
    // An expresion that unpacks the record.
    pub(crate) unpack: TokenStream,
}

impl Fragments {
    pub(crate) fn new(info: RecordInfo) -> Self {
        let RecordInfo {
            path,
            fields_unpack_error_with,
            fields_verify_with,
            fields_ident,
            fields_pattern_ident,
            fields_type,
        } = info;

        Self {
            pattern: quote!(#path { #(#fields_pattern_ident: #fields_ident),* }),
            pack: quote! {
                #(<#fields_type as bee_packable::Packable>::pack(#fields_ident, packer)?;) *
                Ok(())
            },
            unpack: quote! {
                #(
                    let #fields_ident = <#fields_type as bee_packable::Packable>::unpack::<_, VERIFY>(unpacker).map_packable_err(#fields_unpack_error_with).coerce()?;
                    (#fields_verify_with)(&#fields_ident).map_err(bee_packable::error::UnpackError::from_packable)?;
                )*

                Ok(#path {
                    #(#fields_pattern_ident: #fields_ident,)*
                })
            },
        }
    }
}
