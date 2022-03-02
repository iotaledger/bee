// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{Instant, SystemTime},
};

use bee_message::milestone::MilestoneIndex;
use bee_runtime::event::Bus;
use bee_storage::access::{Batch, Truncate};
use bee_tangle::{solid_entry_point::SolidEntryPoint, Tangle};
use log::{debug, info};

use crate::workers::{
    event::PrunedIndex,
    pruning::{
        batch,
        config::PruningConfig,
        error::Error,
        metrics::{PruningMetrics, Timings},
    },
    storage::{self, StorageBackend},
};

const KEEP_INITIAL_SNAPSHOT_SEPS: usize = 50;

static NUM_PRUNINGS: AtomicUsize = AtomicUsize::new(0);

/// Performs pruning of data from `start_index` to `target_index`.
#[cfg_attr(feature = "trace", trace_tools::observe)]
pub async fn prune<S: StorageBackend>(
    tangle: &Tangle<S>,
    storage: &S,
    bus: &Bus<'_>,
    start_index: MilestoneIndex,
    target_index: MilestoneIndex,
    config: &PruningConfig,
) -> Result<(), Error> {
    let mut timings = Timings::default();
    let mut metrics = PruningMetrics::default();

    if target_index < start_index {
        return Err(Error::InvalidTargetIndex {
            selected: target_index,
            minimum: start_index,
        });
    }

    if start_index != target_index {
        info!(
            "Pruning from milestone {} to milestone {}...",
            start_index, target_index
        );
    }

    for index in *start_index..=*target_index {
        let index = MilestoneIndex(index);

        debug!("Pruning milestone {}...", index);

        //

        // Measurement of the full pruning step.
        let full_prune = Instant::now();

        // Get the current set of SEPs.
        let get_curr_seps = Instant::now();
        let mut curr_seps = tangle.get_solid_entry_points().await;
        timings.get_curr_seps = get_curr_seps.elapsed();

        metrics.curr_seps = curr_seps.len();

        // Start a batch to make changes to the storage in a single atomic step.
        let mut batch = S::batch_begin();

        // Add confirmed data to the delete batch.
        // NOTE: This is the most costly thing during pruning, because it has to perform a past-cone traversal.
        let batch_confirmed_data = Instant::now();
        let (mut new_seps, confirmed_data_metrics) =
            batch::batch_prunable_confirmed_data(tangle, storage, &mut batch, index, &curr_seps)?;
        timings.batch_confirmed_data = batch_confirmed_data.elapsed();

        metrics.new_seps = new_seps.len();
        metrics.messages = confirmed_data_metrics.prunable_messages;
        metrics.edges = confirmed_data_metrics.prunable_edges;
        metrics.indexations = confirmed_data_metrics.prunable_indexations;

        // Keep still relevant SEPs.
        //
        // Note:
        // Currently Bee is reliant on the snapshot file generated by Hornet, which stores the confirmation index
        // of an SEP along with it. It then keeps it long enough to be (pretty) sure the coordinator would reject a
        // message directly referencing it. In Bee, however, we wanted to try a different approach, which doesn't
        // trust the Coordinator's tip selection, and stores the highest confirmation index of any of its direct
        // approvers instead.
        //
        // For the first X milestones we keep the initial SEP set (from the snapshot file) around, after that, we keep
        // only the necessary SEPs (the ones that will be referenced in future prunings).
        let filter_curr_seps = Instant::now();
        if NUM_PRUNINGS.fetch_add(1, Ordering::Relaxed) >= KEEP_INITIAL_SNAPSHOT_SEPS {
            curr_seps.retain(|_, v| **v > *index);
        }
        timings.filter_curr_seps = filter_curr_seps.elapsed();

        metrics.kept_seps = curr_seps.len();

        // Create the union of both sets:
        new_seps.extend(curr_seps);

        let num_next_seps = new_seps.len();

        metrics.next_seps = num_next_seps;

        // Write the new set of SEPs to the storage.
        let batch_new_seps = Instant::now();
        for (new_sep, index) in &new_seps {
            Batch::<SolidEntryPoint, MilestoneIndex>::batch_insert(storage, &mut batch, new_sep, index)
                .map_err(|e| Error::Storage(Box::new(e)))?;
        }
        timings.batch_new_seps = batch_new_seps.elapsed();

        // Replace the old set of SEPs with the new one.
        let replace_seps = Instant::now();
        tangle.replace_solid_entry_points(new_seps).await;
        timings.replace_seps = replace_seps.elapsed();

        // Update entry point index
        tangle.update_entry_point_index(index);

        let batch_milestones = Instant::now();
        let milestone_data_metrics = batch::prune_milestone_data(storage, &mut batch, index, config.prune_receipts())?;
        timings.batch_milestone_data = batch_milestones.elapsed();

        metrics.receipts = milestone_data_metrics.receipts;

        // Add unconfirmed data to the delete batch.
        let batch_unconfirmed_data = Instant::now();
        let unconfirmed_data_metrics = batch::batch_prunable_unconfirmed_data(storage, &mut batch, index)?;
        timings.batch_unconfirmed_data = batch_unconfirmed_data.elapsed();

        metrics.messages += unconfirmed_data_metrics.prunable_messages;
        metrics.edges += unconfirmed_data_metrics.prunable_edges;
        metrics.indexations += unconfirmed_data_metrics.prunable_indexations;

        // Remove old SEPs from the storage.
        //
        // **WARNING**: This operation must come before the batch is committed!
        //
        // TODO: consider batching deletes rather than using Truncate. Is one faster than the other? Do we care if its
        // atomic or not?
        let truncate_old_seps = Instant::now();
        Truncate::<SolidEntryPoint, MilestoneIndex>::truncate(storage).expect("truncating solid entry points failed");
        timings.truncate_curr_seps = truncate_old_seps.elapsed();

        // Execute the batch operation.
        let batch_commit = Instant::now();
        storage
            .batch_commit(batch, true)
            .map_err(|e| Error::Storage(Box::new(e)))?;
        timings.batch_commit = batch_commit.elapsed();

        // Update the pruning index.
        tangle.update_pruning_index(index);

        // Write the updated snapshot info to the storage.
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("error creating timestamp")
            .as_secs();
        let mut snapshot_info = storage::fetch_snapshot_info(storage)
            .map_err(|e| Error::Storage(Box::new(e)))?
            .ok_or(Error::MissingSnapshotInfo)?;
        snapshot_info.update_pruning_index(index);
        snapshot_info.update_timestamp(timestamp);
        storage::insert_snapshot_info(storage, &snapshot_info).map_err(|e| Error::Storage(Box::new(e)))?;

        timings.full_prune = full_prune.elapsed();

        debug!("{:?}.", metrics);
        debug!("{:?}", confirmed_data_metrics);
        debug!("{:?}", unconfirmed_data_metrics);
        debug!("{:?}.", timings);
        debug!(
            "Entry point index now at {} with {} solid entry points..",
            index, num_next_seps
        );
        debug!("Pruned milestone {}.", index);

        bus.dispatch(PrunedIndex { index });
    }

    if start_index == target_index {
        info!("Pruned milestone {}.", start_index);
    } else {
        info!("Pruned from milestone {} to milestone {}.", start_index, target_index);
    }

    Ok(())
}
