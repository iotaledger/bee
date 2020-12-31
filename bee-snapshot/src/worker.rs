// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::SnapshotConfig, download::download_snapshot_file, error::Error, info::SnapshotInfo, kind::Kind,
    snapshot::Snapshot, storage::Backend,
};

use bee_common_pt2::{node::Node, worker::Worker};
use bee_storage::access::Fetch;

use async_trait::async_trait;
use chrono::{offset::TimeZone, Utc};
use log::info;

use std::path::Path;

pub struct SnapshotWorker {}

#[async_trait]
impl<N: Node> Worker<N> for SnapshotWorker
where
    N::Backend: Backend,
{
    type Config = (u64, SnapshotConfig);
    type Error = Error;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (network_id, config) = (config.0, config.1);
        let storage = node.storage();

        match Fetch::<(), SnapshotInfo>::fetch(&*storage, &()).await {
            Ok(Some(_info)) => {}
            Ok(None) => import_snapshots(node, &*storage, network_id, &config).await?,
            Err(e) => return Err(Error::StorageBackend(Box::new(e))),
        }

        Ok(Self {})
    }
}

fn import_snapshot<N: Node>(
    _storage: &N::Backend,
    kind: Kind,
    path: &Path,
    network_id: u64,
) -> Result<Snapshot, Error> {
    let kind_str = format!("{:?}", kind).to_lowercase();

    info!("Importing {} snapshot file {}...", kind_str, &path.to_string_lossy());

    let snapshot = Snapshot::from_file(path)?;

    if snapshot.header().kind() != kind {
        return Err(Error::InvalidKind(kind, snapshot.header().kind()));
    }

    if snapshot.header().network_id() != network_id {
        return Err(Error::NetworkIdMismatch(network_id, snapshot.header().network_id()));
    }

    info!(
        "Imported {} snapshot file from {} with sep index {}, ledger index {}, {} solid entry points{} and {} milestone diffs.",
        kind_str,
        Utc.timestamp(snapshot.header().timestamp() as i64, 0)
            .format("%d-%m-%Y %H:%M:%S"),
        snapshot.header().sep_index(),
        snapshot.header().ledger_index(),
        snapshot.solid_entry_points().len(),
        match snapshot.header().kind() {
            Kind::Full=> format!(", {} outputs", snapshot.outputs_len()),
            Kind::Delta=> "".to_owned()
        },
        snapshot.milestone_diffs_len()
    );

    Ok(snapshot)
}

async fn import_snapshots<N: Node>(
    node: &mut N,
    storage: &N::Backend,
    network_id: u64,
    config: &SnapshotConfig,
) -> Result<(), Error> {
    let full_exists = config.full_path().exists();
    let delta_exists = config.delta_path().exists();

    if !full_exists && delta_exists {
        return Err(Error::OnlyDeltaFileExists);
    } else if !full_exists && !delta_exists {
        download_snapshot_file(config.full_path(), config.download_urls()).await?;
        download_snapshot_file(config.delta_path(), config.download_urls()).await?;
    }

    let full_snapshot = import_snapshot::<N>(storage, Kind::Full, config.full_path(), network_id)?;

    // Load delta file only if both full and delta files already existed or if they have just been downloaded.
    if (full_exists && delta_exists) || (!full_exists && !delta_exists) {
        let _delta_snapshot = import_snapshot::<N>(storage, Kind::Delta, config.delta_path(), network_id)?;
    }

    node.register_resource(SnapshotInfo::new(
        full_snapshot.header().network_id(),
        full_snapshot.header().sep_index(),
        full_snapshot.header().sep_index(),
        full_snapshot.header().sep_index(),
        full_snapshot.header().timestamp(),
    ));
    node.register_resource(full_snapshot.solid_entry_points().clone());

    Ok(())
}
