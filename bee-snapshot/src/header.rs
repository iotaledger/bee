// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{kind::Kind, Error};

use bee_common::packable::{Packable, Read, Write};

const SNAPSHOT_VERSION: u8 = 1;

#[derive(Clone)]
pub struct SnapshotHeader {
    pub(crate) kind: Kind,
    pub(crate) timestamp: u64,
    pub(crate) network_id: u64,
    pub(crate) sep_index: u32,
    pub(crate) ledger_index: u32,
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

    pub fn sep_index(&self) -> u32 {
        self.sep_index
    }

    pub fn ledger_index(&self) -> u32 {
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

    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let version = u8::unpack(reader)?;

        if version != SNAPSHOT_VERSION {
            return Err(Self::Error::InvalidVersion(SNAPSHOT_VERSION, version));
        }

        let kind = Kind::unpack(reader)?;
        let timestamp = u64::unpack(reader)?;
        let network_id = u64::unpack(reader)?;
        let sep_index = u32::unpack(reader)?;
        let ledger_index = u32::unpack(reader)?;

        Ok(Self {
            kind,
            timestamp,
            network_id,
            sep_index,
            ledger_index,
        })
    }
}
