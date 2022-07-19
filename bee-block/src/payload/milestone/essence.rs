// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use alloc::vec::Vec;

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
    constant::PROTOCOL_VERSION,
    parent::Parents,
    payload::milestone::{MerkleRoot, MilestoneId, MilestoneIndex, MilestoneOptions},
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
    protocol_version: u8,
    previous_milestone_id: MilestoneId,
    parents: Parents,
    inclusion_merkle_root: MerkleRoot,
    applied_merkle_root: MerkleRoot,
    metadata: BoxedSlicePrefix<u8, MilestoneMetadataLength>,
    options: MilestoneOptions,
}

impl MilestoneEssence {
    /// Creates a new [`MilestoneEssence`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        index: MilestoneIndex,
        timestamp: u32,
        previous_milestone_id: MilestoneId,
        parents: Parents,
        inclusion_merkle_root: MerkleRoot,
        applied_merkle_root: MerkleRoot,
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
            protocol_version: PROTOCOL_VERSION,
            previous_milestone_id,
            parents,
            inclusion_merkle_root,
            applied_merkle_root,
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

    /// Returns the protocol version of a [`MilestoneEssence`].
    pub fn protocol_version(&self) -> u8 {
        self.protocol_version
    }

    /// Returns the previous milestone ID of a [`MilestoneEssence`].
    pub fn previous_milestone_id(&self) -> &MilestoneId {
        &self.previous_milestone_id
    }

    /// Returns the parents of a [`MilestoneEssence`].
    pub fn parents(&self) -> &Parents {
        &self.parents
    }

    /// Returns the inclusion merkle root of a [`MilestoneEssence`].
    pub fn inclusion_merkle_root(&self) -> &MerkleRoot {
        &self.inclusion_merkle_root
    }

    /// Returns the applied merkle root of a [`MilestoneEssence`].
    pub fn applied_merkle_root(&self) -> &MerkleRoot {
        &self.applied_merkle_root
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
    type UnpackVisitor = ();

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.index.pack(packer)?;
        self.timestamp.pack(packer)?;
        self.protocol_version.pack(packer)?;
        self.previous_milestone_id.pack(packer)?;
        self.parents.pack(packer)?;
        self.inclusion_merkle_root.pack(packer)?;
        self.applied_merkle_root.pack(packer)?;
        self.metadata.pack(packer)?;
        self.options.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
        visitor: &mut Self::UnpackVisitor,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let index = MilestoneIndex::unpack::<_, VERIFY>(unpacker, visitor).coerce()?;
        let timestamp = u32::unpack::<_, VERIFY>(unpacker, visitor).coerce()?;
        let protocol_version = u8::unpack::<_, VERIFY>(unpacker, visitor).coerce()?;

        if VERIFY && protocol_version != PROTOCOL_VERSION {
            return Err(UnpackError::Packable(Error::ProtocolVersionMismatch {
                expected: PROTOCOL_VERSION,
                actual: protocol_version,
            }));
        }

        let previous_milestone_id = MilestoneId::unpack::<_, VERIFY>(unpacker, visitor).coerce()?;
        let parents = Parents::unpack::<_, VERIFY>(unpacker, visitor)?;
        let inclusion_merkle_root = MerkleRoot::unpack::<_, VERIFY>(unpacker, visitor).coerce()?;
        let applied_merkle_root = MerkleRoot::unpack::<_, VERIFY>(unpacker, visitor).coerce()?;

        let metadata = BoxedSlicePrefix::<u8, MilestoneMetadataLength>::unpack::<_, VERIFY>(unpacker, visitor)
            .map_packable_err(|e| Error::InvalidMilestoneMetadataLength(e.into_prefix_err().into()))?;

        let options = MilestoneOptions::unpack::<_, VERIFY>(unpacker, visitor)?;

        Ok(Self {
            index,
            timestamp,
            protocol_version,
            previous_milestone_id,
            parents,
            inclusion_merkle_root,
            applied_merkle_root,
            metadata,
            options,
        })
    }
}
