// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_message::MessageId;

use crypto::hashes::{Digest, Output};

use std::marker::PhantomData;

/// Leaf domain separation prefix.
const LEAF_HASH_PREFIX: u8 = 0x00;
/// Node domain separation prefix.
const NODE_HASH_PREFIX: u8 = 0x01;

// TODO hasher re-creation or finalize_reset + hasher field ?

/// Computes the largest power of two inferior to `n`.
#[inline]
fn largest_power_of_two(n: u32) -> usize {
    1 << (32 - n.leading_zeros() - 1)
}

/// A Merkle hasher based on a digest function.
pub(crate) struct MerkleHasher<D> {
    marker: PhantomData<D>,
}

impl<D: Default + Digest> MerkleHasher<D> {
    /// Creates a new Merkle hasher.
    pub(crate) fn new() -> Self {
        Self { marker: PhantomData }
    }

    /// Returns the digest of the empty hash.
    fn empty(&mut self) -> Output<D> {
        D::digest(&[])
    }

    /// Returns the digest of a Merkle leaf.
    fn leaf(&mut self, message_id: MessageId) -> Output<D> {
        let mut hasher = D::default();

        hasher.update([LEAF_HASH_PREFIX]);
        hasher.update(message_id);
        hasher.finalize()
    }

    /// Returns the digest of a Merkle node.
    fn node(&mut self, message_ids: &[MessageId]) -> Output<D> {
        let mut hasher = D::default();
        let (left, right) = message_ids.split_at(largest_power_of_two(message_ids.len() as u32 - 1));

        hasher.update([NODE_HASH_PREFIX]);
        hasher.update(self.digest_inner(left));
        hasher.update(self.digest_inner(right));
        hasher.finalize()
    }

    /// Returns the digest of a list of hashes as an `Output<D>`.
    fn digest_inner(&mut self, message_ids: &[MessageId]) -> Output<D> {
        match message_ids.len() {
            0 => self.empty(),
            1 => self.leaf(message_ids[0]),
            _ => self.node(message_ids),
        }
    }

    /// Returns the digest of a list of hashes as a `Vec<u8>`.
    pub(crate) fn digest(&mut self, message_ids: &[MessageId]) -> Vec<u8> {
        self.digest_inner(message_ids).to_vec()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use crypto::hashes::blake2b::Blake2b256;

    use std::str::FromStr;

    #[test]
    fn tree() {
        let mut hashes = Vec::new();

        for hash in [
            "52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649",
            "81855ad8681d0d86d1e91e00167939cb6694d2c422acd208a0072939487f6999",
            "eb9d18a44784045d87f3c67cf22746e995af5a25367951baa2ff6cd471c483f1",
            "5fb90badb37c5821b6d95526a41a9504680b4e7c8b763a1b1d49d4955c848621",
            "6325253fec738dd7a9e28bf921119c160f0702448615bbda08313f6a8eb668d2",
            "0bf5059875921e668a5bdf2c7fc4844592d2572bcd0668d2d6c52f5054e2d083",
            "6bf84c7174cb7476364cc3dbd968b0f7172ed85794bb358b0c3b525da1786f9f",
        ]
        .iter()
        {
            hashes.push(MessageId::from_str(hash).unwrap());
        }

        let hash = MerkleHasher::<Blake2b256>::new().digest(&hashes);

        assert_eq!(
            hex::encode(hash),
            "bf67ce7ba23e8c0951b5abaec4f5524360d2c26d971ff226d3359fa70cdb0beb"
        )
    }
}
