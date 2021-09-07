// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{bytes::rand_bytes, number::rand_number_range};

use bee_message::payload::indexation::{IndexationPayload, INDEXATION_INDEX_LENGTH_RANGE};

/// Generates a random [`IndexationPayload`].
pub fn rand_indexation_payload() -> IndexationPayload {
    IndexationPayload::new(
        rand_bytes(rand_number_range(INDEXATION_INDEX_LENGTH_RANGE) as usize),
        rand_bytes(rand_number_range(0..=255)),
    )
    .unwrap()
}
