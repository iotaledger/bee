// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;
use bee_pow::{
    providers::{ConstantBuilder, Miner, MinerBuilder, ProviderBuilder},
    score::compute_pow_score,
};
use bee_test::rand::message::rand_message_id;

#[test]
fn pow_default_provider() {
    let message = MessageBuilder::<Miner>::new()
        .with_network_id(0)
        .with_parents(Parents::new(vec![rand_message_id(), rand_message_id()]).unwrap())
        .finish()
        .unwrap();

    let message_bytes = message.pack_new();
    let score = compute_pow_score(&message_bytes);

    assert!(score >= 4000f64);
}

#[test]
fn pow_provider() {
    let message = MessageBuilder::new()
        .with_network_id(0)
        .with_parents(Parents::new(vec![rand_message_id(), rand_message_id()]).unwrap())
        .with_nonce_provider(
            MinerBuilder::new().with_num_workers(num_cpus::get()).finish(),
            10000f64,
            None,
        )
        .finish()
        .unwrap();

    let message_bytes = message.pack_new();
    let score = compute_pow_score(&message_bytes);

    assert!(score >= 10000f64);
}

#[test]
fn invalid_length() {
    let res = MessageBuilder::new()
        .with_network_id(0)
        .with_parents(Parents::new(vec![rand_message_id(), rand_message_id()]).unwrap())
        .with_nonce_provider(ConstantBuilder::new().with_value(42).finish(), 10000f64, None)
        .with_payload(
            IndexationPayload::new(&[42], &[0u8; MESSAGE_LENGTH_MAX])
                .unwrap()
                .into(),
        )
        .finish();

    assert!(matches!(res, Err(Error::InvalidMessageLength(len)) if len == MESSAGE_LENGTH_MAX + 96));
}

#[test]
fn unpack_valid_no_remaining_bytes() {
    assert!(Message::unpack(
        &mut vec![
            42, 0, 0, 0, 0, 0, 0, 0, 2, 140, 28, 186, 52, 147, 145, 96, 9, 105, 89, 78, 139, 3, 71, 249, 97, 149, 190,
            63, 238, 168, 202, 82, 140, 227, 66, 173, 19, 110, 93, 117, 34, 225, 202, 251, 10, 156, 58, 144, 225, 54,
            79, 62, 38, 20, 121, 95, 90, 112, 109, 6, 166, 126, 145, 13, 62, 52, 68, 248, 135, 223, 119, 137, 13, 0, 0,
            0, 0, 21, 205, 91, 7, 0, 0, 0, 0,
        ]
        .as_slice()
    )
    .is_ok())
}

#[test]
fn unpack_invalid_remaining_bytes() {
    assert!(matches!(
        Message::unpack(
            &mut vec![
                42, 0, 0, 0, 0, 0, 0, 0, 2, 140, 28, 186, 52, 147, 145, 96, 9, 105, 89, 78, 139, 3, 71, 249, 97, 149,
                190, 63, 238, 168, 202, 82, 140, 227, 66, 173, 19, 110, 93, 117, 34, 225, 202, 251, 10, 156, 58, 144,
                225, 54, 79, 62, 38, 20, 121, 95, 90, 112, 109, 6, 166, 126, 145, 13, 62, 52, 68, 248, 135, 223, 119,
                137, 13, 0, 0, 0, 0, 21, 205, 91, 7, 0, 0, 0, 0, 42
            ]
            .as_slice()
        ),
        Err(Error::RemainingBytesAfterMessage)
    ))
}
