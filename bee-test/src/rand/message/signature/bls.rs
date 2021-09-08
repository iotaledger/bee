// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::bytes::rand_bytes_array;

use bee_message::signature::BlsSignature;

/// Generates a random [`BlsSignature`].
pub fn rand_bls_signature() -> BlsSignature {
    BlsSignature::new(rand_bytes_array())
}
