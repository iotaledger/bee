// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use quote::quote;
use syn::{parse2, Attribute, Field, Fields, Path, Result};

use crate::{parse::filter_attrs, record_info::RecordInfo, unpack_error_info::UnpackErrorInfo};

pub(crate) struct StructInfo {
    pub(crate) unpack_error: UnpackErrorInfo,
    pub(crate) inner: RecordInfo,
}

impl StructInfo {
    pub(crate) fn new(path: Path, fields: &Fields, attrs: &[Attribute]) -> Result<Self> {
        let filtered_attrs = filter_attrs(attrs);

        let unpack_error = UnpackErrorInfo::new(filtered_attrs, || match fields.iter().next() {
            Some(Field { ty, .. }) => parse2(quote!(<#ty as bee_packable::Packable>::UnpackError)),
            None => parse2(quote!(core::convert::Infallible)),
        })?;

        let inner = RecordInfo::new(path, fields, &unpack_error.with)?;

        Ok(Self { unpack_error, inner })
    }
}
