// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::rand::bytes::rand_bytes_array;

use bee_message::signature::Ed25519Signature;

/// Generates a random [`Ed25519Signature`].
pub fn rand_ed25519_signature() -> Ed25519Signature {
    Ed25519Signature::new(rand_bytes_array(), rand_bytes_array())
}
