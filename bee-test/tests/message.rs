// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_test::rand::message::rand_message;

#[test]
fn message() {
    for _ in 0..100 {
        rand_message();
    }
}
