// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{
        snapshot::{
            DeltaSnapshotHeader, FullSnapshotHeader, MilestoneDiff, SnapshotHeader, SnapshotInfo, SnapshotKind,
        },
        BalanceDiffs, CreatedOutput, TreasuryOutput,
    },
    workers::{
        consensus::worker::migration_from_milestone,
        error::Error,
        snapshot::{config::SnapshotConfig, download::download_snapshot_file, error::Error as SnapshotError},
        storage::{self, apply_balance_diffs, apply_milestone, create_output, rollback_milestone, StorageBackend},
    },
};

use bee_common::packable::{Packable, Read};
use bee_message::{
    milestone::MilestoneIndex,
    output::{self, Output, OutputId},
    payload::Payload,
    MessageId,
};
use bee_storage::access::{Insert, Truncate};
use bee_tangle::solid_entry_point::SolidEntryPoint;

use chrono::{offset::TimeZone, Utc};
use log::{info, warn};

use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::BufReader,
    path::Path,
};

fn snapshot_reader(path: &Path) -> Result<BufReader<File>, Error> {
    Ok(BufReader::new(
        OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| Error::Snapshot(SnapshotError::Io(e)))?,
    ))
}

async fn import_solid_entry_points<R: Read, B: StorageBackend>(
    reader: &mut R,
    storage: &B,
    sep_count: u64,
    index: MilestoneIndex,
) -> Result<(), Error> {
    Truncate::<SolidEntryPoint, MilestoneIndex>::truncate(storage)
        .await
        .map_err(|e| Error::Storage(Box::new(e)))?;
    for _ in 0..sep_count {
        Insert::<SolidEntryPoint, MilestoneIndex>::insert(&*storage, &SolidEntryPoint::unpack(reader)?, &index)
            .await
            .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    Ok(())
}

async fn import_outputs<R: Read, B: StorageBackend>(
    reader: &mut R,
    storage: &B,
    output_count: u64,
) -> Result<(), Error> {
    let mut balance_diffs = BalanceDiffs::new();

    for _ in 0..output_count {
        let message_id = MessageId::unpack(reader)?;
        let output_id = OutputId::unpack(reader)?;
        let output = Output::unpack(reader)?;
        let created_output = CreatedOutput::new(message_id, output);

        create_output(&*storage, &output_id, &created_output).await?;
        balance_diffs.output_add(created_output.inner())?;
    }

    apply_balance_diffs(&*storage, &balance_diffs).await
}

async fn import_milestone_diffs<R: Read, B: StorageBackend>(
    reader: &mut R,
    storage: &B,
    milestone_diff_count: u64,
) -> Result<(), Error> {
    for _ in 0..milestone_diff_count {
        let diff = MilestoneDiff::unpack(reader)?;
        let index = diff.milestone().essence().index();
        // Unwrap is fine because ledger index was inserted just before.
        let ledger_index = *storage::fetch_ledger_index(&*storage).await?.unwrap();
        let mut balance_diffs = BalanceDiffs::new();
        let mut consumed = HashMap::new();

        for (_, output) in diff.created().iter() {
            balance_diffs.output_add(output.inner())?;
        }

        for (output_id, (created_output, consumed_output)) in diff.consumed().iter() {
            balance_diffs.output_sub(created_output.inner())?;
            consumed.insert(*output_id, (created_output.clone(), consumed_output.clone()));
        }

        let migration = if let Some(Payload::Receipt(receipt)) = diff.milestone().essence().receipt() {
            let consumed_treasury = diff
                .consumed_treasury()
                .ok_or(Error::Snapshot(SnapshotError::MissingConsumedTreasury))?
                .clone();

            Some(
                migration_from_milestone(
                    index,
                    diff.milestone().id(),
                    receipt,
                    TreasuryOutput::new(consumed_treasury.0, consumed_treasury.1),
                )
                .await?,
            )
        } else {
            None
        };

        if index == MilestoneIndex(ledger_index + 1) {
            apply_milestone(&*storage, index, diff.created(), &consumed, &balance_diffs, &migration).await?;
        } else if index == MilestoneIndex(ledger_index) {
            rollback_milestone(&*storage, index, diff.created(), &consumed, &balance_diffs, &migration).await?;
        } else {
            return Err(Error::Snapshot(SnapshotError::UnexpectedMilestoneDiffIndex(index)));
        }
    }

    Ok(())
}

fn check_header(header: &SnapshotHeader, kind: SnapshotKind, network_id: u64) -> Result<(), Error> {
    if kind != header.kind() {
        Err(Error::Snapshot(SnapshotError::UnexpectedSnapshotKind(
            kind,
            header.kind(),
        )))
    } else if network_id != header.network_id() {
        Err(Error::Snapshot(SnapshotError::NetworkIdMismatch(
            network_id,
            header.network_id(),
        )))
    } else {
        Ok(())
    }
}

async fn import_full_snapshot<B: StorageBackend>(storage: &B, path: &Path, network_id: u64) -> Result<(), Error> {
    info!("Importing full snapshot file {}...", &path.to_string_lossy());

    let mut reader = snapshot_reader(path)?;
    let header = SnapshotHeader::unpack(&mut reader)?;

    check_header(&header, SnapshotKind::Full, network_id)?;

    let full_header = FullSnapshotHeader::unpack(&mut reader)?;

    if header.ledger_index() < header.sep_index() {
        return Err(Error::Snapshot(SnapshotError::LedgerSepIndexesInconsistency(
            header.ledger_index(),
            header.sep_index(),
        )));
    }
    if (*(header.ledger_index() - header.sep_index())) as usize != full_header.milestone_diff_count() as usize {
        return Err(Error::Snapshot(SnapshotError::InvalidMilestoneDiffsCount(
            (*(header.ledger_index() - header.sep_index())) as usize,
            full_header.milestone_diff_count() as usize,
        )));
    }

    storage::insert_treasury_output(
        &*storage,
        &TreasuryOutput::new(
            output::TreasuryOutput::new(full_header.treasury_output_amount())?,
            *full_header.treasury_output_milestone_id(),
        ),
    )
    .await?;

    storage::insert_ledger_index(storage, &header.ledger_index().into()).await?;
    storage::insert_snapshot_info(
        storage,
        &SnapshotInfo::new(
            network_id,
            header.sep_index(),
            header.sep_index(),
            header.sep_index(),
            header.timestamp(),
        ),
    )
    .await?;

    import_solid_entry_points(&mut reader, storage, full_header.sep_count(), header.sep_index()).await?;
    import_outputs(&mut reader, storage, full_header.output_count()).await?;
    import_milestone_diffs(&mut reader, storage, full_header.milestone_diff_count()).await?;

    if reader.bytes().next().is_some() {
        return Err(Error::Snapshot(SnapshotError::RemainingBytes));
    }

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

async fn import_delta_snapshot<B: StorageBackend>(storage: &B, path: &Path, network_id: u64) -> Result<(), Error> {
    info!("Importing delta snapshot file {}...", &path.to_string_lossy());

    let mut reader = snapshot_reader(path)?;
    let header = SnapshotHeader::unpack(&mut reader)?;

    check_header(&header, SnapshotKind::Delta, network_id)?;

    let delta_header = DeltaSnapshotHeader::unpack(&mut reader)?;

    if header.sep_index() < header.ledger_index() {
        return Err(Error::Snapshot(SnapshotError::LedgerSepIndexesInconsistency(
            header.ledger_index(),
            header.sep_index(),
        )));
    }
    if (*(header.sep_index() - header.ledger_index())) as usize != delta_header.milestone_diff_count() as usize {
        return Err(Error::Snapshot(SnapshotError::InvalidMilestoneDiffsCount(
            (*(header.sep_index() - header.ledger_index())) as usize,
            delta_header.milestone_diff_count() as usize,
        )));
    }

    storage::insert_ledger_index(storage, &header.ledger_index().into()).await?;
    storage::insert_snapshot_info(
        storage,
        &SnapshotInfo::new(
            network_id,
            header.sep_index(),
            header.sep_index(),
            header.sep_index(),
            header.timestamp(),
        ),
    )
    .await?;

    import_solid_entry_points(&mut reader, storage, delta_header.sep_count(), header.sep_index()).await?;
    import_milestone_diffs(&mut reader, storage, delta_header.milestone_diff_count()).await?;

    if reader.bytes().next().is_some() {
        return Err(Error::Snapshot(SnapshotError::RemainingBytes));
    }

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

pub(crate) async fn import_snapshots<B: StorageBackend>(
    storage: &B,
    network_id: u64,
    config: &SnapshotConfig,
) -> Result<(), Error> {
    let full_exists = config.full_path().exists();
    let delta_exists = config.delta_path().map_or(false, Path::exists);

    if !full_exists && delta_exists {
        return Err(Error::Snapshot(SnapshotError::OnlyDeltaSnapshotFileExists));
    }

    if !full_exists {
        download_snapshot_file(config.full_path(), config.download_urls()).await?;
    }

    // Full snapshot file exists from now on.

    import_full_snapshot(storage, config.full_path(), network_id).await?;

    if let Some(delta_path) = config.delta_path() {
        if !delta_exists
            && download_snapshot_file(delta_path, config.download_urls())
                .await
                .is_err()
        {
            warn!("Could not download the delta snapshot file and it will not be imported.");
        }

        if delta_path.exists() {
            import_delta_snapshot(storage, delta_path, network_id).await?;
        }
    }

    Ok(())
}
