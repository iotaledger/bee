// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{bytes::rand_bytes_array, number::rand_number};

use bee_message::payload::drng::CollectiveBeaconPayload;

/// Generates a random [`CollectiveBeaconPayload`].
pub fn rand_collective_beacon_payload() -> CollectiveBeaconPayload {
    CollectiveBeaconPayload::builder()
        .with_instance_id(rand_number())
        .with_round(rand_number())
        .with_prev_signature(rand_bytes_array())
        .with_signature(rand_bytes_array())
        .with_distributed_public_key(rand_bytes_array())
        .finish()
        .unwrap()
}
