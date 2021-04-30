// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_test::rand::message::rand_message_id;
use bee_vote::{
    statement::{OpinionStatement, Timestamp},
    Opinion,
};

#[test]
fn packed_len() {
    let timestamp = Timestamp {
        id: rand_message_id(),
        opinion: OpinionStatement {
            opinion: Opinion::Like,
            round: 1,
        },
    };

    let packed = timestamp.pack_new();

    assert_eq!(packed.len(), timestamp.packed_len());
    assert_eq!(packed.len(), 32 + 2);
}

#[test]
fn pack_unpack_valid() {
    let timestamp = Timestamp {
        id: rand_message_id(),
        opinion: OpinionStatement {
            opinion: Opinion::Like,
            round: 1,
        },
    };

    let packed = timestamp.pack_new();

    assert_eq!(timestamp, Packable::unpack(&mut packed.as_slice()).unwrap());
}
