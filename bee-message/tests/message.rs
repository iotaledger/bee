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
        .with_parents(vec![rand_message_id(), rand_message_id()])
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
        .with_parents(vec![rand_message_id(), rand_message_id()])
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
        .with_parents(vec![rand_message_id(), rand_message_id()])
        .with_nonce_provider(ConstantBuilder::new().with_value(42).finish(), 10000f64, None)
        .with_payload(
            IndexationPayload::new("42".to_owned(), &[0u8; MESSAGE_LENGTH_MAX])
                .unwrap()
                .into(),
        )
        .finish();

    assert!(matches!(res, Err(Error::InvalidMessageLength(len)) if len == MESSAGE_LENGTH_MAX + 97));
}
