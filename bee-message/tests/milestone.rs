// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{milestone::Milestone, prelude::*};
use bee_packable::PackableExt;
use bee_test::rand::{milestone::rand_milestone, number::rand_number};

use std::str::FromStr;

const MESSAGE_ID: &str = "9e23e9fccb816af4ad355c27d904b6a6e88618e0bed1b640df3d4c19f4579bc9";

#[test]
fn debug_impl() {
    let milestone = Milestone::new(MessageId::from_str(MESSAGE_ID).unwrap(), 0);

    assert_eq!(
        format!("{:?}", milestone),
        "Milestone { message_id: MessageId(9e23e9fccb816af4ad355c27d904b6a6e88618e0bed1b640df3d4c19f4579bc9), timestamp: 0 }",
    );
}

#[test]
fn packed_len() {
    let milestone = rand_milestone();
    assert_eq!(milestone.packed_len(), milestone.pack_to_vec().len());
    assert_eq!(milestone.packed_len(), 32 + 8);
}

#[test]
fn pack_unpack() {
    let milestone = rand_milestone();
    let packed = milestone.pack_to_vec();

    assert_eq!(Milestone::unpack_verified(&mut packed.as_slice()).unwrap(), milestone);
}

#[test]
fn getters() {
    let message_id = MessageId::from_str(MESSAGE_ID).unwrap();
    let timestamp = rand_number::<u64>();
    let milestone = Milestone::new(message_id, timestamp);

    assert_eq!(message_id, *milestone.message_id());
    assert_eq!(timestamp, milestone.timestamp());
}
