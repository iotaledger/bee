// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_block::{
    payload::milestone::{MilestoneId, MilestoneIndex},
    protocol::ProtocolParemeters,
};
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
            return Err(UnpackError::Packable(Error::UnsupportedVersion(
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
#[derive(Clone)]
pub struct FullSnapshotHeader {
    genesis_milestone_index: MilestoneIndex,
    target_milestone_index: MilestoneIndex,
    target_milestone_timestamp: u32,
    target_milestone_id: MilestoneId,
    ledger_milestone_index: MilestoneIndex,
    treasury_output_milestone_id: MilestoneId,
    treasury_output_amount: u64,
    protocol_parameters: ProtocolParemeters,
    output_count: u64,
    milestone_diff_count: u64,
    sep_count: u64,
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

    /// Returns the protocol parameters of a [`FullSnapshotHeader`].
    pub fn protocol_parameters(&self) -> &ProtocolParemeters {
        &self.protocol_parameters
    }

    /// Returns the output count of a [`FullSnapshotHeader`].
    pub fn output_count(&self) -> u64 {
        self.output_count
    }

    /// Returns the milestone diff count of a [`FullSnapshotHeader`].
    pub fn milestone_diff_count(&self) -> u64 {
        self.milestone_diff_count
    }

    /// Returns the SEP count of a [`FullSnapshotHeader`].
    pub fn sep_count(&self) -> u64 {
        self.sep_count
    }
}

// This can't be derived as there is an additional length prefix required by Hornet.
impl Packable for FullSnapshotHeader {
    type UnpackError = Error;

    fn pack<P: Packer>(&self, packer: &mut P) -> Result<(), P::Error> {
        self.genesis_milestone_index.pack(packer)?;
        self.target_milestone_index.pack(packer)?;
        self.target_milestone_timestamp.pack(packer)?;
        self.target_milestone_id.pack(packer)?;
        self.ledger_milestone_index.pack(packer)?;
        self.treasury_output_milestone_id.pack(packer)?;
        self.treasury_output_amount.pack(packer)?;
        // This is only required in Hornet.
        (self.protocol_parameters.packed_len() as u16).pack(packer)?;
        self.protocol_parameters.pack(packer)?;
        self.output_count.pack(packer)?;
        self.milestone_diff_count.pack(packer)?;
        self.sep_count.pack(packer)?;

        Ok(())
    }

    fn unpack<U: Unpacker, const VERIFY: bool>(
        unpacker: &mut U,
    ) -> Result<Self, UnpackError<Self::UnpackError, U::Error>> {
        let genesis_milestone_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).coerce()?;
        let target_milestone_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).coerce()?;
        let target_milestone_timestamp = u32::unpack::<_, VERIFY>(unpacker).coerce()?;
        let target_milestone_id = MilestoneId::unpack::<_, VERIFY>(unpacker).coerce()?;
        let ledger_milestone_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).coerce()?;
        let treasury_output_milestone_id = MilestoneId::unpack::<_, VERIFY>(unpacker).coerce()?;
        let treasury_output_amount = u64::unpack::<_, VERIFY>(unpacker).coerce()?;
        // This is only required in Hornet.
        let _protocol_parameters_length = u16::unpack::<_, VERIFY>(unpacker).coerce()?;
        let protocol_parameters = ProtocolParemeters::unpack::<_, VERIFY>(unpacker).coerce()?;
        let output_count = u64::unpack::<_, VERIFY>(unpacker).coerce()?;
        let milestone_diff_count = u64::unpack::<_, VERIFY>(unpacker).coerce()?;
        let sep_count = u64::unpack::<_, VERIFY>(unpacker).coerce()?;

        Ok(Self {
            genesis_milestone_index,
            target_milestone_index,
            target_milestone_timestamp,
            target_milestone_id,
            ledger_milestone_index,
            treasury_output_milestone_id,
            treasury_output_amount,
            protocol_parameters,
            output_count,
            milestone_diff_count,
            sep_count,
        })
    }
}

/// Describes a snapshot header specific to delta snapshots.
#[derive(Clone, Packable)]
pub struct DeltaSnapshotHeader {
    sep_count: u64,
    milestone_diff_count: u64,
}

impl DeltaSnapshotHeader {
    /// Returns the solid entry point count of a `DeltaSnapshotHeader`.
    pub fn sep_count(&self) -> u64 {
        self.sep_count
    }

    /// Returns the milestone diff count of a `DeltaSnapshotHeader`.
    pub fn milestone_diff_count(&self) -> u64 {
        self.milestone_diff_count
    }
}
