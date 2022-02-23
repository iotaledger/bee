// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::FoundryId;

impl_id!(TokenTag, 12, "TODO.");

#[cfg(feature = "serde1")]
string_serde_impl!(TokenTag);

use alloc::vec::Vec;

impl_id!(TokenId, 38, "TODO.");

#[cfg(feature = "serde1")]
string_serde_impl!(TokenId);

impl TokenId {
    /// Builds a new [`TokenId`] from its components.
    pub fn build(foundry_id: FoundryId, token_tag: TokenTag) -> Self {
        Self(
            foundry_id
                .as_ref()
                .iter()
                .chain(token_tag.as_ref())
                .copied()
                .collect::<Vec<u8>>()
                .try_into()
                // SAFETY: the lengths are known.
                .unwrap(),
        )
    }

    /// Returns the [`FoundryId`] of the [`TokenId`].
    pub fn foundry_id(&self) -> FoundryId {
        // SAFETY: the lengths are known.
        FoundryId::new(self.0[0..FoundryId::LENGTH].try_into().unwrap())
    }

    /// Returns the [`TokenTag`] of the [`TokenId`].
    pub fn token_tag(&self) -> TokenTag {
        // SAFETY: the lengths are known.
        TokenTag::new(self.0[FoundryId::LENGTH..].try_into().unwrap())
    }
}
