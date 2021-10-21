// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::hashes::sha::SHA256;
use hash32::{FnvHasher, Hasher as _};

pub(crate) use crypto::hashes::sha::SHA256_LEN;

/// Returns the sha256 hash of the given packet bytes in its byte representation.
pub(crate) fn sha256(bytes: &[u8]) -> [u8; SHA256_LEN] {
    let mut digest = [0; SHA256_LEN];
    SHA256(bytes, &mut digest);
    digest
}

pub(crate) fn fnv32(network_name: &impl AsRef<str>) -> u32 {
    // ```go
    // gossipServiceKeyHash := fnv.New32a()
    // gossipServiceKeyHash.Write([]byte(a.p2pServiceKey))
    // networkID := gossipServiceKeyHash.Sum32()
    // ```
    let mut hasher = FnvHasher::default();
    hasher.write(network_name.as_ref().as_bytes());
    hasher.finish()
}
