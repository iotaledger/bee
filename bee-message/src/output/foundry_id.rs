// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::AliasAddress,
    output::{AliasId, TokenScheme},
};

use alloc::vec::Vec;

impl_id!(FoundryId, 26, "Defines the unique identifier of a foundry.");

#[cfg(feature = "serde1")]
string_serde_impl!(FoundryId);

impl FoundryId {
    /// Builds a new [`FoundryId`] from its components.
    pub fn build(alias_id: AliasId, serial_number: u32, token_scheme: TokenScheme) -> Self {
        Self(
            [AliasAddress::KIND]
                .iter()
                .chain(alias_id.as_ref())
                .chain(&serial_number.to_le_bytes())
                .chain(&[token_scheme as u8])
                .copied()
                .collect::<Vec<u8>>()
                .try_into()
                // SAFETY: the lengths are known.
                .unwrap(),
        )
    }

    /// Returns the [`AliasId`] of the [`FoundryId`].
    pub fn alias_id(&self) -> AliasId {
        // SAFETY: the lengths are known.
        AliasId::new(self.0[0..AliasId::LENGTH].try_into().unwrap())
    }

    /// Returns the serial number of the [`FoundryId`].
    pub fn serial_number(&self) -> u32 {
        // SAFETY: the lengths are known.
        u32::from_le_bytes(
            self.0[AliasId::LENGTH..AliasId::LENGTH + core::mem::size_of::<u32>()]
                .try_into()
                .unwrap(),
        )
    }

    /// Returns the [`TokenScheme`] of the [`FoundryId`].
    pub fn token_scheme(&self) -> TokenScheme {
        // SAFETY: the lengths are known and the token scheme kind has to be valid.
        (*self.0.last().unwrap()).try_into().unwrap()
    }
}
