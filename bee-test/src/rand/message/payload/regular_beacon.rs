// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{bytes::rand_bytes_array, number::rand_number};

use bee_message::payload::drng::BeaconPayload;

/// Generates a random [`BeaconPayload`].
pub fn rand_beacon_payload() -> BeaconPayload {
    BeaconPayload::builder()
        .with_instance_id(rand_number())
        .with_round(rand_number())
        .with_partial_public_key(rand_bytes_array())
        .with_partial_signature(rand_bytes_array())
        .finish()
        .unwrap()
}
