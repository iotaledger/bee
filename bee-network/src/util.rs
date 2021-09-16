// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::signatures::ed25519;

/// Converts an ED25519 public key into its 'base58' string representation.
#[allow(dead_code)]
pub fn to_public_key_string(pk: &ed25519::PublicKey) -> String {
    bs58::encode(pk.as_ref()).into_string()
}

/// Creates an ED25519 public key from its 'base58' string representation.
pub fn from_public_key_string(pk: &str) -> ed25519::PublicKey {
    let bytes = bs58::decode(pk).into_vec().expect("error decoding public key string");

    if bytes.len() != 32 {
        panic!("invalid public key string");
    }

    let mut pk = [0u8; 32];
    pk.copy_from_slice(&bytes[..32]);

    ed25519::PublicKey::try_from_bytes(pk).expect("error creating public key from bytes")
}
