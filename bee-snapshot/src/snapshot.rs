// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    header::SnapshotHeader, kind::Kind, milestone_diff::MilestoneDiff, output::Output,
    solid_entry_points::SolidEntryPoints, Error,
};

use bee_common::packable::{Packable, Read, Write};
use bee_message::MessageId;

use log::{error, info};

use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    path::Path,
};

pub struct Snapshot {
    pub(crate) header: SnapshotHeader,
    pub(crate) solid_entry_points: SolidEntryPoints,
    pub(crate) outputs: Box<[Output]>,
    pub(crate) milestone_diffs: Box<[MilestoneDiff]>,
}

impl Snapshot {
    pub fn header(&self) -> &SnapshotHeader {
        &self.header
    }

    pub fn solid_entry_points(&self) -> &SolidEntryPoints {
        &self.solid_entry_points
    }

    pub fn outputs_len(&self) -> usize {
        self.outputs.len()
    }

    pub fn milestone_diffs_len(&self) -> usize {
        self.milestone_diffs.len()
    }

    pub fn from_file(path: &Path) -> Result<Snapshot, Error> {
        let mut reader = BufReader::new(OpenOptions::new().read(true).open(path).map_err(Error::Io)?);

        // TODO unwrap
        Ok(Snapshot::unpack(&mut reader).unwrap())
    }

    pub fn to_file(&self, path: &Path) -> Result<(), Error> {
        let mut writer = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(path)
                .map_err(Error::Io)?,
        );

        // TODO unwrap
        self.pack(&mut writer).unwrap();

        Ok(())
    }
}

impl Packable for Snapshot {
    type Error = Error;

    fn packed_len(&self) -> usize {
        let mut len = self.header.packed_len();
        len += (self.solid_entry_points.len() as u64).packed_len();
        for s in self.solid_entry_points.iter() {
            len += s.packed_len();
        }
        len += (self.outputs.len() as u64).packed_len();
        for o in self.outputs.iter() {
            len += o.packed_len();
        }
        len += (self.milestone_diffs.len() as u64).packed_len();
        for m in self.milestone_diffs.iter() {
            len += m.packed_len();
        }

        len
    }

    // TODO stream ?
    fn pack<W: Write>(&self, writer: &mut W) -> Result<(), Self::Error> {
        self.header.pack(writer)?;

        (self.solid_entry_points.len() as u64).pack(writer)?;
        if self.header.kind() == Kind::Full {
            (self.outputs.len() as u64).pack(writer)?;
        }
        (self.milestone_diffs.len() as u64).pack(writer)?;

        for s in self.solid_entry_points.iter() {
            s.pack(writer)?;
        }
        if self.header.kind() == Kind::Full {
            for o in self.outputs.iter() {
                o.pack(writer)?;
            }
        }
        for m in self.milestone_diffs.iter() {
            m.pack(writer)?;
        }

        Ok(())
    }

    // TODO stream ?
    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let header = SnapshotHeader::unpack(reader)?;

        let sep_count = u64::unpack(reader)? as usize;
        let output_count = if header.kind() == Kind::Full {
            u64::unpack(reader)? as usize
        } else {
            0
        };
        let milestone_diff_count = u64::unpack(reader)? as usize;

        match header.kind() {
            Kind::Full => {
                if header.ledger_index() < header.sep_index() {
                    return Err(Error::LedgerSepIndexesInconsistency(
                        header.ledger_index(),
                        header.sep_index(),
                    ));
                }
                if (header.ledger_index() - header.sep_index()) as usize != milestone_diff_count {
                    return Err(Error::InvalidMilestoneDiffsCount(
                        (header.ledger_index() - header.sep_index()) as usize,
                        milestone_diff_count,
                    ));
                }
            }
            Kind::Delta => {
                if header.sep_index() < header.ledger_index() {
                    return Err(Error::LedgerSepIndexesInconsistency(
                        header.ledger_index(),
                        header.sep_index(),
                    ));
                }
                if (header.sep_index() - header.ledger_index()) as usize != milestone_diff_count {
                    return Err(Error::InvalidMilestoneDiffsCount(
                        (header.sep_index() - header.ledger_index()) as usize,
                        milestone_diff_count,
                    ));
                }
            }
        }

        let mut solid_entry_points = Vec::with_capacity(sep_count);
        for _ in 0..sep_count {
            solid_entry_points.push(MessageId::unpack(reader)?);
        }

        let mut outputs = Vec::with_capacity(output_count);
        if header.kind() == Kind::Full {
            for _ in 0..output_count {
                outputs.push(Output::unpack(reader)?);
            }
        }

        let mut milestone_diffs = Vec::with_capacity(milestone_diff_count);
        for _ in 0..milestone_diff_count {
            milestone_diffs.push(MilestoneDiff::unpack(reader)?);
        }

        Ok(Self {
            header,
            solid_entry_points: SolidEntryPoints::new(solid_entry_points.into_boxed_slice()),
            outputs: outputs.into_boxed_slice(),
            milestone_diffs: milestone_diffs.into_boxed_slice(),
        })
    }
}

#[allow(dead_code)] // TODO: When pruning is enabled
pub(crate) fn snapshot(path: &Path, index: u32) -> Result<(), Error> {
    info!("Creating snapshot at index {}...", index);

    let ls = Snapshot {
        header: SnapshotHeader {
            kind: Kind::Full,
            timestamp: 0,
            network_id: 0,
            sep_index: 0,
            ledger_index: 0,
        },
        solid_entry_points: SolidEntryPoints::new(Box::new([])),
        outputs: Box::new([]),
        milestone_diffs: Box::new([]),
    };

    // let file = path.to_string() + "_tmp";

    if let Err(e) = ls.to_file(&path) {
        // TODO unwrap
        error!("Failed to write snapshot to file {}: {:?}.", path.to_str().unwrap(), e);
    }

    info!("Created snapshot at index {}.", index);

    Ok(())
}
