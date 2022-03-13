// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::output::FoundryId;

use packable::{packer::SlicePacker, Packable};

impl_id!(pub TokenTag, 12, "TODO.");

#[cfg(feature = "serde1")]
string_serde_impl!(TokenTag);

impl_id!(pub TokenId, 38, "TODO.");

#[cfg(feature = "serde1")]
string_serde_impl!(TokenId);

impl TokenId {
    /// Builds a new [`TokenId`] from its components.
    pub fn build(foundry_id: &FoundryId, token_tag: &TokenTag) -> Self {
        let mut bytes = [0u8; TokenId::LENGTH];
        let mut packer = SlicePacker::new(&mut bytes);

        // SAFETY: packing to an array of the correct length can't fail.
        foundry_id.pack(&mut packer).unwrap();
        token_tag.pack(&mut packer).unwrap();

        TokenId::new(bytes)
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
