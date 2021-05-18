// Copyright 2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod batch;
mod error;
mod metrics;

pub mod condition;
pub mod config;

use self::{
    config::PruningConfig,
    error::Error,
    metrics::{PruningMetrics, TimingMetrics},
};

use crate::workers::{event::PrunedIndex, storage::StorageBackend};

use bee_message::prelude::MilestoneIndex;
use bee_runtime::event::Bus;
use bee_storage::access::{Batch, Truncate};
use bee_tangle::{solid_entry_point::SolidEntryPoint, MsTangle};

use log::*;

use std::time::Instant;

const _UNCONFIRMED_PRUNING_DELAY_FACTOR: u32 = 4;

pub async fn prune<S: StorageBackend>(
    tangle: &MsTangle<S>,
    storage: &S,
    bus: &Bus<'_>,
    target_index: MilestoneIndex,
    config: &PruningConfig,
    _below_max_depth: u32,
) -> Result<(), Error> {
    let mut timings = TimingMetrics::default();
    let mut pruning_metrics = PruningMetrics::default();

    let start_index = tangle.get_pruning_index() + 1;

    if target_index > start_index {
        info!("Pruning milestones {}..{}", start_index, target_index);
    } else if target_index == start_index {
        info!("Pruning milestone {}", target_index);
    } else {
        return Err(Error::InvalidTargetIndex {
            minimum: start_index,
            found: target_index,
        });
    }

    let full_prune = Instant::now();

    let get_old_seps = Instant::now();
    let old_seps = tangle.get_solid_entry_points().await;
    timings.get_old_seps = get_old_seps.elapsed();

    let mut batch = S::batch_begin();

    let batch_del_confirmed = Instant::now();
    let (new_seps, traversal_metrics) =
        batch::add_confirmed_data(tangle, &storage, &mut batch, target_index, &old_seps).await?;
    timings.batch_del_confirmed = batch_del_confirmed.elapsed();

    pruning_metrics.messages = traversal_metrics.prunable_messages;
    pruning_metrics.edges = traversal_metrics.prunable_edges;
    pruning_metrics.indexations = traversal_metrics.prunable_indexations;

    let num_new_seps = new_seps.len();

    let batch_ins_new_seps = Instant::now();
    for (new_sep, index) in &new_seps {
        Batch::<SolidEntryPoint, MilestoneIndex>::batch_insert(storage, &mut batch, new_sep, index)
            .map_err(|e| Error::BatchOperation(Box::new(e)))?;
    }
    timings.batch_ins_new_seps = batch_ins_new_seps.elapsed();

    let replace_seps = Instant::now();
    tangle.replace_solid_entry_points(new_seps).await;
    timings.replace_seps = replace_seps.elapsed();

    tangle.update_entry_point_index(target_index);

    pruning_metrics.milestones = batch::add_milestones(storage, &mut batch, start_index, target_index).await?;
    pruning_metrics.output_diffs = batch::add_output_diffs(storage, &mut batch, start_index, target_index).await?;

    if config.prune_receipts() {
        pruning_metrics.receipts += batch::add_receipts(storage, &mut batch, start_index, target_index).await?;
    }

    let truncate_old_seps = Instant::now();
    // WARN: This operation must come before the batch is committed.
    // TODO: consider batching deletes rather than using Truncate. Is one faster than the other?
    Truncate::<SolidEntryPoint, MilestoneIndex>::truncate(storage)
        .await
        .unwrap();
    timings.truncate_old_seps = truncate_old_seps.elapsed();

    let batch_commit = Instant::now();
    storage
        .batch_commit(batch, true)
        .await
        .map_err(|e| Error::BatchOperation(Box::new(e)))?;
    timings.batch_commit = batch_commit.elapsed();

    tangle.update_pruning_index(target_index);

    timings.full_prune = full_prune.elapsed();

    debug!("Pruning results: {:?}.", pruning_metrics);
    debug!("Traversal metrics: {:?}", traversal_metrics);
    debug!("Timing results: {:?}.", timings);

    info!("Selected {} new solid entry points.", num_new_seps,);
    info!("Entry point index now at {}.", target_index,);
    info!("Pruning index now at {}.", target_index);

    bus.dispatch(PrunedIndex { index: target_index });

    Ok(())
}

// mod debugging {
//     use super::*;
//     use once_cell::sync::OnceCell;
//     use std::{collections::HashMap, sync::Mutex};

//     // This checklist ensures, that no confirmed message will be collected twice by the past-cone
//     // traversal of 'target_index'.
//     pub fn unique_confirmed_checklist() -> &'static Mutex<HashMap<MessageId, MilestoneIndex>> {
//         static INSTANCE1: OnceCell<Mutex<HashMap<MessageId, MilestoneIndex>>> = OnceCell::new();
//         INSTANCE1.get_or_init(|| Mutex::new(HashMap::default()))
//     }

//     // This checklist ensures, that no unconfirmed message is referenced under two different milestone
//     // indexes.
//     pub fn unique_unconfirmed_checklist() -> &'static Mutex<HashMap<MessageId, MilestoneIndex>> {
//         static INSTANCE2: OnceCell<Mutex<HashMap<MessageId, MilestoneIndex>>> = OnceCell::new();
//         INSTANCE2.get_or_init(|| Mutex::new(HashMap::default()))
//     }
// }

// // TEMP: make sure, that the collect process doesn't yield duplicate message ids.
//     // **NOTE**: For some reason the enclosing block is necessary in async function despite the manual drop.
//     {
//         let mut removal_list = debugging::unique_confirmed_checklist().lock().unwrap();
//         for confirmed_id in &confirmed {
//             if let Some(index) = removal_list.get(confirmed_id) {
//                 error!(
//                     "Collected confirmed message {} twice. First added during {}, now at {}. This is a bug!",
//                     confirmed_id, index, pruning_target_index
//                 );
//             } else {
//                 removal_list.insert(*confirmed_id, pruning_target_index);
//             }
//         }
//         // dbg!(removal_list.len());
//         drop(removal_list); // just here to be explicit about it
//     }

// // TEMPORARILY ADD TO REMOVAL LIST
//         {
//             let mut removal_list = debugging::unique_unconfirmed_checklist().lock().unwrap();
//             for unconfirmed_id in received.iter().map(|(_, b)| b.message_id()) {
//                 if let Some(index) = removal_list.get(unconfirmed_id) {
//                     error!(
//                         "Collected UNconfirmed message {} twice. First added during {}, now at {}. This is a bug!",
//                         unconfirmed_id, index, pruning_target_index
//                     );
//                 } else {
//                     removal_list.insert(*unconfirmed_id, pruning_target_index);
//                 }
//             }
//             // dbg!(removal_list.len());
//             drop(removal_list); // just here to be explicit about it
//         }
