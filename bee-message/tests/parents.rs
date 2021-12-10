// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::{parents::Parents, Error, MessageId};
use bee_test::rand::message::{rand_message_id, rand_message_ids};

use std::ops::Deref;

#[test]
fn new_valid_iter() {
    let inner = rand_message_ids(8);
    let parents = Parents::new(inner.clone()).unwrap();

    let parents_vec = parents.iter().copied().collect::<Vec<MessageId>>();

    assert_eq!(inner, parents_vec[0..].to_vec());
}

#[test]
fn new_valid_deref() {
    let inner = rand_message_ids(8);
    let parents = Parents::new(inner.clone()).unwrap();

    assert_eq!(parents.deref(), &inner);
}

#[test]
fn new_invalid_more_than_max() {
    let mut inner = vec![rand_message_id()];

    for _ in 0..8 {
        Parents::new(inner.clone()).unwrap();
        inner.push(rand_message_id());
        inner.sort();
    }

    assert!(matches!(Parents::new(inner), Err(Error::InvalidParentsCount(9))));
}

#[test]
fn new_invalid_not_sorted() {
    let mut inner = rand_message_ids(8);
    inner.reverse();

    assert!(matches!(Parents::new(inner), Err(Error::ParentsNotUniqueSorted)));
}

#[test]
fn new_invalid_not_unique() {
    let mut inner = rand_message_ids(7);
    inner.push(*inner.last().unwrap());

    assert!(matches!(Parents::new(inner), Err(Error::ParentsNotUniqueSorted)));
}

#[test]
fn packed_len() {
    let parents = Parents::new(rand_message_ids(5)).unwrap();

    assert_eq!(parents.packed_len(), 1 + 5 * 32);
    assert_eq!(parents.pack_new().len(), 1 + 5 * 32);
}

#[test]
fn pack_unpack_valid() {
    let parents_1 = Parents::new(rand_message_ids(8)).unwrap();
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

#[test]
fn unpack_invalid_not_sorted() {
    let mut inner = rand_message_ids(8);
    inner.reverse();

    // Remove 8 byte vector length field and replace with 1 byte, to represent message parents.
    let mut packed = (8u8).pack_new();
    let mut packed_messages = inner.pack_new().split_at(core::mem::size_of::<u64>()).1.to_vec();
    packed.append(&mut packed_messages);

    let parents = Parents::unpack(&mut packed.as_slice());

    assert!(matches!(parents, Err(Error::ParentsNotUniqueSorted)));
}

#[test]
fn upnack_invalid_not_unique() {
    let mut inner = rand_message_ids(7);
    inner.push(*inner.last().unwrap());

    // Remove 8 byte vector length field and replace with 1 byte, to represent message parents.
    let mut packed = (8u8).pack_new();
    let mut packed_messages = inner.pack_new().split_at(std::mem::size_of::<u64>()).1.to_vec();
    packed.append(&mut packed_messages);

    let parents = Parents::unpack(&mut packed.as_slice());

    assert!(matches!(parents, Err(Error::ParentsNotUniqueSorted)));
}
