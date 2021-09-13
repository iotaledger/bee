// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Module providing random address generation utilities.
pub mod address;
/// Module providing random input generation utilities.
pub mod input;
/// Module providing random output generation utilities.
pub mod output;
/// Module providing random parent generation utilities.
pub mod parents;
/// Module providing random payload generation utilities.
pub mod payload;
/// Module providing random signature generation utilities.
pub mod signature;
/// Module providing random unlock block generation utilities.
pub mod unlock;

use crate::rand::{bool::rand_bool, bytes::rand_bytes_array, number::rand_number};

use bee_message::{Message, MessageBuilder, MessageId};

/// Generates a random [`MessageId`].
pub fn rand_message_id() -> MessageId {
    MessageId::new(rand_bytes_array())
}

/// Generates a random [`Message`].
pub fn rand_message() -> Message {
    let mut builder = MessageBuilder::new()
        .with_parents(parents::rand_parents())
        .with_issuer_public_key(rand_bytes_array())
        .with_issue_timestamp(rand_number())
        .with_sequence_number(rand_number());

    if rand_bool() {
        builder = builder.with_payload(payload::rand_payload());
    }

    builder
        .with_nonce(rand_number())
        .with_signature(rand_bytes_array())
        .finish()
        .unwrap()
}
