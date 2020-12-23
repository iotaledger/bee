// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::bytes::rand_bytes_32;

use bee_message::payload::transaction::TransactionId;

pub fn rand_transaction_id() -> TransactionId {
    TransactionId::new(rand_bytes_32())
}
