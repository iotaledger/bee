// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_crypto::ternary::{Hash, HASH_LENGTH};
use bee_ternary::Btrit;

#[test]
fn hash_weight() {
    for i in 0..20 {
        let mut hash = Hash::zeros();
        hash.as_trits_mut().set(HASH_LENGTH - i - 1, Btrit::PlusOne);
        assert_eq!(hash.weight(), i as u8);
    }
}
