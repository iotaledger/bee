// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{
    message::{payload::rand_transaction_id, rand_message_id},
    number::{rand_number, rand_number_range},
    vec::rand_vec,
};

use bee_message::payload::fpc::{Conflict, FpcPayload, Opinion, Timestamp};

/// Generates a random [`Opinion`].
pub fn rand_opinion() -> Opinion {
    match rand_number::<u32>() % 3 {
        0 => Opinion::Like,
        1 => Opinion::Dislike,
        2 => Opinion::Unknown,
        _ => unreachable!(),
    }
}

/// Generates a random [`Conflict`].
pub fn rand_conflict() -> Conflict {
    Conflict::new(rand_transaction_id(), rand_opinion(), rand_number_range(0..=127))
}

/// Generates a random [`Timestamp`].
pub fn rand_timestamp() -> Timestamp {
    Timestamp::new(rand_message_id(), rand_opinion(), rand_number_range(0..=127))
}

/// Generates a random [`FpcPayload`].
pub fn rand_fpc_payload() -> FpcPayload {
    FpcPayload::builder()
        .with_conflicts(rand_vec(rand_conflict, rand_number_range(0..=10)))
        .with_timestamps(rand_vec(rand_timestamp, rand_number_range(0..=10)))
        .finish()
        .unwrap()
}
