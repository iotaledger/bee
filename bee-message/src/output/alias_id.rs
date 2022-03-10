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
