// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod signature_locked_asset;
mod signature_locked_single;

use crate::rand::{
    message::payload::transaction::rand_transaction_id,
    number::{rand_number, rand_number_range},
};

pub use signature_locked_asset::rand_signature_locked_asset_output;
pub use signature_locked_single::rand_signature_locked_single_output;

use bee_message::output::{
    Output, OutputId, SignatureLockedAssetOutput, SignatureLockedSingleOutput, OUTPUT_INDEX_RANGE,
};

/// Generates a random [`OutputId`].
pub fn rand_output_id() -> OutputId {
    OutputId::new(rand_transaction_id(), rand_number_range(OUTPUT_INDEX_RANGE)).unwrap()
}

/// Generates a random [`Output`].
pub fn rand_output() -> Output {
    match rand_number::<u8>() % 2 {
        SignatureLockedSingleOutput::KIND => rand_signature_locked_single_output().into(),
        SignatureLockedAssetOutput::KIND => rand_signature_locked_asset_output().into(),
        _ => unreachable!(),
    }
}
