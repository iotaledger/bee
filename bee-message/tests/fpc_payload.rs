// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::ops::Deref;

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
            Conflict::new(TransactionId::from(rand_bytes_array()), 0, 0),
            Conflict::new(TransactionId::from(rand_bytes_array()), 0, 1),
            Conflict::new(TransactionId::from(rand_bytes_array()), 1, 2),
        ]))
        .with_timestamps(Timestamps::new(vec![
            Timestamp::new(MessageId::from(rand_bytes_array()), 0, 0),
            Timestamp::new(MessageId::from(rand_bytes_array()), 0, 1),
            Timestamp::new(MessageId::from(rand_bytes_array()), 1, 2),
        ]))
        .finish();

    assert!(fpc.is_ok());
}

#[test]
fn conflict_accessors_eq() {
    let transaction_id = TransactionId::from(rand_bytes_array());
    let conflict = Conflict::new(transaction_id.clone(), 0, 0);  

    assert_eq!(*conflict.transaction_id(), transaction_id);
    assert_eq!(conflict.opinion(), 0);
    assert_eq!(conflict.round(), 0);
}

#[test]
fn timestamp_accessors_eq() {
    let message_id = MessageId::from(rand_bytes_array());
    let timestamp = Timestamp::new(message_id.clone(), 0, 0);  

    assert_eq!(*timestamp.message_id(), message_id);
    assert_eq!(timestamp.opinion(), 0);
    assert_eq!(timestamp.round(), 0);
}

#[test]
fn accessors_eq() {
    let conflicts = Conflicts::new(vec![
        Conflict::new(TransactionId::from(rand_bytes_array()), 0, 0),
        Conflict::new(TransactionId::from(rand_bytes_array()), 0, 1),
        Conflict::new(TransactionId::from(rand_bytes_array()), 1, 2),
    ]);

    let timestamps = Timestamps::new(vec![
        Timestamp::new(MessageId::from(rand_bytes_array()), 0, 0),
        Timestamp::new(MessageId::from(rand_bytes_array()), 0, 1),
        Timestamp::new(MessageId::from(rand_bytes_array()), 1, 2),
    ]);

    let fpc = FpcPayload::builder()
        .with_version(0)
        .with_conflicts(conflicts.clone())
        .with_timestamps(timestamps.clone())
        .finish()
        .unwrap();
    
    assert_eq!(fpc.version(), 0);
    assert_eq!(fpc.conflicts().cloned().collect::<Vec<Conflict>>(), conflicts.deref());
    assert_eq!(fpc.timestamps().cloned().collect::<Vec<Timestamp>>(), timestamps.deref());
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
            Conflict::new(TransactionId::from(rand_bytes_array()), 0, 0),
            Conflict::new(TransactionId::from(rand_bytes_array()), 0, 1),
            Conflict::new(TransactionId::from(rand_bytes_array()), 1, 2),
        ]))
        .with_timestamps(Timestamps::new(vec![
            Timestamp::new(MessageId::from(rand_bytes_array()), 0, 0),
            Timestamp::new(MessageId::from(rand_bytes_array()), 0, 1),
            Timestamp::new(MessageId::from(rand_bytes_array()), 1, 2),
        ]))
        .finish()
        .unwrap();

    let fpc_b = FpcPayload::unpack_from_slice(fpc_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(fpc_a, fpc_b);
}
