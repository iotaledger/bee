// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::prelude::*;
use bee_packable::Packable;

use bee_test::rand::bytes::rand_bytes_array;

#[test]
fn kind() {
    assert_eq!(FpcPayload::KIND, 2);
}

#[test]
fn new_valid() {
    let fpc = FpcPayload::builder()
        .with_version(0)
        .with_conflicts(Conflicts::new(vec![
            Conflict::new(TransactionId::from(rand_bytes_array::<32>()), 0, 0),
            Conflict::new(TransactionId::from(rand_bytes_array::<32>()), 0, 1),
            Conflict::new(TransactionId::from(rand_bytes_array::<32>()), 1, 2),
        ]))
        .with_timestamps(Timestamps::new(vec![
            Timestamp::new(MessageId::from(rand_bytes_array::<32>()), 0, 0),
            Timestamp::new(MessageId::from(rand_bytes_array::<32>()), 0, 1),
            Timestamp::new(MessageId::from(rand_bytes_array::<32>()), 1, 2),
        ]))
        .finish();

    assert!(fpc.is_ok());
}

#[test]
fn unpack_valid() {
    let mut bytes = vec![0u8, 3, 0, 0, 0];

    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![0, 0]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![0, 1]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![1, 2]);

    bytes.extend(vec![3, 0, 0, 0]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![0, 0]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![0, 1]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![1, 2]);

    let fpc = FpcPayload::unpack_from_slice(bytes);

    assert!(fpc.is_ok());
}

#[test]
fn round_trip() {
    let fpc_a = FpcPayload::builder()
        .with_version(0)
        .with_conflicts(Conflicts::new(vec![
            Conflict::new(TransactionId::from(rand_bytes_array::<32>()), 0, 0),
            Conflict::new(TransactionId::from(rand_bytes_array::<32>()), 0, 1),
            Conflict::new(TransactionId::from(rand_bytes_array::<32>()), 1, 2),
        ]))
        .with_timestamps(Timestamps::new(vec![
            Timestamp::new(MessageId::from(rand_bytes_array::<32>()), 0, 0),
            Timestamp::new(MessageId::from(rand_bytes_array::<32>()), 0, 1),
            Timestamp::new(MessageId::from(rand_bytes_array::<32>()), 1, 2),
        ]))
        .finish()
        .unwrap();

    let fpc_b = FpcPayload::unpack_from_slice(fpc_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(fpc_a, fpc_b);
}
