// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_pow::{
    providers::{MinerBuilder, NonceProvider, NonceProviderBuilder},
    score::compute_pow_score,
};
use bee_test::rand::bytes::rand_bytes;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

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
    let signal = Arc::new(AtomicBool::new(false));
    let miner = MinerBuilder::new()
        .with_num_workers(4)
        .with_signal(signal.clone())
        .finish();
    let bytes = rand_bytes(256);

    let now = std::time::Instant::now();

    std::thread::spawn(move || miner.nonce(&bytes[0..248], 100000f64).unwrap());

    std::thread::sleep(std::time::Duration::from_secs(1));

    signal.store(true, Ordering::Relaxed);

    assert!(now.elapsed().as_secs() < 2);
}
