// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;
use bee_test::rand::message::{rand_message_id, rand_message_ids};

#[test]
fn new_valid() {
    let first = rand_message_id();
    let others = rand_message_ids(7);
    let parents = Parents::new(first, others.clone()).unwrap();

    let parents_vec = parents.iter().copied().collect::<Vec<MessageId>>();

    assert_eq!(first, parents_vec[0]);
    assert_eq!(others, parents_vec[1..].to_vec());
}

#[test]
fn new_invalid_more_than_max() {
    let first = rand_message_id();
    let mut others = Vec::new();

    for _ in 0..8 {
        Parents::new(first, others.clone()).unwrap();
        others.push(rand_message_id())
    }

    assert!(matches!(
        Parents::new(first, others.clone()),
        Err(Error::InvalidParentsCount(9))
    ));
}

// TODO add packed_len test

#[test]
fn pack_unpack_valid() {
    let first = rand_message_id();
    let others = rand_message_ids(7);
    let parents_1 = Parents::new(first, others.clone()).unwrap();
    let parents_2 = Parents::unpack(&mut parents_1.pack_new().as_slice()).unwrap();

    assert_eq!(parents_1, parents_2);
}

#[test]
fn pack_unpack_invalid_less_than_min() {
    let bytes = vec![
        0, 227, 127, 245, 158, 220, 152, 191, 107, 27, 218, 187, 247, 227, 25, 215, 141, 92, 95, 138, 21, 98, 20, 83,
        206, 92, 26, 62, 9, 221, 81, 191, 4, 96, 54, 232, 50, 83, 49, 236, 80, 189, 251, 191, 192, 122, 206, 202, 209,
        145, 50, 168, 233, 176, 12, 164, 138, 207, 22, 96, 82, 189, 64, 188, 130,
    ];

    assert!(matches!(
        Parents::unpack(&mut bytes.as_slice()),
        Err(Error::InvalidParentsCount(0))
    ));
}

#[test]
fn pack_unpack_invalid_more_than_max() {
    let bytes = vec![
        9, 227, 127, 245, 158, 220, 152, 191, 107, 27, 218, 187, 247, 227, 25, 215, 141, 92, 95, 138, 21, 98, 20, 83,
        206, 92, 26, 62, 9, 221, 81, 191, 4, 96, 54, 232, 50, 83, 49, 236, 80, 189, 251, 191, 192, 122, 206, 202, 209,
        145, 50, 168, 233, 176, 12, 164, 138, 207, 22, 96, 82, 189, 64, 188, 130,
    ];

    assert!(matches!(
        Parents::unpack(&mut bytes.as_slice()),
        Err(Error::InvalidParentsCount(9))
    ));
}
