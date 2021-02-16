// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_snapshot::{error::Error as SnapshotError, header::SnapshotHeader};

use chrono::{offset::TimeZone, Utc};
use structopt::StructOpt;
use thiserror::Error;

use std::{fs::OpenOptions, io::BufReader, path::Path};

#[derive(Debug, Error)]
pub enum SnapshotInfoError {
    #[error("{0}")]
    Io(std::io::Error),
    #[error("{0}")]
    InvalidSnapshotHeader(SnapshotError),
}

#[derive(Debug, StructOpt)]
pub struct SnapshotInfoTool {
    path: String,
}

pub fn exec(tool: &SnapshotInfoTool) -> Result<(), SnapshotInfoError> {
    let mut reader = BufReader::new(
        OpenOptions::new()
            .read(true)
            .open(Path::new(&tool.path))
            .map_err(SnapshotInfoError::Io)?,
    );
    let header = SnapshotHeader::unpack(&mut reader).map_err(SnapshotInfoError::InvalidSnapshotHeader)?;

    println!("Type:\t\t\t{:?}", header.kind());
    println!(
        "Timestamp:\t\t{} ({})",
        header.timestamp(),
        Utc.timestamp(header.timestamp() as i64, 0).format("%d-%m-%Y %H:%M:%S")
    );
    println!("Network ID:\t\t{}", header.network_id());
    println!("SEP index:\t\t{}", *header.sep_index());
    println!("Ledger index:\t\t{}", *header.ledger_index());
    println!("SEP count:\t\t{}", header.sep_count());
    println!("Outputs count:\t\t{}", header.output_count());
    println!("Milestone diffs count:\t{}", header.milestone_diff_count());

    Ok(())
}
