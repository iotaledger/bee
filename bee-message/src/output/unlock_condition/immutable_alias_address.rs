// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::address::AliasAddress;

use derive_more::From;

/// Defines the permanent [`AliasAddress`] that owns this output.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ImmutableAliasAddressUnlockCondition(AliasAddress);

impl ImmutableAliasAddressUnlockCondition {
    /// The [`UnlockCondition`](crate::output::UnlockCondition) kind of an [`ImmutableAliasAddressUnlockCondition`].
    pub const KIND: u8 = 6;

    /// Creates a new [`ImmutableAliasAddressUnlockCondition`].
    #[inline(always)]
    pub fn new(address: AliasAddress) -> Self {
        Self(address)
    }

    /// Returns the alias address of a [`ImmutableAliasAddressUnlockCondition`].
    #[inline(always)]
    pub fn address(&self) -> &AliasAddress {
        &self.0
    }
}
