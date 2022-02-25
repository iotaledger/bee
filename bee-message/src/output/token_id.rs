// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::{foundry::TOKEN_TAG_LENGTH, FoundryId};

use alloc::vec::Vec;

impl_id!(TokenId, 38, "TODO.");

#[cfg(feature = "serde1")]
string_serde_impl!(TokenId);

impl TokenId {
    /// Creates a new `TokenId`.
    pub fn build(foundry_id: FoundryId, token_tag: [u8; TOKEN_TAG_LENGTH]) -> Self {
        Self(
            foundry_id
                .as_ref()
                .iter()
                .chain(&token_tag)
                .copied()
                .collect::<Vec<u8>>()
                .try_into()
                // Safe to unwrap because we know the lengths.
                .unwrap(),
        )
    }
}
