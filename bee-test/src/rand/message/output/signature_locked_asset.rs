// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    bytes::rand_bytes_array,
    message::address::rand_address,
    number::{rand_number, rand_number_range},
    vec::rand_vec,
};

use bee_message::output::{AssetBalance, AssetId, SignatureLockedAssetOutput};

/// Generates a random [`AssetId`].
pub fn rand_asset_id() -> AssetId {
    AssetId::new(rand_bytes_array())
}

/// Generates a random [`AssetBalance`].
pub fn rand_asset_balance() -> AssetBalance {
    AssetBalance::new(rand_asset_id(), rand_number())
}

/// Generates a random [`SignatureLockedAssetOutput`] with an asset balance list of length 1..=10.
pub fn rand_signature_locked_asset_output() -> SignatureLockedAssetOutput {
    let asset_balances_length_range = 1..=10;

    SignatureLockedAssetOutput::new(
        rand_address(),
        rand_vec(rand_asset_balance, rand_number_range(asset_balances_length_range)),
    )
    .unwrap()
}
