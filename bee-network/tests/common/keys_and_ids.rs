// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p_core::PeerId;

use std::{iter::repeat, str::FromStr};

pub fn gen_constant_peer_id() -> PeerId {
    PeerId::from_str("12D3KooWJWEKvSFbben74C7H4YtKjhPMTDxd7gP7zxWSUEeF27st").unwrap()
}

#[cfg(feature = "full")]
pub mod full {
    use super::*;
    use libp2p::identity::ed25519::Keypair;

    pub fn gen_random_peer_id() -> PeerId {
        PeerId::from_public_key(libp2p_core::PublicKey::Ed25519(Keypair::generate().public()))
    }

    pub fn gen_deterministic_peer_id(gen: char) -> PeerId {
        let mut peer_id = String::from("12D3Koo");
        peer_id.push_str(&repeat(gen.to_ascii_uppercase()).take(45).collect::<String>()[..]);

        PeerId::from_str(&peer_id[..]).unwrap()
    }

    pub fn get_deterministic_keys(gen: char) -> Keypair {
        let hex_chars = ('0'..'9').chain('a'..'f').collect::<Vec<_>>();
        if !hex_chars.contains(&gen) {
            panic!("invalid generator");
        }

        let identity_kp = repeat(gen).take(128).collect::<String>();
        let mut hex_kp = hex::decode(identity_kp).expect("hex decode");

        Keypair::decode(&mut hex_kp[..]).expect("keypair decode")
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
