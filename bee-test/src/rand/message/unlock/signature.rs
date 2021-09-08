// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::message::signature::rand_signature;

use bee_message::unlock::SignatureUnlock;

/// Generates a random [`SignatureUnlock`].
pub fn rand_signature_unlock() -> SignatureUnlock {
    SignatureUnlock::new(rand_signature())
}
