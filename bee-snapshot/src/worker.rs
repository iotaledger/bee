// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::SnapshotConfig, download::download_snapshot_file, error::Error, info::SnapshotInfo, kind::Kind,
    snapshot::Snapshot, storage::StorageBackend,
};

use bee_common_pt2::{node::Node, worker::Worker};
use bee_ledger::model::LedgerIndex;
use bee_storage::access::{Fetch, Insert, Truncate};
use bee_tangle::{milestone::MilestoneIndex, solid_entry_point::SolidEntryPoint};

use async_trait::async_trait;
use chrono::{offset::TimeZone, Utc};
use log::info;

use std::path::Path;

pub struct SnapshotWorker {}

#[async_trait]
impl<N> Worker<N> for SnapshotWorker
where
    N: Node,
    N::Backend: StorageBackend,
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

async fn import_snapshot<N>(
    node: &mut N,
    storage: &N::Backend,
    kind: Kind,
    path: &Path,
    network_id: u64,
) -> Result<Snapshot, Error>
where
    N: Node,
    N::Backend: StorageBackend,
{
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
        *snapshot.header().sep_index(),
        *snapshot.header().ledger_index(),
        snapshot.solid_entry_points().len(),
        match snapshot.header().kind() {
            Kind::Full=> format!(", {} outputs", snapshot.outputs_len()),
            Kind::Delta=> "".to_owned()
        },
        snapshot.milestone_diffs_len()
    );

    Insert::<(), LedgerIndex>::insert(storage, &(), &LedgerIndex::new(snapshot.header().ledger_index()))
        .await
        .map_err(|e| Error::StorageBackend(Box::new(e)))?;

    Truncate::<SolidEntryPoint, MilestoneIndex>::truncate(storage)
        .await
        .map_err(|e| Error::StorageBackend(Box::new(e)))?;

    for sep in snapshot.solid_entry_points.iter() {
        Insert::<SolidEntryPoint, MilestoneIndex>::insert(storage, &sep, &snapshot.header().sep_index())
            .await
            .map_err(|e| Error::StorageBackend(Box::new(e)))?;
    }

    node.register_resource(SnapshotInfo::new(
        snapshot.header().network_id(),
        snapshot.header().sep_index(),
        snapshot.header().sep_index(),
        snapshot.header().sep_index(),
        snapshot.header().timestamp(),
    ));

    Ok(snapshot)
}

async fn import_snapshots<N>(
    node: &mut N,
    storage: &N::Backend,
    network_id: u64,
    config: &SnapshotConfig,
) -> Result<(), Error>
where
    N: Node,
    N::Backend: StorageBackend,
{
    let full_exists = config.full_path().exists();
    let delta_exists = config.delta_path().exists();

    if !full_exists && delta_exists {
        return Err(Error::OnlyDeltaFileExists);
    } else if !full_exists && !delta_exists {
        download_snapshot_file(config.full_path(), config.download_urls()).await?;
        download_snapshot_file(config.delta_path(), config.download_urls()).await?;
    }

    let _full_snapshot = import_snapshot::<N>(node, storage, Kind::Full, config.full_path(), network_id).await?;

    // Load delta file only if both full and delta files already existed or if they have just been downloaded.
    if (full_exists && delta_exists) || (!full_exists && !delta_exists) {
        let _delta_snapshot = import_snapshot::<N>(node, storage, Kind::Delta, config.delta_path(), network_id).await?;
    }

    Ok(())
}
