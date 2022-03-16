// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::OutputId;

impl_id!(pub NftId, 20, "TODO.");

#[cfg(feature = "serde1")]
string_serde_impl!(NftId);

impl From<OutputId> for NftId {
    fn from(output_id: OutputId) -> Self {
        Self::from(output_id.hash())
    }
}

impl NftId {
    ///
    pub fn or_from_output_id(self, output_id: OutputId) -> NftId {
        if self.is_null() { NftId::from(output_id) } else { self }
    }
}
