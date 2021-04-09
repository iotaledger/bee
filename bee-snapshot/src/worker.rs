// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    config::SnapshotConfig,
    download::download_snapshot_file,
    error::Error,
    header::{DeltaSnapshotHeader, FullSnapshotHeader, SnapshotHeader},
    info::SnapshotInfo,
    kind::Kind,
    milestone_diff::MilestoneDiff,
    storage::StorageBackend,
};

use bee_common::packable::{Packable, Read};
use bee_message::{
    ledger_index::LedgerIndex,
    milestone::MilestoneIndex,
    output::{CreatedOutput, Output, OutputId},
    payload::milestone::MilestoneId,
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
    pub treasury_output_rx: flume::Receiver<(MilestoneId, u64)>,
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
        let (treasury_output_tx, treasury_output_rx) = flume::unbounded();
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
                        treasury_output_tx,
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
            treasury_output_rx,
            full_sep_rx,
            delta_sep_rx,
            output_rx,
            full_diff_rx,
            delta_diff_rx,
        })
    }
}

async fn store_ledger_index<B: StorageBackend>(storage: &B, ledger_index: MilestoneIndex) -> Result<(), Error> {
    Insert::<(), LedgerIndex>::insert(storage, &(), &LedgerIndex::new(ledger_index))
        .await
        .map_err(|e| Error::StorageBackend(Box::new(e)))
}

async fn store_snapshot_info<B: StorageBackend>(
    storage: &B,
    network_id: u64,
    sep_index: MilestoneIndex,
    timestamp: u64,
) -> Result<(), Error> {
    Insert::<(), SnapshotInfo>::insert(
        &*storage,
        &(),
        &SnapshotInfo::new(network_id, sep_index, sep_index, sep_index, timestamp),
    )
    .await
    .map_err(|e| Error::StorageBackend(Box::new(e)))
}

async fn import_seps<R: Read>(
    reader: &mut R,
    sep_count: u64,
    sep_index: MilestoneIndex,
    sep_tx: flume::Sender<(SolidEntryPoint, MilestoneIndex)>,
) -> Result<(), Error> {
    for _ in 0..sep_count {
        let _ = sep_tx.send_async((SolidEntryPoint::unpack(reader)?, sep_index)).await;
    }

    Ok(())
}

async fn import_outputs<R: Read>(
    reader: &mut R,
    output_count: u64,
    output_tx: flume::Sender<(OutputId, CreatedOutput)>,
) -> Result<(), Error> {
    for _ in 0..output_count {
        let message_id = MessageId::unpack(reader)?;
        let output_id = OutputId::unpack(reader)?;
        let output = Output::unpack(reader)?;

        let _ = output_tx
            .send_async((output_id, CreatedOutput::new(message_id, output)))
            .await;
    }

    Ok(())
}

async fn import_milestone_diffs<R: Read>(
    reader: &mut R,
    milestone_diff_count: u64,
    diff_tx: flume::Sender<MilestoneDiff>,
) -> Result<(), Error> {
    for _ in 0..milestone_diff_count {
        let _ = diff_tx.send_async(MilestoneDiff::unpack(reader)?).await;
    }

    Ok(())
}

async fn import_full_snapshot<B: StorageBackend>(
    storage: &B,
    path: &Path,
    network_id: u64,
    treasury_output_tx: flume::Sender<(MilestoneId, u64)>,
    sep_tx: flume::Sender<(SolidEntryPoint, MilestoneIndex)>,
    output_tx: flume::Sender<(OutputId, CreatedOutput)>,
    diff_tx: flume::Sender<MilestoneDiff>,
) -> Result<(), Error> {
    info!("Importing full snapshot file {}...", &path.to_string_lossy());

    let mut reader = BufReader::new(OpenOptions::new().read(true).open(path).map_err(Error::Io)?);

    let header = SnapshotHeader::unpack(&mut reader)?;

    if header.kind() != Kind::Full {
        return Err(Error::UnexpectedKind(Kind::Full, header.kind()));
    }

    if header.network_id() != network_id {
        return Err(Error::NetworkIdMismatch(header.network_id(), network_id));
    }

    let full_header = FullSnapshotHeader::unpack(&mut reader)?;

    if header.ledger_index() < header.sep_index() {
        return Err(Error::LedgerSepIndexesInconsistency(
            header.ledger_index(),
            header.sep_index(),
        ));
    }
    if (*(header.ledger_index() - header.sep_index())) as usize != full_header.milestone_diff_count() as usize {
        return Err(Error::InvalidMilestoneDiffsCount(
            (*(header.ledger_index() - header.sep_index())) as usize,
            full_header.milestone_diff_count() as usize,
        ));
    }

    store_ledger_index(storage, header.ledger_index()).await?;
    store_snapshot_info(storage, network_id, header.sep_index(), header.timestamp()).await?;

    let _ = treasury_output_tx
        .send_async((
            *full_header.treasury_output_milestone_id(),
            full_header.treasury_output_amount(),
        ))
        .await;

    import_seps(&mut reader, full_header.sep_count(), header.sep_index(), sep_tx).await?;
    import_outputs(&mut reader, full_header.output_count(), output_tx).await?;
    import_milestone_diffs(&mut reader, full_header.milestone_diff_count(), diff_tx).await?;

    // TODO check nothing left

    info!(
        "Imported full snapshot file from {} with sep index {}, ledger index {}, {} solid entry points, {} outputs and {} milestone diffs.",
        Utc.timestamp(header.timestamp() as i64, 0).format("%d-%m-%Y %H:%M:%S"),
        *header.sep_index(),
        *header.ledger_index(),
        full_header.sep_count(),
        full_header.output_count(),
        full_header.milestone_diff_count()
    );

    Ok(())
}

async fn import_delta_snapshot<B: StorageBackend>(
    storage: &B,
    path: &Path,
    network_id: u64,
    sep_tx: flume::Sender<(SolidEntryPoint, MilestoneIndex)>,
    diff_tx: flume::Sender<MilestoneDiff>,
) -> Result<(), Error> {
    info!("Importing delta snapshot file {}...", &path.to_string_lossy());

    let mut reader = BufReader::new(OpenOptions::new().read(true).open(path).map_err(Error::Io)?);

    let header = SnapshotHeader::unpack(&mut reader)?;

    if header.kind() != Kind::Delta {
        return Err(Error::UnexpectedKind(Kind::Delta, header.kind()));
    }

    if header.network_id() != network_id {
        return Err(Error::NetworkIdMismatch(header.network_id(), network_id));
    }

    let delta_header = DeltaSnapshotHeader::unpack(&mut reader)?;

    if header.sep_index() < header.ledger_index() {
        return Err(Error::LedgerSepIndexesInconsistency(
            header.ledger_index(),
            header.sep_index(),
        ));
    }
    if (*(header.sep_index() - header.ledger_index())) as usize != delta_header.milestone_diff_count() as usize {
        return Err(Error::InvalidMilestoneDiffsCount(
            (*(header.sep_index() - header.ledger_index())) as usize,
            delta_header.milestone_diff_count() as usize,
        ));
    }

    store_ledger_index(storage, header.ledger_index()).await?;
    store_snapshot_info(storage, network_id, header.sep_index(), header.timestamp()).await?;

    import_seps(&mut reader, delta_header.sep_count(), header.sep_index(), sep_tx).await?;
    import_milestone_diffs(&mut reader, delta_header.milestone_diff_count(), diff_tx).await?;

    // TODO check nothing left

    info!(
        "Imported delta snapshot file from {} with sep index {}, ledger index {}, {} solid entry points and {} milestone diffs.",
        Utc.timestamp(header.timestamp() as i64, 0).format("%d-%m-%Y %H:%M:%S"),
        *header.sep_index(),
        *header.ledger_index(),
        delta_header.sep_count(),
        delta_header.milestone_diff_count()
    );

    Ok(())
}

async fn import_snapshots<B: StorageBackend>(
    storage: ResourceHandle<B>,
    network_id: u64,
    config: SnapshotConfig,
    treasury_output_tx: flume::Sender<(MilestoneId, u64)>,
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

    import_full_snapshot(
        &*storage,
        config.full_path(),
        network_id,
        treasury_output_tx,
        full_sep_tx,
        output_tx,
        full_diff_tx,
    )
    .await?;

    // Load delta file only if both full and delta files already existed or if they have just been downloaded.
    if (full_exists && delta_exists) || (!full_exists && !delta_exists) {
        import_delta_snapshot(&*storage, config.delta_path(), network_id, delta_sep_tx, delta_diff_tx).await?;
    }

    Ok(())
}
