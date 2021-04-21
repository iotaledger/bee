// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::snapshot::{error::Error, kind::Kind};

use bee_common::packable::{Packable, Read, Write};
use bee_message::{milestone::MilestoneIndex, payload::milestone::MilestoneId};

const SNAPSHOT_VERSION: u8 = 1;

#[derive(Clone)]
pub struct SnapshotHeader {
    kind: Kind,
    timestamp: u64,
    network_id: u64,
    sep_index: MilestoneIndex,
    ledger_index: MilestoneIndex,
}

impl SnapshotHeader {
    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn network_id(&self) -> u64 {
        self.network_id
    }

    pub fn sep_index(&self) -> MilestoneIndex {
        self.sep_index
    }

    pub fn ledger_index(&self) -> MilestoneIndex {
        self.ledger_index
    }
}

impl Packable for SnapshotHeader {
    type Error = Error;

    fn packed_len(&self) -> usize {
        SNAPSHOT_VERSION.packed_len()
            + self.kind.packed_len()
            + self.timestamp.packed_len()
            + self.network_id.packed_len()
            + self.sep_index.packed_len()
            + self.ledger_index.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        SNAPSHOT_VERSION.pack(writer)?;
        self.kind.pack(writer)?;
        self.timestamp.pack(writer)?;
        self.network_id.pack(writer)?;
        self.sep_index.pack(writer)?;
        self.ledger_index.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let version = u8::unpack_inner::<R, CHECK>(reader)?;

        if CHECK && SNAPSHOT_VERSION != version {
            return Err(Self::Error::UnsupportedVersion(SNAPSHOT_VERSION, version));
        }

        let kind = Kind::unpack_inner::<R, CHECK>(reader)?;
        let timestamp = u64::unpack_inner::<R, CHECK>(reader)?;
        let network_id = u64::unpack_inner::<R, CHECK>(reader)?;
        let sep_index = MilestoneIndex::unpack_inner::<R, CHECK>(reader)?;
        let ledger_index = MilestoneIndex::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self {
            kind,
            timestamp,
            network_id,
            sep_index,
            ledger_index,
        })
    }
}

#[derive(Clone)]
pub struct FullSnapshotHeader {
    sep_count: u64,
    output_count: u64,
    milestone_diff_count: u64,
    treasury_output_milestone_id: MilestoneId,
    treasury_output_amount: u64,
}

impl FullSnapshotHeader {
    pub fn sep_count(&self) -> u64 {
        self.sep_count
    }

    pub fn output_count(&self) -> u64 {
        self.output_count
    }

    pub fn milestone_diff_count(&self) -> u64 {
        self.milestone_diff_count
    }

    pub fn treasury_output_milestone_id(&self) -> &MilestoneId {
        &self.treasury_output_milestone_id
    }

    pub fn treasury_output_amount(&self) -> u64 {
        self.treasury_output_amount
    }
}

impl Packable for FullSnapshotHeader {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.sep_count.packed_len()
            + self.output_count.packed_len()
            + self.milestone_diff_count.packed_len()
            + self.treasury_output_milestone_id.packed_len()
            + self.treasury_output_amount.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.sep_count.pack(writer)?;
        self.output_count.pack(writer)?;
        self.milestone_diff_count.pack(writer)?;
        self.treasury_output_milestone_id.pack(writer)?;
        self.treasury_output_amount.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let sep_count = u64::unpack_inner::<R, CHECK>(reader)?;
        let output_count = u64::unpack_inner::<R, CHECK>(reader)?;
        let milestone_diff_count = u64::unpack_inner::<R, CHECK>(reader)?;
        let treasury_output_milestone_id = MilestoneId::unpack_inner::<R, CHECK>(reader)?;
        let treasury_output_amount = u64::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self {
            sep_count,
            output_count,
            milestone_diff_count,
            treasury_output_milestone_id,
            treasury_output_amount,
        })
    }
}

#[derive(Clone)]
pub struct DeltaSnapshotHeader {
    sep_count: u64,
    milestone_diff_count: u64,
}

impl DeltaSnapshotHeader {
    pub fn sep_count(&self) -> u64 {
        self.sep_count
    }

    pub fn milestone_diff_count(&self) -> u64 {
        self.milestone_diff_count
    }
}

impl Packable for DeltaSnapshotHeader {
    type Error = Error;

    fn packed_len(&self) -> usize {
        self.sep_count.packed_len() + self.milestone_diff_count.packed_len()
    }

    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.sep_count.pack(writer)?;
        self.milestone_diff_count.pack(writer)?;

        Ok(())
    }

    fn unpack_inner<R: Read + ?Sized, const CHECK: bool>(reader: &mut R) -> Result<Self, Self::Error> {
        let sep_count = u64::unpack_inner::<R, CHECK>(reader)?;
        let milestone_diff_count = u64::unpack_inner::<R, CHECK>(reader)?;

        Ok(Self {
            sep_count,
            milestone_diff_count,
        })
    }
}
