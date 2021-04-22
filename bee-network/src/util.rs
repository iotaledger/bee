// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

#![cfg(test)]

use libp2p::identity::Keypair;
use libp2p_core::PeerId;

use std::str::FromStr;

pub fn gen_random_peer_id() -> PeerId {
    PeerId::from_public_key(Keypair::generate_ed25519().public())
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

pub fn gen_constant_peer_id() -> PeerId {
    PeerId::from_str("12D3KooWJWEKvSFbben74C7H4YtKjhPMTDxd7gP7zxWSUEeF27st").unwrap()
}

pub fn gen_random_keys() -> Keypair {
    Keypair::generate_ed25519()
}

pub fn gen_constant_keys() -> Keypair {
    todo!()
}

pub fn gen_deterministic_keys() -> Keypair {
    // let shex = std::iter::repeat(generator.to_ascii_uppercase())
    //     .take(120)
    //     .collect::<String>()[..];
    // let hex = hex::decode(&shex)
    todo!()
}
