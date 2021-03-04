// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::{
    hashes::{blake2b::Blake2b256, Digest},
    signatures::ed25519::SecretKey,
};

use structopt::StructOpt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Ed25519Error {
    #[error("Invalid public key length: {0}")]
    InvalidPublicKeyLength(usize),
    #[error("Invalid public key hexadecimal: {0}")]
    InvalidPublicKeyHex(String),
    #[error("Secret generation failed")]
    SecretGenerationFailed,
}

#[derive(Clone, Debug, StructOpt)]
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
                return Err(Ed25519Error::InvalidPublicKeyLength(public.len()));
            }
            let bytes = hex::decode(public).map_err(|_| Ed25519Error::InvalidPublicKeyHex(public.clone()))?;
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
