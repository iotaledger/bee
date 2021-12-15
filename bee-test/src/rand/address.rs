// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{bytes::rand_bytes_array, number::rand_number};

use bee_message::{
    address::{Address, AliasAddress, Ed25519Address, NftAddress},
    output::{AliasId, NftId},
};

/// Generates a random Ed25519 address.
pub fn rand_ed25519_address() -> Ed25519Address {
    Ed25519Address::new(rand_bytes_array())
}

/// Generates a random alias address.
pub fn rand_alias_address() -> AliasAddress {
    AliasAddress::new(AliasId::from(rand_bytes_array()))
}

/// Generates a random NFT address.
pub fn rand_nft_address() -> NftAddress {
    NftAddress::new(NftId::from(rand_bytes_array()))
}

/// Generates a random address.
pub fn rand_address() -> Address {
    #[allow(clippy::modulo_one)]
    match rand_number::<u64>() % 3 {
        0 => rand_ed25519_address().into(),
        1 => rand_alias_address().into(),
        2 => rand_nft_address().into(),
        _ => unreachable!(),
    }
}
