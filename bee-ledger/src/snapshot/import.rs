// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use crate::{
    consensus::{
        dust::DUST_THRESHOLD,
        storage::{
            self, apply_output_diffs, create_output, rollback_output_diffs, store_balance_diffs, StorageBackend,
        },
        worker::migration_from_milestone,
    },
    snapshot::{
        config::SnapshotConfig,
        download::download_snapshot_file,
        error::Error,
        header::{DeltaSnapshotHeader, FullSnapshotHeader, SnapshotHeader},
        info::SnapshotInfo,
        kind::Kind,
        milestone_diff::MilestoneDiff,
    },
    types::{BalanceDiffs, TreasuryOutput},
};

use bee_common::packable::{Packable, Read};
use bee_message::{
    milestone::MilestoneIndex,
    output::{self, CreatedOutput, Output, OutputId},
    payload::Payload,
    MessageId,
};
use bee_storage::access::{Insert, Truncate};
use bee_tangle::{solid_entry_point::SolidEntryPoint, MsTangle};

use chrono::{offset::TimeZone, Utc};
use log::info;

use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::BufReader,
    path::Path,
};

fn snapshot_reader(path: &Path) -> Result<BufReader<File>, Error> {
    Ok(BufReader::new(
        OpenOptions::new().read(true).open(path).map_err(Error::Io)?,
    ))
}

async fn import_solid_entry_points<R: Read, B: StorageBackend>(
    reader: &mut R,
    storage: &B,
    tangle: &MsTangle<B>,
    sep_count: u64,
    index: MilestoneIndex,
) -> Result<(), Error> {
    // TODO also clear tangle SEPs
    Truncate::<SolidEntryPoint, MilestoneIndex>::truncate(storage)
        .await
        .unwrap();
    for _ in 0..sep_count {
        let sep = SolidEntryPoint::unpack(reader)?;
        tangle.add_solid_entry_point(sep, index).await;
        // TODO somewhere else ?
        Insert::<SolidEntryPoint, MilestoneIndex>::insert(&*storage, &sep, &index)
            .await
            .unwrap();
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

        // TODO handle unwrap
        // TODO batch
        create_output(&*storage, &output_id, &created_output).await.unwrap();
        match created_output.inner() {
            Output::SignatureLockedSingle(output) => {
                balance_diffs.amount_add(*output.address(), output.amount());
                if output.amount() < DUST_THRESHOLD {
                    balance_diffs.dust_output_inc(*output.address());
                }
            }
            Output::SignatureLockedDustAllowance(output) => {
                balance_diffs.amount_add(*output.address(), output.amount());
                balance_diffs.dust_allowance_add(*output.address(), output.amount());
            }
            output => return Err(Error::UnsupportedOutputKind(output.kind())),
        }
    }

    store_balance_diffs(&*storage, &balance_diffs)
        .await
        .map_err(|e| Error::Consumer(Box::new(e)))?;

    Ok(())
}

async fn import_milestone_diffs<R: Read, B: StorageBackend>(
    reader: &mut R,
    storage: &B,
    milestone_diff_count: u64,
) -> Result<(), Error> {
    for _ in 0..milestone_diff_count {
        let diff = MilestoneDiff::unpack(reader)?;
        let index = diff.milestone().essence().index();
        // Unwrap is fine because we just inserted the ledger index.
        // TODO unwrap
        let ledger_index = *storage::fetch_ledger_index(&*storage).await.unwrap().unwrap();

        let mut balance_diffs = BalanceDiffs::new();

        for (_, output) in diff.created().iter() {
            match output.inner() {
                Output::SignatureLockedSingle(output) => {
                    balance_diffs.amount_add(*output.address(), output.amount());
                    if output.amount() < DUST_THRESHOLD {
                        balance_diffs.dust_output_inc(*output.address());
                    }
                }
                Output::SignatureLockedDustAllowance(output) => {
                    balance_diffs.amount_add(*output.address(), output.amount());
                    balance_diffs.dust_allowance_add(*output.address(), output.amount());
                }
                output => return Err(Error::UnsupportedOutputKind(output.kind())),
            }
        }

        let mut consumed = HashMap::new();

        for (output_id, (created_output, consumed_output)) in diff.consumed().iter() {
            match created_output.inner() {
                Output::SignatureLockedSingle(output) => {
                    balance_diffs.amount_sub(*output.address(), output.amount());
                    if output.amount() < DUST_THRESHOLD {
                        balance_diffs.dust_output_dec(*output.address());
                    }
                }
                Output::SignatureLockedDustAllowance(output) => {
                    balance_diffs.amount_sub(*output.address(), output.amount());
                    balance_diffs.dust_allowance_sub(*output.address(), output.amount());
                }
                output => return Err(Error::UnsupportedOutputKind(output.kind())),
            }
            consumed.insert(*output_id, (*consumed_output).clone());
        }

        let migration = if let Some(Payload::Receipt(receipt)) = diff.milestone().essence().receipt() {
            // There should be a consumed treasury if there is a receipt.
            let consumed_treasury = diff.consumed_treasury().unwrap().clone();

            Some(
                migration_from_milestone(
                    index,
                    diff.milestone().id(),
                    receipt,
                    TreasuryOutput::new(consumed_treasury.0, consumed_treasury.1),
                )
                .await
                .map_err(|e| Error::Consumer(Box::new(e)))?,
            )
        } else {
            None
        };

        match index {
            index if index == MilestoneIndex(ledger_index + 1) => {
                // TODO unwrap until we merge both crates
                apply_output_diffs(&*storage, index, diff.created(), &consumed, &balance_diffs, &migration)
                    .await
                    .unwrap();
            }
            index if index == MilestoneIndex(ledger_index) => {
                // TODO unwrap until we merge both crates
                rollback_output_diffs(&*storage, index, diff.created(), &consumed)
                    .await
                    .unwrap();
            }
            _ => return Err(Error::UnexpectedDiffIndex(index)),
        }
    }

    Ok(())
}

async fn import_full_snapshot<B: StorageBackend>(
    storage: &B,
    tangle: &MsTangle<B>,
    path: &Path,
    network_id: u64,
) -> Result<(), Error> {
    info!("Importing full snapshot file {}...", &path.to_string_lossy());

    let mut reader = snapshot_reader(path)?;

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

    storage::store_unspent_treasury_output(
        &*storage,
        &TreasuryOutput::new(
            output::TreasuryOutput::new(full_header.treasury_output_amount())?,
            *full_header.treasury_output_milestone_id(),
        ),
    )
    .await
    .map_err(|e| Error::Consumer(Box::new(e)))?;

    storage::insert_ledger_index(storage, &header.ledger_index().into())
        .await
        .map_err(|e| Error::Consumer(Box::new(e)))?;
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
    .await
    .map_err(|e| Error::Consumer(Box::new(e)))?;

    import_solid_entry_points(
        &mut reader,
        storage,
        tangle,
        full_header.sep_count(),
        header.sep_index(),
    )
    .await?;
    import_outputs(&mut reader, storage, full_header.output_count()).await?;
    import_milestone_diffs(&mut reader, storage, full_header.milestone_diff_count()).await?;

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
    tangle: &MsTangle<B>,
    path: &Path,
    network_id: u64,
) -> Result<(), Error> {
    info!("Importing delta snapshot file {}...", &path.to_string_lossy());

    let mut reader = snapshot_reader(path)?;

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

    storage::insert_ledger_index(storage, &header.ledger_index().into())
        .await
        .map_err(|e| Error::Consumer(Box::new(e)))?;
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
    .await
    .map_err(|e| Error::Consumer(Box::new(e)))?;

    import_solid_entry_points(
        &mut reader,
        storage,
        tangle,
        delta_header.sep_count(),
        header.sep_index(),
    )
    .await?;
    import_milestone_diffs(&mut reader, storage, delta_header.milestone_diff_count()).await?;

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

pub(crate) async fn import_snapshots<B: StorageBackend>(
    storage: &B,
    tangle: &MsTangle<B>,
    network_id: u64,
    config: &SnapshotConfig,
) -> Result<(), Error> {
    let full_exists = config.full_path().exists();
    let delta_exists = config.delta_path().exists();

    if !full_exists && delta_exists {
        return Err(Error::OnlyDeltaSnapshotFileExists);
    } else if !full_exists && !delta_exists {
        download_snapshot_file(config.full_path(), config.download_urls()).await?;
        download_snapshot_file(config.delta_path(), config.download_urls()).await?;
    }

    import_full_snapshot(storage, tangle, config.full_path(), network_id).await?;

    // Load delta file only if both full and delta files already existed or if they have just been downloaded.
    if (full_exists && delta_exists) || (!full_exists && !delta_exists) {
        import_delta_snapshot(storage, tangle, config.delta_path(), network_id).await?;
    }

    Ok(())
}
