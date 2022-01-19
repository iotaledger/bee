// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_gossip::Keypair;

use pem::Pem;

use std::{fs, path::Path};

const PRIVATE_KEY_TAG: &str = "PRIVATE KEY";

#[derive(Debug, thiserror::Error)]
pub enum PemFileError {
    #[error("reading the identity file failed: {0}")]
    Read(std::io::Error),
    #[error("writing the identity file failed: {0}")]
    Write(#[from] std::io::Error),
    #[error("could not parse PEM file")]
    Parse,
    #[error("could not decode keypair")]
    DecodeKeypair,
}

fn keypair_to_pem_entry(keypair: &Keypair) -> String {
    let pem_entry = Pem {
        tag: PRIVATE_KEY_TAG.into(),
        contents: keypair.encode().to_vec(),
    };
    pem::encode(&pem_entry)
}

fn pem_entry_to_keypair(pem_entry: String) -> Result<Keypair, PemFileError> {
    let mut pem = pem::parse(pem_entry).or(Err(PemFileError::Parse))?;
    Keypair::decode(&mut pem.contents).or(Err(PemFileError::DecodeKeypair))
}

pub fn read_keypair_from_pem_file<P: AsRef<Path>>(path: P) -> Result<Keypair, PemFileError> {
    match fs::read_to_string(path) {
        // Todo: Should we handle PEM files with multiple keys?
        Ok(pem_file) => pem_entry_to_keypair(pem_file),
        Err(e) => return Err(PemFileError::Read(e)),
    }
}

pub fn write_keypair_to_pem_file<P: AsRef<Path>>(path: P, keypair: &Keypair) -> Result<(), PemFileError> {
    fs::write(path, keypair_to_pem_entry(keypair)).map_err(PemFileError::Write)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pem_keypair_round_trip() {
        let keypair = Keypair::generate();
        let pem_entry = keypair_to_pem_entry(&keypair);
        let parsed_keypair = pem_entry_to_keypair(pem_entry).unwrap();
        assert_eq!(keypair.public(), parsed_keypair.public());
        assert_eq!(keypair.secret().as_ref(), parsed_keypair.secret().as_ref());
    }
}
