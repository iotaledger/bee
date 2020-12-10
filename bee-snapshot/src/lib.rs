// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod download;

pub(crate) mod constants;
pub(crate) mod kind;
pub(crate) mod pruning;
// pub(crate) mod worker;

pub mod config;
pub mod error;
pub mod event;
pub mod header;
pub mod milestone_diff;
pub mod output;
pub mod snapshot;
pub mod spent;

pub(crate) use download::download_snapshot_files;

pub use error::Error;
pub use header::SnapshotHeader;
pub use snapshot::Snapshot;

use bee_common::node::Node;
// use bee_protocol::{event::LatestSolidMilestoneChanged, MilestoneIndex};

use chrono::{offset::TimeZone, Utc};
use log::info;

use std::path::Path;

// TODO change return type

pub async fn init<N: Node>(
    // tangle: &MsTangle<B>,
    config: &config::SnapshotConfig,
    node_builder: N::Builder,
) -> Result<(N::Builder, Snapshot), Error> {
    if !Path::new(config.full_path()).exists() || !Path::new(config.delta_path()).exists() {
        download_snapshot_files(config).await?;
    }

    info!("Loading snapshot full file {}...", config.full_path());
    let snapshot_full = Snapshot::from_file(config.full_path())?;
    info!(
        "Loaded snapshot full file from {} with {} solid entry points.",
        Utc.timestamp(snapshot_full.header().timestamp() as i64, 0).to_rfc2822(),
        snapshot_full.solid_entry_points().len(),
    );

    info!("Loading snapshot delta file {}...", config.delta_path());
    let snapshot_delta = Snapshot::from_file(config.delta_path())?;
    info!(
        "Loaded snapshot delta file from {} with {} solid entry points.",
        Utc.timestamp(snapshot_delta.header().timestamp() as i64, 0)
            .to_rfc2822(),
        snapshot_delta.solid_entry_points().len(),
    );

    // The genesis transaction must be marked as SEP with snapshot index during loading a snapshot because coordinator
    // bootstraps the network by referencing the genesis tx.
    // snapshot.solid_entry_points().insert(MessageId::null());

    // node_builder = node_builder.with_worker_cfg::<worker::SnapshotWorker>(config.clone());

    Ok((node_builder, snapshot_full))
}

pub fn events<N: Node>(_node: &N) {
    // let snapshot_worker = node.worker::<worker::SnapshotWorker>().unwrap().tx.clone();
    //
    // node.resource::<Bus>().add_listener(move |latest_solid_milestone: &LatestSolidMilestoneChanged| {
    //     if let Err(e) = snapshot_worker.send(worker::SnapshotWorkerEvent(latest_solid_milestone.0.clone())) {
    //         warn!(
    //             "Failed to send milestone {} to snapshot worker: {:?}.",
    //             *latest_solid_milestone.0.index(),
    //             e
    //         )
    //     }
    // });
}
