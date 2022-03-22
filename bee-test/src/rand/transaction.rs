// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::payload::transaction::TransactionId;

use crate::rand::bytes::rand_bytes_32;

/// Generates a random transaction id.
pub fn rand_transaction_id() -> TransactionId {
    TransactionId::new(rand_bytes_32())
}
