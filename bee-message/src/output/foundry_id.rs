// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::AliasAddress,
    output::{AliasId, TokenScheme},
};

impl_id!(FoundryId, 26, "Defines the unique identifier of a foundry.");

#[cfg(feature = "serde1")]
string_serde_impl!(FoundryId);

impl FoundryId {
    /// Createa a new `FoundryId`
    pub fn build(alias_id: AliasId, serial_number: [u8; 4], token_scheme: TokenScheme) -> Self {
        Self(
            [AliasAddress::KIND]
                .iter()
                .chain(alias_id.as_ref())
                .chain(&serial_number)
                .chain(&[token_scheme as u8])
                .copied()
                .collect::<Vec<u8>>()
                .try_into()
                // safe to unwrap because we know the lengths
                .unwrap(),
        )
    }
}
