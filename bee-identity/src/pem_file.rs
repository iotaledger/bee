// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Manages persistence of identities using the PEM file format.

use std::{fs, path::Path};

use ed25519::KeypairBytes;
use pkcs8::{DecodePrivateKey, EncodePrivateKey, LineEnding};

use crate::ed25519::{Keypair, SecretKey};

/// PEM file errors.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum PemFileError {
    #[error("reading the identity file failed: {0}")]
    Read(std::io::Error),
    #[error("writing the identity file failed: {0}")]
    Write(std::io::Error),
    #[error("could not parse PEM file")]
    Parse,
    #[error("could not decode keypair")]
    DecodeKeypair,
}

fn pem_entry_to_keypair(pem_entry: String) -> Result<Keypair, PemFileError> {
    let KeypairBytes { mut secret_key, .. } = KeypairBytes::from_pkcs8_pem(&pem_entry).or(Err(PemFileError::Parse))?;
    let secret = SecretKey::from_bytes(&mut secret_key).or(Err(PemFileError::DecodeKeypair))?;
    Ok(secret.into())
}

fn keypair_to_pem_entry(keypair: &Keypair) -> Result<String, PemFileError> {
    let secret_key: [u8; 32] = keypair
        .secret()
        .as_ref()
        .try_into()
        .or(Err(PemFileError::DecodeKeypair))?;
    let keypair_bytes = KeypairBytes {
        secret_key,
        public_key: None,
    };
    match keypair_bytes.to_pkcs8_pem(LineEnding::default()) {
        Ok(zeroize_string) => Ok(zeroize_string.to_string()),
        Err(_) => Err(PemFileError::DecodeKeypair),
    }
}

pub(crate) fn read_keypair_from_pem_file<P: AsRef<Path>>(path: P) -> Result<Keypair, PemFileError> {
    match fs::read_to_string(path) {
        Ok(pem_file) => pem_entry_to_keypair(pem_file),
        Err(e) => Err(PemFileError::Read(e)),
    }
}

pub(crate) fn write_keypair_to_pem_file<P: AsRef<Path>>(path: P, keypair: &Keypair) -> Result<(), PemFileError> {
    fs::write(path, keypair_to_pem_entry(keypair)?).map_err(PemFileError::Write)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn pem_keypair_round_trip() {
        let keypair = Keypair::generate();
        let secret = keypair.secret();
        let pem_entry = keypair_to_pem_entry(&secret.into()).unwrap();
        let parsed_keypair = pem_entry_to_keypair(pem_entry).unwrap();
        assert_eq!(keypair.public(), parsed_keypair.public());
        assert_eq!(keypair.secret().as_ref(), parsed_keypair.secret().as_ref());
    }

    #[test]
    fn no_entries() {
        let pem_entries = "";
        assert!(matches!(
            pem_entry_to_keypair(pem_entries.into()),
            Err(PemFileError::Parse)
        ));
    }

    #[test]
    fn single_entry() {
        let pem_entry = concat!(
            "-----BEGIN PRIVATE KEY-----\n",
            "MC4CAQAwBQYDK2VwBCIEIPQ8j9xL2WvxWk2Z/sCocRxywwWAfEgvXxcrSSfX9tUH\n",
            "-----END PRIVATE KEY-----\n",
        );
        let keypair = pem_entry_to_keypair(pem_entry.into()).unwrap();
        let mut decoded = [0u8; 64];
        hex::decode_to_slice("f43c8fdc4bd96bf15a4d99fec0a8711c72c305807c482f5f172b4927d7f6d507f3eef70378022bd42fe0cdb799a2b909d42eace03da33b63c4c32c695a9729c2", &mut decoded).unwrap();
        let parsed = Keypair::decode(&mut decoded).unwrap();
        assert_eq!(keypair.secret().as_ref(), parsed.secret().as_ref());
    }

    #[test]
    fn multiple_entries() {
        let pem_entries = concat!(
            "-----BEGIN PRIVATE KEY-----\n",
            "MC4CAQAwBQYDK2VwBCIEIGQ9cgUtF454R2VotN/W5VCcYWhnEuwOsYtsqKRoIeIz\n",
            "-----END PRIVATE KEY-----\n",
            "-----BEGIN PRIVATE KEY-----\n",
            "MC4CAQAwBQYDK2VwBCIEIPf9H/AJTY1wy3PKu9B//fJxZ6QemTpmSAnPV8Gpt97f\n",
            "-----END PRIVATE KEY-----\n",
        );
        assert!(matches!(
            pem_entry_to_keypair(pem_entries.into()),
            Err(PemFileError::Parse)
        ));
    }
}
