// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::{blake2b, ed25519::SecretKey};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Ed25519Tool {
    /// Generates an Ed25519 address from a public key.
    Address { public: String },
    /// Generates a pair of Ed25519 public/private keys.
    Keys,
}

pub fn exec(tool: &Ed25519Tool) {
    match tool {
        Ed25519Tool::Address { public } => {
            // TODO check size of public key
            let bytes = hex::decode(public).unwrap();
            let mut hash = [0u8; 32];
            blake2b::hash(&bytes, &mut hash);

            println!("Your ed25519 address:\t{}", hex::encode(hash));
        }
        Ed25519Tool::Keys => {
            let private = SecretKey::generate().unwrap();
            let public = private.public_key();

            println!("Your ed25519 private key:\t{}", hex::encode(private.to_le_bytes()));
            println!(
                "Your ed25519 public key:\t{}",
                hex::encode(public.to_compressed_bytes())
            );
        }
    }
}
