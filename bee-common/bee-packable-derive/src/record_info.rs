// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::field_info::{FieldInfo, IdentOrIndex};

use syn::{Expr, Fields, Ident, Path, Result, Type};

pub(crate) struct RecordInfo {
    pub(crate) path: Path,
    pub(crate) fields_unpack_error_with: Vec<Expr>,
    pub(crate) fields_pattern_ident: Vec<IdentOrIndex>,
    pub(crate) fields_ident: Vec<Ident>,
    pub(crate) fields_type: Vec<Type>,
}

impl RecordInfo {
    pub(crate) fn new(path: Path, fields: &Fields, default_unpack_error_with: &Expr) -> Result<Self> {
        let len = fields.len();
        let mut fields_unpack_error_with = Vec::with_capacity(len);
        let mut fields_ident = Vec::with_capacity(len);
        let mut fields_pattern_ident = Vec::with_capacity(len);
        let mut fields_type = Vec::with_capacity(len);

        for (index, field) in fields.iter().enumerate() {
            let FieldInfo {
                unpack_error_with,
                ident,
                pattern_ident,
                ty,
            } = FieldInfo::new(field, default_unpack_error_with, index)?;

            fields_unpack_error_with.push(unpack_error_with);
            fields_ident.push(ident);
            fields_pattern_ident.push(pattern_ident);
            fields_type.push(ty);
        }

        Ok(Self {
            path,
            fields_unpack_error_with,
            fields_pattern_ident,
            fields_ident,
            fields_type,
        })
    }
}
