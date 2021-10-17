// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::hashes::sha::SHA256;

pub(crate) use crypto::hashes::sha::SHA256_LEN;

/// Returns the sha256 hash of the given packet bytes in its byte representation.
pub(crate) fn sha256(bytes: &[u8]) -> [u8; SHA256_LEN] {
    let mut digest = [0; SHA256_LEN];
    SHA256(bytes, &mut digest);
    digest
}
