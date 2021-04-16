// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_pow::{
    providers::{
        miner::{MinerBuilder, MinerCancel},
        NonceProvider, NonceProviderBuilder,
    },
    score::compute_pow_score,
};
use bee_test::rand::bytes::rand_bytes;

#[test]
fn miner_provide() {
    let miner = MinerBuilder::new().with_num_workers(4).finish();
    let mut bytes = rand_bytes(256);

    let nonce = miner.nonce(&bytes[0..248], 4000f64).unwrap();
    bytes[248..].copy_from_slice(&nonce.to_le_bytes());

    assert!(compute_pow_score(&bytes) >= 4000f64);
}

#[test]
fn miner_abort() {
    let cancel = MinerCancel::new();
    let miner = MinerBuilder::new()
        .with_num_workers(4)
        .with_cancel(cancel.clone())
        .finish();
    let bytes = rand_bytes(256);

    let now = std::time::Instant::now();

    let handle = std::thread::spawn(move || miner.nonce(&bytes[0..248], 100000f64).unwrap());

    std::thread::sleep(std::time::Duration::from_secs(1));

    cancel.trigger();

    assert!(now.elapsed().as_secs() < 2);
    assert!(matches!(handle.join(), Ok(0)));
}
