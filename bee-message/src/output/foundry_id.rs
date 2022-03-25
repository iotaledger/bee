// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::AliasAddress,
    output::{AliasId, TokenScheme},
};

use packable::{packer::SlicePacker, Packable};

impl_id!(pub FoundryId, 26, "Defines the unique identifier of a foundry.");

#[cfg(feature = "serde1")]
string_serde_impl!(FoundryId);

impl FoundryId {
    /// Builds a new [`FoundryId`] from its components.
    pub fn build(alias_address: &AliasAddress, serial_number: u32, token_scheme: &TokenScheme) -> Self {
        let mut bytes = [0u8; FoundryId::LENGTH];
        let mut packer = SlicePacker::new(&mut bytes);

        // PANIC: packing to an array of the correct length can't fail.
        alias_address.pack(&mut packer).unwrap();
        serial_number.pack(&mut packer).unwrap();
        token_scheme.kind().pack(&mut packer).unwrap();

        FoundryId::new(bytes)
    }

    /// Returns the [`AliasId`] of the [`FoundryId`].
    pub fn alias_id(&self) -> AliasId {
        // PANIC: the lengths are known.
        AliasId::new(self.0[0..AliasId::LENGTH].try_into().unwrap())
    }

    /// Returns the serial number of the [`FoundryId`].
    pub fn serial_number(&self) -> u32 {
        // PANIC: the lengths are known.
        u32::from_le_bytes(
            self.0[AliasId::LENGTH..AliasId::LENGTH + core::mem::size_of::<u32>()]
                .try_into()
                .unwrap(),
        )
    }

    /// Returns the [`TokenScheme`] kind of the [`FoundryId`].
    pub fn token_scheme_kind(&self) -> u8 {
        // PANIC: the lengths are known.
        u8::from_le_bytes(
            self.0[AliasId::LENGTH + core::mem::size_of::<u32>()..AliasId::LENGTH + core::mem::size_of::<u32>() + 1]
                .try_into()
                .unwrap(),
        )
    }
}
