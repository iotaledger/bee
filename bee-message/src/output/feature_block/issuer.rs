// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use derive_more::From;

use crate::address::Address;

/// Identifies the validated issuer of the UTXO state machine.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct IssuerFeatureBlock(Address);

impl IssuerFeatureBlock {
    /// The [`FeatureBlock`](crate::output::FeatureBlock) kind of an [`IssuerFeatureBlock`].
    pub const KIND: u8 = 1;

    /// Creates a new [`IssuerFeatureBlock`].
    #[inline(always)]
    pub fn new(address: Address) -> Self {
        Self(address)
    }

    /// Returns the issuer [`Address`].
    #[inline(always)]
    pub fn address(&self) -> &Address {
        &self.0
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use crate::address::dto::AddressDto;

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct IssuerFeatureBlockDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub address: AddressDto,
    }
}
