// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{kind::Kind, Error};

use bee_common::packable::{Packable, Read, Write};
use bee_message::milestone::MilestoneIndex;

const SNAPSHOT_VERSION: u8 = 1;

#[derive(Clone)]
pub struct SnapshotHeader {
    pub(crate) kind: Kind,
    pub(crate) timestamp: u64,
    pub(crate) network_id: u64,
    pub(crate) sep_index: MilestoneIndex,
    pub(crate) ledger_index: MilestoneIndex,
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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
        let version = u8::unpack(reader)?;

        if version != SNAPSHOT_VERSION {
            return Err(Self::Error::InvalidVersion(SNAPSHOT_VERSION, version));
        }

        let kind = Kind::unpack(reader)?;
        let timestamp = u64::unpack(reader)?;
        let network_id = u64::unpack(reader)?;
        let sep_index = MilestoneIndex::unpack(reader)?;
        let ledger_index = MilestoneIndex::unpack(reader)?;

        Ok(Self {
            kind,
            timestamp,
            network_id,
            sep_index,
            ledger_index,
        })
    }
}
