// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    milestone::MilestoneIndex,
    parent::Parents,
    payload::{option_payload_pack, option_payload_packed_len, option_payload_unpack, OptionalPayload, Payload},
    Error,
};

use bee_common::{
    ord::is_unique_sorted,
    packable::{Read, Write},
};
use bee_packable::{
    bounded::BoundedU8,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::VecPrefix,
    unpacker::Unpacker,
};

use crypto::hashes::{blake2b::Blake2b256, Digest};

use alloc::vec::Vec;
use core::ops::RangeInclusive;

pub(crate) type PublicKeyCount = BoundedU8<
    { *MilestoneEssence::PUBLIC_KEY_COUNT_RANGE.start() },
    { *MilestoneEssence::PUBLIC_KEY_COUNT_RANGE.end() },
>;

/// Essence of a milestone payload.
/// This is the signed part of a milestone payload.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde1", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestoneEssence {
    index: MilestoneIndex,
    timestamp: u64,
    parents: Parents,
    merkle_proof: [u8; MilestoneEssence::MERKLE_PROOF_LENGTH],
    next_pow_score: u32,
    next_pow_score_milestone_index: u32,
    public_keys: VecPrefix<[u8; MilestoneEssence::PUBLIC_KEY_LENGTH], PublicKeyCount>,
    receipt: OptionalPayload,
}

impl MilestoneEssence {
    /// Length of a milestone merkle proof.
    pub const MERKLE_PROOF_LENGTH: usize = 32;
    /// Range of allowed milestones public key numbers.
    pub const PUBLIC_KEY_COUNT_RANGE: RangeInclusive<u8> = 1..=255;
    /// Length of a milestone public key.
    pub const PUBLIC_KEY_LENGTH: usize = 32;

    /// Creates a new [`MilestoneEssence`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        index: MilestoneIndex,
        timestamp: u64,
        parents: Parents,
        merkle_proof: [u8; MilestoneEssence::MERKLE_PROOF_LENGTH],
        next_pow_score: u32,
        next_pow_score_milestone_index: u32,
        public_keys: Vec<[u8; MilestoneEssence::PUBLIC_KEY_LENGTH]>,
        receipt: Option<Payload>,
    ) -> Result<Self, Error> {
        let public_keys = VecPrefix::<[u8; MilestoneEssence::PUBLIC_KEY_LENGTH], PublicKeyCount>::try_from(public_keys)
            .map_err(Error::MilestoneInvalidPublicKeyCount)?;

        Self::from_vec_prefix(
            index,
            timestamp,
            parents,
            merkle_proof,
            next_pow_score,
            next_pow_score_milestone_index,
            public_keys,
            receipt.into(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn from_vec_prefix(
        index: MilestoneIndex,
        timestamp: u64,
        parents: Parents,
        merkle_proof: [u8; MilestoneEssence::MERKLE_PROOF_LENGTH],
        next_pow_score: u32,
        next_pow_score_milestone_index: u32,
        public_keys: VecPrefix<[u8; MilestoneEssence::PUBLIC_KEY_LENGTH], PublicKeyCount>,
        receipt: OptionalPayload,
    ) -> Result<Self, Error> {
        if next_pow_score == 0 && next_pow_score_milestone_index != 0
            || next_pow_score != 0 && next_pow_score_milestone_index <= *index
        {
            return Err(Error::InvalidPowScoreValues {
                nps: next_pow_score,
                npsmi: next_pow_score_milestone_index,
            });
        }

        if !is_unique_sorted(public_keys.iter()) {
            return Err(Error::MilestonePublicKeysNotUniqueSorted);
        }

        if !matches!(*receipt, None | Some(Payload::Receipt(_))) {
            // Safe to unwrap since it's known not to be None.
            return Err(Error::InvalidPayloadKind(
                Into::<Option<Payload>>::into(receipt).unwrap().kind(),
            ));
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

    /// Returns the index of a [`MilestoneEssence`].
    pub fn index(&self) -> MilestoneIndex {
        self.index
    }

    /// Returns the timestamp of a [`MilestoneEssence`].
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Returns the parents of a [`MilestoneEssence`].
    pub fn parents(&self) -> &Parents {
        &self.parents
    }

    /// Returns the merkle proof of a [`MilestoneEssence`].
    pub fn merkle_proof(&self) -> &[u8] {
        &self.merkle_proof
    }

    /// Returns the next proof of work score of a [`MilestoneEssence`].
    pub fn next_pow_score(&self) -> u32 {
        self.next_pow_score
    }

    /// Returns the newt proof of work index of a [`MilestoneEssence`].
    pub fn next_pow_score_milestone_index(&self) -> u32 {
        self.next_pow_score_milestone_index
    }

    /// Returns the public keys of a [`MilestoneEssence`].
    pub fn public_keys(&self) -> &Vec<[u8; MilestoneEssence::PUBLIC_KEY_LENGTH]> {
        &self.public_keys
    }

    /// Returns the optional receipt of a [`MilestoneEssence`].
    pub fn receipt(&self) -> Option<&Payload> {
        self.receipt.as_ref()
    }

    /// Hashes the [`MilestoneEssence`] to be signed.
    pub fn hash(&self) -> [u8; 32] {
        use bee_common::packable::Packable;
        Blake2b256::digest(&self.pack_new()).into()
    }
}

impl bee_packable::Packable for MilestoneEssence {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.index.pack(packer)?;
        self.timestamp.pack(packer)?;
        self.parents.pack(packer)?;
        self.merkle_proof.pack(packer)?;
        self.next_pow_score.pack(packer)?;
        self.next_pow_score_milestone_index.pack(packer)?;
        self.public_keys.pack(packer)?;
        self.receipt.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).infallible()?;
        let timestamp = u64::unpack::<_, VERIFY>(unpacker).infallible()?;
        let parents = Parents::unpack::<_, VERIFY>(unpacker)?;

        let merkle_proof = <[u8; MilestoneEssence::MERKLE_PROOF_LENGTH]>::unpack::<_, VERIFY>(unpacker).infallible()?;

        let next_pow_score = u32::unpack::<_, VERIFY>(unpacker).infallible()?;
        let next_pow_score_milestone_index = u32::unpack::<_, VERIFY>(unpacker).infallible()?;

        let public_keys = VecPrefix::<[u8; Self::PUBLIC_KEY_LENGTH], PublicKeyCount>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|err| Error::MilestoneInvalidSignatureCount(err.into_prefix().into()))?;

        let receipt = OptionalPayload::unpack::<_, VERIFY>(unpacker)?;

        Self::from_vec_prefix(
            index,
            timestamp,
            parents,
            merkle_proof,
            next_pow_score,
            next_pow_score_milestone_index,
            public_keys,
            receipt,
        )
        .map_err(UnpackError::Packable)
    }
}

impl bee_common::packable::Packable for MilestoneEssence {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.index.packed_len()
            + self.timestamp.packed_len()
            + self.parents.packed_len()
            + MilestoneEssence::MERKLE_PROOF_LENGTH
            + self.next_pow_score.packed_len()
            + self.next_pow_score_milestone_index.packed_len()
            + 0u8.packed_len()
            + self.public_keys.len() * MilestoneEssence::PUBLIC_KEY_LENGTH
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
        for public_key in self.public_keys.iter() {
            public_key.pack(writer)?;
        }
        option_payload_pack(writer, self.receipt.as_ref())?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let index = MilestoneIndex::unpack_inner::<R, CHECK>(reader)?;
        let timestamp = u64::unpack_inner::<R, CHECK>(reader)?;
        let parents = Parents::unpack_inner::<R, CHECK>(reader)?;

        let merkle_proof = <[u8; MilestoneEssence::MERKLE_PROOF_LENGTH]>::unpack_inner::<R, CHECK>(reader)?;

        let next_pow_score = u32::unpack_inner::<R, CHECK>(reader)?;
        let next_pow_score_milestone_index = u32::unpack_inner::<R, CHECK>(reader)?;

        let public_keys_len = u8::unpack_inner::<R, CHECK>(reader)? as usize;
        let mut public_keys = Vec::with_capacity(public_keys_len);
        for _ in 0..public_keys_len {
            public_keys.push(<[u8; MilestoneEssence::PUBLIC_KEY_LENGTH]>::unpack_inner::<R, CHECK>(
                reader,
            )?);
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
