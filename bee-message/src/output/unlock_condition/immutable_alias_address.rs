// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    address::{Address, AliasAddress},
    Error,
};

use derive_more::From;

/// Defines the permanent [`AliasAddress`] that owns this output.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct ImmutableAliasAddressUnlockCondition(#[packable(verify_with = verify_alias_address)] Address);

impl ImmutableAliasAddressUnlockCondition {
    /// The [`UnlockCondition`](crate::output::UnlockCondition) kind of an [`ImmutableAliasAddressUnlockCondition`].
    pub const KIND: u8 = 6;

    /// Creates a new [`ImmutableAliasAddressUnlockCondition`].
    #[inline(always)]
    pub fn new(address: AliasAddress) -> Self {
        Self(Address::Alias(address))
    }

    /// Returns the address of a [`ImmutableAliasAddressUnlockCondition`].
    #[inline(always)]
    pub fn address(&self) -> &AliasAddress {
        // An ImmutableAliasAddressUnlockCondition must have an AliasAddress.
        if let Address::Alias(alias_address) = &self.0 {
            alias_address
        } else {
            // It has already been validated at construction that the address is an `AliasAddress`.
            unreachable!();
        }
    }
}

fn verify_alias_address<const VERIFY: bool>(address: &Address) -> Result<(), Error> {
    if VERIFY && !address.is_alias() {
        Err(Error::InvalidAddressKind(address.kind()))
    } else {
        Ok(())
    }
}
