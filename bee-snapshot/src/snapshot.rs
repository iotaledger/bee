// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{header::SnapshotHeader, kind::Kind, milestone_diff::MilestoneDiff, Error};

use bee_common::packable::{Packable, Read, Write};
use bee_ledger::model::Output;
use bee_message::{
    milestone::MilestoneIndex,
    payload::transaction::{self, OutputId},
    solid_entry_point::SolidEntryPoint,
    MessageId,
};

use log::{error, info};

use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    path::Path,
};

pub struct Snapshot {
    pub(crate) header: SnapshotHeader,
    pub(crate) solid_entry_points: Box<[SolidEntryPoint]>,
    pub(crate) outputs: HashMap<OutputId, Output>,
    // A vector and not a hashmap because we need to keep the order intact.
    pub(crate) milestone_diffs: Vec<MilestoneDiff>,
}

impl Snapshot {
    pub fn header(&self) -> &SnapshotHeader {
        &self.header
    }

    pub fn solid_entry_points(&self) -> &[SolidEntryPoint] {
        &self.solid_entry_points
    }

    pub fn outputs(&self) -> &HashMap<OutputId, Output> {
        &self.outputs
    }

    pub fn milestone_diffs(&self) -> &Vec<MilestoneDiff> {
        &self.milestone_diffs
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
        for sep in self.solid_entry_points.iter() {
            len += sep.packed_len();
        }
        len += (self.outputs.len() as u64).packed_len();
        for (output_id, output) in self.outputs.iter() {
            len += output_id.packed_len();
            len += output.packed_len();
        }
        len += (self.milestone_diffs.len() as u64).packed_len();
        for diff in self.milestone_diffs.iter() {
            len += diff.packed_len();
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

        for sep in self.solid_entry_points.iter() {
            sep.pack(writer)?;
        }
        if self.header.kind() == Kind::Full {
            for (output_id, output) in self.outputs.iter() {
                output.message_id().pack(writer)?;
                output_id.pack(writer)?;
                output.inner().pack(writer)?;
            }
        }
        for diff in self.milestone_diffs.iter() {
            diff.pack(writer)?;
        }

        Ok(())
    }

    // TODO stream ?
    fn unpack<R: Read + ?Sized>(reader: &mut R) -> Result<Self, Self::Error> {
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
                if (*(header.ledger_index() - header.sep_index())) as usize != milestone_diff_count {
                    return Err(Error::InvalidMilestoneDiffsCount(
                        (*(header.ledger_index() - header.sep_index())) as usize,
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
                if (*(header.sep_index() - header.ledger_index())) as usize != milestone_diff_count {
                    return Err(Error::InvalidMilestoneDiffsCount(
                        (*(header.sep_index() - header.ledger_index())) as usize,
                        milestone_diff_count,
                    ));
                }
            }
        }

        let mut solid_entry_points = Vec::with_capacity(sep_count);
        for _ in 0..sep_count {
            solid_entry_points.push(SolidEntryPoint::unpack(reader)?);
        }

        let mut outputs = HashMap::with_capacity(output_count);
        if header.kind() == Kind::Full {
            for _ in 0..output_count {
                let message_id = MessageId::unpack(reader)?;
                let output_id = OutputId::unpack(reader)?;
                let output = transaction::Output::unpack(reader)?;
                outputs.insert(output_id, Output::new(message_id, output));
            }
        }

        let mut milestone_diffs = Vec::with_capacity(milestone_diff_count);
        for _ in 0..milestone_diff_count {
            milestone_diffs.push(MilestoneDiff::unpack(reader)?);
        }

        Ok(Self {
            header,
            solid_entry_points: solid_entry_points.into_boxed_slice(),
            outputs,
            milestone_diffs,
        })
    }
}

#[allow(dead_code)] // TODO: When pruning is enabled
pub(crate) fn snapshot(path: &Path, index: MilestoneIndex) -> Result<(), Error> {
    info!("Creating snapshot at index {}...", *index);

    let ls = Snapshot {
        header: SnapshotHeader {
            kind: Kind::Full,
            timestamp: 0,
            network_id: 0,
            sep_index: MilestoneIndex(0),
            ledger_index: MilestoneIndex(0),
        },
        solid_entry_points: Box::new([]),
        outputs: HashMap::new(),
        milestone_diffs: Vec::new(),
    };

    // let file = path.to_string() + "_tmp";

    if let Err(e) = ls.to_file(&path) {
        // TODO unwrap
        error!("Failed to write snapshot to file {}: {:?}.", path.to_str().unwrap(), e);
    }

    info!("Created snapshot at index {}.", *index);

    Ok(())
}
