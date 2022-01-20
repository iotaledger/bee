// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_gossip::{Keypair, SecretKey};

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
    #[error("found multiple PEM entries")]
    MultipleEntries,
    #[error("no PEM entries")]
    NoEntries,
}

fn keypair_to_pem_entry(keypair: &Keypair) -> String {
    let pem_entry = Pem {
        tag: PRIVATE_KEY_TAG.into(),
        contents: keypair.secret().as_ref().to_vec(),
    };
    pem::encode(&pem_entry)
}

fn pem_entry_to_keypair(pem_entry: String) -> Result<Keypair, PemFileError> {
    let mut pems = pem::parse_many(pem_entry).or(Err(PemFileError::Parse))?;
    // We only support a single PEM entry per file.
    if pems.is_empty() {
        Err(PemFileError::NoEntries)
    } else if pems.len() == 1 {
        // Safety: We just checked the length.
        let secret = SecretKey::from_bytes(&mut pems[0].contents).or(Err(PemFileError::DecodeKeypair))?;
        Ok(secret.into())
    } else {
        Err(PemFileError::MultipleEntries)
    }
}

pub fn read_keypair_from_pem_file<P: AsRef<Path>>(path: P) -> Result<Keypair, PemFileError> {
    match fs::read_to_string(path) {
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
        let secret = keypair.secret();
        let pem_entry = keypair_to_pem_entry(&secret.into());
        let parsed_keypair = pem_entry_to_keypair(pem_entry).unwrap();
        assert_eq!(keypair.public(), parsed_keypair.public());
        assert_eq!(keypair.secret().as_ref(), parsed_keypair.secret().as_ref());
    }

    #[test]
    fn no_entries() {
        let pem_entries = "";
        assert!(matches!(
            pem_entry_to_keypair(pem_entries.into()),
            Err(PemFileError::NoEntries)
        ));
    }

    #[test]
    fn single_entry() {
        // This entry was generated with the Hornet node software.
        let pem_entry = r#"
            -----BEGIN PRIVATE KEY-----
            MC4CAQAwBQYDK2VwBCIEIF4Pg6pHREq+RQDpkaU/f3MkIFcUXSjM80yabh7P9q4r
            -----END PRIVATE KEY-----
        "#;
        let bytes = pem_entry_to_keypair(pem_entry.into()).unwrap().encode();
        let hex_encoded = hex::encode(bytes);
        assert_eq!(&hex_encoded, "12D3KooWKncxbqs3ddRvW54116WaWfYj2jLKC6wAqcGVqsuUXSs7");
    }

    #[test]
    fn multiple_entries() {
        let pem_entries = r#"
            -----BEGIN PRIVATE KEY-----
            MC4CAQAwBQYDK2VwBCIEIGQ9cgUtF454R2VotN/W5VCcYWhnEuwOsYtsqKRoIeIz
            -----END PRIVATE KEY-----
            -----BEGIN PRIVATE KEY-----
            MC4CAQAwBQYDK2VwBCIEIPf9H/AJTY1wy3PKu9B//fJxZ6QemTpmSAnPV8Gpt97f
            -----END PRIVATE KEY-----
        "#;
        assert!(matches!(
            pem_entry_to_keypair(pem_entries.into()),
            Err(PemFileError::MultipleEntries)
        ));
    }
}
