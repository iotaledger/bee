// Copyright 2020-2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_gossip::{Keypair, SecretKey};

use pkcs8::{
    der::Document, AlgorithmIdentifier, DecodePrivateKey, EncodePrivateKey, LineEnding, ObjectIdentifier, der::Decodable,
    PrivateKeyDocument, PrivateKeyInfo,
};

use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum PemFileError {
    #[error("reading the identity file failed: {0}")]
    Read(pkcs8::Error),
    #[error("writing the identity file failed: {0}")]
    Write(pkcs8::Error),
    #[error("could not parse PEM file")]
    Parse,
    #[error("could not decode keypair")]
    DecodeKeypair,
    #[error("found multiple PEM entries")]
    MultipleEntries,
    #[error("no PEM entries")]
    NoEntries,
}

// fn keypair_to_pem_entry(keypair: &Keypair) -> String {
//     let secret = keypair.secret();
//     let pk_info = PrivateKeyInfo::new(secret.as_ref()).unwrap();
//     pk_info.to_pem(LineEnding::default()).
// }

// fn pem_entry_to_keypair(pem_entry: String) -> Result<Keypair, PemFileError> {
//     let mut pkcs8_doc : PrivateKeyDocument = pem_entry.parse().or(Err(PemFileError::Parse))?;
//     // We only support a single PEM entry per file.
//     if pems.is_empty() {
//         Err(PemFileError::NoEntries)
//     } else if pems.len() == 1 {
//         // Safety: We just checked the length.
//         let secret = SecretKey::from_bytes(&mut pems[0].contents).or(Err(PemFileError::DecodeKeypair))?;
//         Ok(secret.into())
//     } else {
//         Err(PemFileError::MultipleEntries)
//     }
// }

// According to: https://crypto.stackexchange.com/questions/81023/oid-for-ed25519
const ED25519_DER_ENCODING: &str = "1.3.101.112"; // This is the one that golang/crypto uses.

// fn pem_entry_to_keypair(pem_entry: String) -> Result<Keypair, PemFileError> {
//     // let pem_doc = PrivateKeyDocument::from_pkcs8_(&pem_entry).or(Err(PemFileError::Parse))?;
//     let pem_doc = PrivateKeyDocument::from_pem(&pem_entry);
//     let pk_info = pem_doc.decode();
//     let mut private_key = pk_info.private_key.to_owned();
//     let secret = SecretKey::from_bytes(&mut private_key).unwrap();
//     Ok(secret.into())
// }

pub fn read_keypair_from_pem_file<P: AsRef<Path>>(path: P) -> Result<Keypair, PemFileError> {
    let pkcs8_doc = PrivateKeyDocument::read_pkcs8_pem_file(path).map_err(PemFileError::Read)?;
    let pk_info = pkcs8_doc.decode();
    let private_key = pk_info.private_key;
    println!("pk length: {}", private_key.len());
    println!("pk[0..2]: {:?}", &private_key[0..2]);
    let mut private_key = private_key[2..].to_owned();
    let secret = SecretKey::from_bytes(&mut private_key).unwrap();
    Ok(secret.into())
}

pub fn write_keypair_to_pem_file<P: AsRef<Path>>(path: P, keypair: &Keypair) -> Result<(), PemFileError> {
    let secret = keypair.secret();
    let algorithm = AlgorithmIdentifier {
        oid: ED25519_DER_ENCODING.parse::<ObjectIdentifier>().unwrap(),
        parameters: None,
    };
    let pk_info = PrivateKeyInfo::new(algorithm, secret.as_ref());
    let pk_doc = PrivateKeyDocument::try_from(pk_info).unwrap();
    pk_doc
        .write_pkcs8_pem_file(path, LineEnding::default())
        .map_err(PemFileError::Write)
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn pem_keypair_round_trip() {
//         let keypair = Keypair::generate();
//         let secret = keypair.secret();
//         let pem_entry = keypair_to_pem_entry(&secret.into());
//         let parsed_keypair = pem_entry_to_keypair(pem_entry).unwrap();
//         assert_eq!(keypair.public(), parsed_keypair.public());
//         assert_eq!(keypair.secret().as_ref(), parsed_keypair.secret().as_ref());
//     }

//     #[test]
//     fn no_entries() {
//         let pem_entries = "";
//         assert!(matches!(
//             pem_entry_to_keypair(pem_entries.into()),
//             Err(PemFileError::NoEntries)
//         ));
//     }

//#[test]
// fn single_entry() {
//     // This entry was generated with the Hornet node software.
//     let pem_entry = r#"-----BEGIN PRIVATE KEY-----
//             MC4CAQAwBQYDK2VwBCIEIF4Pg6pHREq+RQDpkaU/f3MkIFcUXSjM80yabh7P9q4r
// -----END PRIVATE KEY-----
//         "#;
//     let bytes = pem_entry_to_keypair(pem_entry.into()).unwrap().encode();
//     let hex_encoded = hex::encode(bytes);
//     assert_eq!(&hex_encoded, "12D3KooWKncxbqs3ddRvW54116WaWfYj2jLKC6wAqcGVqsuUXSs7");
// }

//     #[test]
//     fn multiple_entries() {
//         let pem_entries = r#"
//             -----BEGIN PRIVATE KEY-----
//             MC4CAQAwBQYDK2VwBCIEIGQ9cgUtF454R2VotN/W5VCcYWhnEuwOsYtsqKRoIeIz
//             -----END PRIVATE KEY-----
//             -----BEGIN PRIVATE KEY-----
//             MC4CAQAwBQYDK2VwBCIEIPf9H/AJTY1wy3PKu9B//fJxZ6QemTpmSAnPV8Gpt97f
//             -----END PRIVATE KEY-----
//         "#;
//         assert!(matches!(
//             pem_entry_to_keypair(pem_entries.into()),
//             Err(PemFileError::MultipleEntries)
//         ));
//     }
// }
