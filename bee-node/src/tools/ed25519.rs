// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::{
    ed25519::SecretKey,
    hashes::{blake2b::Blake2b256, Digest},
};

use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Ed25519Error {
    #[error("{0}")]
    InvalidPublicKey(String),
    #[error("Secret generation failed")]
    SecretGenerationFailed,
}

#[derive(Debug, StructOpt)]
pub enum Ed25519Tool {
    /// Generates an Ed25519 address from a public key.
    Address { public: String },
    /// Generates a pair of Ed25519 public/private keys.
    Keys,
}

pub fn exec(tool: &Ed25519Tool) -> Result<(), Ed25519Error> {
    match tool {
        Ed25519Tool::Address { public } => {
            if public.len() != 32 {
                return Err(Ed25519Error::InvalidPublicKey(public.clone()));
            }
            let bytes = hex::decode(public).map_err(|_| Ed25519Error::InvalidPublicKey(public.clone()))?;
            let hash = Blake2b256::digest(&bytes);

            println!("Your ed25519 address:\t{}", hex::encode(hash));
        }
        Ed25519Tool::Keys => {
            let private = SecretKey::generate().map_err(|_| Ed25519Error::SecretGenerationFailed)?;
            let public = private.public_key();

            println!("Your ed25519 private key:\t{}", hex::encode(private.to_le_bytes()));
            println!(
                "Your ed25519 public key:\t{}",
                hex::encode(public.to_compressed_bytes())
            );
        }
    }

    Ok(())
}
