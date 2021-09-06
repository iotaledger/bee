// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{message::rand_message_id, number::rand_number_range};

use bee_message::payload::fpc::Timestamp;

/// Generates a random [`Timestamp`].
pub fn rand_timestamp() -> Timestamp {
    Timestamp::new(rand_message_id(), rand_number_range(0..=2), rand_number_range(0..=127))
}
