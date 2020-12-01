// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::bytes::random_bytes_32;

use bee_message::payload::transaction::TransactionId;

pub fn random_transaction_id() -> TransactionId {
    TransactionId::new(random_bytes_32())
}
