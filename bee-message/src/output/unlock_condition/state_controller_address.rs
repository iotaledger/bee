// Copyright 2021-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::address::Address;

use derive_more::From;

/// Defines the State Controller Address that owns this output, that is, it can unlock it with the proper Unlock Block
/// in a transaction that state transitions the alias output.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, From, packable::Packable)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct StateControllerAddressUnlockCondition(Address);

impl StateControllerAddressUnlockCondition {
    /// The [`UnlockCondition`](crate::output::UnlockCondition) kind of an [`StateControllerAddressUnlockCondition`].
    pub const KIND: u8 = 4;

    /// Creates a new [`StateControllerAddressUnlockCondition`].
    #[inline(always)]
    pub fn new(address: Address) -> Self {
        Self(address)
    }

    /// Returns the address of a [`StateControllerAddressUnlockCondition`].
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
    pub struct StateControllerAddressUnlockConditionDto {
        #[serde(rename = "type")]
        pub kind: u8,
        pub address: AddressDto,
    }
}
