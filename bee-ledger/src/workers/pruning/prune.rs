// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use super::{
    batch,
    config::PruningConfig,
    error::Error,
    metrics::{PruningMetrics, Timings},
};

use crate::workers::{event::PrunedIndex, storage::StorageBackend};

use bee_message::prelude::MilestoneIndex;
use bee_runtime::event::Bus;
use bee_storage::access::{Batch, Truncate};
use bee_tangle::{solid_entry_point::SolidEntryPoint, MsTangle};

use log::*;

use std::time::Instant;

const RETAIN_SNAPSHOT_SEPS: u32 = 50;

/// Performs pruning of old data until `target_index`.
pub async fn prune<S: StorageBackend>(
    tangle: &MsTangle<S>,
    storage: &S,
    bus: &Bus<'_>,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
    config: &PruningConfig,
) -> Result<(), Error> {
    let mut timings = Timings::default();
    let mut pruning_metrics = PruningMetrics::default();

    let snapshot_index = tangle.get_snapshot_index();

    if target_index < start_index {
        return Err(Error::InvalidTargetIndex {
            minimum: start_index,
            found: target_index,
        });
    }

    for index in *start_index..=*target_index {
        let index = index.into();
        info!("Pruning milestone {}...", index);

        // Measurement of the full pruning step.
        let full_prune = Instant::now();

        // Get the current set of SEPs.
        let get_curr_seps = Instant::now();
        let mut curr_seps = tangle.get_solid_entry_points().await;
        timings.get_curr_seps = get_curr_seps.elapsed();

        pruning_metrics.curr_seps = curr_seps.len();

        // Start a batch to make changes to the storage in a single atomic step.
        let mut batch = S::batch_begin();

        // Add confirmed data to the delete batch.
        // NOTE: This is the most costly thing during pruning, because it has to perform a past-cone traversal.
        let batch_confirmed = Instant::now();
        let (mut new_seps, confirmed_metrics) =
            batch::delete_confirmed_data(tangle, &storage, &mut batch, index, &curr_seps).await?;
        timings.batch_confirmed = batch_confirmed.elapsed();

        pruning_metrics.new_seps = new_seps.len();
        pruning_metrics.messages = confirmed_metrics.prunable_messages;
        pruning_metrics.edges = confirmed_metrics.prunable_edges;
        pruning_metrics.indexations = confirmed_metrics.prunable_indexations;

        // Keep still relevant SEPs.
        //
        // Note:
        // Currently SEPs from the snapshot are stored with their confirming milestone index, but what we need is
        // its "youngest" approver (highest confirmation index) in order to decide how long we need to keep it. Hence,
        // we need to keep the "snapshot-SEPs" around for some time.
        let filter_curr_seps = Instant::now();
        if start_index < snapshot_index + RETAIN_SNAPSHOT_SEPS {
            curr_seps.retain(|_, v| **v > *index || *v == snapshot_index);
        } else {
            curr_seps.retain(|_, v| **v > *index);
        }
        timings.filter_curr_seps = filter_curr_seps.elapsed();

        pruning_metrics.kept_seps = curr_seps.len();

        // Create the union of both sets:
        new_seps.extend(curr_seps);

        let num_next_seps = new_seps.len();

        pruning_metrics.next_seps = num_next_seps;

        // Write the new set of SEPs to the storage.
        let batch_new_seps = Instant::now();
        for (new_sep, index) in &new_seps {
            Batch::<SolidEntryPoint, MilestoneIndex>::batch_insert(storage, &mut batch, new_sep, index)
                .map_err(|e| Error::BatchOperation(Box::new(e)))?;
        }
        timings.batch_new_seps = batch_new_seps.elapsed();

        // Replace the old set of SEPs with the new one.
        let replace_seps = Instant::now();
        tangle.replace_solid_entry_points(new_seps).await;
        timings.replace_seps = replace_seps.elapsed();

        // Update entry point index
        tangle.update_entry_point_index(index);

        // Add prunable milestones to the delete batch.
        let batch_milestones = Instant::now();
        pruning_metrics.milestones = batch::delete_milestones(storage, &mut batch, index, index).await?;
        timings.batch_milestones = batch_milestones.elapsed();

        // Add prunable output diffs to the delete batch.
        let batch_output_diffs = Instant::now();
        pruning_metrics.output_diffs = batch::delete_output_diffs(storage, &mut batch, index, index).await?;
        timings.batch_output_diffs = batch_output_diffs.elapsed();

        // Add prunable receipts the delete batch (if wanted).
        if config.prune_receipts() {
            let batch_receipts = Instant::now();
            pruning_metrics.receipts += batch::delete_receipts(storage, &mut batch, index, index).await?;
            timings.batch_receipts = batch_receipts.elapsed();
        }

        // Add unconfirmed data to the delete batch.
        let batch_unconfirmed = Instant::now();
        let unconfirmed_metrics = batch::delete_unconfirmed_data(storage, &mut batch, index, index).await?;
        timings.batch_unconfirmed = batch_unconfirmed.elapsed();

        pruning_metrics.messages += unconfirmed_metrics.prunable_messages;
        pruning_metrics.edges += unconfirmed_metrics.prunable_edges;
        pruning_metrics.indexations += unconfirmed_metrics.prunable_indexations;

        // Remove old SEPs from the storage.
        //
        // **NOTE**: This operation must come before the batch is committed.
        //
        // TODO: consider batching deletes rather than using Truncate. Is one faster than the other? Do we care if its
        // atomic or not?
        let truncate_old_seps = Instant::now();
        Truncate::<SolidEntryPoint, MilestoneIndex>::truncate(storage).unwrap();
        timings.truncate_curr_seps = truncate_old_seps.elapsed();

        // Execute the batch operation.
        let batch_commit = Instant::now();
        storage
            .batch_commit(batch, true)
            .map_err(|e| Error::BatchOperation(Box::new(e)))?;
        timings.batch_commit = batch_commit.elapsed();

        // Update the pruning index.
        tangle.update_pruning_index(index);

        timings.full_prune = full_prune.elapsed();

        debug!("{:?}.", pruning_metrics);
        debug!("{:?}", confirmed_metrics);
        debug!("{:?}", unconfirmed_metrics);
        debug!("{:?}.", timings);

        info!("{} solid entry points.", num_next_seps,);
        info!("Entry point index now at {}.", index,);
        info!("Pruning index now at {}.", index);

        bus.dispatch(PrunedIndex { index });
    }

    Ok(())
}
