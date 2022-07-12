// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::payload::milestone::{MilestoneId, MilestoneIndex, MilestoneOption, ParametersMilestoneOption};
use packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable, PackableExt,
};

use crate::types::{error::Error, snapshot::SnapshotKind};

const SNAPSHOT_VERSION: u8 = 2;

/// Describes a snapshot header common to full and delta snapshots.
#[derive(Clone)]
pub struct SnapshotHeader {
    kind: SnapshotKind,
    timestamp: u32,
    network_id: u64,
    sep_index: MilestoneIndex,
    ledger_index: MilestoneIndex,
}

impl SnapshotHeader {
    /// The length, in bytes, of a `SnapshotHeader`.
    pub const LENGTH: usize = 26;

    /// Returns the kind of a `SnapshotHeader`.
    pub fn kind(&self) -> SnapshotKind {
        self.kind
    }

    /// Returns the timestamp of a `SnapshotHeader`.
    pub fn timestamp(&self) -> u32 {
        self.timestamp
    }

    /// Returns the network id of a `SnapshotHeader`.
    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    /// Returns the solid entry point index of a `SnapshotHeader`.
    pub fn sep_index(&self) -> MilestoneIndex {
        self.sep_index
    }

    /// Returns the ledger index of a `SnapshotHeader`.
    pub fn ledger_index(&self) -> MilestoneIndex {
        self.ledger_index
    }
}

impl Packable for SnapshotHeader {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        SNAPSHOT_VERSION.pack(packer)?;

        self.kind.pack(packer)?;
        self.timestamp.pack(packer)?;
        self.network_id.pack(packer)?;
        self.sep_index.pack(packer)?;
        self.ledger_index.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack::<_, VERIFY>(unpacker).coerce()?;

        if VERIFY && SNAPSHOT_VERSION != version {
            return Err(UnpackError::Packable(Error::UnsupportedSnapshotVersion(
                SNAPSHOT_VERSION,
                version,
            )));
        }

        let kind = SnapshotKind::unpack::<_, VERIFY>(unpacker)?;
        let timestamp = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let network_id = u64::unpack::<_, VERIFY>(unpacker).coerce()?;
        let sep_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).coerce()?;
        let ledger_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).coerce()?;

        Ok(Self {
            kind,
            timestamp,
            network_id,
            sep_index,
            ledger_index,
        })
    }
}

/// Describes a snapshot header specific to full snapshots.
#[derive(Clone, Debug)]
pub struct FullSnapshotHeader {
    genesis_milestone_index: MilestoneIndex,
    target_milestone_index: MilestoneIndex,
    target_milestone_timestamp: u32,
    target_milestone_id: MilestoneId,
    ledger_milestone_index: MilestoneIndex,
    treasury_output_milestone_id: MilestoneId,
    treasury_output_amount: u64,
    parameters_milestone_option: MilestoneOption,
    output_count: u64,
    milestone_diff_count: u32,
    sep_count: u16,
}

impl FullSnapshotHeader {
    /// Returns the genesis milestone index of a [`FullSnapshotHeader`].
    pub fn genesis_milestone_index(&self) -> MilestoneIndex {
        self.genesis_milestone_index
    }

    /// Returns the target milestone index of a [`FullSnapshotHeader`].
    pub fn target_milestone_index(&self) -> MilestoneIndex {
        self.target_milestone_index
    }

    /// Returns the target milestone timestamp of a [`FullSnapshotHeader`].
    pub fn target_milestone_timestamp(&self) -> u32 {
        self.target_milestone_timestamp
    }

    /// Returns the target milestone ID of a [`FullSnapshotHeader`].
    pub fn target_milestone_id(&self) -> &MilestoneId {
        &self.target_milestone_id
    }

    /// Returns the ledger milestone index of a [`FullSnapshotHeader`].
    pub fn ledger_milestone_index(&self) -> MilestoneIndex {
        self.ledger_milestone_index
    }

    /// Returns the treasury output milestone ID of a [`FullSnapshotHeader`].
    pub fn treasury_output_milestone_id(&self) -> &MilestoneId {
        &self.treasury_output_milestone_id
    }

    /// Returns the treasury output amount of a [`FullSnapshotHeader`].
    pub fn treasury_output_amount(&self) -> u64 {
        self.treasury_output_amount
    }

    /// Returns the parameters milestone option of a [`FullSnapshotHeader`].
    pub fn parameters_milestone_option(&self) -> &MilestoneOption {
        &self.parameters_milestone_option
    }

    /// Returns the output count of a [`FullSnapshotHeader`].
    pub fn output_count(&self) -> u64 {
        self.output_count
    }

    /// Returns the milestone diff count of a [`FullSnapshotHeader`].
    pub fn milestone_diff_count(&self) -> u32 {
        self.milestone_diff_count
    }

    /// Returns the SEP count of a [`FullSnapshotHeader`].
    pub fn sep_count(&self) -> u16 {
        self.sep_count
    }
}

impl Packable for FullSnapshotHeader {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        SNAPSHOT_VERSION.pack(packer)?;
        SnapshotKind::Full.pack(packer)?;

        self.genesis_milestone_index.pack(packer)?;
        self.target_milestone_index.pack(packer)?;
        self.target_milestone_timestamp.pack(packer)?;
        self.target_milestone_id.pack(packer)?;
        self.ledger_milestone_index.pack(packer)?;
        self.treasury_output_milestone_id.pack(packer)?;
        self.treasury_output_amount.pack(packer)?;
        // This is only required in Hornet.
        (self.parameters_milestone_option.packed_len() as u16).pack(packer)?;
        self.parameters_milestone_option.pack(packer)?;
        self.output_count.pack(packer)?;
        self.milestone_diff_count.pack(packer)?;
        self.sep_count.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack::<_, VERIFY>(unpacker).coerce()?;

        if VERIFY && SNAPSHOT_VERSION != version {
            return Err(UnpackError::Packable(Error::UnsupportedSnapshotVersion(
                SNAPSHOT_VERSION,
                version,
            )));
        }

        let kind = SnapshotKind::unpack::<_, VERIFY>(unpacker).coerce()?;

        if VERIFY && kind != SnapshotKind::Full {
            return Err(UnpackError::Packable(Error::InvalidSnapshotKind(kind as u8)));
        }

        let genesis_milestone_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).coerce()?;
        let target_milestone_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).coerce()?;
        let target_milestone_timestamp = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let target_milestone_id = MilestoneId::unpack::<_, VERIFY>(unpacker).coerce()?;
        let ledger_milestone_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).coerce()?;
        let treasury_output_milestone_id = MilestoneId::unpack::<_, VERIFY>(unpacker).coerce()?;
        let treasury_output_amount = u64::unpack::<_, VERIFY>(unpacker).coerce()?;
        // This is only required in Hornet.
        let _parameters_milestone_option_length = u16::unpack::<_, VERIFY>(unpacker).coerce()?;
        let parameters_milestone_option = MilestoneOption::unpack::<_, VERIFY>(unpacker).coerce()?;
        let output_count = u64::unpack::<_, VERIFY>(unpacker).coerce()?;
        let milestone_diff_count = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let sep_count = u16::unpack::<_, VERIFY>(unpacker).coerce()?;

        Ok(Self {
            genesis_milestone_index,
            target_milestone_index,
            target_milestone_timestamp,
            target_milestone_id,
            ledger_milestone_index,
            treasury_output_milestone_id,
            treasury_output_amount,
            parameters_milestone_option,
            output_count,
            milestone_diff_count,
            sep_count,
        })
    }
}

/// Describes a snapshot header specific to delta snapshots.
#[derive(Clone, Debug)]
pub struct DeltaSnapshotHeader {
    target_milestone_index: MilestoneIndex,
    target_milestone_timestamp: u32,
    full_snapshot_target_milestone_id: MilestoneId,
    sep_file_offset: u64,
    milestone_diff_count: u32,
    sep_count: u16,
}

impl DeltaSnapshotHeader {
    /// Returns the target milestone index of a [`DeltaSnapshotHeader`].
    pub fn target_milestone_index(&self) -> MilestoneIndex {
        self.target_milestone_index
    }

    /// Returns the target milestone timestamp of a [`DeltaSnapshotHeader`].
    pub fn target_milestone_timestamp(&self) -> u32 {
        self.target_milestone_timestamp
    }

    /// Returns the full snapshot target milestone ID of a [`DeltaSnapshotHeader`].
    pub fn full_snapshot_target_milestone_id(&self) -> &MilestoneId {
        &self.full_snapshot_target_milestone_id
    }

    /// Returns the SEP file offset of a [`DeltaSnapshotHeader`].
    pub fn sep_file_offset(&self) -> u64 {
        self.sep_file_offset
    }

    /// Returns the milestone diff count of a [`DeltaSnapshotHeader`].
    pub fn milestone_diff_count(&self) -> u32 {
        self.milestone_diff_count
    }

    /// Returns the SEP count of a [`DeltaSnapshotHeader`].
    pub fn sep_count(&self) -> u16 {
        self.sep_count
    }
}

impl Packable for DeltaSnapshotHeader {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        SNAPSHOT_VERSION.pack(packer)?;
        SnapshotKind::Delta.pack(packer)?;

        self.target_milestone_index.pack(packer)?;
        self.target_milestone_timestamp.pack(packer)?;
        self.full_snapshot_target_milestone_id.pack(packer)?;
        self.sep_file_offset.pack(packer)?;
        self.milestone_diff_count.pack(packer)?;
        self.sep_count.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let version = u8::unpack::<_, VERIFY>(unpacker).coerce()?;

        if VERIFY && SNAPSHOT_VERSION != version {
            return Err(UnpackError::Packable(Error::UnsupportedSnapshotVersion(
                SNAPSHOT_VERSION,
                version,
            )));
        }

        let kind = SnapshotKind::unpack::<_, VERIFY>(unpacker).coerce()?;

        if VERIFY && kind != SnapshotKind::Delta {
            return Err(UnpackError::Packable(Error::InvalidSnapshotKind(kind as u8)));
        }

        let target_milestone_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).coerce()?;
        let target_milestone_timestamp = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let full_snapshot_target_milestone_id = MilestoneId::unpack::<_, VERIFY>(unpacker).coerce()?;
        let sep_file_offset = u64::unpack::<_, VERIFY>(unpacker).coerce()?;
        let milestone_diff_count = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let sep_count = u16::unpack::<_, VERIFY>(unpacker).coerce()?;

        Ok(Self {
            target_milestone_index,
            target_milestone_timestamp,
            full_snapshot_target_milestone_id,
            sep_file_offset,
            milestone_diff_count,
            sep_count,
        })
    }
}
