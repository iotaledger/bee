// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::workers::{
    error::Error,
    snapshot::{config::SnapshotConfig, error::Error as SnapshotError, import::import_snapshots},
    storage::{self, StorageBackend},
};

use bee_message::milestone::MilestoneIndex;
use bee_runtime::{node::Node, worker::Worker};
use bee_storage::{access::AsStream, backend::StorageBackend as _, system::StorageHealth};
use bee_tangle::{solid_entry_point::SolidEntryPoint, MsTangle, TangleWorker};

use async_trait::async_trait;

use chrono::{offset::TimeZone, Utc};
use futures::stream::{StreamExt, TryStreamExt};
use log::info;

use std::{any::TypeId, collections::HashMap};

pub struct SnapshotWorker {}

#[async_trait]
impl<N: Node> Worker<N> for SnapshotWorker
where
    N::Backend: StorageBackend,
{
    type Config = (u64, SnapshotConfig);
    type Error = Error;

    fn dependencies() -> &'static [TypeId] {
        vec![TypeId::of::<TangleWorker>()].leak()
    }

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (network_id, snapshot_config) = config;
        let tangle = node.resource::<MsTangle<N::Backend>>();
        let storage = node.storage();

        if let Some(info) = storage::fetch_snapshot_info(&*storage).await? {
            if info.network_id() != network_id {
                return Err(Error::Snapshot(SnapshotError::NetworkIdMismatch(
                    info.network_id(),
                    network_id,
                )));
            }

            info!(
                "Loaded snapshot from {} with snapshot index {}, entry point index {} and pruning index {}.",
                Utc.timestamp(info.timestamp() as i64, 0).format("%d-%m-%Y %H:%M:%S"),
                *info.snapshot_index(),
                *info.entry_point_index(),
                *info.pruning_index(),
            );
        } else if let Err(e) = import_snapshots(&*storage, network_id, &snapshot_config).await {
            (*storage)
                .set_health(StorageHealth::Corrupted)
                .await
                .map_err(|e| Error::Storage(Box::new(e)))?;
            return Err(e);
        }

        let solid_entry_points = AsStream::<SolidEntryPoint, MilestoneIndex>::stream(&*storage)
            .await
            .map_err(|e| Error::Storage(Box::new(e)))?
            .map(|result| result.map_err(|e| Error::Storage(Box::new(e))))
            .try_collect::<HashMap<SolidEntryPoint, MilestoneIndex>>()
            .await?;
        // Unwrap is fine because ledger index was either just inserted or already present in storage.
        let ledger_index = MilestoneIndex(*storage::fetch_ledger_index(&*storage).await?.unwrap());
        // Unwrap is fine because snapshot info was either just inserted or already present in storage.
        let snapshot_info = storage::fetch_snapshot_info(&*storage).await?.unwrap();

        tangle.replace_solid_entry_points(solid_entry_points).await;
        tangle.update_snapshot_index(snapshot_info.snapshot_index());
        tangle.update_pruning_index(snapshot_info.pruning_index());
        tangle.update_solid_milestone_index(ledger_index);
        tangle.update_confirmed_milestone_index(ledger_index);
        tangle.update_latest_milestone_index(ledger_index);

        Ok(Self {})
    }
}
