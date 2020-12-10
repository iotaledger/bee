// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_message::prelude::*;
use bee_pow::{
    miner::{Miner, MinerBuilder},
    provider::ProviderBuilder,
    score::compute_pow_score,
};

use core::str::FromStr;

const PARENT: &str = "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649";

#[test]
fn message_pow_default_provider() {
    let message = MessageBuilder::<Miner>::new()
        .with_network_id(0)
        .with_parent1(MessageId::from_str(PARENT).unwrap())
        .with_parent2(MessageId::from_str(PARENT).unwrap())
        .finish()
        .unwrap();

    let message_bytes = message.pack_new();
    let score = compute_pow_score(&message_bytes);

    assert!(score >= 4000f64);
}

#[test]
fn message_pow_provider() {
    let message = MessageBuilder::new()
        .with_network_id(0)
        .with_parent1(MessageId::from_str(PARENT).unwrap())
        .with_parent2(MessageId::from_str(PARENT).unwrap())
        .with_nonce_provider(MinerBuilder::new().with_num_workers(num_cpus::get()).finish(), 10000f64)
        .finish()
        .unwrap();

    let message_bytes = message.pack_new();
    let score = compute_pow_score(&message_bytes);

    assert!(score >= 10000f64);
}
