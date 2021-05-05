// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    milestone::MilestoneIndex,
    parents::Parents,
    payload::{option_payload_pack, option_payload_packed_len, option_payload_unpack, Payload},
    Error,
};

use bee_common::{
    ord::is_unique_sorted,
    packable::{Packable, Read, Write},
};

use crypto::hashes::{blake2b::Blake2b256, Digest};

use alloc::vec::Vec;
use core::ops::RangeInclusive;

/// Range of allowed milestones public key numbers.
pub const MILESTONE_PUBLIC_KEY_COUNT_RANGE: RangeInclusive<usize> = 1..=255;
/// Length of a milestone merkle proof.
pub const MILESTONE_MERKLE_PROOF_LENGTH: usize = 32;
/// Length of a milestone public key.
pub const MILESTONE_PUBLIC_KEY_LENGTH: usize = 32;

/// Essence of a milestone payload.
/// This is the signed part of a milestone payload.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestonePayloadEssence {
    index: MilestoneIndex,
    timestamp: u64,
    parents: Parents,
    merkle_proof: [u8; MILESTONE_MERKLE_PROOF_LENGTH],
    next_pow_score: u32,
    next_pow_score_milestone_index: u32,
    public_keys: Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>,
    receipt: Option<Payload>,
}

impl MilestonePayloadEssence {
    /// Creates a new `MilestonePayloadEssence`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        index: MilestoneIndex,
        timestamp: u64,
        parents: Parents,
        merkle_proof: [u8; MILESTONE_MERKLE_PROOF_LENGTH],
        next_pow_score: u32,
        next_pow_score_milestone_index: u32,
        public_keys: Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>,
        receipt: Option<Payload>,
    ) -> Result<Self, Error> {
        if next_pow_score == 0 && next_pow_score_milestone_index != 0
            || next_pow_score != 0 && next_pow_score_milestone_index <= *index
        {
            return Err(Error::InvalidPowScoreValues(
                next_pow_score,
                next_pow_score_milestone_index,
            ));
        }

        if !MILESTONE_PUBLIC_KEY_COUNT_RANGE.contains(&public_keys.len()) {
            return Err(Error::MilestoneInvalidPublicKeyCount(public_keys.len()));
        }

        if !is_unique_sorted(public_keys.iter()) {
            return Err(Error::MilestonePublicKeysNotUniqueSorted);
        }

        if !matches!(receipt, None | Some(Payload::Receipt(_))) {
            // Safe to unwrap since it's known not to be None.
            return Err(Error::InvalidPayloadKind(receipt.unwrap().kind()));
        }

        Ok(Self {
            index,
            timestamp,
            parents,
            merkle_proof,
            next_pow_score,
            next_pow_score_milestone_index,
            public_keys,
            receipt,
        })
    }

    /// Returns the index of a `MilestonePayloadEssence`.
    pub fn index(&self) -> MilestoneIndex {
        self.index
    }

    /// Returns the timestamp of a `MilestonePayloadEssence`.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Returns the parents of a `MilestonePayloadEssence`.
    pub fn parents(&self) -> &Parents {
        &self.parents
    }

    /// Returns the merkle proof of a `MilestonePayloadEssence`.
    pub fn merkle_proof(&self) -> &[u8] {
        &self.merkle_proof
    }

    /// Returns the next proof of work score of a `MilestonePayloadEssence`.
    pub fn next_pow_score(&self) -> u32 {
        self.next_pow_score
    }

    /// Returns the newt proof of work index of a `MilestonePayloadEssence`.
    pub fn next_pow_score_milestone_index(&self) -> u32 {
        self.next_pow_score_milestone_index
    }

    /// Returns the public keys of a `MilestonePayloadEssence`.
    pub fn public_keys(&self) -> &Vec<[u8; MILESTONE_PUBLIC_KEY_LENGTH]> {
        &self.public_keys
    }

    /// Returns the optional receipt of a `MilestonePayloadEssence`.
    pub fn receipt(&self) -> Option<&Payload> {
        self.receipt.as_ref()
    }

    /// Hashes the `MilestonePayloadEssence to be signed.`
    pub fn hash(&self) -> [u8; 32] {
        Blake2b256::digest(&self.pack_new()).into()
    }
}

impl Packable for MilestonePayloadEssence {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.index.packed_len()
            + self.timestamp.packed_len()
            + self.parents.packed_len()
            + MILESTONE_MERKLE_PROOF_LENGTH
            + self.next_pow_score.packed_len()
            + self.next_pow_score_milestone_index.packed_len()
            + 0u8.packed_len()
            + self.public_keys.len() * MILESTONE_PUBLIC_KEY_LENGTH
            + option_payload_packed_len(self.receipt.as_ref())
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.index.pack(writer)?;
        self.timestamp.pack(writer)?;
        self.parents.pack(writer)?;
        self.merkle_proof.pack(writer)?;
        self.next_pow_score.pack(writer)?;
        self.next_pow_score_milestone_index.pack(writer)?;
        (self.public_keys.len() as u8).pack(writer)?;
        for public_key in &self.public_keys {
            public_key.pack(writer)?;
        }
        option_payload_pack(writer, self.receipt.as_ref())?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = MilestoneIndex::unpack_inner::<R, CHECK>(reader)?;
        let timestamp = u64::unpack_inner::<R, CHECK>(reader)?;
        let parents = Parents::unpack_inner::<R, CHECK>(reader)?;

        let merkle_proof = <[u8; MILESTONE_MERKLE_PROOF_LENGTH]>::unpack_inner::<R, CHECK>(reader)?;

        let next_pow_score = u32::unpack_inner::<R, CHECK>(reader)?;
        let next_pow_score_milestone_index = u32::unpack_inner::<R, CHECK>(reader)?;

        let public_keys_len = u8::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut public_keys = Vec::with_capacity(public_keys_len);
        for _ in 0..public_keys_len {
            public_keys.push(<[u8; MILESTONE_PUBLIC_KEY_LENGTH]>::unpack_inner::<R, CHECK>(reader)?);
        }

        let (_, receipt) = option_payload_unpack::<R, CHECK>(reader)?;

        Self::new(
            index,
            timestamp,
            parents,
            merkle_proof,
            next_pow_score,
            next_pow_score_milestone_index,
            public_keys,
            receipt,
        )
    }
}
