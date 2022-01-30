// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_common::packable::Packable;
use bee_ledger::types::{
    snapshot::{DeltaSnapshotHeader, FullSnapshotHeader, SnapshotHeader, SnapshotKind},
    Error as SnapshotError,
};

use chrono::{offset::TimeZone, Utc};
use structopt::StructOpt;
use thiserror::Error;

use std::{fs::OpenOptions, io::BufReader, path::Path};

#[derive(Debug, Error)]
pub enum SnapshotInfoError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid snapshot header: {0}")]
    InvalidSnapshotHeader(#[from] SnapshotError),
}

#[derive(Clone, Debug, StructOpt)]
pub struct SnapshotInfoTool {
    path: String,
}

pub fn print_full_header(header: FullSnapshotHeader) {
    println!("SEP count:\t\t\t{}", header.sep_count());
    println!("Outputs count:\t\t\t{}", header.output_count());
    println!("Milestone diffs count:\t\t{}", header.milestone_diff_count());
    println!(
        "Treasury output milestone ID:\t{}",
        header.treasury_output_milestone_id()
    );
    println!("Treasury output amount:\t\t{}", header.treasury_output_amount());
}

pub fn print_delta_header(header: DeltaSnapshotHeader) {
    println!("SEP count:\t\t\t{}", header.sep_count());
    println!("Milestone diffs count:\t\t{}", header.milestone_diff_count());
}

pub fn exec(tool: &SnapshotInfoTool) -> Result<(), SnapshotInfoError> {
    let mut reader = BufReader::new(OpenOptions::new().read(true).open(Path::new(&tool.path))?);

    let header = SnapshotHeader::unpack(&mut reader)?;

    println!("Type:\t\t\t\t{:?}", header.kind());
    println!(
        "Timestamp:\t\t\t{} ({})",
        header.timestamp(),
        Utc.timestamp(header.timestamp() as i64, 0).format("%d-%m-%Y %H:%M:%S")
    );
    println!("Network ID:\t\t\t{}", header.network_id());
    println!("SEP index:\t\t\t{}", *header.sep_index());
    println!("Ledger index:\t\t\t{}", *header.ledger_index());

    match header.kind() {
        SnapshotKind::Full => print_full_header(FullSnapshotHeader::unpack(&mut reader)?),
        SnapshotKind::Delta => print_delta_header(DeltaSnapshotHeader::unpack(&mut reader)?),
    };

    Ok(())
}
