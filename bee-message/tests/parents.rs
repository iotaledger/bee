// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;
use bee_test::rand::message::{rand_message_id, rand_message_ids};

#[test]
fn new_valid() {
    let inner = rand_message_ids(8);
    let parents = Parents::new(inner.clone()).unwrap();

    let parents_vec = parents.iter().copied().collect::<Vec<MessageId>>();

    assert_eq!(inner, parents_vec[0..].to_vec());
}

#[test]
fn new_invalid_more_than_max() {
    let mut inner = vec![rand_message_id()];

    for _ in 0..8 {
        Parents::new(inner.clone()).unwrap();
        inner.push(rand_message_id())
    }

    assert!(matches!(
        Parents::new(inner.clone()),
        Err(Error::InvalidParentsCount(9))
    ));
}

#[test]
fn packed_len() {
    assert_eq!(Parents::new(rand_message_ids(5)).unwrap().packed_len(), 1 + 5 * 32);
}

#[test]
fn pack_unpack_valid() {
    let inner = rand_message_ids(8);
    let parents_1 = Parents::new(inner.clone()).unwrap();
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
