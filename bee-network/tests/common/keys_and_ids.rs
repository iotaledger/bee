// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use libp2p_core::PeerId;

use std::str::FromStr;

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

    pub fn gen_deterministic_peer_id(generator: char) -> PeerId {
        let mut peer_id = String::from("12D3Koo");
        peer_id.push_str(
            &std::iter::repeat(generator.to_ascii_uppercase())
                .take(45)
                .collect::<String>()[..],
        );

        PeerId::from_str(&peer_id[..]).unwrap()
    }

    pub fn gen_random_keys() -> Keypair {
        Keypair::generate()
    }

    pub fn gen_constant_net_id() -> u64 {
        // "testnet7"
        14379272398717627559_u64
    }
}
