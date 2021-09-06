// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{bytes::rand_bytes, number::rand_number_range};

use bee_message::payload::data::DataPayload;

/// Generates a random [`DataPayload`].
pub fn rand_data_payload() -> DataPayload {
    DataPayload::new(rand_bytes(rand_number_range(0..=255))).unwrap()
}
