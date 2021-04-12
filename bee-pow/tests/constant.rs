// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_pow::providers::{ConstantBuilder, NonceProvider, NonceProviderBuilder};
use bee_test::rand::bytes::rand_bytes;

#[test]
fn constant_provide() {
    let bytes = rand_bytes(256);
    let constant = ConstantBuilder::new().with_value(42).finish();
    let nonce = constant.nonce(&bytes[0..248], 4000f64, None).unwrap();

    assert_eq!(nonce, 42);
}
