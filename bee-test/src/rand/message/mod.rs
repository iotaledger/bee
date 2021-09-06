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

use crate::rand::bytes::rand_bytes_array;

use bee_message::MessageId;

/// Generates a random [`MessageId`].
pub fn rand_message_id() -> MessageId {
    MessageId::new(rand_bytes_array())
}
