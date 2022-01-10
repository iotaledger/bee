// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use syn::{parse_quote, Attribute, Field, Fields, Ident, Path, Result};

use crate::{parse::filter_attrs, record_info::RecordInfo, unpack_error_info::UnpackErrorInfo};

pub(crate) struct StructInfo {
    pub(crate) unpack_error: UnpackErrorInfo,
    pub(crate) inner: RecordInfo,
}

impl StructInfo {
    pub(crate) fn new(path: Path, fields: &Fields, attrs: &[Attribute], crate_name: &Ident) -> Result<Self> {
        let filtered_attrs = filter_attrs(attrs);

        let unpack_error = UnpackErrorInfo::new(filtered_attrs, || match fields.iter().next() {
            Some(Field { ty, .. }) => parse_quote!(<#ty as #crate_name::Packable>::UnpackError),
            None => parse_quote!(core::convert::Infallible),
        })?;

        let inner = RecordInfo::new(path, fields, &unpack_error.with)?;

        Ok(Self { unpack_error, inner })
    }
}
