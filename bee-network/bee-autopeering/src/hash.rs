// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::packet::MessageType;

use crypto::hashes::{
    sha::{Sha256, SHA256},
    Digest,
};
use hash32::{FnvHasher, Hasher as _};

pub(crate) use crypto::hashes::sha::SHA256_LEN;

/// Creates the SHA-256 hash of a generic byte sequence.
pub(crate) fn data_hash(data: &[u8]) -> [u8; SHA256_LEN] {
    let mut digest = [0; SHA256_LEN];
    SHA256(data, &mut digest);
    digest
}

/// Creates the SHA-256 hash of a particular network message.
pub(crate) fn message_hash(msg_type: MessageType, msg_data: &[u8]) -> [u8; SHA256_LEN] {
    let mut sha256 = Sha256::new();
    sha256.update([msg_type as u8]);
    sha256.update(msg_data);

    let mut digest = [0u8; SHA256_LEN];
    digest.copy_from_slice(&sha256.finalize());
    digest
}

/// Creates the 32bit fnv hash of the network name.
pub(crate) fn network_hash(network_name: impl AsRef<str>) -> u32 {
    let mut hasher = FnvHasher::default();
    hasher.write(network_name.as_ref().as_bytes());
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compare_message_hash(msg_type: MessageType, msg_data: &[u8]) -> [u8; SHA256_LEN] {
        let mut bytes = vec![0u8; msg_data.len() + 1];
        bytes[0] = msg_type as u8;
        bytes[1..].copy_from_slice(msg_data);

        let mut digest = [0; SHA256_LEN];
        SHA256(&bytes, &mut digest);
        digest
    }

    #[test]
    fn create_message_hash() {
        let msg_type = MessageType::DiscoveryRequest;
        let msg_data = [1u8; 150];

        assert_eq!(
            message_hash(msg_type, &msg_data),
            compare_message_hash(msg_type, &msg_data)
        );
    }
}
