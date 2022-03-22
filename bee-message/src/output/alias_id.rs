// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::OutputId;

impl_id!(pub AliasId, 20, "TODO.");

#[cfg(feature = "serde1")]
string_serde_impl!(AliasId);

impl From<OutputId> for AliasId {
    fn from(output_id: OutputId) -> Self {
        Self::from(output_id.hash())
    }
}

impl AliasId {
    ///
    pub fn or_from_output_id(self, output_id: OutputId) -> Self {
        if self.is_null() {
            Self::from(output_id)
        } else {
            self
        }
    }
}
