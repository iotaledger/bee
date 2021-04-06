// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! A module that provides password utilities.

use argon2::{self, Config};
use rand::Rng;

/// Generates a salt to be used for password hashing.
pub fn generate_salt() -> [u8; 32] {
    rand::thread_rng().gen::<[u8; 32]>()
}

/// Hashes a password together with a salt.
pub fn password_hash(password: &[u8], salt: &[u8]) -> Result<Vec<u8>, String> {
    argon2::hash_raw(password, salt, &Config::default()).map_err(|e| e.to_string())
}

/// Verifies if a password/salt pair matches a password hash.
pub fn password_verify(password: &[u8], salt: &[u8], hash: &[u8]) -> Result<bool, String> {
    Ok(hash == password_hash(password, salt)?)
}
