// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;

pub fn gen_constant_peer_id() -> PeerId {
    "12D3KooWJWEKvSFbben74C7H4YtKjhPMTDxd7gP7zxWSUEeF27st".parse().unwrap()
}

#[cfg(feature = "full")]
mod full {

    use super::*;
    use libp2p::identity::{
        ed25519::{Keypair, SecretKey},
        PublicKey,
    };
    use std::iter::repeat;

    pub fn gen_random_peer_id() -> PeerId {
        PeerId::from_public_key(libp2p_core::PublicKey::Ed25519(Keypair::generate().public()))
    }

    pub fn gen_deterministic_peer_id(gen: impl ToString) -> PeerId {
        let keys = gen_deterministic_keys(gen);
        PeerId::from_public_key(PublicKey::Ed25519(keys.public()))
    }

    pub fn gen_deterministic_keys(gen: impl ToString) -> Keypair {
        let gen = gen.to_string();

        let div = 64 / gen.len();
        let rem = 64 % gen.len();

        let identity_sk = repeat(gen.clone())
            .take(div)
            .chain(gen.chars().map(Into::into).take(rem))
            .collect::<String>();

        // Panic:
        // The input consists only of valid hex chars and the length for the secret key
        // is also correct. Hence, the `unwrap`s are fine.
        let mut hex_sk = hex::decode(identity_sk).expect("invalid generated secret key");
        let sk = SecretKey::from_bytes(&mut hex_sk).unwrap();
        sk.into()
    }

    pub fn get_constant_keys() -> Keypair {
        let identity_kp = "41dbc921b157fe001bcaf7f1f8b97f6eddf8f29e8888afc2ff089d544b9baf45bcd026f7900fd6efaa21958890bcfd05b7b738f724acc3bfa68ba3f33197aee1";

        // Panic:
        // Unwraps below are fine because we checked `identity_kp` for its validity.
        let mut hex_kp = hex::decode(identity_kp).unwrap();

        Keypair::decode(&mut hex_kp[..]).unwrap()
    }

    pub fn gen_random_keys() -> Keypair {
        Keypair::generate()
    }

    pub fn gen_constant_net_id() -> u64 {
        14379272398717627559_u64 // "testnet7"
    }
}

#[cfg(feature = "full")]
pub use full::*;
