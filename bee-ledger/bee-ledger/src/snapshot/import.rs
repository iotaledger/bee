// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, Read},
    path::Path,
};

use bee_block::{
    output::{self, OutputId},
    payload::milestone::MilestoneIndex,
    protocol::ProtocolParameters,
};
use bee_storage::access::{Insert, Truncate};
use bee_tangle::solid_entry_point::SolidEntryPoint;
use log::info;
use packable::{
    unpacker::{IoUnpacker, Unpacker},
    Packable,
};
use time_helper as time;

use crate::{
    consensus::worker::migration_from_milestone,
    error::Error,
    snapshot::{config::SnapshotConfig, download::download_latest_snapshot_files, error::Error as SnapshotError},
    storage::{self, apply_milestone, create_output, rollback_milestone, StorageBackend},
    types::{
        snapshot::{
            DeltaSnapshotHeader, FullSnapshotHeader, MilestoneDiff, SnapshotHeader, SnapshotInfo, SnapshotKind,
        },
        CreatedOutput, TreasuryOutput,
    },
};

fn snapshot_reader(path: &Path) -> Result<BufReader<File>, Error> {
    Ok(BufReader::new(
        OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| Error::Snapshot(SnapshotError::Io(e)))?,
    ))
}

fn import_solid_entry_points<U: Unpacker<Error = std::io::Error>, B: StorageBackend>(
    unpacker: &mut U,
    storage: &B,
    sep_count: u64,
    index: MilestoneIndex,
) -> Result<(), Error> {
    Truncate::<SolidEntryPoint, MilestoneIndex>::truncate(storage).map_err(|e| Error::Storage(Box::new(e)))?;
    for _ in 0..sep_count {
        Insert::<SolidEntryPoint, MilestoneIndex>::insert(
            storage,
            &SolidEntryPoint::unpack::<_, true>(unpacker, &())?,
            &index,
        )
        .map_err(|e| Error::Storage(Box::new(e)))?;
    }

    Ok(())
}

fn import_outputs<U: Unpacker<Error = std::io::Error>, B: StorageBackend>(
    unpacker: &mut U,
    storage: &B,
    output_count: u64,
    protocol_parameters: &ProtocolParameters,
) -> Result<(), Error> {
    for _ in 0..output_count {
        let output_id = OutputId::unpack::<_, true>(unpacker, &())?;
        let created_output = CreatedOutput::unpack::<_, true>(unpacker, protocol_parameters)?;

        create_output(storage, &output_id, &created_output)?;
    }

    Ok(())
}

fn import_milestone_diffs<U: Unpacker<Error = std::io::Error>, B: StorageBackend>(
    unpacker: &mut U,
    storage: &B,
    milestone_diff_count: u64,
    protocol_parameters: &ProtocolParameters,
) -> Result<(), Error> {
    for _ in 0..milestone_diff_count {
        let diff = MilestoneDiff::unpack::<_, true>(unpacker, protocol_parameters)?;
        let index = diff.milestone().essence().index();
        // Unwrap is fine because ledger index was inserted just before.
        let ledger_index = *storage::fetch_ledger_index(storage)?.unwrap();

        let consumed = diff
            .consumed()
            .iter()
            .map::<Result<_, Error>, _>(|(output_id, (created_output, consumed_output))| {
                Ok((*output_id, (created_output.clone(), consumed_output.clone())))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let migration = if let Some(receipt) = diff.milestone().essence().options().receipt() {
            let consumed_treasury = diff
                .consumed_treasury()
                .ok_or(Error::Snapshot(SnapshotError::MissingConsumedTreasury))?
                .clone();

            Some(migration_from_milestone(
                index,
                diff.milestone().id(),
                receipt,
                TreasuryOutput::new(consumed_treasury.0, consumed_treasury.1),
                protocol_parameters,
            )?)
        } else {
            None
        };

        if index == MilestoneIndex(ledger_index + 1) {
            apply_milestone(storage, index, diff.created(), &consumed, &migration)?;
        } else if index == MilestoneIndex(ledger_index) {
            rollback_milestone(storage, index, diff.created(), &consumed, &migration)?;
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

fn import_full_snapshot<B: StorageBackend>(
    storage: &B,
    path: &Path,
    network_id: u64,
    protocol_parameters: &ProtocolParameters,
) -> Result<(), Error> {
    info!("Importing full snapshot file {}...", &path.to_string_lossy());

    let mut unpacker = IoUnpacker::new(snapshot_reader(path)?);
    let header = SnapshotHeader::unpack::<_, true>(&mut unpacker, &())?;

    check_header(&header, SnapshotKind::Full, network_id)?;

    let full_header = FullSnapshotHeader::unpack::<_, true>(&mut unpacker, &())?;

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
        storage,
        &TreasuryOutput::new(
            output::TreasuryOutput::new(full_header.treasury_output_amount(), protocol_parameters)?,
            *full_header.treasury_output_milestone_id(),
        ),
    )?;

    storage::insert_ledger_index(storage, &header.ledger_index().into())?;
    storage::insert_snapshot_info(
        storage,
        &SnapshotInfo::new(
            network_id,
            header.sep_index(),
            header.sep_index(),
            header.sep_index(),
            header.timestamp(),
        ),
    )?;

    import_solid_entry_points(&mut unpacker, storage, full_header.sep_count(), header.sep_index())?;
    import_outputs(&mut unpacker, storage, full_header.output_count(), protocol_parameters)?;
    import_milestone_diffs(
        &mut unpacker,
        storage,
        full_header.milestone_diff_count(),
        protocol_parameters,
    )?;

    if unpacker.into_inner().bytes().next().is_some() {
        return Err(Error::Snapshot(SnapshotError::RemainingBytes));
    }

    info!(
        "Imported full snapshot file from {} with sep index {}, ledger index {}, {} solid entry points, {} outputs and {} milestone diffs.",
        time::format_unix_timestamp(header.timestamp() as i64),
        *header.sep_index(),
        *header.ledger_index(),
        full_header.sep_count(),
        full_header.output_count(),
        full_header.milestone_diff_count()
    );

    Ok(())
}

fn import_delta_snapshot<B: StorageBackend>(
    storage: &B,
    path: &Path,
    network_id: u64,
    protocol_parameters: &ProtocolParameters,
) -> Result<(), Error> {
    info!("Importing delta snapshot file {}...", &path.to_string_lossy());

    let mut unpacker = IoUnpacker::new(snapshot_reader(path)?);
    let header = SnapshotHeader::unpack::<_, true>(&mut unpacker, &())?;

    check_header(&header, SnapshotKind::Delta, network_id)?;

    let delta_header = DeltaSnapshotHeader::unpack::<_, true>(&mut unpacker, &())?;

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

    storage::insert_ledger_index(storage, &header.ledger_index().into())?;
    storage::insert_snapshot_info(
        storage,
        &SnapshotInfo::new(
            network_id,
            header.sep_index(),
            header.sep_index(),
            header.sep_index(),
            header.timestamp(),
        ),
    )?;

    import_solid_entry_points(&mut unpacker, storage, delta_header.sep_count(), header.sep_index())?;
    import_milestone_diffs(
        &mut unpacker,
        storage,
        delta_header.milestone_diff_count(),
        protocol_parameters,
    )?;

    if unpacker.into_inner().bytes().next().is_some() {
        return Err(Error::Snapshot(SnapshotError::RemainingBytes));
    }

    info!(
        "Imported delta snapshot file from {} with sep index {}, ledger index {}, {} solid entry points and {} milestone diffs.",
        time::format_unix_timestamp(header.timestamp() as i64),
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

    // TODO: this is obviously wrong and just temporary
    let protocol_parameters = ProtocolParameters::default();

    if !full_exists && delta_exists {
        return Err(Error::Snapshot(SnapshotError::OnlyDeltaSnapshotFileExists));
    } else if !full_exists && !delta_exists {
        download_latest_snapshot_files(
            network_id,
            config.full_path(),
            config.delta_path(),
            config.download_urls(),
        )
        .await?;
    }

    import_full_snapshot(storage, config.full_path(), network_id, &protocol_parameters)?;

    if let Some(delta_path) = config.delta_path() {
        if delta_path.exists() {
            import_delta_snapshot(storage, delta_path, network_id, &protocol_parameters)?;
        }
    }

    Ok(())
}
