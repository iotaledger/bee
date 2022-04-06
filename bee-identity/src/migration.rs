// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Keypair;

const KEYPAIR_STR_LENGTH: usize = 128;

#[derive(Debug, thiserror::Error)]
pub(crate) enum IdentityMigrationError {
    #[error("hex decoding failed")]
    DecodeHex,
    #[error("keypair decoding failed")]
    DecodeKeypair,
    #[error("invalid keypair")]
    InvalidKeypair,
}

pub(crate) fn migrate_keypair(encoded: String) -> Result<Keypair, IdentityMigrationError> {
    if encoded.len() == KEYPAIR_STR_LENGTH {
        // Decode the keypair from hex.
        let mut decoded = [0u8; 64];
        hex::decode_to_slice(&encoded[..], &mut decoded).map_err(|_| IdentityMigrationError::DecodeHex)?;

        // Decode the keypair from bytes.
        Keypair::decode(&mut decoded).map_err(|_| IdentityMigrationError::DecodeKeypair)
    } else {
        Err(IdentityMigrationError::InvalidKeypair)
    }
}
