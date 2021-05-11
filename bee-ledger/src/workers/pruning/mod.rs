// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! **NB**:
//!
//! In order to ensure that everything that entered the database gets pruned eventually the following mechanism
//! is applied:
//!
//! (1) Incoming messages immediatedly get assigned the LMI (current latest milestone index known by the node)
//! (2) Most of them will get confirmed eventually, and are referencable by a particular CMI (the closest future
//! milestone that references them)
//!
//! However, that means that for a given message X LMI(X) <= CMI(X) may not be true. That is certainly the case for
//! requested messages, that are required to solidify an older milestone. When the message comes in it gets assigned the
//! current LMI, while its confirming milestone might be several milestone indexes ago already. So that would mean that
//! when we try to prune LMI(X) (as unconfirmed) it would fail, because it would have been pruned already at CMI(X).
//!
//! The other way of failure is also possible: A message X was pruned at LMI(X), but when CMI(X) is about to pruned it
//! can't be used to fully traverse the past-cone of that particular milestone, and hence will make it impossible to
//! determine the new SEP set.
//!
//! CASE A: CMI < LMI (requested message)        => pruning at CMI problematic for unconfirmed messages
//!      Solution: just ignore the failure, and assume that the message has been pruned already at CMI, because it was
//! confirmed CASE B: CMI > LMI (regular gossip message)   => pruning at LMI problematic
//!      Solution: use `ADDITIONAL_DELAY` such that CMI < LMI + UNCONFIRMED_PRUNING_DELAY_FACTOR

mod batch;
mod error;

pub mod condition;
pub mod config;

use self::{config::PruningConfig, error::Error};

use crate::workers::{event::PrunedIndex, storage::StorageBackend};

use bee_message::{prelude::MilestoneIndex, MessageId};
use bee_runtime::event::Bus;
use bee_storage::access::{Batch, Truncate};
use bee_tangle::{solid_entry_point::SolidEntryPoint, MsTangle};

use log::*;
use once_cell::sync::OnceCell;

use std::{collections::HashMap, sync::Mutex, time::Instant};

/// Determines how many BMDs we delay pruning of still unconfirmed messages. It basically determines the range to be:
/// [LMI(X)..LMI(X)+BMD*UNCONFIRMED_PRUNING_DELAY_FACTOR] within which a message *MUST* be confirmed, i.e. contains
/// CMI(X), so it basically relies on the Coordinator node to not confirm a message anytime later than that, otherwise
/// traversing the about-to-be-pruned past-cone of a milestone to determine the new SEP set would fail.
///
/// Pruning (if still unconf.) happens x-BMD milestones after its LMI(X)
const UNCONFIRMED_PRUNING_DELAY_FACTOR: u32 = 4;

pub async fn prune<S: StorageBackend>(
    tangle: &MsTangle<S>,
    storage: &S,
    bus: &Bus<'_>,
    pruning_target_index: MilestoneIndex,
    config: &PruningConfig,
    below_max_depth: u32,
) -> Result<(), Error> {
    let start = Instant::now();

    // Start pruning from the last pruning index + 1.
    let start_index = tangle.get_pruning_index() + 1;
    assert!(
        pruning_target_index >= start_index,
        "target_index: {}, start_index: {}",
        pruning_target_index,
        start_index
    );

    // If `start_index == 1` (lowest possible start index), we need to deactivate "unconfirmed" pruning.
    let unconfirmed_additional_pruning_delay = below_max_depth * UNCONFIRMED_PRUNING_DELAY_FACTOR;
    let unconfirmed_start_index = if *start_index >= unconfirmed_additional_pruning_delay {
        Some(start_index - unconfirmed_additional_pruning_delay)
    } else {
        None
    };

    info!("Pruning database until milestone {}...", pruning_target_index);

    // Prepare a batch of delete operations on the database.
    let mut batch = S::batch_begin();

    // We get us a clone of the current SEP set. We are the only ones that make changes to that particular tangle state,
    // so we can be sure it can't be invalidated in the meantime while we do the past-cone traversal to find the new
    // set.
    let old_seps = tangle.get_solid_entry_points().await;
    let num_old_seps = old_seps.len();

    // Batch the data that can be safely pruned. In order to find the correct set of SEPs during this pruning we need
    // to walk the past-cone from the `target_index` backwards, and not step-by-step from `start_index` to
    // `target_index` as this would require additional logic to remove redundant SEPs again. If memory or performance
    // becomes an issue, reduce the pruning `interval` in the config.
    let (num_batched_messages, num_batched_edges, num_batched_indexations, new_seps) =
        batch::add_confirmed_data(tangle, &storage, pruning_target_index, &old_seps, &mut batch).await?;

    let num_new_seps = new_seps.len();

    for (new_sep, index) in &new_seps {
        Batch::<SolidEntryPoint, MilestoneIndex>::batch_insert(storage, &mut batch, new_sep, index)
            .map_err(|e| Error::PruningFailed(Box::new(e)))?;
    }

    tangle.replace_solid_entry_points(new_seps).await;

    // Remember up to which index we determined SEPs.
    tangle.update_entry_point_index(pruning_target_index);
    info!(
        "Entry point index now at {}. (Selected {} new solid entry points).",
        pruning_target_index, num_new_seps,
    );

    // Add milestone related data to the batch.
    let num_batched_milestones = batch::add_milestones(storage, &mut batch, start_index, pruning_target_index).await?;
    let num_batched_output_diffs =
        batch::add_output_diffs(storage, &mut batch, start_index, pruning_target_index).await?;

    // Add receipts optionally.
    let mut num_batched_receipts: usize = 0;
    if config.prune_receipts() {
        num_batched_receipts += batch::add_receipts(storage, &mut batch, start_index, pruning_target_index).await?;
    }

    // WARN: This operation must come before the batch is committed.
    // TODO: consider batching deletes rather than using Truncate. Is one faster than the other?
    Truncate::<SolidEntryPoint, MilestoneIndex>::truncate(storage)
        .await
        .unwrap();

    storage
        .batch_commit(batch, true)
        .await
        // If that error actually happens we set the database to 'corrupted'!
        .map_err(|e| Error::StorageError(Box::new(e)))?;

    info!(
        "Pruned {} milestones, {} output diffs, {} receipts.",
        num_batched_milestones, num_batched_output_diffs, num_batched_receipts
    );
    info!(
        "Pruned {} confirmed messages including {} edges, {} indexations.",
        num_batched_messages, num_batched_edges, num_batched_indexations
    );
    // info!(
    //     "Pruned {} unreferenced messages, including {} messages, {} edges, {} indexations.",
    //     num_received_ids, num_unconf_messages, num_unconf_edges, num_unconf_indexations
    // );

    tangle.update_pruning_index(pruning_target_index);
    info!("Pruning index now at {}.", pruning_target_index);

    info!("Pruning completed in {}.", start.elapsed().as_secs_f64());

    bus.dispatch(PrunedIndex {
        index: pruning_target_index,
    });

    Ok(())
}

mod debugging {
    use super::*;

    // This checklist ensures, that no confirmed message will be collected twice by the past-cone
    // traversal of 'target_index'.
    pub fn unique_confirmed_checklist() -> &'static Mutex<HashMap<MessageId, MilestoneIndex>> {
        static INSTANCE1: OnceCell<Mutex<HashMap<MessageId, MilestoneIndex>>> = OnceCell::new();
        INSTANCE1.get_or_init(|| Mutex::new(HashMap::default()))
    }

    // This checklist ensures, that no unconfirmed message is referenced under two different milestone
    // indexes.
    pub fn unique_unconfirmed_checklist() -> &'static Mutex<HashMap<MessageId, MilestoneIndex>> {
        static INSTANCE2: OnceCell<Mutex<HashMap<MessageId, MilestoneIndex>>> = OnceCell::new();
        INSTANCE2.get_or_init(|| Mutex::new(HashMap::default()))
    }
}
