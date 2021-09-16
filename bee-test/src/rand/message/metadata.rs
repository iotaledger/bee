// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::{bool::rand_bool, bytes::rand_bytes_array, message::payload::rand_opinion, number::rand_number};

use bee_message::MessageMetadata;

/// Generates a random [`MessageMetadata`].
pub fn rand_message_metadata() -> MessageMetadata {
    let mut metadata = MessageMetadata::new(rand_number());

    metadata.flags_mut().set_solid(rand_bool());
    metadata.flags_mut().set_scheduled(rand_bool());
    metadata.flags_mut().set_booked(rand_bool());
    metadata.flags_mut().set_eligible(rand_bool());
    metadata.flags_mut().set_invalid(rand_bool());

    metadata.set_solidification_timestamp(rand_number());
    metadata.set_branch_id(rand_bytes_array());
    metadata.set_opinion(rand_opinion());

    metadata
}
