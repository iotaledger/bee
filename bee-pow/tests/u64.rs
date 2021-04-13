// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_pow::providers::NonceProvider;
use bee_test::rand::bytes::rand_bytes;

#[test]
fn constant_provide() {
    let bytes = rand_bytes(256);
    let nonce_1 = 42;
    let nonce_2 = nonce_1.nonce(&bytes[0..248], 4000f64).unwrap();

    assert_eq!(nonce_1, nonce_2);
}
