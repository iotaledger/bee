// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::types::Error;

use bee_common::packable::{Packable, Read, Write};
use bee_message::milestone::MilestoneIndex;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotInfo {
    network_id: u64,
    snapshot_index: MilestoneIndex,
    entry_point_index: MilestoneIndex,
    pruning_index: MilestoneIndex,
    timestamp: u64,
}

impl SnapshotInfo {
    pub fn new(
        network_id: u64,
        snapshot_index: MilestoneIndex,
        entry_point_index: MilestoneIndex,
        pruning_index: MilestoneIndex,
        timestamp: u64,
    ) -> Self {
        Self {
            network_id,
            snapshot_index,
            entry_point_index,
            pruning_index,
            timestamp,
        }
    }

    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    pub fn snapshot_index(&self) -> MilestoneIndex {
        self.snapshot_index
    }

    pub fn entry_point_index(&self) -> MilestoneIndex {
        self.entry_point_index
    }

    pub fn pruning_index(&self) -> MilestoneIndex {
        self.pruning_index
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl Packable for SnapshotInfo {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.network_id.packed_len()
            + self.snapshot_index.packed_len()
            + self.entry_point_index.packed_len()
            + self.pruning_index.packed_len()
            + self.timestamp.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.network_id.pack(writer)?;
        self.snapshot_index.pack(writer)?;
        self.entry_point_index.pack(writer)?;
        self.pruning_index.pack(writer)?;
        self.timestamp.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let network_id = u64::unpack_inner::<R, CHECK>(reader)?;
        let snapshot_index = MilestoneIndex::unpack_inner::<R, CHECK>(reader)?;
        let entry_point_index = MilestoneIndex::unpack_inner::<R, CHECK>(reader)?;
        let pruning_index = MilestoneIndex::unpack_inner::<R, CHECK>(reader)?;
        let timestamp = u64::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self {
            network_id,
            snapshot_index,
            entry_point_index,
            pruning_index,
            timestamp,
        })
    }
}
