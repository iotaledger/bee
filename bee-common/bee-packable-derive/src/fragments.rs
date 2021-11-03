// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::record_info::RecordInfo;

use proc_macro2::TokenStream;
use quote::quote;

pub(crate) struct Fragments {
    // The pattern used to destructure the record.
    pub(crate) pattern: TokenStream,
    // An expression that returns the packed length of the record.
    pub(crate) packed_len: TokenStream,
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
            fields_ident,
            fields_pattern_ident,
            fields_type,
        } = info;

        Self {
            pattern: quote!(#path { #(#fields_pattern_ident: #fields_ident),* }),
            packed_len: quote!(#(<#fields_type>::packed_len(#fields_ident))+*),
            pack: quote! {
                #(<#fields_type>::pack(#fields_ident, packer)?;) *
                Ok(())
            },
            unpack: quote! {Ok(#path {
                #(#fields_pattern_ident: <#fields_type>::unpack::<_, CHECK>(unpacker).map_packable_err(#fields_unpack_error_with).coerce()?,)*
            })},
        }
    }
}
