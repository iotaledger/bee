// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    message::{address::rand_address, asset_balance::rand_asset_balance, transaction::rand_transaction_id},
    number::{rand_number, rand_number_range},
    vec::vec_rand_length,
};

use bee_message::output::{
    Output, OutputId, SignatureLockedAssetOutput, SignatureLockedSingleOutput, OUTPUT_INDEX_RANGE,
    SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT,
};

/// Generates a random [`OutputId`].
pub fn rand_output_id() -> OutputId {
    OutputId::new(rand_transaction_id(), rand_number_range(OUTPUT_INDEX_RANGE)).unwrap()
}

/// Generates a random [`SignatureLockedAssetOutput`] with an asset balance list of length 1..=10.
pub fn rand_signature_locked_asset_output() -> SignatureLockedAssetOutput {
    let asset_balances_length_range = 1..=10;

    SignatureLockedAssetOutput::new(
        rand_address(),
        vec_rand_length(asset_balances_length_range, rand_asset_balance),
    )
    .unwrap()
}

/// Generates a random [`SignatureLockedSingleOutput`].
pub fn rand_signature_locked_single_output() -> SignatureLockedSingleOutput {
    SignatureLockedSingleOutput::new(rand_address(), rand_number_range(SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT)).unwrap()
}

/// Generates a random [`Output`].
pub fn rand_output() -> Output {
    match rand_number::<u8>() % 2 {
        SignatureLockedSingleOutput::KIND => rand_signature_locked_single_output().into(),
        SignatureLockedAssetOutput::KIND => rand_signature_locked_asset_output().into(),
        _ => unreachable!(),
    }
}
