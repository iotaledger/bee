// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::address::{rand_address, rand_alias_address};

use bee_message::{
    address::Address,
    output::{
        unlock_condition::{
            AddressUnlockCondition, GovernorAddressUnlockCondition, StateControllerAddressUnlockCondition,
        },
        AliasId,
    },
};

/// Generates a random [`StateControllerAddressUnlockCondition`].
pub fn rand_state_controller_address_unlock_condition_different_from(
    alias_id: &AliasId,
) -> StateControllerAddressUnlockCondition {
    let mut address = rand_address();
    if let Address::Alias(mut alias_address) = &mut address {
        while alias_address.id() == alias_id {
            alias_address = rand_alias_address();
        }
    }
    address.into()
}

/// Generates a random [`GovernorAddressUnlockCondition`].
pub fn rand_governor_address_unlock_condition_different_from(alias_id: &AliasId) -> GovernorAddressUnlockCondition {
    let mut address = rand_address();
    if let Address::Alias(mut alias_address) = &mut address {
        while alias_address.id() == alias_id {
            alias_address = rand_alias_address();
        }
    }
    address.into()
}

/// Generates a random [`AddressUnlockCondition`].
pub fn rand_address_unlock_condition() -> AddressUnlockCondition {
    rand_address().into()
}
