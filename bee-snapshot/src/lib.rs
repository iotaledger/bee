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
pub mod info;
pub mod milestone_diff;
pub mod output;
pub mod snapshot;
pub mod spent;
pub mod storage;

pub(crate) use download::download_snapshot_file;

pub use error::Error;
pub use header::SnapshotHeader;
pub use snapshot::Snapshot;

use bee_common_pt2::node::Node;
// use bee_protocol::{event::LatestSolidMilestoneChanged, MilestoneIndex};

use chrono::{offset::TimeZone, Utc};
use log::info;

// TODO change return type

pub async fn init<N: Node>(
    // tangle: &MsTangle<B>,
    config: &config::SnapshotConfig,
    network_id: u64,
    node_builder: N::Builder,
) -> Result<(N::Builder, Snapshot), Error> {
    let full_exists = config.full_path().exists();
    let delta_exists = config.delta_path().exists();

    if !full_exists && delta_exists {
        return Err(Error::OnlyDeltaFileExists);
    } else if !full_exists && !delta_exists {
        download_snapshot_file(config.full_path(), config.download_urls()).await?;
        download_snapshot_file(config.delta_path(), config.download_urls()).await?;
    }

    info!(
        "Loading full snapshot file {}...",
        &config.full_path().to_string_lossy()
    );
    let full_snapshot = Snapshot::from_file(config.full_path())?;
    info!(
        "Loaded full snapshot file from {} with {} solid entry points.",
        Utc.timestamp(full_snapshot.header().timestamp() as i64, 0).to_rfc2822(),
        full_snapshot.solid_entry_points().len(),
    );

    if full_snapshot.header().network_id() != network_id {
        return Err(Error::NetworkIdMismatch(
            network_id,
            full_snapshot.header().network_id(),
        ));
    }

    // Load delta file only if both full and delta files already existed or if they have just been downloaded.
    if (full_exists && delta_exists) || (!full_exists && !delta_exists) {
        info!(
            "Loading delta snapshot file {}...",
            config.delta_path().to_string_lossy()
        );
        let delta_snapshot = Snapshot::from_file(config.delta_path())?;
        info!(
            "Loaded delta snapshot file from {} with {} solid entry points.",
            Utc.timestamp(delta_snapshot.header().timestamp() as i64, 0)
                .to_rfc2822(),
            delta_snapshot.solid_entry_points().len(),
        );

        if delta_snapshot.header().network_id() != network_id {
            return Err(Error::NetworkIdMismatch(
                network_id,
                delta_snapshot.header().network_id(),
            ));
        }
    }

    // The genesis transaction must be marked as SEP with snapshot index during loading a snapshot because coordinator
    // bootstraps the network by referencing the genesis tx.
    // snapshot.solid_entry_points().insert(MessageId::null());

    // node_builder = node_builder.with_worker_cfg::<worker::SnapshotWorker>(config.clone());

    Ok((node_builder, full_snapshot))
}
