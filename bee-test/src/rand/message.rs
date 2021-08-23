// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::bytes::rand_bytes_array;

use bee_message::MessageId;

/// Generates a random [`MessageId`].
pub fn rand_message_id() -> MessageId {
    MessageId::new(rand_bytes_array())
}
