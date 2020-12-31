// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use bee_snapshot::snapshot::Snapshot;

use chrono::{offset::TimeZone, Utc};
use structopt::StructOpt;

use std::path::Path;

#[derive(Debug, StructOpt)]
pub struct SnapshotInfo {
    path: String,
}

pub fn exec(tool: &SnapshotInfo) {
    let snapshot = Snapshot::from_file(Path::new(&tool.path)).unwrap();

    println!("Type:\t\t\t{:?}", snapshot.header().kind());
    println!(
        "Timestamp:\t\t{} ({})",
        snapshot.header().timestamp(),
        Utc.timestamp(snapshot.header().timestamp() as i64, 0)
            .format("%d-%m-%Y %H:%M:%S")
    );
    println!("Network ID:\t\t{}", snapshot.header().network_id());
    println!("SEP index:\t\t{}", snapshot.header().sep_index());
    println!("Ledger index:\t\t{}", snapshot.header().ledger_index());
    println!("SEP count:\t\t{}", snapshot.solid_entry_points().len());
    println!("Outputs count:\t\t{}", snapshot.outputs_len());
    println!("Milestone diffs count:\t{}", snapshot.milestone_diffs_len());
}
