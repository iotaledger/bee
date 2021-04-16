// Copyright 2020 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

mod _config;
mod _error;
mod worker;

// Exports
pub use _config::{PruningConfig, PruningConfigBuilder};

use _error::Error;

use crate::{
    MsTangle,
    traversal,
};

use bee_message::prelude::{MessageId, MilestoneIndex};
use bee_storage::backend::StorageBackend;

use log::{info, warn};

use std::collections::HashMap;

pub(crate) const SEP_CHECK_THRESHOLD_PAST: u32 = 50;
pub(crate) const SEP_CHECK_THRESHOLD_FUTURE: u32 = 50;
pub(crate) const ADDITIONAL_PRUNING_THRESHOLD: u32 = 50;

/// Checks whether any direct approver of the given transaction was confirmed by a
/// milestone which is above the target milestone.
pub fn is_solid_entry_point<B: StorageBackend>(tangle: &MsTangle<B>, message_id: &MessageId) -> Result<bool, Error> {
    // Check if there is any approver of the transaction was confirmed by newer milestones.



    let milestone_index = if let Some(metadata) = tangle.get_metadata(message_id) {
        metadata.milestone_index()
    } else {
        return Err(Error::MetadataNotFound(*message_id));
    };





    let mut is_solid = false;
    traversal::visit_children_follow_parent1(
        tangle,
        *message_id,
        |_, metadata| {
            if is_solid {
                return false;
            }
            // `true` when one of the current tx's approver was confirmed by a newer milestone_index.
            is_solid = metadata.flags().is_confirmed() && metadata.milestone_index() > milestone_index;
            true
        },
        |_, _, _| {},
    );

    Ok(is_solid)
}

/// Determines the new set of SEPs.
pub fn get_new_solid_entry_points<B: StorageBackend>(
    tangle: &MsTangle<B>,
    target_index: MilestoneIndex,
) -> Result<HashMap<MessageId, MilestoneIndex>, Error> {



    // This will be the new set of SEPs. A SEP is a `MessageId`s with the `MilestoneIndex` that confirmed it.
    let new_seps: HashMap<MessageId, MilestoneIndex> = HashMap::new();

    // We go through all milestones [target_index-THRESHOLD, target_index)
    for index in (*target_index - SEP_CHECK_THRESHOLD_PAST)..*target_index {

        // Determine the actual `message_id` of the current milestone index. If the index is not found in the Tangle the algorithm returns with an `Error::MilestoneNotFound`.
        let ms_message_id = if let Some(message_id) = tangle.get_milestone_message_id(MilestoneIndex(index)) {
            message_id
        } else {
            return Err(Error::MilestoneNotFoundInTangle(index));
        };

        // Get all the approvees/parents confirmed by the milestone.
        traversal::visit_parents_depth_first(
            tangle,
            ms_message_id,
            |_, _, metadata| *metadata.milestone_index() >= index,
            |message_id, _, metadata| {
                if metadata.flags().is_confirmed() && is_solid_entry_point(tangle, &message_id).unwrap() {
                    // Find all tails.
                    helper::on_all_tails(&**tangle, *message_id, |message_id, _tx, metadata| {
                        new_seps.insert(*message_id, metadata.milestone_index());
                    });
                }
            },
            |_, _, _| {},
            |_| {},
        );
    }
    Ok(new_seps)
}use bee_protocol::{
    tangle::{helper, MsTangle},
    MilestoneIndex,
};
use bee_storage::storage::Backend;
use bee_tangle::traversal;{
    unimplemented!()
}

// TODO remove the confirmed transactions in the database.
pub fn prune_transactions(_hashes: Vec<Hash>) -> u32 {
    unimplemented!()
}

// TODO prunes the milestone metadata and the ledger diffs from the database for the given milestone.
pub fn prune_milestone(_milestone_index: MilestoneIndex) {
    // Delete ledger_diff for milestone with milestone_index.
    // Delete milestone storage (if we have this) for milestone with milestone_index.
    unimplemented!()
}

// NOTE we don't prune cache, but only prune the database.
pub fn prune_database<B: Backend>(tangle: &MsTangle<B>, mut target_index: MilestoneIndex) -> Result<(), Error> {
    let target_index_max = MilestoneIndex(
        *tangle.get_snapshot_index() - SOLID_ENTRY_POINT_CHECK_THRESHOLD_PAST - ADDITIONAL_PRUNING_THRESHOLD - 1,
    );
    if target_index > target_index_max {
        target_index = target_index_max;
    }
    // Update the solid entry points in the static MsTangle.
    let new_solid_entry_points = get_new_solid_entry_points(tangle, target_index)?;

    // Clear the solid_entry_points in the static MsTangle.
    tangle.clear_solid_entry_points();

    // TODO update the whole solid_entry_points in the static MsTangle w/o looping.
    for (hash, milestone_index) in new_solid_entry_points.into_iter() {
        tangle.add_solid_entry_point(hash, milestone_index);
    }

    // We have to set the new solid entry point index.
    // This way we can cleanly prune even if the pruning was aborted last time.
    tangle.update_entry_point_index(target_index);

    prune_unconfirmed_transactions(&tangle.get_pruning_index());

    // Iterate through all milestones that have to be pruned.
    for milestone_index in *tangle.get_pruning_index() + 1..*target_index + 1 {
        info!("Pruning milestone {}...", milestone_index);

        // TODO calculate the deleted tx count and visited tx count if needed
        let pruned_unconfirmed_transactions_count = prune_unconfirmed_transactions(&MilestoneIndex(milestone_index));

        // NOTE Actually we don't really need the tail, and only need one of the milestone tx.
        //      In gohornet, we start from the tail milestone tx.
        let milestone_hash;
        if let Some(hash) = tangle.get_milestone_hash(MilestoneIndex(milestone_index)) {
            milestone_hash = hash;
        } else {
            warn!("Pruning milestone {} failed! Milestone not found!", milestone_index);
            continue;
        }

        let mut transactions_to_prune: Vec<Hash> = Vec::new();

        // Traverse the approvees of the milestone transaction.
        traversal::visit_parents_depth_first(
            tangle,
            milestone_hash,
            |_, _, _| {
                // NOTE everything that was referenced by that milestone can be pruned
                //      (even transactions of older milestones)
                true
            },
            |hash, _, _| transactions_to_prune.push(*hash),
            |_, _, _| {},
            |_| {},
        );

        // NOTE The metadata of solid entry points can be deleted from the database,
        //      because we only need the hashes of them, and don't have to trace their parents.
        let transactions_to_prune_count = transactions_to_prune.len();
        let pruned_transactions_count = prune_transactions(transactions_to_prune);

        prune_milestone(MilestoneIndex(milestone_index));

        tangle.update_pruning_index(MilestoneIndex(milestone_index));
        info!(
            "Pruning milestone {}. Pruned {}/{} confirmed transactions. Pruned {} unconfirmed transactions.",
            milestone_index,
            pruned_transactions_count,
            transactions_to_prune_count,
            pruned_unconfirmed_transactions_count
        );
        // TODO trigger pruning milestone index changed event if needed.
        //      notify peers about our new pruning milestone index by
        //      broadcast_heartbeat()
    }
    Ok(())
}
