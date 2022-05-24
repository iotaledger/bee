// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use derive_more::From;

use crate::{
    address::{Address, AliasAddress},
    Error,
};

/// Defines the permanent [`AliasAddress`] that owns this output.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    pub fn address(&self) -> &Address {
        // An ImmutableAliasAddressUnlockCondition must have an AliasAddress.
        // It has already been validated at construction that the address is an `AliasAddress`.
        debug_assert!(&self.0.is_alias());
        &self.0
    }
}

fn verify_alias_address<const VERIFY: bool>(address: &Address) -> Result<(), Error> {
    if VERIFY && !address.is_alias() {
        Err(Error::InvalidAddressKind(address.kind()))
    } else {
        Ok(())
    }
}

#[cfg(feature = "dto")]
#[allow(missing_docs)]
pub mod dto {
    use serde::{Deserialize, Serialize};

    use crate::address::dto::AddressDto;

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct ImmutableAliasAddressUnlockConditionDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub address: AddressDto,
    }
}
