// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crypto::hashes::{blake2b::Blake2b256, Digest};
use packable::{
    bounded::BoundedU16,
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    prefix::BoxedSlicePrefix,
    unpacker::Unpacker,
    Packable, PackableExt,
};

use crate::{
    milestone::MilestoneIndex,
    parent::Parents,
    payload::milestone::{MilestoneId, MilestoneOptions},
    Error,
};

pub(crate) type MilestoneMetadataLength = BoundedU16<{ u16::MIN }, { u16::MAX }>;

/// Essence of a milestone payload.
/// This is the signed part of a milestone payload.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MilestoneEssence {
    index: MilestoneIndex,
    timestamp: u32,
    previous_milestone_id: MilestoneId,
    parents: Parents,
    confirmed_merkle_proof: [u8; MilestoneEssence::MERKLE_PROOF_LENGTH],
    applied_merkle_proof: [u8; MilestoneEssence::MERKLE_PROOF_LENGTH],
    metadata: BoxedSlicePrefix<u8, MilestoneMetadataLength>,
    options: MilestoneOptions,
}

impl MilestoneEssence {
    /// Length of a milestone merkle proof.
    pub const MERKLE_PROOF_LENGTH: usize = 32;

    /// Creates a new [`MilestoneEssence`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        index: MilestoneIndex,
        timestamp: u32,
        previous_milestone_id: MilestoneId,
        parents: Parents,
        confirmed_merkle_proof: [u8; MilestoneEssence::MERKLE_PROOF_LENGTH],
        applied_merkle_proof: [u8; MilestoneEssence::MERKLE_PROOF_LENGTH],
        metadata: Vec<u8>,
        options: MilestoneOptions,
    ) -> Result<Self, Error> {
        let metadata = metadata
            .into_boxed_slice()
            .try_into()
            .map_err(Error::InvalidMilestoneMetadataLength)?;

        Ok(Self {
            index,
            timestamp,
            previous_milestone_id,
            parents,
            confirmed_merkle_proof,
            applied_merkle_proof,
            metadata,
            options,
        })
    }

    /// Returns the index of a [`MilestoneEssence`].
    pub fn index(&self) -> MilestoneIndex {
        self.index
    }

    /// Returns the timestamp of a [`MilestoneEssence`].
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }

    /// Returns the previous milestone ID of a [`MilestoneEssence`].
    pub fn previous_milestone_id(&self) -> &MilestoneId {
        &self.previous_milestone_id
    }

    /// Returns the parents of a [`MilestoneEssence`].
    pub fn parents(&self) -> &Parents {
        &self.parents
    }

    /// Returns the confirmed merkle proof of a [`MilestoneEssence`].
    pub fn confirmed_merkle_proof(&self) -> &[u8] {
        &self.confirmed_merkle_proof
    }

    /// Returns the applied merkle proof of a [`MilestoneEssence`].
    pub fn applied_merkle_proof(&self) -> &[u8] {
        &self.applied_merkle_proof
    }

    /// Returns the metadata.
    pub fn metadata(&self) -> &[u8] {
        &self.metadata
    }

    /// Returns the options of a [`MilestoneEssence`].
    pub fn options(&self) -> &MilestoneOptions {
        &self.options
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
        self.previous_milestone_id.pack(packer)?;
        self.parents.pack(packer)?;
        self.confirmed_merkle_proof.pack(packer)?;
        self.applied_merkle_proof.pack(packer)?;
        self.metadata.pack(packer)?;
        self.options.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).coerce()?;
        let timestamp = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let previous_milestone_id = MilestoneId::unpack::<_, VERIFY>(unpacker).coerce()?;
        let parents = Parents::unpack::<_, VERIFY>(unpacker)?;
        let confirmed_merkle_proof =
            <[u8; MilestoneEssence::MERKLE_PROOF_LENGTH]>::unpack::<_, VERIFY>(unpacker).coerce()?;
        let applied_merkle_proof =
            <[u8; MilestoneEssence::MERKLE_PROOF_LENGTH]>::unpack::<_, VERIFY>(unpacker).coerce()?;

        let metadata = BoxedSlicePrefix::<u8, MilestoneMetadataLength>::unpack::<_, VERIFY>(unpacker)
            .map_packable_err(|e| Error::InvalidMilestoneMetadataLength(e.into_prefix_err().into()))?;

        let options = MilestoneOptions::unpack::<_, VERIFY>(unpacker)?;

        Ok(Self {
            index,
            timestamp,
            previous_milestone_id,
            parents,
            confirmed_merkle_proof,
            applied_merkle_proof,
            metadata,
            options,
        })
    }
}
