// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::{
    payload::{
        fpc::{Conflict, FpcPayload, Opinion, Timestamp},
        transaction::TransactionId,
        MessagePayload,
    },
    MessageId,
};
use bee_packable::Packable;
use bee_test::rand::bytes::rand_bytes_array;

#[test]
fn kind() {
    assert_eq!(FpcPayload::KIND, 2);
}

#[test]
fn version() {
    assert_eq!(FpcPayload::VERSION, 0);
}

#[test]
fn new_valid() {
    let fpc = FpcPayload::builder()
        .with_conflicts(vec![
            Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Dislike, 0),
            Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Dislike, 1),
            Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Like, 2),
        ])
        .with_timestamps(vec![
            Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Dislike, 0),
            Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Dislike, 1),
            Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Like, 2),
        ])
        .finish();

    assert!(fpc.is_ok());
}

#[test]
fn conflict_accessors_eq() {
    let transaction_id = TransactionId::from(rand_bytes_array());
    let conflict = Conflict::new(transaction_id, Opinion::Dislike, 0);

    assert_eq!(conflict.transaction_id(), &transaction_id);
    assert_eq!(conflict.opinion(), Opinion::Dislike);
    assert_eq!(conflict.round(), 0);
}

#[test]
fn timestamp_accessors_eq() {
    let message_id = MessageId::from(rand_bytes_array());
    let timestamp = Timestamp::new(message_id, Opinion::Dislike, 0);

    assert_eq!(timestamp.message_id(), &message_id);
    assert_eq!(timestamp.opinion(), Opinion::Dislike);
    assert_eq!(timestamp.round(), 0);
}

#[test]
fn accessors_eq() {
    let conflicts = vec![
        Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Dislike, 0),
        Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Dislike, 1),
        Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Like, 2),
    ];

    let timestamps = vec![
        Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Dislike, 0),
        Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Dislike, 1),
        Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Like, 2),
    ];

    let fpc = FpcPayload::builder()
        .with_conflicts(conflicts.clone())
        .with_timestamps(timestamps.clone())
        .finish()
        .unwrap();

    assert_eq!(fpc.conflicts().cloned().collect::<Vec<Conflict>>(), conflicts);
    assert_eq!(fpc.timestamps().cloned().collect::<Vec<Timestamp>>(), timestamps);
}

#[test]
fn unpack_valid() {
    let mut bytes = vec![3, 0, 0, 0];

    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![1, 0]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![1, 1]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![2, 2]);

    bytes.extend(vec![3, 0, 0, 0]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![1, 0]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![1, 1]);
    bytes.extend(rand_bytes_array::<32>());
    bytes.extend(vec![2, 2]);

    let fpc = FpcPayload::unpack_from_slice(bytes);

    assert!(fpc.is_ok());
}

#[test]
fn packable_round_trip() {
    let fpc_a = FpcPayload::builder()
        .with_conflicts(vec![
            Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Dislike, 0),
            Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Dislike, 1),
            Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Like, 2),
        ])
        .with_timestamps(vec![
            Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Dislike, 0),
            Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Dislike, 1),
            Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Like, 2),
        ])
        .finish()
        .unwrap();

    let fpc_b = FpcPayload::unpack_from_slice(fpc_a.pack_to_vec().unwrap()).unwrap();

    assert_eq!(fpc_a, fpc_b);
}

#[test]
fn serde_round_trip() {
    let fpc_payload_1 = FpcPayload::builder()
        .with_conflicts(vec![
            Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Dislike, 0),
            Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Dislike, 1),
            Conflict::new(TransactionId::from(rand_bytes_array()), Opinion::Like, 2),
        ])
        .with_timestamps(vec![
            Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Dislike, 0),
            Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Dislike, 1),
            Timestamp::new(MessageId::from(rand_bytes_array()), Opinion::Like, 2),
        ])
        .finish()
        .unwrap();
    let json = serde_json::to_string(&fpc_payload_1).unwrap();
    let fpc_payload_2 = serde_json::from_str::<FpcPayload>(&json).unwrap();

    assert_eq!(fpc_payload_1, fpc_payload_2);
}
