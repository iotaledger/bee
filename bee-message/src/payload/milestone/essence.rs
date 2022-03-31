// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    milestone::MilestoneIndex,
    parent::Parents,
    payload::{OptionalPayload, Payload},
    Error,
};

use crypto::hashes::{blake2b::Blake2b256, Digest};
use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

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
    receipt: OptionalPayload,
}

impl MilestoneEssence {
    /// Length of a milestone merkle proof.
    pub const MERKLE_PROOF_LENGTH: usize = 32;

    /// Creates a new [`MilestoneEssence`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        index: MilestoneIndex,
        timestamp: u64,
        parents: Parents,
        merkle_proof: [u8; MilestoneEssence::MERKLE_PROOF_LENGTH],
        next_pow_score: u32,
        next_pow_score_milestone_index: u32,
        receipt: Option<Payload>,
    ) -> Result<Self, Error> {
        verify_pow_scores(index, next_pow_score, next_pow_score_milestone_index)?;

        let receipt = OptionalPayload::from(receipt);

        verify_payload(&receipt)?;

        Ok(Self {
            index,
            timestamp,
            parents,
            merkle_proof,
            next_pow_score,
            next_pow_score_milestone_index,
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

    /// Returns the optional receipt of a [`MilestoneEssence`].
    pub fn receipt(&self) -> Option<&Payload> {
        self.receipt.as_ref()
    }

    /// Hashes the [`MilestoneEssence`] to be signed.
    pub fn hash(&self) -> [u8; 32] {
        Blake2b256::digest(&self.pack_to_vec()).into()
    }
}

impl Packable for MilestoneEssence {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.index.pack(packer)?;
        self.timestamp.pack(packer)?;
        self.parents.pack(packer)?;
        self.merkle_proof.pack(packer)?;
        self.next_pow_score.pack(packer)?;
        self.next_pow_score_milestone_index.pack(packer)?;
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

        if VERIFY {
            verify_pow_scores(index, next_pow_score, next_pow_score_milestone_index).map_err(UnpackError::Packable)?;
        }

        let receipt = OptionalPayload::unpack::<_, VERIFY>(unpacker)?;

        if VERIFY {
            verify_payload(&receipt).map_err(UnpackError::Packable)?;
        }

        Ok(Self {
            index,
            timestamp,
            parents,
            merkle_proof,
            next_pow_score,
            next_pow_score_milestone_index,
            receipt,
        })
    }
}

fn verify_pow_scores(
    index: MilestoneIndex,
    next_pow_score: u32,
    next_pow_score_milestone_index: u32,
) -> Result<(), Error> {
    if next_pow_score == 0 && next_pow_score_milestone_index != 0
        || next_pow_score != 0 && next_pow_score_milestone_index <= *index
    {
        Err(Error::InvalidPowScoreValues {
            nps: next_pow_score,
            npsmi: next_pow_score_milestone_index,
        })
    } else {
        Ok(())
    }
}

fn verify_payload(payload: &OptionalPayload) -> Result<(), Error> {
    match &payload.0 {
        Some(Payload::Receipt(_)) | None => Ok(()),
        Some(payload) => Err(Error::InvalidPayloadKind(payload.kind())),
    }
}
