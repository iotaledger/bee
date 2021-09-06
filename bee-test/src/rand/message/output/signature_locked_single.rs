// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{message::address::rand_address, number::rand_number_range};

use bee_message::output::{SignatureLockedSingleOutput, SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT};

/// Generates a random [`SignatureLockedSingleOutput`].
pub fn rand_signature_locked_single_output() -> SignatureLockedSingleOutput {
    SignatureLockedSingleOutput::new(rand_address(), rand_number_range(SIGNATURE_LOCKED_SINGLE_OUTPUT_AMOUNT)).unwrap()
}
