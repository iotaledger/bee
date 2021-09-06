// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    message::{address::rand_address, asset_balance::rand_asset_balance},
    number::rand_number_range,
    vec::rand_vec,
};

use bee_message::output::SignatureLockedAssetOutput;

/// Generates a random [`SignatureLockedAssetOutput`] with an asset balance list of length 1..=10.
pub fn rand_signature_locked_asset_output() -> SignatureLockedAssetOutput {
    let asset_balances_length_range = 1..=10;

    SignatureLockedAssetOutput::new(
        rand_address(),
        rand_vec(rand_asset_balance, rand_number_range(asset_balances_length_range)),
    )
    .unwrap()
}
