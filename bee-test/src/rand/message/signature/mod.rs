// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod bls;
mod ed25519;

use crate::rand::number::rand_number;

pub use bls::rand_bls_signature;
pub use ed25519::rand_ed25519_signature;

use bee_message::signature::{BlsSignature, Ed25519Signature, Signature};

/// Generates a random [`Signature`].
pub fn rand_signature() -> Signature {
    match rand_number::<u8>() % 2 {
        Ed25519Signature::KIND => rand_ed25519_signature().into(),
        BlsSignature::KIND => rand_bls_signature().into(),
        _ => unreachable!(),
    }
}
