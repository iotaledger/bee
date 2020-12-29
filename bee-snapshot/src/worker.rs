// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{config::SnapshotConfig, download::download_snapshot_file, error::Error, snapshot::Snapshot};

use bee_common_pt2::{node::Node, worker::Worker};

use async_trait::async_trait;
use chrono::{offset::TimeZone, Utc};
use log::info;

pub struct SnapshotWorker {}

#[async_trait]
impl<N: Node> Worker<N> for SnapshotWorker {
    type Config = (u64, SnapshotConfig);
    type Error = Error;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (network_id, config) = (config.0, config.1);

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

        node.register_resource(full_snapshot.header().clone());
        node.register_resource(full_snapshot.solid_entry_points().clone());

        Ok(Self {})
    }
}
