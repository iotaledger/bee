// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    bytes::rand_bytes,
    number::{rand_number, rand_number_range},
};

use bee_message::payload::drng::{DkgPayload, EncryptedDeal};

/// Generates a random [`DkgPayload`].
pub fn rand_dkg_payload() -> DkgPayload {
    DkgPayload::builder()
        .with_instance_id(rand_number())
        .with_from_index(rand_number())
        .with_to_index(rand_number())
        .with_deal(
            EncryptedDeal::builder()
                .with_dh_key(rand_bytes(rand_number_range(0..=127)))
                .with_nonce(rand_bytes(rand_number_range(0..=127)))
                .with_encrypted_share(rand_bytes(rand_number_range(0..=127)))
                .with_threshold(rand_number())
                .with_commitments(rand_bytes(rand_number_range(0..=127)))
                .finish()
                .unwrap(),
        )
        .finish()
        .unwrap()
}
