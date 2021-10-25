// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::{error::Error, snapshot::SnapshotKind};

use bee_message::{milestone::MilestoneIndex, payload::milestone::MilestoneId};
use bee_packable::{
    error::{UnpackError, UnpackErrorExt},
    packer::Packer,
    unpacker::Unpacker,
    Packable,
};

const SNAPSHOT_VERSION: u8 = 1;

/// Describes a snapshot header common to full and delta snapshots.
#[derive(Clone)]
pub struct SnapshotHeader {
    kind: SnapshotKind,
    timestamp: u64,
    network_id: u64,
    sep_index: MilestoneIndex,
    ledger_index: MilestoneIndex,
}

impl SnapshotHeader {
    /// Returns the kind of a `SnapshotHeader`.
    pub fn kind(&self) -> SnapshotKind {
        self.kind
    }

    /// Returns the timestamp of a `SnapshotHeader`.
    pub fn timestamp(&self) -> u64 {
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
        let version = u8::unpack::<_, VERIFY>(unpacker).infallible()?;

        if SNAPSHOT_VERSION != version {
            return Err(UnpackError::Packable(Self::UnpackError::UnsupportedVersion(
                SNAPSHOT_VERSION,
                version,
            )));
        }

        let kind = SnapshotKind::unpack::<_, VERIFY>(unpacker)?;
        let timestamp = u64::unpack::<_, VERIFY>(unpacker).infallible()?;
        let network_id = u64::unpack::<_, VERIFY>(unpacker).infallible()?;
        let sep_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).infallible()?;
        let ledger_index = MilestoneIndex::unpack::<_, VERIFY>(unpacker).infallible()?;

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
#[derive(Clone, Packable)]
pub struct FullSnapshotHeader {
    sep_count: u64,
    output_count: u64,
    milestone_diff_count: u64,
    treasury_output_milestone_id: MilestoneId,
    treasury_output_amount: u64,
}

impl FullSnapshotHeader {
    /// Returns the solid entry point count of a `FullSnapshotHeader`.
    pub fn sep_count(&self) -> u64 {
        self.sep_count
    }

    /// Returns the output count of a `FullSnapshotHeader`.
    pub fn output_count(&self) -> u64 {
        self.output_count
    }

    /// Returns the milestone diff count of a `FullSnapshotHeader`.
    pub fn milestone_diff_count(&self) -> u64 {
        self.milestone_diff_count
    }

    /// Returns the treasury output milestone id of a `FullSnapshotHeader`.
    pub fn treasury_output_milestone_id(&self) -> &MilestoneId {
        &self.treasury_output_milestone_id
    }

    /// Returns the treasury output amount of a `FullSnapshotHeader`.
    pub fn treasury_output_amount(&self) -> u64 {
        self.treasury_output_amount
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
