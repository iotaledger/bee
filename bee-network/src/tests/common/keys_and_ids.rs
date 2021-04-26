// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::PeerId;

pub fn gen_constant_peer_id() -> PeerId {
    "12D3KooWJWEKvSFbben74C7H4YtKjhPMTDxd7gP7zxWSUEeF27st".parse().unwrap()
}

#[cfg(feature = "full")]
mod __full {

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
        let hex_chars = ('0'..'9').chain('a'..'f').collect::<Vec<_>>();
        let gen = gen.to_string();
        let gen2 = gen.to_string();
        let chars = gen2.chars();

        for c in gen.chars() {
            if !hex_chars.contains(&c) {
                panic!("invalid generator");
            }
        }

        let div = 64 / gen.len();
        let rem = 64 % gen.len();

        let identity_sk = repeat(gen)
            .take(div)
            .chain(chars.into_iter().map(Into::into).take(rem))
            .collect::<String>();

        let mut hex_sk = hex::decode(identity_sk).unwrap();
        let sk = SecretKey::from_bytes(&mut hex_sk).unwrap();
        sk.into()
    }

    pub fn get_constant_keys() -> Keypair {
        let identity_kp = "41dbc921b157fe001bcaf7f1f8b97f6eddf8f29e8888afc2ff089d544b9baf45bcd026f7900fd6efaa21958890bcfd05b7b738f724acc3bfa68ba3f33197aee1";

        let mut hex_kp = hex::decode(identity_kp).expect("hex decode");

        Keypair::decode(&mut hex_kp[..]).expect("keypair decode")
    }

    pub fn gen_random_keys() -> Keypair {
        Keypair::generate()
    }

    pub fn gen_constant_net_id() -> u64 {
        14379272398717627559_u64 // "testnet7"
    }
}

#[cfg(feature = "full")]
pub use __full::*;
