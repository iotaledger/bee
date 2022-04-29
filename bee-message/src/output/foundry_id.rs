// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use packable::{packer::SlicePacker, Packable};

use crate::{
    address::{Address, AliasAddress},
    output::AliasId,
};

impl_id!(pub FoundryId, 38, "Defines the unique identifier of a foundry.");

#[cfg(feature = "serde")]
string_serde_impl!(FoundryId);

impl FoundryId {
    /// Builds a new [`FoundryId`] from its components.
    pub fn build(alias_address: &AliasAddress, serial_number: u32, token_scheme_kind: u8) -> Self {
        let mut bytes = [0u8; FoundryId::LENGTH];
        let mut packer = SlicePacker::new(&mut bytes);

        // PANIC: packing to an array of the correct length can't fail.
        Address::Alias(*alias_address).pack(&mut packer).unwrap();
        serial_number.pack(&mut packer).unwrap();
        token_scheme_kind.pack(&mut packer).unwrap();

        FoundryId::new(bytes)
    }

    /// Returns the [`AliasAddress`] of the [`FoundryId`].
    pub fn alias_address(&self) -> AliasAddress {
        // PANIC: the lengths are known.
        AliasAddress::from(AliasId::new(self.0[1..AliasId::LENGTH + 1].try_into().unwrap()))
    }

    /// Returns the serial number of the [`FoundryId`].
    pub fn serial_number(&self) -> u32 {
        // PANIC: the lengths are known.
        u32::from_le_bytes(
            self.0[AliasId::LENGTH + 1..AliasId::LENGTH + 1 + core::mem::size_of::<u32>()]
                .try_into()
                .unwrap(),
        )
    }

    /// Returns the [`TokenScheme`](crate::output::TokenScheme) kind of the [`FoundryId`].
    pub fn token_scheme_kind(&self) -> u8 {
        // PANIC: the lengths are known.
        *self.0.last().unwrap()
    }
}
