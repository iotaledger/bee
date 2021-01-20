// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::SnapshotConfig, download::download_snapshot_file, error::Error, info::SnapshotInfo, kind::Kind,
    snapshot::Snapshot, storage::StorageBackend,
};

use bee_ledger::{
    model::Unspent,
    storage::{apply_outputs_diff, check_ledger_state, rollback_outputs_diff},
};
use bee_message::{
    ledger_index::LedgerIndex,
    milestone::MilestoneIndex,
    payload::transaction::{Address, Ed25519Address, Output, OutputId},
    solid_entry_point::SolidEntryPoint,
};
use bee_runtime::{node::Node, worker::Worker};
use bee_storage::access::{Fetch, Insert, Truncate};

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
            Ok(None) => import_snapshots(&*storage, network_id, &config).await?,
            Err(e) => return Err(Error::StorageBackend(Box::new(e))),
        }

        Ok(Self {})
    }
}

async fn import_snapshot<B: StorageBackend>(
    storage: &B,
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
        *snapshot.header().sep_index(),
        *snapshot.header().ledger_index(),
        snapshot.solid_entry_points().len(),
        match snapshot.header().kind() {
            Kind::Full=> format!(", {} outputs", snapshot.outputs().len()),
            Kind::Delta=> "".to_owned()
        },
        snapshot.milestone_diffs().len()
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

    if snapshot.header().kind() == Kind::Full {
        // TODO unify this with ledger crate
        for (output_id, output) in snapshot.outputs().iter() {
            // TODO group them 3 in a function
            Insert::<OutputId, bee_ledger::model::Output>::insert(storage, output_id, &output)
                .await
                .map_err(|e| Error::StorageBackend(Box::new(e)))?;
            Insert::<Unspent, ()>::insert(storage, &((*output_id).into()), &())
                .await
                .map_err(|e| Error::StorageBackend(Box::new(e)))?;
            match output.inner() {
                Output::SignatureLockedSingle(output) => {
                    if let Address::Ed25519(address) = output.address() {
                        Insert::<(Ed25519Address, OutputId), ()>::insert(storage, &(address.clone(), *output_id), &())
                            .await
                            .map_err(|e| Error::StorageBackend(Box::new(e)))?;
                    }
                }
                Output::SignatureLockedDustAllowance(output) => {
                    if let Address::Ed25519(address) = output.address() {
                        Insert::<(Ed25519Address, OutputId), ()>::insert(storage, &(address.clone(), *output_id), &())
                            .await
                            .map_err(|e| Error::StorageBackend(Box::new(e)))?;
                    }
                }
                // TODO
                _ => panic!("Unhandled output type"),
            }
        }
    }

    for diff in snapshot.milestone_diffs() {
        let index = diff.index();
        // Unwrap is fine because we just inserted the ledger index.
        let ledger_index = Fetch::<(), LedgerIndex>::fetch(storage, &())
            .await
            .map_err(|e| Error::StorageBackend(Box::new(e)))?
            .unwrap();

        match index {
            MilestoneIndex(index) if index == *ledger_index + 1 => {
                // TODO unwrap until we merge both crates
                apply_outputs_diff(storage, MilestoneIndex(index), diff.created(), diff.consumed())
                    .await
                    .unwrap();
            }
            MilestoneIndex(index) if index == *ledger_index => {
                // TODO unwrap until we merge both crates
                rollback_outputs_diff(storage, MilestoneIndex(index), diff.created(), diff.consumed())
                    .await
                    .unwrap();
            }
            _ => return Err(Error::UnexpectedDiffIndex(index)),
        }
    }

    // TODO unwrap
    if !check_ledger_state(storage).await.unwrap() {
        return Err(Error::InvalidLedgerState);
    }

    Ok(snapshot)
}

async fn import_snapshots<B: StorageBackend>(
    storage: &B,
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

    let mut snapshot = import_snapshot(storage, Kind::Full, config.full_path(), network_id).await?;

    // Load delta file only if both full and delta files already existed or if they have just been downloaded.
    if (full_exists && delta_exists) || (!full_exists && !delta_exists) {
        snapshot = import_snapshot(storage, Kind::Delta, config.delta_path(), network_id).await?;
    }

    Insert::<(), SnapshotInfo>::insert(
        storage,
        &(),
        &SnapshotInfo::new(
            snapshot.header().network_id(),
            snapshot.header().sep_index(),
            snapshot.header().sep_index(),
            snapshot.header().sep_index(),
            snapshot.header().timestamp(),
        ),
    )
    .await
    .map_err(|e| Error::StorageBackend(Box::new(e)))?;

    Ok(())
}
