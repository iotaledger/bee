// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod common;

use common::default_tangle;

use bee_test::rand::message::{metadata::rand_message_metadata, rand_message, rand_message_id};

#[test]
fn get_none() {
    let tangle = default_tangle();

    assert!(tangle.get(&rand_message_id()).is_none());
}

#[test]
fn insert_get() {
    let (message, metadata) = (rand_message(), rand_message_metadata());
    let message_id = message.id();

    let tangle = default_tangle();

    tangle.insert(message_id, message.clone(), metadata.clone());

    let message_data = tangle.get(&message_id).unwrap();

    assert_eq!(message_data.message(), &message);
    assert_eq!(message_data.metadata(), &metadata);
}
