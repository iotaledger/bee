// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::Error;

use bee_common::packable::{Packable, Read, Write};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotInfo {
    pub(crate) network_id: u64,
    pub(crate) snapshot_index: u32,
    pub(crate) entry_point_index: u32,
    pub(crate) pruning_index: u32,
    pub(crate) timestamp: u64,
}

impl SnapshotInfo {
    pub fn new(
        network_id: u64,
        snapshot_index: u32,
        entry_point_index: u32,
        pruning_index: u32,
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

    pub fn snapshot_index(&self) -> u32 {
        self.snapshot_index
    }

    pub fn entry_point_index(&self) -> u32 {
        self.entry_point_index
    }

    pub fn pruning_index(&self) -> u32 {
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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let network_id = u64::unpack(reader)?;
        let snapshot_index = u32::unpack(reader)?;
        let entry_point_index = u32::unpack(reader)?;
        let pruning_index = u32::unpack(reader)?;
        let timestamp = u64::unpack(reader)?;

        Ok(Self {
            network_id,
            snapshot_index,
            entry_point_index,
            pruning_index,
            timestamp,
        })
    }
}
