// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_gossip::{Keypair, PeerId, PublicKey};

use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct P2pIdentityTool {}

pub fn exec(_tool: &P2pIdentityTool) {
    let keypair = Keypair::generate();
    let public_key = keypair.public();

    println!("Your node identity:\t{}", hex::encode(keypair.encode()));
    println!("Your node public key:\t{}", hex::encode(public_key.encode()));
    println!(
        "Your libp2p peer identity:\t{}",
        PeerId::from_public_key(PublicKey::Ed25519(public_key))
    );
}
