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

pub(crate) use download::download_snapshot;

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
    if !Path::new(config.path()).exists() {
        download_snapshot(config).await?;
    }
    info!("Loading snapshot file {}...", config.path());

    let snapshot = Snapshot::from_file(config.path())?;

    info!(
        "Loaded snapshot file from {} with {} solid entry points.",
        Utc.timestamp(snapshot.header().timestamp() as i64, 0).to_rfc2822(),
        snapshot.solid_entry_points().len(),
    );

    // The genesis transaction must be marked as SEP with snapshot index during loading a snapshot because coordinator
    // bootstraps the network by referencing the genesis tx.
    // snapshot.solid_entry_points().insert(MessageId::null());

    // node_builder = node_builder.with_worker_cfg::<worker::SnapshotWorker>(config.clone());

    Ok((node_builder, snapshot))
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
