// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_network::{Keypair, PeerId, PublicKey};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct P2pIdentityTool {}

pub fn exec(_tool: &P2pIdentityTool) {
    let keypair = Keypair::generate();
    let public = keypair.public();

    println!("Your p2p private key:\t{}", hex::encode(keypair.encode()));
    println!("Your p2p public key:\t{}", hex::encode(public.encode()));
    println!(
        "Your p2p PeerID:\t{}",
        PeerId::from_public_key(PublicKey::Ed25519(public))
    );
}
