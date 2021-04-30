// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_test::rand::transaction::rand_transaction_id;
use bee_vote::{Opinion, statement::{Conflict, OpinionStatement}};

#[test]
fn packed_len() {
    let conflict = Conflict { 
        id: rand_transaction_id(),
        opinion: OpinionStatement {
            opinion: Opinion::Like,
            round: 1,
        },
    };

    let packed = conflict.pack_new();

    assert_eq!(packed.len(), conflict.packed_len());
    assert_eq!(packed.len(), 32 + 2);
}

#[test]
fn pack_unpack_valid() {
    let conflict = Conflict {
        id: rand_transaction_id(),
        opinion: OpinionStatement {
            opinion: Opinion::Like,
            round: 1,
        },
    };

    let packed = conflict.pack_new();

    assert_eq!(conflict, Packable::unpack(&mut packed.as_slice()).unwrap());
}