// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::SnapshotConfig, download::download_snapshot_file, error::Error, header::SnapshotHeader, info::SnapshotInfo,
    kind::Kind, milestone_diff::MilestoneDiff, storage::StorageBackend,
};

use bee_common::packable::Packable;
use bee_message::{
    ledger_index::LedgerIndex,
    milestone::MilestoneIndex,
    output::{CreatedOutput, Output, OutputId},
    solid_entry_point::SolidEntryPoint,
    MessageId,
};
use bee_runtime::{node::Node, resource::ResourceHandle, worker::Worker};
use bee_storage::access::{Fetch, Insert};

use async_trait::async_trait;
use chrono::{offset::TimeZone, Utc};
use log::info;
use tokio::task;

use std::{fs::OpenOptions, io::BufReader, path::Path};

pub struct SnapshotWorker {
    pub full_sep_rx: flume::Receiver<(SolidEntryPoint, MilestoneIndex)>,
    pub delta_sep_rx: flume::Receiver<(SolidEntryPoint, MilestoneIndex)>,
    pub output_rx: flume::Receiver<(OutputId, CreatedOutput)>,
    pub full_diff_rx: flume::Receiver<MilestoneDiff>,
    pub delta_diff_rx: flume::Receiver<MilestoneDiff>,
}

#[async_trait]
impl<N> Worker<N> for SnapshotWorker
where
    N: Node,
    N::Backend: StorageBackend,
{
    type Config = (u64, SnapshotConfig);
    type Error = Error;

    async fn start(node: &mut N, config: Self::Config) -> Result<Self, Self::Error> {
        let (full_sep_tx, full_sep_rx) = flume::unbounded();
        let (delta_sep_tx, delta_sep_rx) = flume::unbounded();
        let (output_tx, output_rx) = flume::unbounded();
        let (full_diff_tx, full_diff_rx) = flume::unbounded();
        let (delta_diff_tx, delta_diff_rx) = flume::unbounded();

        let (network_id, config) = (config.0, config.1);
        let storage = node.storage();

        match Fetch::<(), SnapshotInfo>::fetch(&*storage, &()).await {
            Ok(None) => {
                task::spawn(async move {
                    // TODO handle error
                    import_snapshots(
                        storage,
                        network_id,
                        config,
                        full_sep_tx,
                        delta_sep_tx,
                        output_tx,
                        full_diff_tx,
                        delta_diff_tx,
                    )
                    .await
                });
            }
            Ok(Some(info)) => {
                if info.network_id() != network_id {
                    return Err(Error::NetworkIdMismatch(info.network_id(), network_id));
                }

                info!(
                    "Loaded snapshot from {} with snapshot index {}, entry point index {} and pruning index {}.",
                    Utc.timestamp(info.timestamp() as i64, 0).format("%d-%m-%Y %H:%M:%S"),
                    *info.snapshot_index(),
                    *info.entry_point_index(),
                    *info.pruning_index(),
                );
            }
            Err(e) => return Err(Error::StorageBackend(Box::new(e))),
        }

        Ok(Self {
            full_sep_rx,
            delta_sep_rx,
            output_rx,
            full_diff_rx,
            delta_diff_rx,
        })
    }
}

async fn import_snapshot<B: StorageBackend>(
    storage: &B,
    kind: Kind,
    path: &Path,
    network_id: u64,
    sep_tx: flume::Sender<(SolidEntryPoint, MilestoneIndex)>,
    output_tx: Option<flume::Sender<(OutputId, CreatedOutput)>>,
    diff_tx: flume::Sender<MilestoneDiff>,
) -> Result<(), Error> {
    let kind_str = format!("{:?}", kind).to_lowercase();

    info!("Importing {} snapshot file {}...", kind_str, &path.to_string_lossy());

    let mut reader = BufReader::new(OpenOptions::new().read(true).open(path).map_err(Error::Io)?);

    let header = SnapshotHeader::unpack(&mut reader)?;

    if kind != header.kind() {
        return Err(Error::UnexpectedKind(kind, header.kind()));
    }

    if header.network_id() != network_id {
        return Err(Error::NetworkIdMismatch(header.network_id(), network_id));
    }

    match header.kind() {
        Kind::Full => {
            if header.ledger_index() < header.sep_index() {
                return Err(Error::LedgerSepIndexesInconsistency(
                    header.ledger_index(),
                    header.sep_index(),
                ));
            }
            if (*(header.ledger_index() - header.sep_index())) as usize != header.milestone_diff_count() as usize {
                return Err(Error::InvalidMilestoneDiffsCount(
                    (*(header.ledger_index() - header.sep_index())) as usize,
                    header.milestone_diff_count() as usize,
                ));
            }
        }
        Kind::Delta => {
            if header.sep_index() < header.ledger_index() {
                return Err(Error::LedgerSepIndexesInconsistency(
                    header.ledger_index(),
                    header.sep_index(),
                ));
            }
            if (*(header.sep_index() - header.ledger_index())) as usize != header.milestone_diff_count() as usize {
                return Err(Error::InvalidMilestoneDiffsCount(
                    (*(header.sep_index() - header.ledger_index())) as usize,
                    header.milestone_diff_count() as usize,
                ));
            }
        }
    }

    Insert::<(), LedgerIndex>::insert(storage, &(), &LedgerIndex::new(header.ledger_index()))
        .await
        .map_err(|e| Error::StorageBackend(Box::new(e)))?;

    Insert::<(), SnapshotInfo>::insert(
        &*storage,
        &(),
        &SnapshotInfo::new(
            header.network_id(),
            header.sep_index(),
            header.sep_index(),
            header.sep_index(),
            header.timestamp(),
        ),
    )
    .await
    .map_err(|e| Error::StorageBackend(Box::new(e)))?;

    for _ in 0..header.sep_count() {
        let _ = sep_tx
            .send_async((SolidEntryPoint::unpack(&mut reader)?, header.sep_index()))
            .await;
    }

    if header.kind() == Kind::Full {
        let output_tx = output_tx.unwrap();
        for _ in 0..header.output_count() {
            let message_id = MessageId::unpack(&mut reader)?;
            let output_id = OutputId::unpack(&mut reader)?;
            let output = Output::unpack(&mut reader)?;

            if !matches!(
                output,
                Output::SignatureLockedSingle(_) | Output::SignatureLockedDustAllowance(_),
            ) {
                return Err(Error::UnsupportedOutputKind(output.kind()));
            }

            let _ = output_tx
                .send_async((output_id, CreatedOutput::new(message_id, output)))
                .await;
        }
    }

    for _ in 0..header.milestone_diff_count() {
        let _ = diff_tx.send_async(MilestoneDiff::unpack(&mut reader)?).await;
    }

    // TODO check nothing left

    info!(
        "Imported {} snapshot file from {} with sep index {}, ledger index {}, {} solid entry points{} and {} milestone diffs.",
        kind_str,
        Utc.timestamp(header.timestamp() as i64, 0)
            .format("%d-%m-%Y %H:%M:%S"),
        *header.sep_index(),
        *header.ledger_index(),
        header.sep_count(),
        match header.kind() {
            Kind::Full=> format!(", {} outputs", header.output_count()),
            Kind::Delta=> "".to_owned()
        },
        header.milestone_diff_count()
    );

    Ok(())
}

async fn import_snapshots<B: StorageBackend>(
    storage: ResourceHandle<B>,
    network_id: u64,
    config: SnapshotConfig,
    full_sep_tx: flume::Sender<(SolidEntryPoint, MilestoneIndex)>,
    delta_sep_tx: flume::Sender<(SolidEntryPoint, MilestoneIndex)>,
    output_tx: flume::Sender<(OutputId, CreatedOutput)>,
    full_diff_tx: flume::Sender<MilestoneDiff>,
    delta_diff_tx: flume::Sender<MilestoneDiff>,
) -> Result<(), Error> {
    let full_exists = config.full_path().exists();
    let delta_exists = config.delta_path().exists();

    if !full_exists && delta_exists {
        return Err(Error::OnlyDeltaFileExists);
    } else if !full_exists && !delta_exists {
        download_snapshot_file(config.full_path(), config.download_urls()).await?;
        download_snapshot_file(config.delta_path(), config.download_urls()).await?;
    }

    import_snapshot(
        &*storage,
        Kind::Full,
        config.full_path(),
        network_id,
        full_sep_tx,
        Some(output_tx),
        full_diff_tx,
    )
    .await?;

    // Load delta file only if both full and delta files already existed or if they have just been downloaded.
    if (full_exists && delta_exists) || (!full_exists && !delta_exists) {
        import_snapshot(
            &*storage,
            Kind::Delta,
            config.delta_path(),
            network_id,
            delta_sep_tx,
            None,
            delta_diff_tx,
        )
        .await?;
    }

    Ok(())
}
