// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::address::Address;

use derive_more::From;

/// Identifies the validated issuer of an output.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, bee_packable::Packable)]
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
