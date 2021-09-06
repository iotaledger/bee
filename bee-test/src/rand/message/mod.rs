// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

/// Module providing random address generation utilities.
pub mod address;
/// Module providing random asset balance generation utilities.
pub mod asset_balance;
/// Module providing random (FPC) conflict utilities.
pub mod conflict;
/// Module providing random input generation utilities.
pub mod input;
/// Module providing random output generation utilities.
pub mod output;
/// Module providing random parent generation utilities.
pub mod parents;
/// Module providing random payload generation utilities.
pub mod payload;
/// Module providing random salt generation utilities.
pub mod salt;
/// Module providing random signature generation utilities.
pub mod signature;
/// Module providing randoom (FPC) timestamp utilities.
pub mod timestamp;
/// Module providing random transaction generation utilities.
pub mod transaction;
/// Module providing random unlock block generation utilities.
pub mod unlock;

use crate::rand::bytes::rand_bytes_array;

use bee_message::MessageId;

/// Generates a random [`MessageId`].
pub fn rand_message_id() -> MessageId {
    MessageId::new(rand_bytes_array())
}
